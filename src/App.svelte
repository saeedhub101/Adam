<script lang="ts">
  import { onMount } from "svelte";
  import { t, type Lang } from "./lib/i18n";
  import { AdamScene } from "./lib/scene";
  import { setCharacterSize, setIgnoreCursorEvents, savePosition, loadPosition } from "./lib/desktop";
  import { addMemory, listMemories, searchMemories, type Memory } from "./lib/memory";
  import { addReminder, listReminders, completeReminder, type Reminder } from "./lib/reminders";
  import { cloudChat, hasApiKey, saveApiKey, deleteApiKey, type ChatMessage } from "./lib/cloud";

  let memories: Memory[] = [];
  let memoryQuery = "";
  let memoryOpen = false;
  let dragX = 0;
  let dragY = 0;
  let reminders: Reminder[] = [];
  let reminderOpen = false;
  let reminderTitle = "";
  let reminderDue = "";\n  let chatOpen = false; let chatInput = ""; let chatReply = ""; let cloudKey = ""; let cloudReady = false; let chatBusy = false;\n  const cloudConfig = { baseUrl: "https://api.openai.com/v1", model: "gpt-4o-mini" };

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
    const position = await loadPosition(); if (position) await savePosition(position.x, position.y);\n    cloudReady = await hasApiKey();
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

  async function refreshReminders() { reminders = await listReminders(); }

  async function createReminder() {
    if (!reminderTitle.trim() || !reminderDue) return;
    await addReminder(reminderTitle.trim(), new Date(reminderDue).toISOString());
    reminderTitle = ""; reminderDue = "";
    await refreshReminders();
  }

  async function saveCloudKey() { if (!cloudKey.trim()) return; await saveApiKey(cloudKey); cloudKey = ""; cloudReady = true; }\n  async function sendChat() { if (!chatInput.trim() || chatBusy || !cloudReady) return; chatBusy = true; const input = chatInput.trim(); chatInput = ""; try { const messages: ChatMessage[] = [{ role: "system", content: "You are Adam, a helpful desktop AI companion. Reply in " + (lang === "ar" ? "Arabic" : "English") + " unless the user asks otherwise." }, { role: "user", content: input }]; chatReply = await cloudChat(cloudConfig, messages); } catch (e) { chatReply = String(e); } finally { chatBusy = false; } }\n\n  async function resizeAdam() {
    await setCharacterSize(size);
    scene?.resize();
  }

  async function startDrag(e: PointerEvent) {
    if (showMenu) return;
    dragging = true; lastX = e.clientX; lastY = e.clientY;
    const pos = await loadPosition();
    dragX = pos?.x ?? 0; dragY = pos?.y ?? 0;
    (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
  }
  function drag(e: PointerEvent) {
    if (!dragging) return;
    const dx = e.clientX-lastX, dy=e.clientY-lastY;
    lastX=e.clientX; lastY=e.clientY;
    dragX += dx; dragY += dy;
    void savePosition(dragX, dragY);
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
      <button on:click|stopPropagation={() => chatOpen = !chatOpen}>{lang === "ar" ? "محادثة الذكاء الاصطناعي" : "AI Chat"}</button>\n      {#if chatOpen}<div class="chat-panel">{#if !cloudReady}<input type="password" placeholder="API key" bind:value={cloudKey} /><button on:click|stopPropagation={saveCloudKey}>Save key</button>{:else}<input placeholder={lang === "ar" ? "اكتب لآدم" : "Message Adam"} bind:value={chatInput} on:keydown={(e) => e.key === "Enter" && sendChat()} /><button disabled={chatBusy} on:click|stopPropagation={sendChat}>{chatBusy ? "..." : "Send"}</button><button on:click|stopPropagation={async () => { await deleteApiKey(); cloudReady = false; }}>Remove key</button>{/if}{#if chatReply}<div class="chat-reply">{chatReply}</div>{/if}</div>{/if}\n      <button class:active={listening} on:click|stopPropagation={toggleVoice}>{listening ? "● " : "🎙 "} {listening ? (lang === "ar" ? "استماع..." : "Listening...") : (lang === "ar" ? "الميكروفون" : "Microphone")}</button>
      {#if transcript}<div class="transcript">{transcript}</div>{/if}
      <div class="memory-panel">
        <button on:click|stopPropagation={async () => { reminderOpen = !reminderOpen; if (reminderOpen) await refreshReminders(); }}>
          {lang === "ar" ? "تذكيرات" : "Reminders"}
        </button>
        {#if reminderOpen}
          <div class="reminder-panel">
            <input placeholder={lang === "ar" ? "عنوان التذكير" : "Reminder title"} bind:value={reminderTitle} />
            <input type="datetime-local" bind:value={reminderDue} />
            <button on:click|stopPropagation={createReminder}>{lang === "ar" ? "إضافة" : "Add"}</button>
            {#each reminders.slice(0, 8) as reminder}
              <div class:completed={reminder.completed} class="reminder-item">
                <span>{reminder.title}</span>
                <small>{new Date(reminder.dueAt).toLocaleString()}</small>
                {#if !reminder.completed}<button on:click|stopPropagation={async () => { await completeReminder(reminder.id); await refreshReminders(); }}>✓</button>{/if}
              </div>
            {/each}
          </div>
        {/if}

        <input placeholder={lang === "ar" ? "ابحث في الذاكرة" : "Search memory"} bind:value={memoryQuery} />
<button on:click|stopPropagation={async () => { memoryOpen = !memoryOpen; if (memoryOpen) memories = memoryQuery.trim() ? await searchMemories(memoryQuery) : await listMemories(); }}>
          {lang === "ar" ? "ذاكرة" : "Memory"}
        </button>
        {#if memoryOpen}
        {#each memories.slice(0, 5) as memory}
          <div class="memory-item">{memory.content}</div>
        {/each}
        {/if}
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