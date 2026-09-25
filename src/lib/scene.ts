import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { FBXLoader } from "three/addons/loaders/FBXLoader.js";

export type CharacterCapabilities = {
  loaded: boolean;
  hasRig: boolean;
  hasAnimations: boolean;
  hasFacialMorphs: boolean;
  animationCount: number;
  boneMap: Record<string, boolean>;
};
type Emotion = "neutral" | "happy" | "thinking" | "confused" | "surprised" | "listening" | "speaking";
type State = "idle" | "walk" | "run" | "gesture";

const ALIASES: Record<string, string[]> = {
  head: ["head"], neck: ["neck"], spine: ["spine", "spine1", "spine2"],
  leftArm: ["leftarm", "leftupperarm"], rightArm: ["rightarm", "rightupperarm"],
  leftForeArm: ["leftforearm", "leftlowerarm"], rightForeArm: ["rightforearm", "rightlowerarm"],
  leftHand: ["lefthand"], rightHand: ["righthand"], leftUpLeg: ["leftupleg", "leftthigh"],
  rightUpLeg: ["rightupleg", "rightthigh"], leftLeg: ["leftleg", "leftlowerleg"],
  rightLeg: ["rightleg", "rightlowerleg"], leftFoot: ["leftfoot"], rightFoot: ["rightfoot"],
  leftEye: ["lefteye"], rightEye: ["righteye"]
};

export class AdamScene {
  renderer: THREE.WebGLRenderer;
  camera: THREE.OrthographicCamera;
  scene = new THREE.Scene();
  clock = new THREE.Clock();
  root = new THREE.Group();

  private urls: string[] = [];
  private quality: "auto" | "low" | "medium" | "high" = "auto";
  private contextLost = false;
  private mixer?: THREE.AnimationMixer;
  private actions: THREE.AnimationAction[] = [];
  private active?: THREE.AnimationAction;
  private model?: THREE.Object3D;
  private bones = new Map<string, THREE.Object3D>();
  private bases = new Map<THREE.Object3D, THREE.Euler>();
  private aliases = new Map<string, THREE.Object3D>();
  private morphs: Array<{ mesh: THREE.Mesh; index: number; name: string }> = [];
  private blinks: Array<{ mesh: THREE.Mesh; index: number }> = [];
  private emotion: Emotion = "neutral";
  private talking = false;
  private blinkTimer = 2.5;
  private blinkValue = 0;
  private gestures: string[] = [];
  private state: State = "idle";
  private stateIndex = new Map<string, number>();
  private idleIndex = -1;
  private locomotion = 0;
  private idleTime = 0;
  private talkTime = 0;
  private proceduralOffsets = new Map<THREE.Object3D, THREE.Euler>();
  private gestureTime = 0;
  private modelAspect = 0.62;
  private caps: CharacterCapabilities = { loaded: false, hasRig: false, hasAnimations: false, hasFacialMorphs: false, animationCount: 0, boneMap: {} };

