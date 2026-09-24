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
    try {
      const gltf = await this.loader.loadAsync(url);
      this.root.add(gltf.scene);
      this.fit(gltf.scene);
      if (gltf.animations.length) {
        this.mixer = new THREE.AnimationMixer(gltf.scene);
        this.mixer.clipAction(gltf.animations[0]).play();
      }
    } catch {
      this.createFallback();
    }
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
    this.root.rotation.y = Math.sin(performance.now() * .0007) * .025;
    this.renderer.render(this.scene, this.camera);
  };
}