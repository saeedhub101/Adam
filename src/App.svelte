<script lang="ts">
  import { onMount } from "svelte";
  import { t, type Lang } from "./lib/i18n";
  import { AdamScene } from "./lib/scene";
  import { setCharacterSize, setIgnoreCursorEvents, savePosition, loadPosition } from "./lib/desktop";

  let canvas: HTMLCanvasElement;
  let scene: AdamScene;
  let lang: Lang = "en";
  let size = 100;
  let showMenu = false;
  let dragging = false;
  let lastX = 0;
  let lastY = 0;

  onMount(async () => {
    scene = new AdamScene(canvas);
    await scene.load("/adam.glb");
    window.addEventListener("resize", () => scene.resize());
    await loadPosition();
    await setIgnoreCursorEvents(false);
  });

  async function chooseCharacter() {
    const input = document.createElement("input");
    input.type = "file"; input.accept = ".glb,.gltf";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      const url = URL.createObjectURL(file);
      await scene.load(url);
      setTimeout(() => URL.revokeObjectURL(url), 60000);
    };
    input.click();
  }

  async function resizeAdam() {
    await setCharacterSize(size);
    scene?.resize();
  }

  function startDrag(e: PointerEvent) {
    dragging = true; lastX = e.clientX; lastY = e.clientY;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  async function drag(e: PointerEvent) {
    if (!dragging) return;
    const dx = e.clientX-lastX, dy=e.clientY-lastY;
    lastX=e.clientX; lastY=e.clientY;
    const pos = await loadPosition();
    if (pos) await savePosition(pos.x+dx, pos.y+dy);
  }
  function stopDrag() { dragging=false; }

  async function toggleThrough() {
    await setIgnoreCursorEvents(!showMenu);
    showMenu = !showMenu;
  }
</script>

<svelte:window on:keydown={(e) => e.key === "Escape" && (showMenu = false)} />

<div class="stage" on:pointerdown={startDrag} on:pointermove={drag} on:pointerup={stopDrag}>
  <canvas bind:this={canvas}></canvas>
  <div class="bubble">{t(lang, "idle")}</div>
  {#if showMenu}
    <div class="menu">
      <button on:click|stopPropagation={chooseCharacter}>{t(lang,"changeCharacter")}</button>
      <label>{t(lang,"size")} {size}% <input type="range" min="60" max="160" bind:value={size} on:input={resizeAdam}/></label>
      <button on:click|stopPropagation={() => lang = lang === "en" ? "ar" : "en"}>{lang === "en" ? "العربية" : "English"}</button>
      <button on:click|stopPropagation={toggleThrough}>✓</button>
    </div>
  {/if}
</div>