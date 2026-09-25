import { invoke } from "@tauri-apps/api/core";

export type WindowState = { x: number; y: number; size: number };

export async function setIgnoreCursorEvents(ignore: boolean) {
  await invoke("set_ignore_cursor_events", { ignore });
}
export async function setCharacterSize(size: number) {
  await invoke("set_character_size", { size: Math.round(size) });
}
export async function setCharacterDimensions(width: number, height: number, size: number) {
  await invoke("set_character_dimensions", { width: Math.round(width), height: Math.round(height), size: Math.round(size) });
}
export async function savePosition(x: number, y: number) {
  await invoke("save_position", { x: Math.round(x), y: Math.round(y) });
}
export async function loadPosition(): Promise<WindowState | null> {
  return await invoke("load_position");
}

export type CaptureResult = { width: number; height: number; png_base64: string };
export type ComputerWindow = [number, string];

export async function captureDesktop(): Promise<CaptureResult> {
  return await invoke("screen_capture");
}

export async function analyzeScreen(baseUrl: string, model: string, prompt: string, pngBase64: string): Promise<{ text: string }> {
  return await invoke("vision_analyze", { baseUrl, model, prompt, pngBase64 });
}

export async function listComputerWindows(): Promise<ComputerWindow[]> {
  return await invoke("computer_windows");
}

export async function computerClick(x: number, y: number, double = false) {
  await invoke("computer_click", { x: Math.round(x), y: Math.round(y), double });
}

export async function computerType(value: string) {
  await invoke("computer_type", { text: value });
}

export async function computerKey(virtualKey: number) {
  await invoke("computer_key", { virtualKey });
}
