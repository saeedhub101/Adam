import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";

export type CharacterCapabilities = {
  loaded: boolean;
  hasRig: boolean;
  hasAnimations: boolean;
  hasFacialMorphs: boolean;
  animationCount: number;
};

export class AdamScene {
  renderer: THREE.WebGLRenderer;
  camera: THREE.OrthographicCamera;
  scene = new THREE.Scene();
  clock = new THREE.Clock();
  root = new THREE.Group();
  private loader = new GLTFLoader();
  private mixer?: THREE.AnimationMixer;
  private actions: THREE.AnimationAction[] = [];
  private activeAction?: THREE.AnimationAction;
  private model?: THREE.Object3D;
  private bones = new Map<string, THREE.Object3D>();
  private baseRotations = new Map<string, THREE.Euler>();
  private morphTargets: Array<{ mesh: THREE.Mesh; index: number }> = [];
  private talking = false;
  private talkTime = 0;
  private idleTime = 0;
  private idleClipIndex = -1;
  private capabilities: CharacterCapabilities = {
    loaded: false, hasRig: false, hasAnimations: false, hasFacialMorphs: false, animationCount: 0
  };

  constructor(canvas: HTMLCanvasElement) {
    this.renderer = new THREE.WebGLRenderer({
      canvas, alpha: true, antialias: true, preserveDrawingBuffer: false,
      powerPreference: "high-performance"
    });
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2.5));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
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

  getCapabilities(): CharacterCapabilities { return { ...this.capabilities }; }

  async load(url: string): Promise<CharacterCapabilities> {
    this.root.clear();
    this.mixer?.stopAllAction();
    this.mixer = undefined;
    this.actions = [];
    this.activeAction = undefined;
    this.model = undefined;
    this.bones.clear();
    this.baseRotations.clear();
    this.morphTargets = [];
    this.talking = false;
    this.talkTime = 0;
    this.idleTime = 0;
    this.idleClipIndex = -1;
    this.capabilities = {
      loaded: false, hasRig: false, hasAnimations: false, hasFacialMorphs: false, animationCount: 0
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

  private indexBones(obj: THREE.Object3D) {
    obj.traverse((child) => {
      if (!child.name) return;
      this.bones.set(child.name.toLowerCase(), child);
      if (child instanceof THREE.Bone) this.baseRotations.set(child.name.toLowerCase(), child.rotation.clone());
    });
  }

  listAnimations() { return this.actions.length; }
  setTalking(value: boolean) { this.talking = value; }

  private indexMouthMorphs(obj: THREE.Object3D) {
    obj.traverse((child) => {
      const mesh = child as THREE.Mesh;
      if (!(mesh instanceof THREE.Mesh) || !mesh.morphTargetDictionary || !mesh.morphTargetInfluences) return;
      for (const [name, index] of Object.entries(mesh.morphTargetDictionary)) {
        if (/viseme|mouth|jaw|open|aa|ah|speech|talk/i.test(name)) {
          this.morphTargets.push({ mesh, index });
        }
      }
    });
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
    next.reset().setEffectiveWeight(1).setEffectiveTimeScale(1).fadeIn(0.25).play();
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
    for (const name of names) {
      const found = this.bones.get(name.toLowerCase());
      if (found) return found;
    }
    return undefined;
  }

  private applyProceduralIdle(dt: number) {
    if ((this.activeAction && this.idleClipIndex >= 0) || !this.model || !this.capabilities.hasRig) return;
    this.idleTime += dt;
    const t = this.idleTime;
    const spine = this.findBone("spine", "spine1", "spine_1");
    const head = this.findBone("head");
    const neck = this.findBone("neck");
    const leftArm = this.findBone("leftarm", "left_arm", "leftupperarm");
    const rightArm = this.findBone("rightarm", "right_arm", "rightupperarm");
    const leftLeg = this.findBone("leftleg", "left_leg", "leftlowerleg");
    const rightLeg = this.findBone("rightleg", "right_leg", "rightlowerleg");
    const leftFoot = this.findBone("leftfoot", "left_foot");
    const rightFoot = this.findBone("rightfoot", "right_foot");

    const set = (bone: THREE.Object3D | undefined, axis: "x" | "y" | "z", value: number) => {
      if (!bone) return;
      const base = this.baseRotations.get(bone.name.toLowerCase());
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
    this.capabilities = { loaded: true, hasRig: false, hasAnimations: false, hasFacialMorphs: false, animationCount: 0 };
  }

  private fit(obj: THREE.Object3D) {
    const box = new THREE.Box3().setFromObject(obj);
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());
    obj.position.sub(center);
    const scale = 1.7 / Math.max(size.x, size.y, size.z, 0.01);
    obj.scale.setScalar(scale);
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
    this.renderer.render(this.scene, this.camera);
  };
}
