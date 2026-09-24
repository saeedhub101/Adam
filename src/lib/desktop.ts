import { invoke } from "@tauri-apps/api/core";
export async function setIgnoreCursorEvents(ignore: boolean) {
  await invoke("set_ignore_cursor_events", { ignore });
}
export async function setCharacterSize(size: number) {
  await invoke("set_character_size", { size });
}
export async function savePosition(x: number, y: number) {
  await invoke("save_position", { x, y });
}
export async function loadPosition(): Promise<{x:number;y:number}|null> {
  return await invoke("load_position");
}