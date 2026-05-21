pub const CHAT_WIDTH: i32 = 514;
pub const CHAT_HEIGHT: i32 = 700;
pub const CHAT_HEAD_GAP: i32 = 32;
pub const COMPANION_WIDTH: i32 = 340;
pub const COMPANION_HEIGHT: i32 = 230;

pub fn companion_top_anchor(companion_x: i32, companion_y: i32) -> (i32, i32) {
    (companion_x + (COMPANION_WIDTH / 2), companion_y)
}

pub fn chat_position_above_companion(companion_center_x: i32, companion_top_y: i32) -> (i32, i32) {
    (companion_center_x - (CHAT_WIDTH / 2), companion_top_y - CHAT_HEIGHT - CHAT_HEAD_GAP)
}
