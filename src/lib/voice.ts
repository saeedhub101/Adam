type Status = "idle" | "loading" | "ready";
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
  private handler?: ResultHandler;
  listening = false;
  private statusCb: (status: Status) => void;

  constructor(statusCb: (status: Status) => void) {
    this.statusCb = statusCb;
  }

  async load(): Promise<void> {
    if (this.recognizer) return;
    this.statusCb("loading");
    const { pipeline, env } = await import("@huggingface/transformers");
    env.allowLocalModels = false;
    env.useBrowserCache = true;
    this.recognizer = await pipeline("automatic-speech-recognition", "Xenova/whisper-tiny");
    this.statusCb("ready");
  }

  async start(language: "ar" | "en", handler: ResultHandler): Promise<void> {
    await this.load();
    this.handler = handler;
    this.stream = await navigator.mediaDevices.getUserMedia({ audio: { channelCount: 1, echoCancellation: true, noiseSuppression: true } });
    this.audio = new AudioContext({ sampleRate: 16000 });
    this.source = this.audio.createMediaStreamSource(this.stream);
    this.processor = this.audio.createScriptProcessor(4096, 1, 1);
    this.chunks = [];
    this.speaking = false;
    this.startedAt = performance.now();
    this.listening = true;
    this.processor.onaudioprocess = (event) => {
      const input = event.inputBuffer.getChannelData(0);
      const copy = new Float32Array(input.length);
      copy.set(input);
      this.chunks.push(copy);
      let energy = 0;
      for (let i = 0; i < input.length; i++) energy += input[i] * input[i];
      const rms = Math.sqrt(energy / input.length);
      if (rms > 0.018) this.speaking = true;
      if (this.speaking && rms < 0.012 && performance.now() - this.startedAt > 700) void this.finish(language);
    };
    this.source.connect(this.processor);
    this.processor.connect(this.audio.destination);
  }

  stop(): void {
    this.listening = false;
    this.processor?.disconnect();
    this.source?.disconnect();
    this.stream?.getTracks().forEach((t) => t.stop());
    void this.audio?.close();
    this.processor = undefined;
    this.source = undefined;
    this.stream = undefined;
    this.audio = undefined;
    this.chunks = [];
    this.speaking = false;
  }

  private async finish(language: "ar" | "en") {
    if (!this.listening) return;
    this.listening = false;
    const chunks = this.chunks;
    this.stop();
    const length = chunks.reduce((n, c) => n + c.length, 0);
    const pcm = new Float32Array(length);
    let offset = 0;
    for (const chunk of chunks) { pcm.set(chunk, offset); offset += chunk.length; }
    if (pcm.length < 16000 * 0.25) return;
    const result = await this.recognizer(pcm, { language, task: "transcribe", return_timestamps: false });
    const text = typeof result?.text === "string" ? result.text.trim() : "";
    if (text && this.handler) await this.handler(text);
  }
}
