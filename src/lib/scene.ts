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

export class AdamScene {
  renderer: THREE.WebGLRenderer;
  camera: THREE.OrthographicCamera;
  scene = new THREE.Scene();
  clock = new THREE.Clock();
  root = new THREE.Group();
  private loader = new GLTFLoader();
  private objectUrls: string[] = [];
  private quality: "auto" | "low" | "medium" | "high" = "auto";
  private contextLost = false;
  private mixer?: THREE.AnimationMixer;
  private actions: THREE.AnimationAction[] = [];
  private activeAction?: THREE.AnimationAction;
  private model?: THREE.Object3D;
  private bones = new Map<string, THREE.Object3D>();
  private baseRotations = new Map<THREE.Object3D, THREE.Euler>();
  private morphTargets: Array<{ mesh: THREE.Mesh; index: number }> = [];
  private talking = false;
  private blinkTargets: Array<{ mesh: THREE.Mesh; index: number }> = [];
  private blinkTime = 2.5;
  private blinkActive = false;
  private emotion: "neutral" | "happy" | "thinking" | "confused" | "surprised" | "listening" | "speaking" = "neutral";
  private gestureQueue: string[] = [];
  private talkTime = 0;
  private idleTime = 0;
  private idleClipIndex = -1;
  private state: "idle" | "walk" | "run" | "gesture" = "idle";
  private stateIndex = new Map<string, number>();
  private boneAliases = new Map<string, THREE.Object3D>();
  private capabilities: CharacterCapabilities = {
    loaded: false, hasRig: false, hasAnimations: false, hasFacialMorphs: false, animationCount: 0, boneMap: {}
  };

