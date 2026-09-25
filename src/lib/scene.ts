import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";

export class AdamScene {
  renderer: THREE.WebGLRenderer; camera: THREE.OrthographicCamera; scene = new THREE.Scene(); clock = new THREE.Clock(); root = new THREE.Group();
  mixer?: THREE.AnimationMixer; private loader = new GLTFLoader(); private actions: THREE.AnimationAction[] = []; private activeAction?: THREE.AnimationAction; private model?: THREE.Object3D;
  private bones = new Map<string, THREE.Object3D>(); private idleTime = 0; private baseRotations = new Map<string, THREE.Euler>(); private morphTargets:Array<{mesh:THREE.Mesh;index:number}>=[];
  private talking=false; private talkTime=0; private idleClipIndex=-1;
  constructor(canvas:HTMLCanvasElement){
    this.renderer=new THREE.WebGLRenderer({canvas,alpha:true,antialias:true,preserveDrawingBuffer:false});
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio||1,2.5)); this.renderer.outputColorSpace=THREE.SRGBColorSpace;
    this.camera=new THREE.OrthographicCamera(-1,1,1,-1,.1,100); this.camera.position.set(0,0,8);
    this.scene.add(new THREE.AmbientLight(0xffffff,2.2)); const key=new THREE.DirectionalLight(0xffffff,2.5); key.position.set(2,4,6); this.scene.add(key); this.scene.add(this.root); this.resize(); this.animate();
  }
  async load(url:string){
    this.root.clear(); this.mixer=undefined; this.actions=[]; this.activeAction=undefined; this.model=undefined; this.bones.clear(); this.baseRotations.clear(); this.morphTargets=[]; this.talking=false;
    try{
      const gltf=await this.loader.loadAsync(url); this.model=gltf.scene; this.root.add(gltf.scene); this.indexBones(gltf.scene); this.indexMouthMorphs(gltf.scene); this.fit(gltf.scene);
      if(gltf.animations.length){ this.mixer=new THREE.AnimationMixer(gltf.scene); this.actions=gltf.animations.map(c=>this.mixer!.clipAction(c)); this.actions.forEach(a=>{a.enabled=true;a.clampWhenFinished=false;});
        this.idleClipIndex=this.actions.findIndex(a=>/idle|breath|stand|rest/i.test(a.getClip().name)); if(this.idleClipIndex>=0)this.playAnimation(this.idleClipIndex);
      }
    }catch{this.createFallback();}
  }
  private indexBones(obj:THREE.Object3D){obj.traverse(c=>{if(!c.name)return;this.bones.set(c.name.toLowerCase(),c);if(c instanceof THREE.Bone)this.baseRotations.set(c.name.toLowerCase(),c.rotation.clone());});}
  listAnimations(){return this.actions.length;} setTalking(v:boolean){this.talking=v;}
  private indexMouthMorphs(obj:THREE.Object3D){obj.traverse(c=>{if(!(c instanceof THREE.Mesh)||!c.morphTargetDictionary||!c.morphTargetInfluences)return;for(const [name,index] of Object.entries(c.morphTargetDictionary)){if(/viseme|mouth|jaw|open|aa|ah|speech|talk/i.test(name))this.morphTargets.push({mesh:c,index});}});}
  private applyTalking(dt:number){if(!this.morphTargets.length)return;this.talkTime+=dt;const amount=this.talking?.18+Math.max(0,Math.sin(this.talkTime*17))*.5:0;for(const target of this.morphTargets){if(target.mesh.morphTargetInfluences)target.mesh.morphTargetInfluences[target.index]=THREE.MathUtils.lerp(target.mesh.morphTargetInfluences[target.index]||0,amount,Math.min(1,dt*12));}}
  playAnimation(index:number){if(!this.actions.length)return;const next=this.actions[Math.max(0,Math.min(index,this.actions.length-1))];if(this.activeAction===next)return;this.activeAction?.fadeOut(.25);next.reset().setEffectiveWeight(1).setEffectiveTimeScale(1).fadeIn(.25).play();this.activeAction=next;this.idleTime=0;}
  playAnimationByName(name:string){const i=this.actions.findIndex(a=>a.getClip().name.toLowerCase()===name.toLowerCase()||a.getClip().name.toLowerCase().includes(name.toLowerCase()));if(i>=0)this.playAnimation(i);}
  pauseAnimation(){if(this.activeAction)this.activeAction.paused=true;} resumeAnimation(){if(this.activeAction)this.activeAction.paused=false;}
  stopAnimation(){this.activeAction?.fadeOut(.2);this.activeAction=undefined;this.idleClipIndex=-1;this.idleTime=0;}
  private findBone(...names:string[]){for(const n of names){const x=this.bones.get(n.toLowerCase());if(x)return x;}return undefined;}
  private applyProceduralIdle(dt:number){if((this.activeAction&&this.idleClipIndex>=0)||!this.model)return;this.idleTime+=dt;const t=this.idleTime;const spine=this.findBone("spine","spine1","spine_1"),head=this.findBone("head"),neck=this.findBone("neck"),la=this.findBone("leftarm","left_arm","leftupperarm"),ra=this.findBone("rightarm","right_arm","rightupperarm"),ll=this.findBone("leftleg","left_leg","leftlowerleg"),rl=this.findBone("rightleg","right_leg","rightlowerleg"),lf=this.findBone("leftfoot","left_foot"),rf=this.findBone("rightfoot","right_foot");const set=(b:THREE.Object3D|undefined,a:"x"|"y"|"z",v:number)=>{if(!b)return;const base=this.baseRotations.get(b.name.toLowerCase());if(base)b.rotation[a]=base[a]+v;};const t1=Math.sin(t*1.7)*.018,s=Math.sin(t*.65)*.012;set(spine,"x",t1);set(neck,"z",s*.5);set(head,"y",Math.sin(t*.48)*.018);set(head,"x",Math.sin(t*.82+1.1)*.012);set(la,"z",Math.sin(t*1.1)*.008);set(ra,"z",-Math.sin(t*1.1)*.008);const step=Math.sin(t*1.35)*.006;set(ll,"x",step);set(rl,"x",-step);set(lf,"x",-step*.5);set(rf,"x",step*.5);}
  createFallback(){const group=new THREE.Group();const body=new THREE.Mesh(new THREE.CapsuleGeometry(.42,1,6,16),new THREE.MeshStandardMaterial({color:0x4f7cff}));const head=new THREE.Mesh(new THREE.SphereGeometry(.43,24,16),new THREE.MeshStandardMaterial({color:0xffd2b3}));head.position.y=.95;const eyeMat=new THREE.MeshBasicMaterial({color:0x172033});for(const x of [-.15,.15]){const e=new THREE.Mesh(new THREE.SphereGeometry(.045,12,8),eyeMat);e.position.set(x,.98,.4);group.add(e);}group.add(body,head);this.root.add(group);this.fit(group);}
  private fit(obj:THREE.Object3D){const box=new THREE.Box3().setFromObject(obj),center=box.getCenter(new THREE.Vector3()),size=box.getSize(new THREE.Vector3());obj.position.sub(center);const scale=1.7/Math.max(size.x,size.y,size.z,.01);obj.scale.setScalar(scale);}
  resize(){const w=this.renderer.domElement.clientWidth||360,h=this.renderer.domElement.clientHeight||520,aspect=w/h;this.camera.left=-aspect;this.camera.right=aspect;this.camera.top=1;this.camera.bottom=-1;this.camera.updateProjectionMatrix();this.renderer.setSize(Math.max(1,w),Math.max(1,h),false);}
  private animate=()=>{requestAnimationFrame(this.animate);const dt=Math.min(this.clock.getDelta(),.05);this.mixer?.update(dt);this.applyProceduralIdle(dt);this.applyTalking(dt);this.root.rotation.y=Math.sin(performance.now()*.0007)*.025;this.renderer.render(this.scene,this.camera);};
}