  constructor(canvas: HTMLCanvasElement) {
    this.renderer = new THREE.WebGLRenderer({ canvas, alpha: true, antialias: true, preserveDrawingBuffer: false, powerPreference: "high-performance" });
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.setPixelRatio(Math.min(devicePixelRatio || 1, 2));
    this.renderer.setClearColor(0, 0);
    this.camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.1, 100);
    this.camera.position.z = 8;
    this.scene.add(new THREE.HemisphereLight(0xffffff, 0x445066, 2));
    const key = new THREE.DirectionalLight(0xffffff, 2.4);
    key.position.set(2, 4, 6);
    this.scene.add(key, this.root);
    this.renderer.domElement.addEventListener("webglcontextlost", (e) => { e.preventDefault(); this.contextLost = true; });
    this.renderer.domElement.addEventListener("webglcontextrestored", () => { this.contextLost = false; this.resize(); });
    this.resize();
    this.animate();
  }

  getCapabilities() { return { ...this.caps, boneMap: { ...this.caps.boneMap } }; }
  getPreferredWindowSize(scalePercent = 100) {
    const scale = THREE.MathUtils.clamp(scalePercent / 100, 0.6, 1.6);
    const height = Math.round(470 * scale);
    const width = Math.round(THREE.MathUtils.clamp(height * this.modelAspect, 190, 520));
    return { width, height };
  }
  getRenderStatus() { return { contextLost: this.contextLost, quality: this.quality, pixelRatio: this.renderer.getPixelRatio(), webgl2: this.renderer.capabilities.isWebGL2 }; }
  setQuality(q: "auto" | "low" | "medium" | "high") {
    this.quality = q;
    const d = devicePixelRatio || 1;
    this.renderer.setPixelRatio(q === "low" ? 1 : q === "medium" ? Math.min(d, 1.5) : q === "high" ? Math.min(d, 2.5) : Math.min(d, 2));
    this.resize();
  }

  private reset() {
    this.root.clear(); this.mixer?.stopAllAction(); this.mixer = undefined; this.actions = []; this.active = undefined; this.model = undefined;
    this.bones.clear(); this.aliases.clear(); this.bases.clear(); this.morphs = []; this.blinks = []; this.stateIndex.clear(); this.gestures = []; this.proceduralOffsets.clear();
    this.emotion = "neutral"; this.talking = false; this.talkTime = 0; this.blinkTimer = 2.5; this.blinkValue = 0; this.state = "idle"; this.idleIndex = -1; this.idleTime = 0; this.locomotion = 0;
    this.modelAspect = 0.62;
    this.caps = { loaded: false, hasRig: false, hasAnimations: false, hasFacialMorphs: false, animationCount: 0, boneMap: {} };
  }

  async load(url: string) {
    this.reset();
    try { const g = await new GLTFLoader().loadAsync(url); this.install(g.scene, g.animations); }
    catch { this.fallback(); }
    return this.getCapabilities();
  }

  async loadFiles(files: File[]) {
    const valid = files.filter((f) => /\.(glb|gltf|fbx|bin|png|jpe?g|webp|ktx2)$/i.test(f.name));
    const main = valid.find((f) => /\.(glb|gltf|fbx)$/i.test(f.name));
    if (!main) throw new Error("Select a GLB, GLTF, or FBX character model.");
    if (main.size > 200 * 1024 * 1024) throw new Error("Character file is larger than 200MB.");
    this.urls.forEach(URL.revokeObjectURL); this.urls = [];
    const map = new Map<string, string>();
    for (const f of valid) { const u = URL.createObjectURL(f); this.urls.push(u); map.set(f.name.toLowerCase(), u); }
    const manager = new THREE.LoadingManager();
    manager.setURLModifier((req) => { const p = decodeURIComponent(req).split(/[?#]/)[0].replace(/\\/g, "/"); const b = p.substring(p.lastIndexOf("/") + 1).toLowerCase(); return map.get(p.toLowerCase()) ?? map.get(b) ?? req; });
    const url = map.get(main.name.toLowerCase());
    if (!url) throw new Error("Unable to create a local model URL.");
    this.reset();
    if (/\.fbx$/i.test(main.name)) { const o = await new FBXLoader(manager).loadAsync(url); this.install(o, o.animations); }
    else { const g = await new GLTFLoader(manager).loadAsync(url); this.install(g.scene, g.animations); }
    return this.getCapabilities();
  }

  private install(object: THREE.Object3D, clips: THREE.AnimationClip[]) {
    this.model = object; this.root.add(object); this.indexBones(object); this.indexMorphs(object); this.fit(object);
    this.caps = { loaded: true, hasRig: this.bones.size > 0, hasAnimations: clips.length > 0, hasFacialMorphs: this.morphs.length > 0, animationCount: clips.length, boneMap: {} };
    for (const key of Object.keys(ALIASES)) this.caps.boneMap[key] = this.aliases.has(key);
    clips.forEach((clip, i) => { const n = clip.name.toLowerCase(); if (!this.stateIndex.has("idle") && /idle|breath|stand|rest/.test(n)) this.stateIndex.set("idle", i); if (!this.stateIndex.has("walk") && /walk|walking|locomotion/.test(n)) this.stateIndex.set("walk", i); if (!this.stateIndex.has("run") && /run|jog|sprint/.test(n)) this.stateIndex.set("run", i); if (!this.stateIndex.has("gesture") && /wave|gesture|greet|point|clap|nod|thumb/.test(n)) this.stateIndex.set("gesture", i); });
    if (clips.length) { this.mixer = new THREE.AnimationMixer(object); this.actions = clips.map((c) => this.mixer!.clipAction(c)); this.idleIndex = this.actions.findIndex((a) => /idle|breath|stand|rest/i.test(a.getClip().name)); if (this.idleIndex >= 0) this.play(this.idleIndex); }
  }

  private indexBones(obj: THREE.Object3D) {
    obj.traverse((o) => { if (!o.name) return; const n = o.name.toLowerCase().replace(/mixamorig[:_]?/g, "").replace(/[^a-z0-9]/g, ""); this.bones.set(n, o); if (o instanceof THREE.Bone) this.bases.set(o, o.rotation.clone()); });
    for (const [key, names] of Object.entries(ALIASES)) for (const name of names) { const bone = this.bones.get(name); if (bone) { this.aliases.set(key, bone); break; } }
  }

  private indexMorphs(obj: THREE.Object3D) {
    obj.traverse((o) => { const m = o as THREE.Mesh; if (!(m instanceof THREE.Mesh) || !m.morphTargetDictionary || !m.morphTargetInfluences) return; for (const [name, index] of Object.entries(m.morphTargetDictionary)) { const n = name.toLowerCase(); if (/blink|eyeclose|eyelid|closeeye/.test(n)) this.blinks.push({ mesh: m, index }); if (/viseme|mouth|jaw|open|aa|ah|speech|talk|smile|happy|joy|frown|sad|confus|surpris|think|brow/.test(n)) this.morphs.push({ mesh: m, index, name: n }); } });
  }

  setEmotion(e: Emotion) { this.emotion = e; }
  getEmotion() { return this.emotion; }
  setTalking(v: boolean) { this.talking = v; if (!v) this.talkTime = 0; if (v) this.emotion = "speaking"; else if (this.emotion === "speaking") this.emotion = "neutral"; }
  queueGesture(name: string) { if (name.trim()) this.gestures.push(name.trim().toLowerCase()); }
  getAnimationState() { return this.state; }
  getLocomotionDistance() { return this.locomotion; }
  setState(s: State) { this.gestureTime = 0; this.state = s; const i = this.stateIndex.get(s); if (i !== undefined) this.play(i); else if (s === "idle" && this.idleIndex >= 0) this.play(this.idleIndex); this.idleTime = 0; }
  listAnimations() { return this.actions.map((a) => a.getClip().name); }
  playAnimationByName(name: string) { const i = this.actions.findIndex((a) => a.getClip().name.toLowerCase().includes(name.toLowerCase())); if (i >= 0) this.play(i); }

  private play(index: number) {
    if (!this.actions.length) return;
    const next = this.actions[Math.max(0, Math.min(index, this.actions.length - 1))];
    if (!next || this.active === next) return;
    this.active?.fadeOut(0.22); next.reset().setEffectiveWeight(1).setEffectiveTimeScale(1).fadeIn(0.22);
    if (this.state === "gesture") next.setLoop(THREE.LoopOnce, 1).clampWhenFinished = true;
    next.play(); this.active = next;
  }

  lookAtScreenPoint(x: number, y: number, w: number, h: number) {
    if (w <= 0 || h <= 0) return;
    const nx = THREE.MathUtils.clamp(x / w * 2 - 1, -1, 1); const ny = THREE.MathUtils.clamp(y / h * 2 - 1, -1, 1);
    const set = (key: string, axis: "x" | "y", value: number) => { const bone = this.aliases.get(key); const base = bone ? this.bases.get(bone) : undefined; if (bone && base) bone.rotation[axis] = THREE.MathUtils.lerp(bone.rotation[axis], base[axis] + value, 0.2); };
    set("head", "y", THREE.MathUtils.degToRad(nx * 15)); set("head", "x", THREE.MathUtils.degToRad(-ny * 10));
    set("leftEye", "y", THREE.MathUtils.degToRad(nx * 15)); set("rightEye", "y", THREE.MathUtils.degToRad(nx * 15));
    set("leftEye", "x", THREE.MathUtils.degToRad(-ny * 10)); set("rightEye", "x", THREE.MathUtils.degToRad(-ny * 10));
  }

  private applyEmotion(dt: number) {
    if (this.talking) this.talkTime += dt;
    const pattern = this.emotion === "happy" ? /smile|happy|joy/ : this.emotion === "surprised" ? /surpris|wide|oh/ : this.emotion === "confused" ? /confus|frown|sad/ : this.emotion === "thinking" ? /think|brow/ : this.emotion === "speaking" ? /mouth|jaw|open|speech|talk|viseme|aa|ah|ee|oh|ou/ : /$a/;
    for (const m of this.morphs) { let target = pattern.test(m.name) ? (this.emotion === "speaking" ? 0.15 : 0.35) : 0; if (this.talking && /viseme|mouth|jaw|open|speech|talk|aa|ah|ee|oh|ou/.test(m.name)) target = 0.06 + Math.max(0, Math.sin(this.talkTime * (5.5 + (m.index % 3)))) * 0.18; if (m.mesh.morphTargetInfluences) m.mesh.morphTargetInfluences[m.index] = THREE.MathUtils.lerp(m.mesh.morphTargetInfluences[m.index] ?? 0, target, Math.min(1, dt * 12)); }
  }

  private applyBlink(dt: number) {
    if (!this.blinks.length) return;
    this.blinkTimer -= dt;
    if (this.blinkTimer <= 0) { this.blinkTimer = 0.14; this.blinkValue = 1; }
    else if (this.blinkValue > 0) { this.blinkValue = Math.max(0, this.blinkValue - dt * 10); if (this.blinkValue === 0) this.blinkTimer = 2.5 + Math.random() * 4; }
    for (const b of this.blinks) if (b.mesh.morphTargetInfluences) b.mesh.morphTargetInfluences[b.index] = THREE.MathUtils.lerp(b.mesh.morphTargetInfluences[b.index] ?? 0, this.blinkValue, Math.min(1, dt * 24));
  }

  private removeProcedural() {
    for (const [bone, offset] of this.proceduralOffsets) { bone.rotation.x -= offset.x; bone.rotation.y -= offset.y; bone.rotation.z -= offset.z; }
    this.proceduralOffsets.clear();
  }

  private applyProcedural(dt: number) {
    if (!this.model || !this.caps.hasRig) return;
    this.idleTime += dt; const t = this.idleTime;
    const set = (key: string, axis: "x" | "y" | "z", value: number) => { const bone = this.aliases.get(key); if (!bone || !value) return; const offset = this.proceduralOffsets.get(bone) ?? new THREE.Euler(0, 0, 0); offset[axis] += value; this.proceduralOffsets.set(bone, offset); bone.rotation[axis] += value; };
    set("spine", "x", Math.sin(t * 1.7) * 0.018); set("neck", "z", Math.sin(t * 0.65) * 0.006); set("head", "y", Math.sin(t * 0.48) * 0.018); set("head", "x", Math.sin(t * 0.82) * 0.012);
    const activeClip = this.active?.isRunning() ?? false;
    if (!activeClip && (this.state === "walk" || this.state === "run")) {
      const speed = this.state === "run" ? 4.2 : 2.6; const amp = this.state === "run" ? 0.55 : 0.32; const swing = Math.sin(t * speed) * amp;
      set("leftUpLeg", "x", swing); set("rightUpLeg", "x", -swing); set("leftLeg", "x", -swing * 0.65); set("rightLeg", "x", swing * 0.65); set("leftArm", "x", -swing * 0.35); set("rightArm", "x", swing * 0.35); set("leftForeArm", "x", -swing * 0.18); set("rightForeArm", "x", swing * 0.18); set("leftHand", "z", Math.sin(t * speed) * 0.05); set("rightHand", "z", -Math.sin(t * speed) * 0.05); set("leftFoot", "x", swing * 0.25); set("rightFoot", "x", -swing * 0.25); this.locomotion += Math.abs(swing) * dt * (this.state === "run" ? 1.4 : 0.8);
    }
    if (!activeClip && this.state === "gesture") { this.gestureTime += dt; const wave = Math.sin(this.gestureTime * 8) * Math.min(0.45, this.gestureTime * 2); set("rightArm", "z", -0.25); set("rightForeArm", "z", -0.2); set("rightHand", "y", wave); if (this.gestureTime >= 1.2) { this.gestureTime = 0; this.state = "idle"; } } else if (this.state !== "gesture") this.gestureTime = 0;
  }
  private fallback() {
    const group = new THREE.Group();
    const body = new THREE.Mesh(new THREE.CapsuleGeometry(0.42, 1, 6, 16), new THREE.MeshStandardMaterial({ color: 0x4f7cff }));
    const head = new THREE.Mesh(new THREE.SphereGeometry(0.43, 24, 16), new THREE.MeshStandardMaterial({ color: 0xffd2b3 }));
    head.position.y = 0.95; group.add(body, head); this.root.add(group); this.model = group; this.fit(group);
    this.caps = { loaded: true, hasRig: false, hasAnimations: false, hasFacialMorphs: false, animationCount: 0, boneMap: {} };
  }

  private fit(obj: THREE.Object3D) {
    obj.updateWorldMatrix(true, true);
    const box = new THREE.Box3().setFromObject(obj);
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());
    this.modelAspect = THREE.MathUtils.clamp(size.x / Math.max(size.y, 0.01), 0.35, 1.4);
    obj.position.sub(center);
    obj.scale.setScalar(1.72 / Math.max(size.y, 0.01));
  }
  resize() {
    const w = this.renderer.domElement.clientWidth || 300;
    const h = this.renderer.domElement.clientHeight || 470;
    const aspect = w / h;
    const halfHeight = 0.94;
    const halfWidth = Math.max(halfHeight * aspect, 0.86 * this.modelAspect);
    this.camera.left = -halfWidth;
    this.camera.right = halfWidth;
    this.camera.top = halfHeight;
    this.camera.bottom = -halfHeight;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(Math.max(1, w), Math.max(1, h), false);
  }
  dispose() { this.urls.forEach(URL.revokeObjectURL); this.urls = []; this.root.clear(); this.renderer.dispose(); }

  private animate = () => {
    requestAnimationFrame(this.animate);
    const dt = Math.min(this.clock.getDelta(), 0.05);
    this.removeProcedural(); this.mixer?.update(dt); this.applyProcedural(dt); this.applyEmotion(dt); this.applyBlink(dt);
    if (this.state === "gesture" && this.active && !this.active.isRunning()) { this.active = undefined; this.state = "idle"; if (this.idleIndex >= 0) this.play(this.idleIndex); }
    if (this.gestures.length && (!this.active || this.state === "idle")) { const gesture = this.gestures.shift(); if (gesture) { this.state = "gesture"; this.playAnimationByName(gesture); } }
    if (!this.contextLost) this.renderer.render(this.scene, this.camera);
  };
}
