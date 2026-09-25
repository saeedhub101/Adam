import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
export type ChatMessage={role:"system"|"user"|"assistant";content:string};
export type CloudConfig={baseUrl:string;model:string};
export async function saveApiKey(key:string){await invoke("cloud_save_key",{key});}
export async function hasApiKey():Promise<boolean>{return await invoke<boolean>("cloud_has_key");}
export async function deleteApiKey(){await invoke("cloud_delete_key");}
export async function cloudChat(config:CloudConfig,messages:ChatMessage[]):Promise<string>{return await invoke<string>("cloud_chat",{baseUrl:config.baseUrl,model:config.model,messages});}


export async function cloudChatStream(config:CloudConfig,messages:ChatMessage[],onDelta:(delta:string)=>void):Promise<string>{
 if(!config.baseUrl.trim() || !config.model.trim()) throw new Error("Provider URL and model are required.");
 const id=crypto.randomUUID();
 const unlisten=await listen<string>(`adam://cloud-chunk/${id}`,(event)=>onDelta(event.payload));
 try { return await invoke<string>("cloud_chat_stream",{requestId:id,baseUrl:config.baseUrl,model:config.model,messages}); }
 finally { await unlisten(); }
}
