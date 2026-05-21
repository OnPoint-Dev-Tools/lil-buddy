
use image::{imageops::FilterType, RgbaImage};
use serde::Deserialize;
use std::{
    io::{self, BufRead, Write},
    net::TcpStream,
    sync::{mpsc, Arc},
    time::{Duration, Instant},
};
use wgpu::util::DeviceExt;
use winit::{
    dpi::{LogicalPosition, LogicalSize, PhysicalPosition},
    event::{ElementState, Event, MouseButton, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{WindowBuilder, WindowLevel},
};

const PET_W: u32 = 340;
const PET_H: u32 = 230;
const DEFAULT_SPRITE_SIZE: u32 = 120;
const DEFAULT_SPRITE_Y: i32 = 88;
const IDLE_WALK_INTERVAL_MIN_SECS: u64 = 7;
const IDLE_WALK_INTERVAL_MAX_SECS: u64 = 11;

#[derive(Debug, Clone, Copy)]
struct CompanionScale {
    sprite_size: u32,
    sprite_y: i32,
}

impl CompanionScale {
    fn from_size(value: &str) -> Self {
        match value.trim() {
            "small" => Self { sprite_size: 96, sprite_y: 112 },
            "medium" => Self { sprite_size: DEFAULT_SPRITE_SIZE, sprite_y: DEFAULT_SPRITE_Y },
            "large" => Self { sprite_size: 170, sprite_y: 60 },
            _ => Self { sprite_size: DEFAULT_SPRITE_SIZE, sprite_y: DEFAULT_SPRITE_Y },
        }
    }

    fn sprite_x(self) -> i32 {
        ((PET_W - self.sprite_size) / 2) as i32
    }

    fn walk_bound_x(self) -> i32 {
        ((PET_W - self.sprite_size) / 2) as i32 - 8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mood {
    Hello,
    Idle,
    WalkLeft,
    WalkRight,
    Thinking,
    Working,
    Command,
    Done,
    Error,
}

impl Mood {
    fn from_str(value: &str) -> Self {
        match value {
            "hello" => Self::Hello,
            "walk-left" => Self::WalkLeft,
            "walk-right" => Self::WalkRight,
            "thinking" => Self::Thinking,
            "working" => Self::Working,
            "command" => Self::Command,
            "done" => Self::Done,
            "error" => Self::Error,
            _ => Self::Idle,
        }
    }

    fn fps(self) -> u64 {
        match self {
            Self::Idle => 1,
            Self::Hello => 5,
            Self::WalkLeft | Self::WalkRight => 8,
            Self::Thinking => 4,
            Self::Working => 6,
            Self::Command | Self::Done | Self::Error => 5,
        }
    }

    fn loops(self) -> bool {
        matches!(self, Self::Idle | Self::WalkLeft | Self::WalkRight | Self::Thinking | Self::Working)
    }

    fn frames(self, bank: &FrameBank) -> &[Frame] {
        match self {
            Self::Hello => &bank.hello,
            Self::Idle => &bank.idle_open,
            Self::WalkLeft => &bank.walk_left,
            Self::WalkRight => &bank.walk_right,
            Self::Thinking => &bank.thinking,
            Self::Working => &bank.working,
            Self::Command => &bank.command,
            Self::Done => &bank.done,
            Self::Error => &bank.error,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
enum NativeCommand {
    #[serde(rename = "mood")]
    Mood { mood: String },
    #[serde(rename = "pose")]
    Pose { pose: String },
    #[serde(rename = "category")]
    Category { category: String, pose: String },
    #[serde(rename = "walk")]
    Walk { direction: String, distance: i32, duration_ms: u64 },
    #[serde(rename = "show")]
    Show,
    #[serde(rename = "hide")]
    Hide,
    #[serde(rename = "quit")]
    Quit,
}

#[derive(Clone)]
struct Frame {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
}

struct FrameBank {
    hello: Vec<Frame>,
    idle_open: Vec<Frame>,
    idle_blink: Frame,
    idle: Vec<Frame>,
    walk_left: Vec<Frame>,
    walk_right: Vec<Frame>,
    thinking: Vec<Frame>,
    working: Vec<Frame>,
    command: Vec<Frame>,
    done: Vec<Frame>,
    error: Vec<Frame>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeMode {
    Idle,
    Thinking,
    Working,
    Command,
    Done,
    Error,
}

struct WalkState {
    started_at: Instant,
    duration: Duration,
    start_x: i32,
    y: i32,
    start_offset_x: i32,
    target_offset_x: i32,
    direction: i32,
    distance: i32,
    return_to_idle: bool,
}

struct TimedClip {
    started_at: Instant,
    duration: Duration,
    return_to: Mood,
}

struct Runtime {
    mode: RuntimeMode,
    requested_mood: Mood,
    mood: Mood,
    frame_index: usize,
    last_frame_at: Instant,
    walk: Option<WalkState>,
    timed_clip: Option<TimedClip>,
    static_pose: Option<String>,
    next_idle_wave_at: Instant,
    next_idle_walk_at: Instant,
    next_blink_at: Instant,
    blink_until: Option<Instant>,
    blink_interval_flip: bool,
    next_walk_direction: i32,
    sprite_offset_x: i32,
    bubble_label: Option<String>,
    active_event_category: Option<String>,
    active_event_pose: Option<String>,
    idle_wave_pose: String,
}

struct DragState {
    press_cursor: PhysicalPosition<f64>,
    moved: bool,
    native_dragging: bool,
}

#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 2],
    uv: [f32; 2],
}

struct GpuRenderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

macro_rules! companion_png {
    ($base:literal, $path:literal) => {
        include_bytes!(concat!("../../src/assets/companions/", $base, "-v2/", $path))
    };
}

macro_rules! companion_bank {
    ($base:literal) => {
        FrameBank {
            idle_open: load_frames(&[
                companion_png!($base, "idle/01.png"),
            ]),
            idle_blink: load_frame(companion_png!($base, "idle/03.png")),
            idle: load_frames(&[
                companion_png!($base, "idle/01.png"), companion_png!($base, "idle/02.png"), companion_png!($base, "idle/03.png"),
                companion_png!($base, "idle/04.png"), companion_png!($base, "idle/05.png"), companion_png!($base, "idle/06.png"),
            ]),
            hello: load_frames(&[
                companion_png!($base, "hello/01.png"), companion_png!($base, "hello/02.png"), companion_png!($base, "hello/03.png"),
                companion_png!($base, "hello/04.png"), companion_png!($base, "hello/05.png"), companion_png!($base, "hello/06.png"),
            ]),
            walk_right: load_walk_frames(&[
                companion_png!($base, "walk-right/01.png"), companion_png!($base, "walk-right/02.png"), companion_png!($base, "walk-right/03.png"), companion_png!($base, "walk-right/04.png"),
                companion_png!($base, "walk-right/05.png"), companion_png!($base, "walk-right/06.png"), companion_png!($base, "walk-right/07.png"), companion_png!($base, "walk-right/08.png"),
            ]),
            walk_left: load_walk_frames(&[
                companion_png!($base, "walk-left/01.png"), companion_png!($base, "walk-left/02.png"), companion_png!($base, "walk-left/03.png"), companion_png!($base, "walk-left/04.png"),
                companion_png!($base, "walk-left/05.png"), companion_png!($base, "walk-left/06.png"), companion_png!($base, "walk-left/07.png"), companion_png!($base, "walk-left/08.png"),
            ]),
            thinking: load_frames(&[
                companion_png!($base, "thinking/01.png"), companion_png!($base, "thinking/02.png"), companion_png!($base, "thinking/03.png"),
                companion_png!($base, "thinking/04.png"), companion_png!($base, "thinking/05.png"), companion_png!($base, "thinking/06.png"),
            ]),
            working: load_frames(&[
                companion_png!($base, "working/01.png"), companion_png!($base, "working/02.png"), companion_png!($base, "working/03.png"),
                companion_png!($base, "working/04.png"), companion_png!($base, "working/05.png"), companion_png!($base, "working/06.png"),
            ]),
            command: load_frames(&[
                companion_png!($base, "command/01.png"), companion_png!($base, "command/02.png"), companion_png!($base, "command/03.png"),
                companion_png!($base, "command/04.png"), companion_png!($base, "command/05.png"), companion_png!($base, "command/06.png"),
            ]),
            done: load_frames(&[
                companion_png!($base, "done/01.png"), companion_png!($base, "done/02.png"), companion_png!($base, "done/03.png"),
                companion_png!($base, "done/04.png"), companion_png!($base, "done/05.png"), companion_png!($base, "done/06.png"),
            ]),
            error: load_frames(&[
                companion_png!($base, "error/01.png"), companion_png!($base, "error/02.png"), companion_png!($base, "error/03.png"),
                companion_png!($base, "error/04.png"), companion_png!($base, "error/05.png"), companion_png!($base, "error/06.png"),
            ]),
        }
    };
}

fn alpha_bounds(image: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (width, height) = image.dimensions();
    let mut min_x = width;
    let mut min_y = height;
    let mut max_x = 0;
    let mut max_y = 0;
    let mut found = false;

    for y in 0..height {
        for x in 0..width {
            if image.get_pixel(x, y)[3] > 0 {
                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x);
                max_y = max_y.max(y);
                found = true;
            }
        }
    }

    found.then_some((min_x, min_y, max_x, max_y))
}

fn load_frame(bytes: &[u8]) -> Frame {
    let image = image::load_from_memory(bytes)
        .expect("companion frame should decode")
        .resize_exact(DEFAULT_SPRITE_SIZE, DEFAULT_SPRITE_SIZE, FilterType::Lanczos3)
        .to_rgba8();

    Frame {
        pixels: image.into_raw(),
        width: DEFAULT_SPRITE_SIZE,
        height: DEFAULT_SPRITE_SIZE,
    }
}

fn load_walk_frame(bytes: &[u8]) -> Frame {
    let image = image::load_from_memory(bytes)
        .expect("companion frame should decode")
        .to_rgba8();

    let cropped = if let Some((min_x, min_y, max_x, max_y)) = alpha_bounds(&image) {
        image::imageops::crop_imm(&image, min_x, min_y, max_x - min_x + 1, max_y - min_y + 1).to_image()
    } else {
        image
    };

    let walk_target = ((DEFAULT_SPRITE_SIZE as f32) * 0.84).round() as u32;
    let fitted = image::DynamicImage::ImageRgba8(cropped)
        .resize(walk_target, walk_target, FilterType::Lanczos3)
        .to_rgba8();

    let mut canvas = RgbaImage::new(DEFAULT_SPRITE_SIZE, DEFAULT_SPRITE_SIZE);
    let x = ((DEFAULT_SPRITE_SIZE - fitted.width()) / 2) as i64;
    let y = (DEFAULT_SPRITE_SIZE.saturating_sub(fitted.height()).saturating_sub(4)) as i64;
    image::imageops::overlay(&mut canvas, &fitted, x, y);

    Frame {
        pixels: canvas.into_raw(),
        width: DEFAULT_SPRITE_SIZE,
        height: DEFAULT_SPRITE_SIZE,
    }
}

fn load_frames(items: &[&[u8]]) -> Vec<Frame> {
    items.iter().map(|bytes| load_frame(bytes)).collect()
}

fn load_walk_frames(items: &[&[u8]]) -> Vec<Frame> {
    items.iter().map(|bytes| load_walk_frame(bytes)).collect()
}

fn should_flip_walk_bank(character_id: &str) -> bool {
    matches!(
        character_id,
        "red-beanie" | "pink-hood" | "lavender-bear" | "autumn-vest"
    )
}

fn maybe_flip_walk_bank(character_id: &str, mut bank: FrameBank) -> FrameBank {
    // These generated walk sheets have their visual left/right folders opposite
    // from the native movement direction. Keep the asset files untouched and
    // fix the mapping here so runtime commands stay semantically correct:
    // `walk-left` moves left and `walk-right` moves right.
    if should_flip_walk_bank(character_id) {
        std::mem::swap(&mut bank.walk_left, &mut bank.walk_right);
    }

    bank
}

fn load_bank(character_id: &str) -> FrameBank {
    let bank = match character_id {
        "tan-explorer" => companion_bank!("tan-explorer"),
        "pink-hood" => companion_bank!("pink-hood"),
        "autumn-vest" => companion_bank!("autumn-vest"),
        "green-scout" => companion_bank!("green-scout"),
        "blue-hoodie" => companion_bank!("blue-hoodie"),
        "lavender-bear" => companion_bank!("lavender-bear"),
        "yellow-rain" => companion_bank!("yellow-rain"),
        "red-beanie" => companion_bank!("red-beanie"),
        _ => companion_bank!("tan-explorer"),
    };

    maybe_flip_walk_bank(character_id, bank)
}

fn parse_arg_value(args: &[String], name: &str) -> Option<String> {
    args.iter()
        .position(|item| item == name)
        .and_then(|index| args.get(index + 1))
        .cloned()
}

fn send_ipc(port: u16, line: &str) {
    if port == 0 {
        return;
    }

    if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) {
        let _ = writeln!(stream, "{line}");
    }
}

fn spawn_stdin_reader(tx: mpsc::Sender<NativeCommand>) {
    std::thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines().map_while(Result::ok) {
            match serde_json::from_str::<NativeCommand>(&line) {
                Ok(command) => {
                    let should_quit = matches!(command, NativeCommand::Quit);
                    let _ = tx.send(command);
                    if should_quit {
                        break;
                    }
                }
                Err(error) => {
                    eprintln!("native companion command parse failed: {error}; raw={line}");
                }
            }
        }
    });
}


fn run_idle_behavior(
    runtime: &mut Runtime,
    window: &winit::window::Window,
    start_x: i32,
    start_y: i32,
    companion_scale: CompanionScale,
) {
    let idle_pose = runtime.idle_wave_pose.clone();

    if is_walk_pose(&idle_pose) {
        let direction = runtime.next_walk_direction;
        runtime.next_walk_direction = -runtime.next_walk_direction;
        start_screen_walk(runtime, window, start_x, start_y, direction, 210, 4000, true, companion_scale);
    } else {
        runtime.mode = RuntimeMode::Idle;
        runtime.requested_mood = Mood::Idle;
        set_timed_static_pose(runtime, idle_pose, 2000);
    }
}

fn ease_in_out(progress: f64) -> f64 {
    if progress < 0.5 {
        2.0 * progress * progress
    } else {
        1.0 - (-2.0 * progress + 2.0).powi(2) / 2.0
    }
}

fn set_mood(runtime: &mut Runtime, mood: Mood) {
    runtime.static_pose = None;
    runtime.bubble_label = bubble_label_for_mood(mood).map(str::to_string);

    if runtime.mood == mood {
        return;
    }

    runtime.mood = mood;
    runtime.frame_index = 0;
    runtime.last_frame_at = Instant::now();
}

fn is_walk_pose(pose: &str) -> bool {
    pose == "walk-left" || pose == "walk-right"
}

fn is_terminal_category(category: &str) -> bool {
    category == "done" || category == "error"
}

fn set_timed_static_pose(runtime: &mut Runtime, pose: String, duration_ms: u64) {
    runtime.bubble_label = bubble_label_for_pose(&pose).map(str::to_string);
    runtime.static_pose = Some(pose);
    runtime.timed_clip = Some(TimedClip {
        started_at: Instant::now(),
        duration: Duration::from_millis(duration_ms),
        return_to: Mood::Idle,
    });
    runtime.frame_index = 0;
    runtime.last_frame_at = Instant::now();
}

fn reset_to_idle_system(runtime: &mut Runtime) {
    runtime.mode = RuntimeMode::Idle;
    runtime.requested_mood = Mood::Idle;
    runtime.mood = Mood::Idle;
    runtime.static_pose = None;
    runtime.bubble_label = None;
    runtime.active_event_category = None;
    runtime.active_event_pose = None;
    runtime.walk = None;
    runtime.frame_index = 0;
    runtime.last_frame_at = Instant::now();
    schedule_idle_times(runtime);
}

fn set_event_category(runtime: &mut Runtime, category: String, pose: String) {
    runtime.active_event_category = Some(category.clone());
    runtime.active_event_pose = Some(pose.clone());
    runtime.mode = RuntimeMode::Working;
    runtime.requested_mood = Mood::Idle;
    runtime.blink_until = None;
    runtime.walk = None;

    if is_terminal_category(&category) {
        set_timed_static_pose(runtime, pose, 1800);
    } else if is_walk_pose(&pose) {
        // The real walk is started by the command handler because it needs the window.
        runtime.static_pose = None;
        runtime.timed_clip = None;
        runtime.bubble_label = bubble_label_for_pose(&pose).map(str::to_string);
    } else {
        runtime.bubble_label = bubble_label_for_pose(&pose).map(str::to_string);
        runtime.static_pose = Some(pose);
        runtime.timed_clip = None;
        runtime.frame_index = 0;
        runtime.last_frame_at = Instant::now();
    }
}

fn pose_frame<'a>(bank: &'a FrameBank, pose: &str) -> &'a Frame {
    match pose {
        "intro" => bank.hello.get(1).unwrap_or(&bank.hello[0]),
        "hello" => bank.hello.get(2).unwrap_or(&bank.hello[0]),
        "working" => bank.working.get(2).unwrap_or(&bank.working[0]),
        "walk-right" => bank.walk_right.get(2).unwrap_or(&bank.walk_right[0]),
        "walk-left" => bank.walk_left.get(2).unwrap_or(&bank.walk_left[0]),
        "thinking" => bank.thinking.get(2).unwrap_or(&bank.thinking[0]),
        "command" => bank.command.get(2).unwrap_or(&bank.command[0]),
        "done" => bank.done.get(2).unwrap_or(&bank.done[0]),
        "error" => bank.error.get(2).unwrap_or(&bank.error[0]),
        "idle" => bank.idle_open.first().unwrap_or(&bank.idle[0]),
        _ => bank.idle_open.first().unwrap_or(&bank.idle[0]),
    }
}

