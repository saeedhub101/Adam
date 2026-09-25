import { localModelManager } from "./model_manager";
type Status = "idle" | "loading" | "ready" | "error";
type ResultHandler = (text: string) => Promise<void> | void;

export class LocalWhisperVoice {
  private recognizer: any;
  private stream?: MediaStream;
  private audio?: AudioContext;
  private source?: MediaStreamAudioSourceNode;
  private processor?: ScriptProcessorNode;
  private chunks: Float32Array[] = [];
  private speaking = false;
  private startedAt = 0;
  private lastVoiceAt = 0;
  private handler?: ResultHandler;
  listening = false;
  status: Status = "idle";
  error = "";
  private statusCb: (status: Status) => void;

  constructor(statusCb: (status: Status) => void) { this.statusCb = statusCb; }

  async load(): Promise<void> {
    if (this.recognizer) return;
    this.status = "loading"; this.error = ""; this.statusCb("loading");
    try {
      this.recognizer = await localModelManager.load((state) => { this.status = state.status; this.error = state.error; this.statusCb(state.status); });
    } catch (e) {
      this.status = "error"; this.error = String(e); this.statusCb("error");
      throw e;
    }
  }

  async start(language: "ar" | "en", handler: ResultHandler): Promise<void> {
    await this.load();
    this.handler = handler;
    (window as any).speechSynthesis?.cancel();
    try {
      this.stream = await navigator.mediaDevices.getUserMedia({ audio: { channelCount: 1, echoCancellation: true, noiseSuppression: true, autoGainControl: true } });
    } catch (e) {
      this.status = "error";
      this.error = e instanceof DOMException ? `${e.name}: ${e.message || "Microphone permission denied"}` : String(e);
      this.statusCb("error");
      throw e;
    }
    this.audio = new AudioContext({ sampleRate: 16000 });
    this.source = this.audio.createMediaStreamSource(this.stream);
    this.processor = this.audio.createScriptProcessor(4096, 1, 1);
    this.chunks = []; this.speaking = false;
    this.startedAt = performance.now(); this.lastVoiceAt = this.startedAt; this.listening = true;
    this.processor.onaudioprocess = (event) => {
      if (!this.listening) return;
      const input = event.inputBuffer.getChannelData(0);
      const copy = new Float32Array(input.length); copy.set(input); this.chunks.push(copy);
      let energy = 0; for (let i = 0; i < input.length; i++) energy += input[i] * input[i];
      const rms = Math.sqrt(energy / input.length);
      const now = performance.now();
      if (rms > 0.018) { this.speaking = true; this.lastVoiceAt = now; }
      if (this.speaking && rms < 0.012 && now - this.lastVoiceAt > 550 && now - this.startedAt > 700) void this.finish(language);
    };
    this.source.connect(this.processor);
    this.processor.connect(this.audio.destination);
  }

  stop(): void {
    this.listening = false;
    this.processor?.disconnect(); this.source?.disconnect();
    this.stream?.getTracks().forEach((t) => t.stop());
    void this.audio?.close();
    this.processor = undefined; this.source = undefined; this.stream = undefined; this.audio = undefined;
    this.chunks = []; this.speaking = false;
  }

  private async finish(language: "ar" | "en") {
    if (!this.listening) return;
    const chunks = this.chunks; this.stop();
    const length = chunks.reduce((n, c) => n + c.length, 0);
    const pcm = new Float32Array(length);
    let offset = 0; for (const chunk of chunks) { pcm.set(chunk, offset); offset += chunk.length; }
    if (pcm.length < 16000 * 0.25) return;
    try {
      const result = await this.recognizer(pcm, { language, task: "transcribe", return_timestamps: false });
      const text = typeof result?.text === "string" ? result.text.trim() : "";
      if (text && this.handler) await this.handler(text);
    } catch (e) {
      this.status = "error"; this.error = String(e); this.statusCb("error");
    }
  }
}
