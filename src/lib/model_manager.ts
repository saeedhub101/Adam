export type ModelStatus="idle"|"loading"|"ready"|"error";
const MODEL_ID="Xenova/whisper-tiny";
export class LocalModelManager{
 private status:ModelStatus="idle"; private error=""; private recognizer:any; private progress=0;
 getState(){return {status:this.status,error:this.error,model:MODEL_ID,progress:this.progress};}
 async load(onStatus?:(state:ReturnType<LocalModelManager["getState"]>)=>void){
  if(this.recognizer){this.status="ready";this.progress=100;onStatus?.(this.getState());return this.recognizer;}
  this.status="loading";this.error="";this.progress=0;onStatus?.(this.getState());
  try{
   const {pipeline,env}=await import("@huggingface/transformers");
   env.allowLocalModels=true;env.allowRemoteModels=true;env.useBrowserCache=true;
   this.recognizer=await pipeline("automatic-speech-recognition",MODEL_ID,{progress_callback:(p:any)=>{ if(typeof p?.progress==="number") this.progress=Math.max(0,Math.min(100,p.progress)); onStatus?.(this.getState()); }});
   this.status="ready";onStatus?.(this.getState());return this.recognizer;
  }catch(e){this.status="error";this.error=String(e);onStatus?.(this.getState());throw e;}
 }
 get recognizerInstance(){return this.recognizer;}
}
export const localModelManager=new LocalModelManager();