fn bubble_label_for_pose(pose: &str) -> Option<&'static str> {
    match pose {
        "working" | "walk-left" | "walk-right" => Some("WORKING"),
        "thinking" => Some("THINKING"),
        "command" => Some("CMD"),
        "done" => Some("IM DONE"),
        "error" => Some("ERROR"),
        "hello" | "intro" => Some("HI"),
        _ => None,
    }
}

fn bubble_label_for_mood(mood: Mood) -> Option<&'static str> {
    match mood {
        Mood::Working | Mood::WalkLeft | Mood::WalkRight => Some("WORKING"),
        Mood::Thinking => Some("THINKING"),
        Mood::Command => Some("CMD"),
        Mood::Done => Some("IM DONE"),
        Mood::Error => Some("ERROR"),
        Mood::Hello => Some("HI"),
        Mood::Idle => None,
    }
}

fn glyph_rows(ch: char) -> [&'static str; 7] {
    match ch {
        'C' => ["01110", "10001", "10000", "10000", "10000", "10001", "01110"],
        'D' => ["11110", "10001", "10001", "10001", "10001", "10001", "11110"],
        'E' => ["11111", "10000", "10000", "11110", "10000", "10000", "11111"],
        'H' => ["10001", "10001", "10001", "11111", "10001", "10001", "10001"],
        'I' => ["11111", "00100", "00100", "00100", "00100", "00100", "11111"],
        'K' => ["10001", "10010", "10100", "11000", "10100", "10010", "10001"],
        'M' => ["10001", "11011", "10101", "10101", "10001", "10001", "10001"],
        'N' => ["10001", "11001", "10101", "10011", "10001", "10001", "10001"],
        'O' => ["01110", "10001", "10001", "10001", "10001", "10001", "01110"],
        'R' => ["11110", "10001", "10001", "11110", "10100", "10010", "10001"],
        'T' => ["11111", "00100", "00100", "00100", "00100", "00100", "00100"],
        'W' => ["10001", "10001", "10001", "10101", "10101", "10101", "01010"],
        _ => ["00000", "00000", "00000", "00000", "00000", "00000", "00000"],
    }
}

