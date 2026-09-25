import { invoke } from "@tauri-apps/api/core";
export type ChatMessage={role:"system"|"user"|"assistant";content:string};
export type CloudConfig={baseUrl:string;model:string};
export async function saveApiKey(key:string){await invoke("cloud_save_key",{key});}
export async function hasApiKey():Promise<boolean>{return await invoke<boolean>("cloud_has_key");}
export async function deleteApiKey(){await invoke("cloud_delete_key");}
export async function cloudChat(config:CloudConfig,messages:ChatMessage[]):Promise<string>{return await invoke<string>("cloud_chat",{baseUrl:config.baseUrl,model:config.model,messages});}
