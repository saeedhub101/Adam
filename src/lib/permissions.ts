import { invoke } from "@tauri-apps/api/core";
export type Permission={capability:string;mode:string};
export type Activity={id:number;action:string;detail:string;createdAt:string};
export const permissionModes=["ask","session","always","deny"] as const;
export async function listPermissions():Promise<Permission[]>{return await invoke("permission_list");}
export async function setPermission(capability:string,mode:string){await invoke("permission_set",{capability,mode});}
export async function listActivity(limit=100):Promise<Activity[]>{const rows=await invoke<[number,string,string,string][]>("activity_log",{limit});return rows.map(([id,action,detail,createdAt])=>({id,action,detail,createdAt}));}
export async function listExcludedApps():Promise<string[]>{return await invoke("excluded_apps");}
export async function addExcludedApp(name:string){await invoke("excluded_app_add",{name});}
export async function removeExcludedApp(name:string){await invoke("excluded_app_remove",{name});}
export async function emergencyStop(){await invoke("emergency_stop");}
