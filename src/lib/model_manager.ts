export type ModelStatus="idle"|"loading"|"ready"|"error";
const MODEL_ID="Xenova/whisper-tiny";
export class LocalModelManager{
 private status:ModelStatus="idle"; private error=""; private recognizer:any;
 getState(){return {status:this.status,error:this.error,model:MODEL_ID};}
 async load(onStatus?:(state:ReturnType<LocalModelManager["getState"]>)=>void){
  if(this.recognizer){this.status="ready";onStatus?.(this.getState());return this.recognizer;}
  this.status="loading";this.error="";onStatus?.(this.getState());
  try{
   const {pipeline,env}=await import("@huggingface/transformers");
   env.allowLocalModels=true;env.allowRemoteModels=true;env.useBrowserCache=true;
   this.recognizer=await pipeline("automatic-speech-recognition",MODEL_ID);
   this.status="ready";onStatus?.(this.getState());return this.recognizer;
  }catch(e){this.status="error";this.error=String(e);onStatus?.(this.getState());throw e;}
 }
 get recognizerInstance(){return this.recognizer;}
}
export const localModelManager=new LocalModelManager();