fn blend_pixel(pixels: &mut [u8], width: u32, height: u32, x: i32, y: i32, rgba: [u8; 4]) {
    if x < 0 || y < 0 || x >= width as i32 || y >= height as i32 {
        return;
    }

    let idx = ((y as u32 * width + x as u32) * 4) as usize;
    pixels[idx] = rgba[0];
    pixels[idx + 1] = rgba[1];
    pixels[idx + 2] = rgba[2];
    pixels[idx + 3] = rgba[3];
}

#[allow(clippy::too_many_arguments)]
fn fill_rounded_rect(pixels: &mut [u8], width: u32, height: u32, x: i32, y: i32, w: i32, h: i32, r: i32, fill: [u8; 4], border: [u8; 4]) {
    for py in y..(y + h) {
        for px in x..(x + w) {
            let dx = if px < x + r {
                x + r - px
            } else if px >= x + w - r {
                px - (x + w - r - 1)
            } else {
                0
            };
            let dy = if py < y + r {
                y + r - py
            } else if py >= y + h - r {
                py - (y + h - r - 1)
            } else {
                0
            };

            if dx * dx + dy * dy > r * r {
                continue;
            }

            let edge = px <= x + 1 || px >= x + w - 2 || py <= y + 1 || py >= y + h - 2 || (dx * dx + dy * dy >= (r - 2) * (r - 2));
            blend_pixel(pixels, width, height, px, py, if edge { border } else { fill });
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_text(pixels: &mut [u8], width: u32, height: u32, text: &str, x: i32, y: i32, scale: i32, color: [u8; 4]) {
    let mut cursor_x = x;
    for ch in text.chars() {
        if ch == ' ' {
            cursor_x += 4 * scale;
            continue;
        }

        let rows = glyph_rows(ch);
        for (row_idx, row) in rows.iter().enumerate() {
            for (col_idx, bit) in row.chars().enumerate() {
                if bit != '1' {
                    continue;
                }
                for sy in 0..scale {
                    for sx in 0..scale {
                        blend_pixel(
                            pixels,
                            width,
                            height,
                            cursor_x + col_idx as i32 * scale + sx,
                            y + row_idx as i32 * scale + sy,
                            color,
                        );
                    }
                }
            }
        }

        cursor_x += 6 * scale;
    }
}

fn bubble_frame(label: &str) -> Frame {
    let width = ((label.chars().count() as u32 * 12) + 28).clamp(118, 168);
    let height = 46;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    fill_rounded_rect(
        &mut pixels,
        width,
        height,
        1,
        1,
        width as i32 - 6,
        height as i32 - 12,
        13,
        [255, 252, 244, 238],
        [200, 142, 52, 220],
    );

    // Little thought-tail dots on the left side, flipped down/right toward the companion.
    fill_rounded_rect(&mut pixels, width, height, 18, 33, 12, 8, 4, [255, 252, 244, 220], [200, 142, 52, 190]);
    fill_rounded_rect(&mut pixels, width, height, 31, 40, 7, 5, 3, [255, 252, 244, 200], [200, 142, 52, 170]);

    let text_width = (label.chars().count() as i32 * 6 - 1) * 2;
    let tx = ((width as i32 - 6 - text_width) / 2).max(8);
    draw_text(&mut pixels, width, height, label, tx, 12, 2, [82, 58, 32, 255]);

    Frame { pixels, width, height }
}

fn bubble_rect(sprite_offset_x: i32, bubble: &Frame, scale: CompanionScale) -> [Vertex; 4] {
    let sprite_x = (scale.sprite_x() + sprite_offset_x).clamp(0, PET_W as i32 - scale.sprite_size as i32);
    let w = bubble.width as i32;
    let h = bubble.height as i32;
    let sprite_center = sprite_x + (scale.sprite_size as i32 / 2);

    // Keep the bubble centered above the head while leaving the small tail dots on the left side.
    let mut x = sprite_center - (w / 2) + 8;
    let y = 26;

    if x + w > PET_W as i32 - 4 {
        x = PET_W as i32 - w - 4;
    }
    if x < 4 {
        x = 4;
    }

    quad_vertices_rect(x, y, w, h)
}

fn quad_vertices_rect(x: i32, y: i32, w: i32, h: i32) -> [Vertex; 4] {
    let x0 = (x as f32 / PET_W as f32) * 2.0 - 1.0;
    let y0 = 1.0 - (y as f32 / PET_H as f32) * 2.0;
    let x1 = ((x + w) as f32 / PET_W as f32) * 2.0 - 1.0;
    let y1 = 1.0 - ((y + h) as f32 / PET_H as f32) * 2.0;

    [
        Vertex { position: [x0, y0], uv: [0.0, 0.0] },
        Vertex { position: [x1, y0], uv: [1.0, 0.0] },
        Vertex { position: [x1, y1], uv: [1.0, 1.0] },
        Vertex { position: [x0, y1], uv: [0.0, 1.0] },
    ]
}

fn next_idle_walk_delay_secs(runtime: &Runtime) -> u64 {
    if runtime.next_walk_direction > 0 {
        IDLE_WALK_INTERVAL_MIN_SECS
    } else {
        IDLE_WALK_INTERVAL_MAX_SECS
    }
}

fn schedule_idle_times(runtime: &mut Runtime) {
    let now = Instant::now();
    runtime.next_idle_wave_at = now + Duration::from_secs(15);
    runtime.next_idle_walk_at = now + Duration::from_secs(next_idle_walk_delay_secs(runtime));
    runtime.next_blink_at = now + Duration::from_millis(if runtime.blink_interval_flip { 6500 } else { 4200 });
    runtime.blink_interval_flip = !runtime.blink_interval_flip;
    runtime.blink_until = None;
}

fn start_timed_clip(runtime: &mut Runtime, mood: Mood, duration_ms: u64, return_to: Mood) {
    runtime.static_pose = None;
    runtime.timed_clip = Some(TimedClip {
        started_at: Instant::now(),
        duration: Duration::from_millis(duration_ms),
        return_to,
    });
    set_mood(runtime, mood);
}

#[allow(clippy::too_many_arguments)]
fn start_screen_walk(
    runtime: &mut Runtime,
    window: &winit::window::Window,
    start_x: i32,
    start_y: i32,
    direction_sign: i32,
    distance: i32,
    duration_ms: u64,
    return_to_idle: bool,
    companion_scale: CompanionScale,
) {
    let current = window
        .outer_position()
        .unwrap_or(PhysicalPosition::new(start_x, start_y));

    let mut direction = direction_sign.signum().clamp(-1, 1);
    let walk_bound_x = companion_scale.walk_bound_x().max(0);
    if walk_bound_x == 0 {
        return;
    }

    let start_offset_x = runtime.sprite_offset_x.clamp(-walk_bound_x, walk_bound_x);
    let min_distance = ((walk_bound_x * 2) / 3).clamp(18, 56);
    let requested_distance = distance.abs().clamp(min_distance, walk_bound_x);

    // Bounce before hitting the transparent viewport edge. This makes the
    // movement visible on Wayland/Hyprland even when set_outer_position is ignored.
    if start_offset_x + direction * requested_distance > walk_bound_x {
        direction = -1;
    } else if start_offset_x + direction * requested_distance < -walk_bound_x {
        direction = 1;
    }

    let target_offset_x = (start_offset_x + direction * requested_distance).clamp(-walk_bound_x, walk_bound_x);
    let actual_distance = (target_offset_x - start_offset_x).abs().max(1);
    let scaled_duration_ms = (700 + actual_distance as u64 * 26).clamp(900, 2600);
    let requested_duration_ms = duration_ms.clamp(900, 4200);
    let effective_duration_ms = requested_duration_ms.max(scaled_duration_ms);

    runtime.mode = RuntimeMode::Working;
    runtime.static_pose = None;
    runtime.timed_clip = None;
    runtime.blink_until = None;
    set_mood(runtime, if direction < 0 { Mood::WalkRight } else { Mood::WalkLeft });
    runtime.bubble_label = Some(if return_to_idle { "I NEED WORK" } else { "WORKING" }.to_string());

    runtime.walk = Some(WalkState {
        started_at: Instant::now(),
        duration: Duration::from_millis(effective_duration_ms),
        start_x: current.x,
        y: current.y,
        start_offset_x,
        target_offset_x,
        direction,
        distance: actual_distance,
        return_to_idle,
    });
}

fn quad_vertices(sprite_offset_x: i32, scale: CompanionScale) -> [Vertex; 4] {
    quad_vertices_rect(
        scale.sprite_x() + sprite_offset_x,
        scale.sprite_y,
        scale.sprite_size as i32,
        scale.sprite_size as i32,
    )
}

async fn create_renderer(window: Arc<winit::window::Window>) -> Result<GpuRenderer, String> {
    let instance = wgpu::Instance::default();
    let surface = instance
        .create_surface(window.clone())
        .map_err(|error| format!("create native companion surface: {error}"))?;

    let adapter = if let Some(adapter) = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
    {
        adapter
    } else if let Some(adapter) = instance
        .request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        })
        .await
    {
        adapter
    } else {
        instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::LowPower,
                compatible_surface: Some(&surface),
                force_fallback_adapter: true,
            })
            .await
            .ok_or_else(|| "no compatible GPU adapter found for native companion".to_string())?
    };

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: Some("Lil Buddy companion device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_webgl2_defaults(),
            },
            None,
        )
        .await
        .map_err(|error| format!("request native companion device: {error}"))?;

    let caps = surface.get_capabilities(&adapter);
    let format = caps
        .formats
        .iter()
        .copied()
        .find(|format| format.is_srgb())
        .unwrap_or(caps.formats[0]);

    let alpha_mode = caps
        .alpha_modes
        .iter()
        .copied()
        .find(|mode| matches!(mode, wgpu::CompositeAlphaMode::PreMultiplied | wgpu::CompositeAlphaMode::PostMultiplied))
        .unwrap_or(caps.alpha_modes[0]);

    let config = wgpu::SurfaceConfiguration {
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        format,
        width: PET_W,
        height: PET_H,
        present_mode: wgpu::PresentMode::Fifo,
        alpha_mode,
        view_formats: vec![],
        desired_maximum_frame_latency: 2,
    };

    surface.configure(&device, &config);

    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("Lil Buddy companion sprite shader"),
        source: wgpu::ShaderSource::Wgsl(r#"
struct VertexIn {
  @location(0) position: vec2<f32>,
  @location(1) uv: vec2<f32>,
};

struct VertexOut {
  @builtin(position) position: vec4<f32>,
  @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(input: VertexIn) -> VertexOut {
  var out: VertexOut;
  out.position = vec4<f32>(input.position, 0.0, 1.0);
  out.uv = input.uv;
  return out;
}

@group(0) @binding(0)
var sprite_tex: texture_2d<f32>;

@group(0) @binding(1)
var sprite_sampler: sampler;

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
  return textureSample(sprite_tex, sprite_sampler, input.uv);
}
"#.into()),
    });

    let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Lil Buddy companion texture layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    });

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Lil Buddy companion pipeline layout"),
        bind_group_layouts: &[&bind_group_layout],
        push_constant_ranges: &[],
    });

    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Lil Buddy companion pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        offset: 0,
                        shader_location: 0,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                    wgpu::VertexAttribute {
                        offset: 8,
                        shader_location: 1,
                        format: wgpu::VertexFormat::Float32x2,
                    },
                ],
            }],
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: "fs_main",
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
    });

    let indices: [u16; 6] = [0, 1, 2, 0, 2, 3];

    let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("Lil Buddy companion index buffer"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("Lil Buddy companion sprite sampler"),
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        ..Default::default()
    });

    Ok(GpuRenderer {
        surface,
        device,
        queue,
        config,
        pipeline,
        bind_group_layout,
        sampler,
        index_buffer,
        num_indices: indices.len() as u32,
    })
}

