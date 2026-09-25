<script lang="ts">
  import { onMount } from "svelte";
  import { t, type Lang } from "./lib/i18n";
  import { AdamScene } from "./lib/scene";
  import { setCharacterSize, setIgnoreCursorEvents, savePosition, loadPosition } from "./lib/desktop";
  import { addMemory, listMemories, searchMemories, type Memory } from "./lib/memory";

  let memories: Memory[] = [];
  let memoryQuery = "";

  let canvas: HTMLCanvasElement;
  let scene: AdamScene;
  let lang: Lang = "en";
  let size = 100;
  let showMenu = false;
  let dragging = false;
  let lastX = 0;
  let lastY = 0;
  let listening = false;
  let transcript = "";
  let recognition: any;

  function setupVoice() {
    const Recognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!Recognition) return;
    recognition = new Recognition();
    recognition.continuous = false;
    recognition.interimResults = true;
    recognition.lang = lang === "ar" ? "ar-SA" : "en-US";
    recognition.onstart = () => listening = true;
    recognition.onend = () => listening = false;
    recognition.onerror = () => listening = false;
    recognition.onresult = (event: any) => {
      let text = "";
      for (let i = event.resultIndex; i < event.results.length; i++) text += event.results[i][0].transcript;
      transcript = text.trim();
      if (event.results[event.results.length - 1]?.isFinal && transcript) speak(transcript);
    };
  }

  function toggleVoice() {
    if (!recognition) setupVoice();
    if (!recognition) return;
    if (listening) recognition.stop(); else { recognition.lang = lang === "ar" ? "ar-SA" : "en-US"; recognition.start(); }
  }

  function speak(text: string) {
    if (!text || !(window as any).speechSynthesis) return;
    (window as any).speechSynthesis.cancel();
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.lang = lang === "ar" ? "ar-SA" : "en-US";
    utterance.rate = 1;
    (window as any).speechSynthesis.speak(utterance);
  }

  onMount(async () => {
    scene = new AdamScene(canvas);
    setupVoice();
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
      <button on:click|stopPropagation={() => { lang = lang === "en" ? "ar" : "en"; setupVoice(); }}>{lang === "en" ? "العربية" : "English"}</button>
      <button class:active={listening} on:click|stopPropagation={toggleVoice}>{listening ? "● " : "🎙 "} {listening ? (lang === "ar" ? "استماع..." : "Listening...") : (lang === "ar" ? "الميكروفون" : "Microphone")}</button>
      {#if transcript}<div class="transcript">{transcript}</div>{/if}
      <div class="memory-panel">
        <input placeholder={lang === "ar" ? "ابحث في الذاكرة" : "Search memory"} bind:value={memoryQuery} />
        <button on:click|stopPropagation={async () => { memories = memoryQuery.trim() ? await searchMemories(memoryQuery) : await listMemories(); }}>
          {lang === "ar" ? "ذاكرة" : "Memory"}
        </button>
        {#each memories.slice(0, 5) as memory}
          <div class="memory-item">{memory.content}</div>
        {/each}
        {#if transcript}
          <button on:click|stopPropagation={async () => { await addMemory(transcript, "voice"); memories = await listMemories(); }}>
            {lang === "ar" ? "حفظ الكلام" : "Save transcript"}
          </button>
        {/if}
      </div>
      <button on:click|stopPropagation={toggleThrough}>✓</button>
    </div>
  {/if}
</div>