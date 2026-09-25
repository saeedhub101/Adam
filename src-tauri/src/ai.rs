use keyring::Entry;
use reqwest::{Client, StatusCode};
use std::time::Duration;
use serde::{Deserialize, Serialize};
use futures_util::StreamExt;

const SERVICE: &str = "Adam";
const USER: &str = "cloud_api_key";

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatRequest { pub base_url: String, pub model: String, pub messages: Vec<Message> }
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Message { pub role: String, pub content: String }
#[derive(Debug, Deserialize)]
struct ChatResponse { choices: Vec<Choice> }
#[derive(Debug, Deserialize)]
struct Choice { message: Message }

fn entry() -> Result<Entry, String> { Entry::new(SERVICE, USER).map_err(|e| e.to_string()) }
pub fn save_key(key: &str) -> Result<(), String> { if key.trim().is_empty(){return Err("API key cannot be empty".into())} entry()?.set_password(key.trim()).map_err(|e|e.to_string()) }
pub fn has_key() -> Result<bool, String> { match entry()?.get_password(){Ok(_)=>Ok(true),Err(keyring::Error::NoEntry)=>Ok(false),Err(e)=>Err(e.to_string())} }
pub fn delete_key() -> Result<(), String> { match entry()?.delete_credential(){Ok(())|Err(keyring::Error::NoEntry)=>Ok(()),Err(e)=>Err(e.to_string())} }
pub async fn chat(req: ChatRequest)->Result<String,String>{
 let key=entry()?.get_password().map_err(|e|e.to_string())?;
 if req.base_url.trim().is_empty() || req.model.trim().is_empty() { return Err("Provider URL and model are required".into()); }
 let url=format!("{}/chat/completions",req.base_url.trim_end_matches('/'));
 let body=serde_json::json!({"model":req.model,"messages":req.messages,"stream":false});
 let client=Client::builder().connect_timeout(Duration::from_secs(10)).timeout(Duration::from_secs(90)).build().map_err(|e|e.to_string())?;
 let response=client.post(url).bearer_auth(key).json(&body).send().await.map_err(|e|format!("Provider connection failed: {}",e))?;
 let status=response.status();
 if !status.is_success(){
   let body=response.text().await.unwrap_or_default();
   let detail=body.chars().take(800).collect::<String>();
   let label=match status { StatusCode::UNAUTHORIZED => "Authentication failed", StatusCode::TOO_MANY_REQUESTS => "Rate limit reached", s if s.is_server_error() => "Provider server error", _ => "Provider request failed" };
   return Err(format!("{} (HTTP {}): {}",label,status.as_u16(),detail));
 }
 let data:ChatResponse=response.json().await.map_err(|e|e.to_string())?;
 data.choices.first().map(|c|c.message.content.clone()).ok_or_else(||"Provider returned no response".into())
}

pub async fn chat_stream(app: &tauri::AppHandle, request_id: &str, req: ChatRequest)->Result<String,String>{
 let key=entry()?.get_password().map_err(|e|e.to_string())?;
 if req.base_url.trim().is_empty() || req.model.trim().is_empty() { return Err("Provider URL and model are required".into()); }
 let url=format!("{}/chat/completions",req.base_url.trim_end_matches('/'));
 let body=serde_json::json!({"model":req.model,"messages":req.messages,"stream":true});
 let client=Client::builder().connect_timeout(Duration::from_secs(10)).timeout(Duration::from_secs(120)).build().map_err(|e|e.to_string())?;
 let response=client.post(url).bearer_auth(key).json(&body).send().await.map_err(|e|format!("Provider connection failed: {}",e))?;
 let status=response.status();
 if !status.is_success(){ let body=response.text().await.unwrap_or_default(); return Err(format!("Provider request failed (HTTP {}): {}",status.as_u16(),body.chars().take(800).collect::<String>())); }
 let mut stream=response.bytes_stream();
 let mut buffer=String::new();
 let mut answer=String::new();
 while let Some(item)=stream.next().await {
   let chunk=item.map_err(|e|e.to_string())?;
   buffer.push_str(&String::from_utf8_lossy(&chunk));
   while let Some(pos)=buffer.find("\n") {
     let line=buffer[..pos].trim_end_matches('\r').to_string();
     buffer.drain(..=pos);
     if let Some(data)=line.strip_prefix("data: ") {
       if data.trim()=="[DONE]" { continue; }
       if let Ok(v)=serde_json::from_str::<serde_json::Value>(data) {
         if let Some(delta)=v["choices"][0]["delta"]["content"].as_str() {
           answer.push_str(delta);
           let _=tauri::Emitter::emit(app, &format!("adam://cloud-chunk/{}",request_id), delta);
         }
       }
     }
   }
 }
 let _=tauri::Emitter::emit(app, &format!("adam://cloud-done/{}",request_id), &answer);
 Ok(answer)
}
