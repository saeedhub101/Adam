<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import { t, type Lang } from "./lib/i18n";
  import { AdamScene } from "./lib/scene";
  import { setCharacterSize, setIgnoreCursorEvents, savePosition, loadPosition } from "./lib/desktop";
  import { addMemory, listMemories, searchMemories, updateMemory, deleteMemory, type Memory } from "./lib/memory";
  import { addReminder, listReminders, completeReminder, type Reminder } from "./lib/reminders";
  import { cloudChatStream, hasApiKey, saveApiKey, deleteApiKey, type ChatMessage } from "./lib/cloud";
  import { LocalWhisperVoice } from "./lib/voice";
  import { listPermissions, setPermission, listActivity, emergencyStop, type Permission, type Activity } from "./lib/permissions";

  let memories: Memory[] = [];
  let memoryQuery = "";
  let memoryOpen = false;
  let dragX = $state(0);
  let dragY = $state(0);
  let reminders: Reminder[] = $state([]);
  let reminderOpen = $state(false);
  let reminderTitle = $state("");
  let reminderDue = $state("");
  let chatOpen = $state(false); let chatInput = $state(""); let chatReply = $state(""); let cloudKey = $state(""); let cloudReady = $state(false); let chatBusy = $state(false);
  let provider = $state("openai"); let model = $state("gpt-4o-mini");
  let persona = $state("You are Adam, a helpful desktop AI companion. Be concise, friendly, and practical.");
  let offlineFallback = $state(true); let customBaseUrl = $state("");
  const providers = { openai: { name: "OpenAI", baseUrl: "https://api.openai.com/v1", model: "gpt-4o-mini" }, openrouter: { name: "OpenRouter", baseUrl: "https://openrouter.ai/api/v1", model: "openai/gpt-4o-mini" }, custom: { name: "Custom OpenAI-compatible", baseUrl: "", model: "" } } as const;

  let canvas: HTMLCanvasElement;
  let scene: AdamScene;
  let lang: Lang = $state("en");
  let size = $state(100);
  let showMenu = $state(false);
  let dragging = $state(false);
  let lastX = $state(0);
  let lastY = $state(0);
  let listening = $state(false);
  let transcript = $state("");
  let recognition: any = $state();
  let safetyOpen = $state(false); let permissions: Permission[] = $state([]); let activity: Activity[] = $state([]);
  let localVoice: LocalWhisperVoice | undefined = $state(); let whisperReady = $state(false); let voiceBusy = $state(false);

  async function setupLocalVoice() {
    if (!localVoice) localVoice = new LocalWhisperVoice((status) => { whisperReady = status === "ready"; voiceBusy = status === "loading"; });
  }

  async function toggleLocalVoice() {
    await setupLocalVoice();
    if (!localVoice) return;
    if (localVoice.listening) { localVoice.stop(); return; }
    await localVoice.start(lang === "ar" ? "ar" : "en", async (text) => { transcript = text; await sendChat(text); });
  }

  function setupVoice() {
    const Recognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!Recognition) return;
    recognition?.abort?.();
    recognition = new Recognition();
    recognition.continuous = false;
    recognition.interimResults = true;
    recognition.lang = lang === "ar" ? "ar-SA" : "en-US";
    recognition.onstart = () => { (window as any).speechSynthesis?.cancel(); scene?.setTalking(false); listening = true; };
    recognition.onend = () => { listening = false; };
    recognition.onerror = () => { listening = false; scene?.setTalking(false); };
    recognition.onresult = async (event: any) => {
      let text = "";
      for (let i = event.resultIndex; i < event.results.length; i++) text += event.results[i][0].transcript;
      transcript = text.trim();
      if (event.results[event.results.length - 1]?.isFinal && transcript) await sendChat(transcript);
    };
  }

  function toggleVoice() {
    if (!recognition) setupVoice();
    if (!recognition) return;
    if (listening) recognition.stop();
    else {
      (window as any).speechSynthesis?.cancel();
      scene?.setTalking(false);
      transcript = "";
      recognition.lang = lang === "ar" ? "ar-SA" : "en-US";
      try { recognition.start(); } catch {}
    }
  }

  function speak(text: string) {
    if (!text || !(window as any).speechSynthesis) return;
    (window as any).speechSynthesis.cancel();
    scene?.setTalking(true);
    const utterance = new SpeechSynthesisUtterance(text);
    utterance.lang = lang === "ar" ? "ar-SA" : "en-US";
    utterance.rate = 1;
    utterance.onend = () => scene?.setTalking(false);
    utterance.onerror = () => scene?.setTalking(false);
    (window as any).speechSynthesis.speak(utterance);
  }

  onMount(async () => {
    scene = new AdamScene(canvas);
    setupVoice();
    await scene.load("/adam.glb");
    window.addEventListener("resize", () => scene.resize());
    const position = await loadPosition(); if (position) await savePosition(position.x, position.y);
    cloudReady = await hasApiKey();
    try { const cfg = JSON.parse(localStorage.getItem("adam-cloud-config") || "{}"); provider = cfg.provider ?? provider; model = cfg.model ?? model; customBaseUrl = cfg.customBaseUrl ?? ""; persona = cfg.persona ?? persona; offlineFallback = cfg.offlineFallback ?? true; } catch {}
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

  async function saveCloudKey() { if (!cloudKey.trim()) return; await saveApiKey(cloudKey); cloudKey = ""; cloudReady = true; }
  function activeConfig() { const p = providers[provider as keyof typeof providers]; return { baseUrl: provider === "custom" ? customBaseUrl : p.baseUrl, model: model || p.model }; }
  function saveCloudConfig() { localStorage.setItem("adam-cloud-config", JSON.stringify({ provider, model, customBaseUrl, persona, offlineFallback })); chatReply = lang === "ar" ? "تم حفظ الإعدادات." : "AI settings saved."; }
  function selectProvider() { const p = providers[provider as keyof typeof providers]; if (provider !== "custom") model = p.model; }
  function offlineReply(input: string) { const q = input.toLowerCase(); if (q.includes("hello") || q.includes("hi") || q.includes("مرحبا")) return lang === "ar" ? "مرحباً، أنا آدم. أعمل حالياً في الوضع المحلي." : "Hello, I’m Adam. I’m currently working in local mode."; if (q.includes("time") || q.includes("الوقت")) return new Date().toLocaleString(); return lang === "ar" ? "لا أستطيع الوصول إلى نموذج السحابة الآن، لكنني ما زلت متاحاً للمهام المحلية والذاكرة والتذكيرات." : "I can’t reach the cloud model now, but memory and reminders are still available."; }
  async function sendChat(inputOverride?: string) {
    const input = (inputOverride ?? chatInput).trim();
    if (!input || chatBusy) return;
    if (!inputOverride) chatInput = "";
    chatBusy = true;
    try {
      const localReply = await invoke<string | null>("local_brain_execute", { input, language: lang });
      if (localReply) {
        chatReply = localReply;
        speak(localReply);
        return;
      }

      let reply = "";
      if (!cloudReady) {
        reply = offlineFallback
          ? (lang === "ar"
              ? "لم أفهم الطلب محلياً ولا يوجد نموذج سحابي متاح حالياً."
              : "I could not understand that locally and no cloud model is available right now.")
          : "API key is not configured.";
      } else {
        const cfg = activeConfig();
        if (!cfg.baseUrl || !cfg.model) throw new Error("Provider URL and model are required.");
        const memoriesForContext = await searchMemories(input, 5);
        const memoryContext = memoriesForContext.length
          ? "\nRelevant local memory:\n" + memoriesForContext.map((m) => "- " + m.content).join("\n")
          : "";
        const messages: ChatMessage[] = [
          { role: "system", content: persona + " Reply in " + (lang === "ar" ? "Arabic" : "English") + " unless the user asks otherwise." + memoryContext },
          { role: "user", content: input }
        ];
        chatReply = "";
        reply = await cloudChatStream(cfg, messages, (delta) => { chatReply += delta; });
      }
      chatReply = reply;
      speak(reply);
    } catch (e) {
      chatReply = offlineFallback ? String(e) : String(e);
      if (offlineFallback) speak(chatReply);
    } finally {
      chatBusy = false;
    }
  }

  async function openSafety() { safetyOpen = !safetyOpen; if (safetyOpen) { permissions = await listPermissions(); activity = await listActivity(); } }
  async function changePermission(capability: string, mode: string) { await setPermission(capability, mode); permissions = await listPermissions(); activity = await listActivity(); }
  async function stopAll() { await emergencyStop(); activity = await listActivity(); }

  async function resizeAdam() {
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
      <button on:click|stopPropagation={() => chatOpen = !chatOpen}>{lang === "ar" ? "محادثة الذكاء الاصطناعي" : "AI Chat"}</button>
      {#if chatOpen}<div class="chat-panel">
          <label>Provider <select bind:value={provider} on:change={selectProvider}>{#each Object.entries(providers) as [key, p]}<option value={key}>{p.name}</option>{/each}</select></label>
          {#if provider === "custom"}<input placeholder="https://your-provider/v1" bind:value={customBaseUrl} />{/if}
          <input placeholder="Model" bind:value={model} />
          <textarea rows="2" placeholder="Adam persona" bind:value={persona}></textarea>
          <label><input type="checkbox" bind:checked={offlineFallback} /> Offline fallback</label>
          <button on:click|stopPropagation={saveCloudConfig}>Save AI settings</button>{#if !cloudReady}<input type="password" placeholder="API key" bind:value={cloudKey} /><button on:click|stopPropagation={saveCloudKey}>Save key</button>{:else}<input placeholder={lang === "ar" ? "اكتب لآدم" : "Message Adam"} bind:value={chatInput} on:keydown={(e) => e.key === "Enter" && sendChat()} /><button disabled={chatBusy} on:click|stopPropagation={() => sendChat()}>{chatBusy ? "..." : "Send"}</button><button on:click|stopPropagation={async () => { await deleteApiKey(); cloudReady = false; }}>Remove key</button>{/if}{#if chatReply}<div class="chat-reply">{chatReply}</div>{/if}</div>{/if}
      <button class:active={listening} on:click|stopPropagation={toggleVoice}>{listening ? "● " : "🎙 "} {listening ? (lang === "ar" ? "استماع..." : "Listening...") : (lang === "ar" ? "الميكروفون" : "Microphone")}</button>
      {#if transcript}<div class="transcript">{transcript}</div>{/if}
      <button class:active={voiceBusy} on:click|stopPropagation={toggleLocalVoice}>{voiceBusy ? "Loading Whisper…" : whisperReady ? "Local Whisper" : "Load Local Whisper"}</button>
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
          <div class="memory-item"><span>{memory.content}</span><button on:click|stopPropagation={async () => { const value = window.prompt("Edit memory", memory.content); if (value !== null && value.trim()) { await updateMemory(memory.id, value.trim(), memory.kind); memories = memoryQuery.trim() ? await searchMemories(memoryQuery) : await listMemories(); } }}>Edit</button><button on:click|stopPropagation={async () => { await deleteMemory(memory.id); memories = memoryQuery.trim() ? await searchMemories(memoryQuery) : await listMemories(); }}>Delete</button></div>
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
<style>.stage{position:relative;width:100vw;height:100vh;overflow:visible;user-select:none}.stage canvas{display:block;width:100%;height:100%}.bubble{position:absolute;left:50%;bottom:8px;transform:translateX(-50%);padding:4px 10px;border-radius:12px;background:rgba(20,25,35,.72);color:white;font:12px sans-serif;pointer-events:none}.menu{position:absolute;right:8px;top:8px;width:260px;max-height:90vh;overflow:auto;padding:10px;border-radius:14px;background:rgba(20,24,32,.94);color:white;font:13px sans-serif;display:flex;flex-direction:column;gap:7px}.menu button,.menu input,.menu select,.menu textarea{font:inherit;border-radius:8px;border:1px solid rgba(255,255,255,.18);padding:7px;box-sizing:border-box}.menu button{background:#2f3746;color:white}.menu input,.menu select,.menu textarea{width:100%;background:#151a22;color:white}.chat-panel{padding:7px;border-radius:10px;background:rgba(255,255,255,.06);display:flex;flex-direction:column;gap:6px}.safety-panel{padding:7px;border-radius:10px;background:rgba(255,255,255,.06);display:flex;flex-direction:column;gap:6px}.safety-panel label{display:flex;justify-content:space-between;gap:6px}.activity-log{max-height:140px;overflow:auto;padding:6px;background:rgba(0,0,0,.18);border-radius:8px}.chat-reply{white-space:pre-wrap;max-height:180px;overflow:auto;padding:7px;border-radius:8px;background:rgba(255,255,255,.08)}</style>
