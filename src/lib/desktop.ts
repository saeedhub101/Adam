import { invoke } from "@tauri-apps/api/core";

export type WindowState = { x: number; y: number; size: number };

export async function setIgnoreCursorEvents(ignore: boolean) {
  await invoke("set_ignore_cursor_events", { ignore });
}
export async function setCharacterSize(size: number) {
  await invoke("set_character_size", { size: Math.round(size) });
}
export async function savePosition(x: number, y: number) {
  await invoke("save_position", { x: Math.round(x), y: Math.round(y) });
}
export async function loadPosition(): Promise<WindowState | null> {
  return await invoke("load_position");
}
