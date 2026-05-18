import { getCurrentWindow } from '@tauri-apps/api/window';

export function ResizeHandle() {
  async function startResize() {
    try {
      await getCurrentWindow().startResizeDragging('SouthEast');
    } catch {
      // Some compositors may not support programmatic resize dragging.
      // Chat is still marked resizable in tauri.conf.json.
    }
  }

  return (
    <button
      className="lm-resize-handle"
      onMouseDown={startResize}
      title="Resize chat"
      aria-label="Resize chat"
    />
  );
}
