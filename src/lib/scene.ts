import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";

export class AdamScene {
  renderer: THREE.WebGLRenderer;
  camera: THREE.OrthographicCamera;
  scene = new THREE.Scene();
  clock = new THREE.Clock();
  root = new THREE.Group();
  mixer?: THREE.AnimationMixer;
  private loader = new GLTFLoader();
  private actions: THREE.AnimationAction[] = [];
  private activeAction?: THREE.AnimationAction;
  private model?: THREE.Object3D;
  private bones = new Map<string, THREE.Object3D>();
  private idleTime = 0;
  private baseRotations = new Map<string, THREE.Euler>();
  private morphTargets: Array<{mesh: THREE.Mesh; index: number}> = [];
  private talking = false;
  private talkTime = 0;

  constructor(canvas: HTMLCanvasElement) {
    this.renderer = new THREE.WebGLRenderer({ canvas, alpha: true, antialias: true });
    this.renderer.setPixelRatio(Math.min(devicePixelRatio, 2));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    this.camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0.1, 100);
    this.camera.position.set(0, 0, 8);
    this.scene.add(new THREE.AmbientLight(0xffffff, 2.2));
    const key = new THREE.DirectionalLight(0xffffff, 2.5);
    key.position.set(2, 4, 6);
    this.scene.add(key);
    this.scene.add(this.root);
    this.resize();
    this.animate();
  }

  async load(url: string) {
    this.root.clear();
    this.mixer = undefined;
    this.actions = [];
    this.activeAction = undefined;
    this.model = undefined;
    this.bones.clear();
    this.baseRotations.clear();
    this.morphTargets = [];
    this.talking = false;
    try {
      const gltf = await this.loader.loadAsync(url);
      this.model = gltf.scene;
      this.root.add(gltf.scene);
      this.indexBones(gltf.scene);
      this.indexMouthMorphs(gltf.scene);
      this.fit(gltf.scene);
      if (gltf.animations.length) {
        this.mixer = new THREE.AnimationMixer(gltf.scene);
        this.actions = gltf.animations.map((clip) => {
          const action = this.mixer!.clipAction(clip);
          action.enabled = true;
          action.clampWhenFinished = false;
          return action;
        });
        this.playAnimation(0);
      }
    } catch {
      this.createFallback();
    }
  }

  private indexBones(obj: THREE.Object3D) {
    obj.traverse((child) => {
      if (!child.name) return;
      this.bones.set(child.name.toLowerCase(), child);
      if (child instanceof THREE.Bone) this.baseRotations.set(child.name.toLowerCase(), child.rotation.clone());
    });
  }

  listAnimations() { return this.actions.length; }

  setTalking(talking: boolean) { this.talking = talking; }

  private indexMouthMorphs(obj: THREE.Object3D) {
    obj.traverse((child) => {
      if (!(child instanceof THREE.Mesh) || !child.morphTargetDictionary || !child.morphTargetInfluences) return;
      for (const [name, index] of Object.entries(child.morphTargetDictionary)) {
        if (/viseme|mouth|jaw|open|aa|ah|speech|talk/i.test(name)) this.morphTargets.push({ mesh: child, index });
      }
    });
  }

  private applyTalking(dt: number) {
    if (!this.morphTargets.length) return;
    this.talkTime += dt;
    const amount = this.talking ? (0.18 + Math.max(0, Math.sin(this.talkTime * 17)) * 0.5) : 0;
    for (const target of this.morphTargets) {
      const current = target.mesh.morphTargetInfluences?.[target.index] ?? 0;
      if (target.mesh.morphTargetInfluences) target.mesh.morphTargetInfluences[target.index] = THREE.MathUtils.lerp(current, amount, Math.min(1, dt * 12));
    }
  }

  playAnimation(index: number) {
    if (!this.actions.length) return;
    const next = this.actions[Math.max(0, Math.min(index, this.actions.length - 1))];
    if (this.activeAction === next) return;
    next.reset().fadeIn(0.25).play();
    this.activeAction?.fadeOut(0.25);
    this.activeAction = next;
  }

  stopAnimation() {
    this.activeAction?.fadeOut(0.2);
    this.activeAction = undefined;
  }

  private findBone(...names: string[]) {
    for (const name of names) {
      const found = this.bones.get(name.toLowerCase());
      if (found) return found;
    }
    return undefined;
  }

  private applyProceduralIdle(dt: number) {
    if (this.activeAction || !this.model) return;
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
    const breathe = Math.sin(t * 1.7) * 0.018;
    const sway = Math.sin(t * 0.65) * 0.012;
    const setDelta = (bone: THREE.Object3D | undefined, axis: "x" | "y" | "z", value: number) => {
      if (!bone) return;
      const base = this.baseRotations.get(bone.name.toLowerCase());
      if (!base) return;
      bone.rotation[axis] = base[axis] + value;
    };
    setDelta(spine, "x", breathe);
    setDelta(neck, "z", sway * 0.5);
    setDelta(head, "y", Math.sin(t * 0.48) * 0.018);
    setDelta(head, "x", Math.sin(t * 0.82 + 1.1) * 0.012);
    setDelta(leftArm, "z", Math.sin(t * 1.1) * 0.008);
    setDelta(rightArm, "z", -Math.sin(t * 1.1) * 0.008);
    const step = Math.sin(t * 1.35) * 0.006;
    setDelta(leftLeg, "x", step);
    setDelta(rightLeg, "x", -step);
    setDelta(leftFoot, "x", -step * 0.5);
    setDelta(rightFoot, "x", step * 0.5);
  }

  createFallback() {
    const group = new THREE.Group();
    const body = new THREE.Mesh(new THREE.CapsuleGeometry(.42, 1.0, 6, 16), new THREE.MeshStandardMaterial({color: 0x4f7cff}));
    const head = new THREE.Mesh(new THREE.SphereGeometry(.43, 24, 16), new THREE.MeshStandardMaterial({color: 0xffd2b3}));
    head.position.y = .95;
    const eyeMat = new THREE.MeshBasicMaterial({color: 0x172033});
    for (const x of [-.15,.15]) { const e = new THREE.Mesh(new THREE.SphereGeometry(.045, 12, 8), eyeMat); e.position.set(x,.98,.4); group.add(e); }
    group.add(body, head); this.root.add(group); this.fit(group);
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
    const w = this.renderer.domElement.clientWidth || 360;
    const h = this.renderer.domElement.clientHeight || 520;
    const aspect = w / h;
    this.camera.left = -aspect;
    this.camera.right = aspect;
    this.camera.top = 1;
    this.camera.bottom = -1;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(w, h, false);
  }

  private animate = () => {
    requestAnimationFrame(this.animate);
    const dt = Math.min(this.clock.getDelta(), .05);
    this.mixer?.update(dt);
    this.applyProceduralIdle(dt);
    this.applyTalking(dt);
    this.root.rotation.y = Math.sin(performance.now() * .0007) * .025;
    this.renderer.render(this.scene, this.camera);
  };
}
