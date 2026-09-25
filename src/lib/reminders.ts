import { invoke } from "@tauri-apps/api/core";

export type Reminder = { id: number; title: string; dueAt: string; completed: boolean };

export async function addReminder(title: string, dueAt: string): Promise<number> {
  return await invoke<number>("reminder_add", { title, dueAt });
}

export async function listReminders(): Promise<Reminder[]> {
  const rows = await invoke<[number,string,string,boolean][]>("reminder_list");
  return rows.map(([id, title, dueAt, completed]) => ({ id, title, dueAt, completed }));
}

export async function completeReminder(id: number): Promise<void> {
  await invoke("reminder_complete", { id });
}