  constructor(canvas: HTMLCanvasElement) {
    this.renderer = new THREE.WebGLRenderer({
      canvas, alpha: true, antialias: true, preserveDrawingBuffer: false,
      powerPreference: "high-performance"
    });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2.0));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.renderer.toneMapping = THREE.ACESFilmicToneMapping;
    this.renderer.toneMappingExposure = 1.05;
    this.renderer.domElement.addEventListener("webglcontextlost", (event) => { event.preventDefault(); this.contextLost = true; });
    this.renderer.domElement.addEventListener("webglcontextrestored", () => { this.contextLost = false; this.renderer.setClearColor(0x000000, 0); this.resize(); });
    this.renderer.setClearColor(0x000000, 0);
    this.camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.1, 100);
    this.camera.position.set(0, 0, 8);
    this.scene.add(new THREE.HemisphereLight(0xffffff, 0x445066, 2.0));
    const key = new THREE.DirectionalLight(0xffffff, 2.4);
    key.position.set(2, 4, 6);
    this.scene.add(key);
    this.scene.add(this.root);
    this.resize();
    this.animate();
  }

  getCapabilities(): CharacterCapabilities { return { ...this.capabilities, boneMap: { ...this.capabilities.boneMap } }; }
  getRenderStatus() { return { contextLost: this.contextLost, quality: this.quality, pixelRatio: this.renderer.getPixelRatio(), webgl2: !!this.renderer.capabilities?.isWebGL2 }; }
  setQuality(quality: "auto" | "low" | "medium" | "high") {
    this.quality = quality;
    const dpr = window.devicePixelRatio || 1;
    const ratio = quality === "low" ? 1 : quality === "medium" ? Math.min(dpr, 1.5) : quality === "high" ? Math.min(dpr, 2.5) : Math.min(dpr, 2);
    this.renderer.setPixelRatio(ratio);
    this.resize();
  }

  async load(url: string): Promise<CharacterCapabilities> {
    this.root.clear();
    this.mixer?.stopAllAction();
    this.mixer = undefined;
    this.actions = [];
    this.activeAction = undefined;
    this.model = undefined;
    this.bones.clear();
    this.boneAliases.clear();
    this.stateIndex.clear();
    this.baseRotations.clear();
    this.morphTargets = [];
    this.blinkTargets = [];
    this.blinkTime = 2.5;
    this.blinkActive = false;
    this.gestureQueue = [];
    this.talking = false;
    this.talkTime = 0;
    this.idleTime = 0;
    this.idleClipIndex = -1;
    this.capabilities = {
      loaded: false, hasRig: false, hasAnimations: false, hasFacialMorphs: false, animationCount: 0, boneMap: {}
    };

    try {
      const gltf = await this.loader.loadAsync(url);
      this.model = gltf.scene;
      this.root.add(gltf.scene);
      this.indexBones(gltf.scene);
      this.indexMouthMorphs(gltf.scene);
      this.fit(gltf.scene);

      this.capabilities.loaded = true;
      this.capabilities.hasRig = this.bones.size > 0;
      this.capabilities.hasFacialMorphs = this.morphTargets.length > 0;
      this.capabilities.animationCount = gltf.animations.length;
      this.capabilities.hasAnimations = gltf.animations.length > 0;
      this.stateIndex.clear();
      gltf.animations.forEach((clip, i) => {
        const n = clip.name.toLowerCase();
        if (this.stateIndex.has("idle") === false && /idle|breath|stand|rest/.test(n)) this.stateIndex.set("idle", i);
        if (this.stateIndex.has("walk") === false && /walk|walking|locomotion/.test(n)) this.stateIndex.set("walk", i);
        if (this.stateIndex.has("run") === false && /run|jog|sprint/.test(n)) this.stateIndex.set("run", i);
        if (this.stateIndex.has("gesture") === false && /wave|gesture|greet|point|clap/.test(n)) this.stateIndex.set("gesture", i);
      });
      for (const key of ["head","neck","spine","leftArm","rightArm","leftForeArm","rightForeArm","leftHand","rightHand","leftUpLeg","rightUpLeg","leftLeg","rightLeg","leftFoot","rightFoot","leftEye","rightEye"]) {
        this.capabilities.boneMap[key] = !!this.boneAliases.get(key);
      }

      if (gltf.animations.length > 0) {
        this.mixer = new THREE.AnimationMixer(gltf.scene);
        this.actions = gltf.animations.map((clip) => this.mixer!.clipAction(clip));
        this.actions.forEach((action) => {
          action.enabled = true;
          action.clampWhenFinished = false;
        });
        this.idleClipIndex = this.actions.findIndex((action) => /idle|breath|stand|rest/i.test(action.getClip().name));
        if (this.idleClipIndex >= 0) this.playAnimation(this.idleClipIndex);
      }
    } catch {
      this.createFallback();
    }
    return this.getCapabilities();
  }

  async loadFiles(files: File[]): Promise<CharacterCapabilities> {
    const valid = files.filter((f) => /\.(glb|gltf|fbx|bin|png|jpg|jpeg|webp|ktx2)$/i.test(f.name));
    const main = valid.find((f) => /\.(glb|gltf)$/i.test(f.name));
    if (!main) throw new Error("No GLB/GLTF model was selected.");
    if (main.size > 200 * 1024 * 1024) throw new Error("Character file is larger than 200MB.");
    this.objectUrls.forEach((u) => URL.revokeObjectURL(u)); this.objectUrls = [];
    const map = new Map<string, string>();
    for (const file of valid) { const url = URL.createObjectURL(file); this.objectUrls.push(url); map.set(file.name.toLowerCase(), url); }
    const manager = new THREE.LoadingManager();
    manager.setURLModifier((requested) => { const clean = decodeURIComponent(requested).split(/[?#]/)[0].replace(/\\/g, "/"); const base = clean.substring(clean.lastIndexOf("/") + 1).toLowerCase(); return map.get(clean.toLowerCase()) ?? map.get(base) ?? requested; });
    const url = map.get(main.name.toLowerCase());
    if (!url) throw new Error("Unable to create a local model URL.");
    if (/\.fbx$/i.test(main.name)) {
      const object = await new FBXLoader(manager).loadAsync(url);
      this.root.clear(); this.mixer?.stopAllAction(); this.mixer = undefined; this.actions = []; this.activeAction = undefined;
      this.model = object; this.bones.clear(); this.boneAliases.clear(); this.stateIndex.clear(); this.baseRotations.clear(); this.morphTargets = []; this.blinkTargets = [];
      this.root.add(object); this.indexBones(object); this.indexMouthMorphs(object); this.fit(object);
      this.capabilities = { loaded: true, hasRig: this.bones.size > 0, hasAnimations: object.animations.length > 0, hasFacialMorphs: this.morphTargets.length > 0, animationCount: object.animations.length, boneMap: {} };
      object.animations.forEach((clip, i) => { const n = clip.name.toLowerCase(); if (!this.stateIndex.has("idle") && /idle|breath|stand|rest/.test(n)) this.stateIndex.set("idle", i); if (!this.stateIndex.has("walk") && /walk|walking|locomotion/.test(n)) this.stateIndex.set("walk", i); if (!this.stateIndex.has("run") && /run|jog|sprint/.test(n)) this.stateIndex.set("run", i); if (!this.stateIndex.has("gesture") && /wave|gesture|greet|point|clap/.test(n)) this.stateIndex.set("gesture", i); });
      for (const key of ["head","neck","spine","leftArm","rightArm","leftForeArm","rightForeArm","leftHand","rightHand","leftUpLeg","rightUpLeg","leftLeg","rightLeg","leftFoot","rightFoot","leftEye","rightEye"]) this.capabilities.boneMap[key] = !!this.boneAliases.get(key);
      if (object.animations.length) { this.mixer = new THREE.AnimationMixer(object); this.actions = object.animations.map((clip) => this.mixer!.clipAction(clip)); this.idleClipIndex = this.actions.findIndex((a) => /idle|breath|stand|rest/i.test(a.getClip().name)); if (this.idleClipIndex >= 0) this.playAnimation(this.idleClipIndex); }
      return this.getCapabilities();
    }
    const loader = new GLTFLoader(manager);
    const gltf = await loader.loadAsync(url);
    this.root.clear(); this.mixer?.stopAllAction(); this.mixer = undefined; this.actions = []; this.activeAction = undefined;
    this.talking = false; this.talkTime = 0; this.idleTime = 0; this.idleClipIndex = -1; this.state = "idle";
    this.model = gltf.scene; this.bones.clear(); this.boneAliases.clear(); this.stateIndex.clear(); this.baseRotations.clear(); this.morphTargets = [];
    this.root.add(gltf.scene); this.indexBones(gltf.scene); this.indexMouthMorphs(gltf.scene); this.fit(gltf.scene);
    this.capabilities = { loaded: true, hasRig: this.bones.size > 0, hasAnimations: gltf.animations.length > 0, hasFacialMorphs: this.morphTargets.length > 0, animationCount: gltf.animations.length, boneMap: {} };
    gltf.animations.forEach((clip, i) => { const n = clip.name.toLowerCase(); if (!this.stateIndex.has("idle") && /idle|breath|stand|rest/.test(n)) this.stateIndex.set("idle", i); if (!this.stateIndex.has("walk") && /walk|walking|locomotion/.test(n)) this.stateIndex.set("walk", i); if (!this.stateIndex.has("run") && /run|jog|sprint/.test(n)) this.stateIndex.set("run", i); if (!this.stateIndex.has("gesture") && /wave|gesture|greet|point|clap/.test(n)) this.stateIndex.set("gesture", i); });
    for (const key of ["head","neck","spine","leftArm","rightArm","leftForeArm","rightForeArm","leftHand","rightHand","leftUpLeg","rightUpLeg","leftLeg","rightLeg","leftFoot","rightFoot","leftEye","rightEye"]) this.capabilities.boneMap[key] = !!this.boneAliases.get(key);
    if (gltf.animations.length) { this.mixer = new THREE.AnimationMixer(gltf.scene); this.actions = gltf.animations.map((clip) => this.mixer!.clipAction(clip)); this.actions.forEach((action) => { action.enabled = true; action.clampWhenFinished = false; }); this.idleClipIndex = this.actions.findIndex((a) => /idle|breath|stand|rest/i.test(a.getClip().name)); if (this.idleClipIndex >= 0) this.playAnimation(this.idleClipIndex); }
    return this.getCapabilities();
  }
  private indexBones(obj: THREE.Object3D) {
    obj.traverse((child) => {
      if (!child.name) return;
      const normalized = child.name.toLowerCase().replace(/mixamorig[:_]?/g, "").replace(/[^a-z0-9]/g, "");
      this.bones.set(normalized, child);
      if (child instanceof THREE.Bone) this.baseRotations.set(child, child.rotation.clone());
    });
    const aliases: Record<string,string[]> = {
      head:["head"], neck:["neck"], spine:["spine","spine1","spine2"],
      leftArm:["leftarm","leftupperarm"], rightArm:["rightarm","rightupperarm"],
      leftForeArm:["leftforearm","leftlowerarm"], rightForeArm:["rightforearm","rightlowerarm"],
      leftHand:["lefthand"], rightHand:["righthand"], leftUpLeg:["leftupleg","leftthigh"], rightUpLeg:["rightupleg","rightthigh"],
      leftLeg:["leftleg","leftlowerleg"], rightLeg:["rightleg","rightlowerleg"], leftFoot:["leftfoot"], rightFoot:["rightfoot"],
      leftEye:["lefteye"], rightEye:["righteye"]
    };
    for (const [key,names] of Object.entries(aliases)) for (const n of names) { const b=this.bones.get(n); if (b) { this.boneAliases.set(key,b); break; } }
  }

  listAnimations() { return this.actions.map((a) => a.getClip().name); }
  getAnimationState() { return this.state; }
  setState(state: "idle"|"walk"|"run"|"gesture") {
    this.state = state;
    const i = this.stateIndex.get(state);
    if (i !== undefined) this.playAnimation(i);
    else {
      this.activeAction?.fadeOut(0.2);
      this.activeAction = undefined;
      if (state === "idle" && this.idleClipIndex >= 0) this.playAnimation(this.idleClipIndex);
      this.idleTime = 0;
    }
  }
  setTalking(value: boolean) { this.talking = value; this.emotion = value ? "speaking" : "neutral"; }
  setEmotion(emotion: "neutral" | "happy" | "thinking" | "confused" | "surprised" | "listening" | "speaking") { this.emotion = emotion; }
  getEmotion() { return this.emotion; }
  queueGesture(name: string) { if (name.trim()) this.gestureQueue.push(name.trim()); }

  lookAtScreenPoint(x: number, y: number, width: number, height: number) {
    const head = this.findBone("head");
    if (!head || width <= 0 || height <= 0) return;
    const nx = THREE.MathUtils.clamp((x / width) * 2 - 1, -1, 1);
    const ny = THREE.MathUtils.clamp((y / height) * 2 - 1, -1, 1);
    const maxYaw = THREE.MathUtils.degToRad(15);
    const maxPitch = THREE.MathUtils.degToRad(10);
    const base = this.baseRotations.get(head);
    if (!base) return;
    head.rotation.y = THREE.MathUtils.lerp(head.rotation.y, base.y + nx * maxYaw, 0.18);
    head.rotation.x = THREE.MathUtils.lerp(head.rotation.x, base.x - ny * maxPitch, 0.18);
    for (const key of ["leftEye", "rightEye"]) {
      const eye = this.findBone(key);
      const eyeBase = eye ? this.baseRotations.get(eye) : undefined;
      if (eye && eyeBase) {
        eye.rotation.y = THREE.MathUtils.lerp(eye.rotation.y, eyeBase.y + nx * THREE.MathUtils.degToRad(8), 0.25);
        eye.rotation.x = THREE.MathUtils.lerp(eye.rotation.x, eyeBase.x - ny * THREE.MathUtils.degToRad(6), 0.25);
      }
    }
  }

  private indexMouthMorphs(obj: THREE.Object3D) {
    obj.traverse((child) => {
      const mesh = child as THREE.Mesh;
      if (!(mesh instanceof THREE.Mesh) || !mesh.morphTargetDictionary || !mesh.morphTargetInfluences) return;
      for (const [name, index] of Object.entries(mesh.morphTargetDictionary)) {
        if (/blink|eyeclose|eyelid|closeeye/i.test(name)) this.blinkTargets.push({ mesh, index });
        if (/viseme|mouth|jaw|open|aa|ah|speech|talk/i.test(name)) {
          this.morphTargets.push({ mesh, index });
        }
      }
    });
  }

  private applyBlink(dt: number) {
    if (!this.blinkTargets.length) return;
    this.blinkTime -= dt;
    if (this.blinkTime <= 0) { this.blinkActive = true; this.blinkTime = 0.12; }
    else if (this.blinkActive) { this.blinkActive = false; this.blinkTime = 2.5 + Math.random() * 4; }
    const amount = this.blinkActive ? 1 : 0;
    for (const target of this.blinkTargets) if (target.mesh.morphTargetInfluences) target.mesh.morphTargetInfluences[target.index] = THREE.MathUtils.lerp(target.mesh.morphTargetInfluences[target.index] || 0, amount, Math.min(1, dt * 20));
  }

  private applyTalking(dt: number) {
    if (!this.morphTargets.length) return;
    this.talkTime += dt;
    const amount = this.talking ? 0.18 + Math.max(0, Math.sin(this.talkTime * 17)) * 0.5 : 0;
    for (const target of this.morphTargets) {
      if (target.mesh.morphTargetInfluences) {
        target.mesh.morphTargetInfluences[target.index] = THREE.MathUtils.lerp(
          target.mesh.morphTargetInfluences[target.index] || 0, amount, Math.min(1, dt * 12)
        );
      }
    }
  }

  playAnimation(index: number) {
    if (!this.actions.length) return;
    const next = this.actions[Math.max(0, Math.min(index, this.actions.length - 1))];
    if (this.activeAction === next) return;
    this.activeAction?.fadeOut(0.25);
    next.reset().setEffectiveWeight(1).setEffectiveTimeScale(1).fadeIn(0.25);
    if (this.state === "gesture") next.setLoop(THREE.LoopOnce, 1).clampWhenFinished = true;
    next.play();
    this.activeAction = next;
    this.idleTime = 0;
  }

  playAnimationByName(name: string) {
    const needle = name.toLowerCase();
    const index = this.actions.findIndex((a) => a.getClip().name.toLowerCase().includes(needle));
    if (index >= 0) this.playAnimation(index);
  }

  pauseAnimation() { if (this.activeAction) this.activeAction.paused = true; }
  resumeAnimation() { if (this.activeAction) this.activeAction.paused = false; }
  stopAnimation() { this.activeAction?.fadeOut(0.2); this.activeAction = undefined; this.idleClipIndex = -1; }

  private findBone(...names: string[]) {
    for (const name of names) { const found = this.boneAliases.get(name) ?? this.bones.get(name.toLowerCase().replace(/[^a-z0-9]/g, "")); if (found) return found; }
    return undefined;
  }

  private applyProceduralIdle(dt: number) {
    if ((this.activeAction && this.stateIndex.has(this.state)) || !this.model || !this.capabilities.hasRig) return;
    this.idleTime += dt;
    const t = this.idleTime;
    const spine = this.findBone("spine");
    const head = this.findBone("head");
    const neck = this.findBone("neck");
    const leftArm = this.findBone("leftArm");
    const rightArm = this.findBone("rightArm");
    const leftLeg = this.findBone("leftLeg");
    const rightLeg = this.findBone("rightLeg");
    const leftFoot = this.findBone("leftFoot");
    const rightFoot = this.findBone("rightFoot");

    const set = (bone: THREE.Object3D | undefined, axis: "x" | "y" | "z", value: number) => {
      if (!bone) return;
      const base = this.baseRotations.get(bone);
      if (base) bone.rotation[axis] = base[axis] + value;
    };

    set(spine, "x", Math.sin(t * 1.7) * 0.018);
    set(neck, "z", Math.sin(t * 0.65) * 0.006);
    set(head, "y", Math.sin(t * 0.48) * 0.018);
    set(head, "x", Math.sin(t * 0.82 + 1.1) * 0.012);
    set(leftArm, "z", Math.sin(t * 1.1) * 0.008);
    set(rightArm, "z", -Math.sin(t * 1.1) * 0.008);
    const step = Math.sin(t * 1.35) * 0.006;
    set(leftLeg, "x", step);
    set(rightLeg, "x", -step);
    set(leftFoot, "x", -step * 0.5);
    set(rightFoot, "x", step * 0.5);
    if (this.state === "walk" || this.state === "run") {
      const speed = this.state === "run" ? 2.8 : 1.8;
      const stride = Math.sin(t * speed) * (this.state === "run" ? 0.28 : 0.16);
      set(leftLeg, "x", stride); set(rightLeg, "x", -stride);
      set(leftArm, "x", -stride * 0.35); set(rightArm, "x", stride * 0.35);
    }
  }

  private createFallback() {
    const group = new THREE.Group();
    const body = new THREE.Mesh(
      new THREE.CapsuleGeometry(0.42, 1, 6, 16),
      new THREE.MeshStandardMaterial({ color: 0x4f7cff })
    );
    const head = new THREE.Mesh(
      new THREE.SphereGeometry(0.43, 24, 16),
      new THREE.MeshStandardMaterial({ color: 0xffd2b3 })
    );
    head.position.y = 0.95;
    const eyeMat = new THREE.MeshBasicMaterial({ color: 0x172033 });
    for (const x of [-0.15, 0.15]) {
      const eye = new THREE.Mesh(new THREE.SphereGeometry(0.045, 12, 8), eyeMat);
      eye.position.set(x, 0.98, 0.4);
      group.add(eye);
    }
    group.add(body, head);
    this.root.add(group);
    this.model = group;
    this.fit(group);
    this.capabilities = { loaded: true, hasRig: false, hasAnimations: false, hasFacialMorphs: false, animationCount: 0, boneMap: {} };
  }

  private fit(obj: THREE.Object3D) {
    obj.updateWorldMatrix(true, true);
    const box = new THREE.Box3().setFromObject(obj);
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());
    obj.position.sub(center);
    const scale = 1.7 / Math.max(size.x, size.y, size.z, 0.01);
    obj.scale.setScalar(scale);
  }

  dispose() {
    this.objectUrls.forEach((u) => URL.revokeObjectURL(u));
    this.objectUrls = [];
    this.root.clear();
    this.renderer.dispose();
  }

  resize() {
    const width = this.renderer.domElement.clientWidth || 360;
    const height = this.renderer.domElement.clientHeight || 520;
    const aspect = width / height;
    this.camera.left = -aspect;
    this.camera.right = aspect;
    this.camera.top = 1;
    this.camera.bottom = -1;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(Math.max(1, width), Math.max(1, height), false);
  }

  private animate = () => {
    requestAnimationFrame(this.animate);
    const dt = Math.min(this.clock.getDelta(), 0.05);
    this.mixer?.update(dt);
    this.applyProceduralIdle(dt);
    this.applyTalking(dt);
    this.applyBlink(dt);
    if (this.state === "gesture" && this.activeAction && !this.activeAction.isRunning()) {
      this.activeAction = undefined;
      this.state = "idle";
      if (this.idleClipIndex >= 0) this.playAnimation(this.idleClipIndex);
    }
    if (this.gestureQueue.length && (!this.activeAction || this.state === "idle")) {
      const nextGesture = this.gestureQueue.shift();
      if (nextGesture) { this.state = "gesture"; this.playAnimationByName(nextGesture); }
    }
    if (!this.contextLost) this.renderer.render(this.scene, this.camera);
  };
}
