import { invoke } from "@tauri-apps/api/core";

export type Memory = { id: number; content: string; kind: string; createdAt: string };

export async function addMemory(content: string, kind = "note"): Promise<number> {
  return await invoke<number>("memory_add", { content, kind });
}

export async function listMemories(limit = 50): Promise<Memory[]> {
  const rows = await invoke<[number,string,string,string][]>("memory_list", { limit });
  return rows.map(([id, content, kind, createdAt]) => ({ id, content, kind, createdAt }));
}

export async function searchMemories(query: string, limit = 20): Promise<Memory[]> {
  const rows = await invoke<[number,string,string,string][]>("memory_search", { query, limit });
  return rows.map(([id, content, kind, createdAt]) => ({ id, content, kind, createdAt }));
}