impl GpuRenderer {
    fn make_bind_group(&self, frame: &Frame) -> wgpu::BindGroup {
        let texture_size = wgpu::Extent3d {
            width: frame.width,
            height: frame.height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Lil Buddy companion texture"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &frame.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * frame.width),
                rows_per_image: Some(frame.height),
            },
            texture_size,
        );

        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Lil Buddy companion bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    fn make_vertex_buffer(&self, vertices: &[Vertex; 4], label: &str) -> wgpu::Buffer {
        self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        })
    }

    fn draw_frame<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        bind_group: &'a wgpu::BindGroup,
        vertex_buffer: &'a wgpu::Buffer,
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, bind_group, &[]);
        pass.set_vertex_buffer(0, vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }

    fn render(&mut self, frame: &Frame, sprite_offset_x: i32, bubble: Option<&Frame>, scale: CompanionScale) {
        let sprite_vertices = quad_vertices(sprite_offset_x, scale);
        let sprite_vertex_buffer = self.make_vertex_buffer(&sprite_vertices, "Lil Buddy sprite vertices");
        let sprite_bind_group = self.make_bind_group(frame);

        let bubble_render = bubble.map(|frame| {
            let vertices = bubble_rect(sprite_offset_x, frame, scale);
            let vertex_buffer = self.make_vertex_buffer(&vertices, "Lil Buddy bubble vertices");
            let bind_group = self.make_bind_group(frame);
            (vertex_buffer, bind_group)
        });

        let output = match self.surface.get_current_texture() {
            Ok(output) => output,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.config);
                return;
            }
            Err(wgpu::SurfaceError::Timeout) => return,
            Err(wgpu::SurfaceError::OutOfMemory) => return,
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Lil Buddy companion encoder"),
        });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Lil Buddy companion render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 0.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            self.draw_frame(&mut pass, &sprite_bind_group, &sprite_vertex_buffer);

            if let Some((bubble_vertex_buffer, bubble_bind_group)) = bubble_render.as_ref() {
                self.draw_frame(&mut pass, bubble_bind_group, bubble_vertex_buffer);
            }
        }

        self.queue.submit(Some(encoder.finish()));
        output.present();
    }
}

pub fn run_from_args() {
    let args: Vec<String> = std::env::args().collect();
    let start_x = parse_arg_value(&args, "--x")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(80);
    let start_y = parse_arg_value(&args, "--y")
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(80);
    let ipc_port = parse_arg_value(&args, "--ipc-port")
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0);
    let idle_wave_pose = parse_arg_value(&args, "--idle-wave-mood")
        .unwrap_or_else(|| "hello".to_string());
    let character_id = parse_arg_value(&args, "--character-id")
        .unwrap_or_else(|| "tan-explorer".to_string());
    let companion_size = parse_arg_value(&args, "--companion-size")
        .unwrap_or_else(|| "medium".to_string());
    let companion_scale = CompanionScale::from_size(&companion_size);

    let bank = load_bank(&character_id);
    let event_loop = EventLoop::new().expect("native companion event loop");
    let window = Arc::new(
        WindowBuilder::new()
            .with_title("Lil Buddy Native Companion")
            .with_inner_size(LogicalSize::new(PET_W as f64, PET_H as f64))
            .with_resizable(false)
            .with_decorations(false)
            .with_transparent(true)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(LogicalPosition::new(start_x as f64, start_y as f64))
            .build(&event_loop)
            .expect("native companion window"),
    );

    window.set_visible(true);

    let mut renderer = match pollster::block_on(create_renderer(window.clone())) {
        Ok(renderer) => renderer,
        Err(error) => {
            eprintln!("native companion renderer failed: {error}");
            return;
        }
    };

    let (tx, rx) = mpsc::channel();
    spawn_stdin_reader(tx);

    let now = Instant::now();
    let mut runtime = Runtime {
        mode: RuntimeMode::Idle,
        requested_mood: Mood::Idle,
        mood: Mood::Idle,
        frame_index: 0,
        last_frame_at: now,
        walk: None,
        timed_clip: Some(TimedClip {
            started_at: now,
            duration: Duration::from_millis(2200),
            return_to: Mood::Idle,
        }),
        static_pose: Some("intro".to_string()),
        next_idle_wave_at: now + Duration::from_secs(15),
        next_idle_walk_at: now + Duration::from_secs(IDLE_WALK_INTERVAL_MIN_SECS),
        next_blink_at: now + Duration::from_millis(4200),
        blink_until: None,
        blink_interval_flip: false,
        next_walk_direction: 1,
        sprite_offset_x: 0,
        bubble_label: Some("HI".to_string()),
        active_event_category: None,
        active_event_pose: None,
        idle_wave_pose,
    };
    let mut drag_state: Option<DragState> = None;
    let mut last_cursor_pos = PhysicalPosition::new(0.0, 0.0);

    let _ = event_loop.run(move |event, target| {
        target.set_control_flow(ControlFlow::WaitUntil(
            Instant::now() + Duration::from_millis(16),
        ));

        while let Ok(command) = rx.try_recv() {
            match command {
                NativeCommand::Mood { mood } => {
                    let requested = Mood::from_str(&mood);
                    if runtime.requested_mood == requested && runtime.timed_clip.is_some() {
                        continue;
                    }
                    runtime.requested_mood = requested;
                    runtime.timed_clip = None;
                    match mood.as_str() {
                        "hello" => {
                            runtime.mode = RuntimeMode::Idle;
                            set_timed_static_pose(&mut runtime, "hello".to_string(), 2000);
                        }
                        "command" => {
                            runtime.mode = RuntimeMode::Command;
                            start_timed_clip(&mut runtime, Mood::Command, 1200, Mood::Idle);
                        }
                        "done" => {
                            runtime.mode = RuntimeMode::Done;
                            start_timed_clip(&mut runtime, Mood::Done, 1400, Mood::Idle);
                        }
                        "error" => {
                            runtime.mode = RuntimeMode::Error;
                            start_timed_clip(&mut runtime, Mood::Error, 1500, Mood::Idle);
                        }
                        "thinking" => {
                            runtime.mode = RuntimeMode::Thinking;
                            set_mood(&mut runtime, Mood::Thinking);
                        }
                        "working" | "walk-left" | "walk-right" => {
                            runtime.mode = RuntimeMode::Working;
                            if mood == "walk-left" {
                                set_mood(&mut runtime, Mood::WalkLeft);
                            } else if mood == "walk-right" {
                                set_mood(&mut runtime, Mood::WalkRight);
                            } else {
                                set_mood(&mut runtime, Mood::Working);
                            }
                        }
                        _ => {
                            runtime.mode = RuntimeMode::Idle;
                            set_mood(&mut runtime, Mood::Idle);
                            schedule_idle_times(&mut runtime);
                        }
                    }
                }
                NativeCommand::Category { category, pose } => {
                    let walk_pose = pose.clone();
                    set_event_category(&mut runtime, category, pose);

                    if is_walk_pose(&walk_pose) {
                        let direction = if walk_pose == "walk-left" { -1 } else { 1 };
                        start_screen_walk(&mut runtime, &window, start_x, start_y, direction, 180, 2600, false, companion_scale);
                    }
                }
                NativeCommand::Pose { pose } => {
                    runtime.requested_mood = Mood::Idle;
                    runtime.blink_until = None;

                    if pose == "walk-left" || pose == "walk-right" {
                        let direction = if pose == "walk-left" { -1 } else { 1 };
                        start_screen_walk(&mut runtime, &window, start_x, start_y, direction, 180, 2600, true, companion_scale);
                    } else {
                        runtime.mode = RuntimeMode::Working;
                        set_timed_static_pose(&mut runtime, pose, 1800);
                    }
                }
                NativeCommand::Walk { direction, distance, duration_ms } => {
                    let direction_sign = if direction == "left" { -1 } else { 1 };
                    start_screen_walk(
                        &mut runtime,
                        &window,
                        start_x,
                        start_y,
                        direction_sign,
                        distance,
                        duration_ms,
                        true,
                        companion_scale,
                    );
                }
                NativeCommand::Show => {
                    window.set_minimized(false);
                    window.set_visible(true);
                }
                NativeCommand::Hide => {
                    window.set_visible(false);
                    window.set_minimized(true);
                    window.set_outer_position(LogicalPosition::new(-32000.0, -32000.0));
                }
                NativeCommand::Quit => target.exit(),
            }
        }

        if let Event::WindowEvent { event, .. } = event {
            match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                    drag_state = Some(DragState {
                        press_cursor: last_cursor_pos,
                        moved: false,
                        native_dragging: false,
                    });
                }
                WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                    if let Some(drag) = drag_state.take() {
                        if drag.moved {
                            if let Ok(pos) = window.outer_position() {
                                send_ipc(ipc_port, &format!("position {} {}", pos.x + runtime.sprite_offset_x, pos.y));
                            }
                        } else if let Ok(pos) = window.outer_position() {
                            let anchor_x = pos.x + runtime.sprite_offset_x + 170;
                            let anchor_y = pos.y;
                            send_ipc(ipc_port, &format!("toggle_chat_at {} {}", anchor_x, anchor_y));
                        } else {
                            send_ipc(ipc_port, "toggle_chat");
                        }
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    last_cursor_pos = position;

                    if let Some(drag) = drag_state.as_mut() {
                        let delta_x = position.x - drag.press_cursor.x;
                        let delta_y = position.y - drag.press_cursor.y;

                        if !drag.moved && (delta_x.abs() >= 6.0 || delta_y.abs() >= 6.0) {
                            drag.moved = true;
                            runtime.walk = None;
                            runtime.timed_clip = None;
                            runtime.static_pose = None;
                            runtime.bubble_label = None;
                            runtime.requested_mood = Mood::Idle;
                            set_mood(&mut runtime, Mood::Idle);

                            if !drag.native_dragging {
                                let _ = window.drag_window();
                                drag.native_dragging = true;
                            }
                        }
                    }
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Right, .. } => {
                    drag_state = None;
                    send_ipc(ipc_port, "open_menu");
                }
                WindowEvent::Resized(size) => {
                    renderer.config.width = size.width.max(1);
                    renderer.config.height = size.height.max(1);
                    renderer.surface.configure(&renderer.device, &renderer.config);
                }
                WindowEvent::RedrawRequested => {
                    let now = Instant::now();

                    if let Some(clip) = &runtime.timed_clip {
                        if clip.started_at.elapsed() >= clip.duration {
                            let return_to = clip.return_to;
                            runtime.timed_clip = None;
                            if return_to == Mood::Idle {
                                reset_to_idle_system(&mut runtime);
                            } else {
                                runtime.static_pose = None;
                                runtime.bubble_label = bubble_label_for_mood(return_to).map(str::to_string);
                                runtime.requested_mood = return_to;
                                set_mood(&mut runtime, return_to);
                            }
                        }
                    }

                    if runtime.active_event_category.is_none() && runtime.mode == RuntimeMode::Idle && runtime.walk.is_none() && runtime.timed_clip.is_none() {
                        if now >= runtime.next_blink_at && runtime.blink_until.is_none() {
                            runtime.blink_until = Some(now + Duration::from_millis(130));
                            let next_delay = if runtime.blink_interval_flip { 6500 } else { 4200 };
                            runtime.blink_interval_flip = !runtime.blink_interval_flip;
                            runtime.next_blink_at = now + Duration::from_millis(next_delay);
                        }

                        if now >= runtime.next_idle_walk_at {
                            run_idle_behavior(&mut runtime, &window, start_x, start_y, companion_scale);
                            runtime.next_idle_wave_at = now + Duration::from_secs(15);
                            runtime.next_idle_walk_at = now + Duration::from_secs(next_idle_walk_delay_secs(&runtime));
                        }
                    }

                    if let Some(walk) = &runtime.walk {
                        let elapsed = walk.started_at.elapsed();
                        let progress = (elapsed.as_secs_f64() / walk.duration.as_secs_f64()).min(1.0);
                        let eased = ease_in_out(progress);
                        let offset_delta = ((walk.target_offset_x - walk.start_offset_x) as f64 * eased).round() as i32;
                        runtime.sprite_offset_x = (walk.start_offset_x + offset_delta).clamp(-companion_scale.walk_bound_x(), companion_scale.walk_bound_x());

                        // Best effort for platforms that allow programmatic window positioning.
                        // Wayland often ignores this; the internal WGPU offset above is the reliable path.
                        let attempted_dx = (walk.distance as f64 * eased).round() as i32 * walk.direction;
                        let next_x = (walk.start_x + attempted_dx).max(0);
                        window.set_outer_position(PhysicalPosition::new(next_x, walk.y));

                        if progress >= 1.0 {
                            runtime.sprite_offset_x = walk.target_offset_x;

                            if walk.return_to_idle {
                                reset_to_idle_system(&mut runtime);
                                runtime.next_idle_walk_at = Instant::now() + Duration::from_secs(next_idle_walk_delay_secs(&runtime));
                                runtime.walk = None;
                            } else {
                                runtime.walk = None;
                                if let Some(pose) = runtime.active_event_pose.clone() {
                                    if is_walk_pose(&pose) {
                                        let direction = if pose == "walk-left" { -1 } else { 1 };
                                        start_screen_walk(&mut runtime, &window, start_x, start_y, direction, 180, 2600, false, companion_scale);
                                    }
                                }
                            }

                            if let Ok(pos) = window.outer_position() {
                                // Save the visual sprite position, not just the transparent viewport origin.
                                send_ipc(ipc_port, &format!("position {} {}", pos.x + runtime.sprite_offset_x, pos.y));
                            }
                        }
                    }

                    let frames = runtime.mood.frames(&bank);
                    let frame_duration = Duration::from_millis(1000 / runtime.mood.fps().max(1));

                    if runtime.last_frame_at.elapsed() >= frame_duration {
                        runtime.last_frame_at = Instant::now();
                        runtime.frame_index += 1;

                        if runtime.frame_index >= frames.len() {
                            if runtime.mood.loops() {
                                runtime.frame_index = 0;
                            } else {
                                runtime.frame_index = frames.len().saturating_sub(1);
                            }
                        }
                    }

                    let draw_index = runtime.frame_index.min(frames.len().saturating_sub(1));

                    let frame = if let Some(pose) = runtime.static_pose.as_deref() {
                        pose_frame(&bank, pose)
                    } else if runtime.mood == Mood::Idle {
                        if let Some(until) = runtime.blink_until {
                            if Instant::now() < until {
                                &bank.idle_blink
                            } else {
                                runtime.blink_until = None;
                                &frames[draw_index]
                            }
                        } else {
                            &frames[draw_index]
                        }
                    } else {
                        &frames[draw_index]
                    };

                    let bubble = runtime.bubble_label.as_deref().map(bubble_frame);
                    renderer.render(frame, runtime.sprite_offset_x, bubble.as_ref(), companion_scale);
                }
                _ => {}
            }

            return;
        }

        if matches!(event, Event::AboutToWait) {
            window.request_redraw();
        }
    });
}
