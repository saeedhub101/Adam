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

  let memories: Memory[] = $state([]);
  let memoryQuery = $state("");
  let memoryOpen = $state(false);
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
  let clickThrough = $state(false);
  let showMenu = $state(false);
  let dragging = $state(false);
  let lastX = $state(0);
  let lastY = $state(0);
  let listening = $state(false);
  let transcript = $state("");
  let recognition: any = $state();
  let safetyOpen = $state(false); let permissions: Permission[] = $state([]); let activity: Activity[] = $state([]);
  let localVoice: LocalWhisperVoice | undefined = $state(); let whisperReady = $state(false); let voiceBusy = $state(false);
  let capabilities = $state<any>({});
  let characterError = $state("");
  let dragOver = $state(false);
  let animationNames = $state<string[]>([]);
  let animationState = $state<"idle"|"walk"|"run"|"gesture">("idle");
  let loadingCharacter = $state(false);
  let loadingProgress = $state(0);
  let renderQuality = $state<"auto"|"low"|"medium"|"high">("auto");
  let renderStatus = $state<any>({});
  let savedCharacters = $state<string[]>([]);
  let lastCharacterName = $state("Adam default");

  const CHARACTER_DB = "adam-character-library";
  const CHARACTER_STORE = "packages";

  function characterDb(): Promise<IDBDatabase> {
    return new Promise((resolve, reject) => {
      const req = indexedDB.open(CHARACTER_DB, 1);
      req.onupgradeneeded = () => req.result.createObjectStore(CHARACTER_STORE);
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
  }

  async function saveCharacterPackage(name: string, files: File[]) {
    const db = await characterDb();
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(CHARACTER_STORE, "readwrite");
      tx.objectStore(CHARACTER_STORE).put(files.map((file) => ({ name: file.name, type: file.type, blob: file })), name);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
    db.close();
  }

  async function loadCharacterPackage(name: string): Promise<File[]> {
    const db = await characterDb();
    const value = await new Promise<any>((resolve, reject) => {
      const req = db.transaction(CHARACTER_STORE, "readonly").objectStore(CHARACTER_STORE).get(name);
      req.onsuccess = () => resolve(req.result);
      req.onerror = () => reject(req.error);
    });
    db.close();
    return Array.isArray(value) ? value.map((item) => new File([item.blob], item.name, { type: item.type || "application/octet-stream" })) : [];
  }

  async function deleteCharacterPackage(name: string) {
    const db = await characterDb();
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(CHARACTER_STORE, "readwrite");
      tx.objectStore(CHARACTER_STORE).delete(name);
      tx.oncomplete = () => resolve();
      tx.onerror = () => reject(tx.error);
    });
    db.close();
  }

  async function restoreLastCharacter() {
    if (!savedCharacters.length) return;
    try {
      const files = await loadCharacterPackage(savedCharacters[0]);
      if (!files.length) return;
      const caps = await scene.loadFiles(files);
      capabilities = caps;
      animationNames = scene.listAnimations();
      lastCharacterName = savedCharacters[0];
    } catch {
      savedCharacters = savedCharacters.slice(1);
      localStorage.setItem("adam-character-history", JSON.stringify(savedCharacters));
    }
  }

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
    const caps = await scene.load("/adam.glb");
    capabilities = caps;
    animationNames = scene.listAnimations();
    renderQuality = (localStorage.getItem("adam-render-quality") as "auto"|"low"|"medium"|"high") || "auto";
    scene.setQuality(renderQuality); renderStatus = scene.getRenderStatus();
    try { savedCharacters = JSON.parse(localStorage.getItem("adam-character-history") || "[]"); } catch { savedCharacters = []; }
    window.addEventListener("resize", () => scene.resize());
    const position = await loadPosition();
    if (position) {
      size = position.size ?? 100;
      dragX = position.x;
      dragY = position.y;
    }
    await restoreLastCharacter();
    cloudReady = await hasApiKey();
    try { const cfg = JSON.parse(localStorage.getItem("adam-cloud-config") || "{}"); provider = cfg.provider ?? provider; model = cfg.model ?? model; customBaseUrl = cfg.customBaseUrl ?? ""; persona = cfg.persona ?? persona; offlineFallback = cfg.offlineFallback ?? true; } catch {}
    await setIgnoreCursorEvents(false);
  });

  async function loadCharacterFiles(files: FileList | File[]) {
    const selected = Array.from(files);
    const main = selected.find((f) => /\.(glb|gltf)$/i.test(f.name));
    if (!main) { characterError = lang === "ar" ? "اختر ملف GLB أو GLTF." : "Select a GLB or GLTF model."; return; }
    characterError = ""; loadingCharacter = true; loadingProgress = 10;
    try {
      const caps = await scene.loadFiles(selected);
      loadingProgress = 100; capabilities = caps; animationNames = scene.listAnimations(); animationState = "idle";
      lastCharacterName = main.name.replace(/\.(glb|gltf|fbx)$/i, "");
      await saveCharacterPackage(lastCharacterName, selected);
      savedCharacters = [lastCharacterName, ...savedCharacters.filter((n) => n !== lastCharacterName)].slice(0, 8);
      localStorage.setItem("adam-character-history", JSON.stringify(savedCharacters));
      renderStatus = scene.getRenderStatus();
    } catch (e) { characterError = e instanceof Error ? e.message : String(e); loadingProgress = 0; }
    finally { loadingCharacter = false; }
  }

  async function loadCharacterFile(file: File) { await loadCharacterFiles([file]); }

  async function chooseCharacter() {
    const input = document.createElement("input");
    input.type = "file"; input.accept = ".glb,.gltf,.fbx,.bin,.png,.jpg,.jpeg,.webp"; input.multiple = true;
    input.onchange = async () => { if (input.files?.length) await loadCharacterFiles(input.files); };
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
      let agentResult = await invoke<any | null>("agent_route", { input, language: lang });
      if (agentResult) {
        if (agentResult.requires_confirmation) {
          const approved = window.confirm(agentResult.message + (lang === "ar" ? "\n\nالسماح بهذه العملية لهذه الجلسة؟" : "\n\nAllow this action for this session?"));
          if (approved) {
            await setPermission(agentResult.intent === "computer.open" ? "computer.open" : agentResult.intent, "session");
            agentResult = await invoke<any | null>("agent_route", { input, language: lang });
          }
        }
        if (agentResult) {
          chatReply = agentResult.message;
          speak(agentResult.message);
          if (agentResult.action === "opened" || agentResult.action === "rejected" || agentResult.action === "blocked") return;
        }
      }

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
    if (showMenu || clickThrough) return;
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

  function trackPointer(e: PointerEvent) {
    if (!scene || showMenu || dragging) return;
    scene.lookAtScreenPoint(e.clientX, e.clientY, window.innerWidth, window.innerHeight);
  }

  async function openMenu() {
    scene?.lookAtScreenPoint(window.innerWidth / 2, window.innerHeight / 2, window.innerWidth, window.innerHeight);
    clickThrough = false;
    await setIgnoreCursorEvents(false);
    showMenu = true;
  }

  async function closeMenu() {
    showMenu = false;
    clickThrough = false;
    await setIgnoreCursorEvents(false);
  }

  async function toggleThrough() {
    clickThrough = !clickThrough;
    showMenu = false;
    await setIgnoreCursorEvents(clickThrough);
  }
</script>

<svelte:window onkeydown={(e) => { if (e.key === "Escape") void closeMenu(); }} />

<div class="stage" class:drag-over={dragOver} ondragover={(e) => { e.preventDefault(); dragOver = true; }} ondragleave={() => dragOver = false} ondrop={(e) => { e.preventDefault(); dragOver = false; const files = e.dataTransfer?.files; if (files?.length) void loadCharacterFiles(files); }} ondblclick={() => void openMenu()} oncontextmenu={(e) => { e.preventDefault(); void openMenu(); }} onpointerdown={startDrag} onpointermove={(e) => { drag(e); trackPointer(e); }} onpointerup={stopDrag}>
  <canvas bind:this={canvas}></canvas>
  <div class="bubble">{t(lang, "idle")}</div>
  {#if showMenu}
    <div class="menu" onpointerdown={(e) => e.stopPropagation()} onpointermove={(e) => e.stopPropagation()}>
      <button onclick={chooseCharacter}>{t(lang,"changeCharacter")}</button><button onclick={() => void restoreLastCharacter()}>{lang === "ar" ? "استعادة الشخصية المحفوظة" : "Restore saved character"}</button><button onclick={async () => { await deleteCharacterPackage(lastCharacterName); savedCharacters = savedCharacters.filter((n) => n !== lastCharacterName); localStorage.setItem("adam-character-history", JSON.stringify(savedCharacters)); }}>{lang === "ar" ? "حذف الشخصية المحفوظة" : "Delete saved character"}</button><small>Current: {lastCharacterName}</small>
      {#if characterError}<div class="error">{characterError}</div>{/if}
      <div class="loading" class:hidden={!loadingCharacter}>Loading character… {loadingProgress}%</div><div class="capabilities">{capabilities.loaded ? `Rig: ${capabilities.hasRig ? "yes" : "no"} · Animations: ${capabilities.animationCount ?? 0} · Face: ${capabilities.hasFacialMorphs ? "yes" : "no"} · Bones: ${Object.values(capabilities.boneMap ?? {}).filter(Boolean).length}/17` : "Loading character…"}</div><div class="quality-row"><label>Quality <select bind:value={renderQuality} onchange={() => { scene?.setQuality(renderQuality); localStorage.setItem("adam-render-quality", renderQuality); renderStatus = scene?.getRenderStatus(); }}><option value="auto">Auto</option><option value="low">Low</option><option value="medium">Medium</option><option value="high">High</option></select></label><small>{renderStatus.contextLost ? "WebGL context lost" : "DPR " + (renderStatus.pixelRatio ?? "—")}</small></div><div class="bone-capabilities">{#each Object.entries(capabilities.boneMap ?? {}) as [bone, present]}<span class:missing={!present}>{present ? "✓" : "✗"} {bone}</span>{/each}</div>{#if savedCharacters.length}<small class="history">Recent: {savedCharacters.join(" · ")}</small>{/if}
      <div class="animation-panel">
        <select bind:value={animationState} onchange={() => scene?.setState(animationState)}><option value="idle">Idle</option><option value="walk">Walk</option><option value="run">Run</option><option value="gesture">Gesture</option></select>
        {#if animationNames.length}<small>{animationNames.join(" · ")}</small>{/if}
      </div>
      <label>{t(lang,"size")} {size}% <input type="range" min="60" max="160" bind:value={size} oninput={resizeAdam}/></label>
      <button onclick={() => { lang = lang === "en" ? "ar" : "en"; setupVoice(); }}>{lang === "en" ? "العربية" : "English"}</button>
      <button onclick={() => chatOpen = !chatOpen}>{lang === "ar" ? "محادثة الذكاء الاصطناعي" : "AI Chat"}</button>
      {#if chatOpen}<div class="chat-panel">
          <label>Provider <select bind:value={provider} onchange={selectProvider}>{#each Object.entries(providers) as [key, p]}<option value={key}>{p.name}</option>{/each}</select></label>
          {#if provider === "custom"}<input placeholder="https://your-provider/v1" bind:value={customBaseUrl} />{/if}
          <input placeholder="Model" bind:value={model} />
          <textarea rows="2" placeholder="Adam persona" bind:value={persona}></textarea>
          <label><input type="checkbox" bind:checked={offlineFallback} /> Offline fallback</label>
          <button onclick={saveCloudConfig}>Save AI settings</button>{#if !cloudReady}<input type="password" placeholder="API key" bind:value={cloudKey} /><button onclick={saveCloudKey}>Save key</button>{:else}<input placeholder={lang === "ar" ? "اكتب لآدم" : "Message Adam"} bind:value={chatInput} onkeydown={(e) => e.key === "Enter" && sendChat()} /><button disabled={chatBusy} onclick={() => sendChat()}>{chatBusy ? "..." : "Send"}</button><button onclick={async () => { await deleteApiKey(); cloudReady = false; }}>Remove key</button>{/if}{#if chatReply}<div class="chat-reply">{chatReply}</div>{/if}</div>{/if}
      <button class:active={listening} onclick={toggleVoice}>{listening ? "● " : "🎙 "} {listening ? (lang === "ar" ? "استماع..." : "Listening...") : (lang === "ar" ? "الميكروفون" : "Microphone")}</button>
      {#if transcript}<div class="transcript">{transcript}</div>{/if}
      <button class:active={voiceBusy} onclick={toggleLocalVoice}>{voiceBusy ? "Loading Whisper…" : whisperReady ? "Local Whisper" : "Load Local Whisper"}</button>
      <div class="memory-panel">
        <button onclick={async () => { reminderOpen = !reminderOpen; if (reminderOpen) await refreshReminders(); }}>
          {lang === "ar" ? "تذكيرات" : "Reminders"}
        </button>
        {#if reminderOpen}
          <div class="reminder-panel">
            <input placeholder={lang === "ar" ? "عنوان التذكير" : "Reminder title"} bind:value={reminderTitle} />
            <input type="datetime-local" bind:value={reminderDue} />
            <button onclick={createReminder}>{lang === "ar" ? "إضافة" : "Add"}</button>
            {#each reminders.slice(0, 8) as reminder}
              <div class:completed={reminder.completed} class="reminder-item">
                <span>{reminder.title}</span>
                <small>{new Date(reminder.dueAt).toLocaleString()}</small>
                {#if !reminder.completed}<button onclick={async () => { await completeReminder(reminder.id); await refreshReminders(); }}>✓</button>{/if}
              </div>
            {/each}
          </div>
        {/if}

        <input placeholder={lang === "ar" ? "ابحث في الذاكرة" : "Search memory"} bind:value={memoryQuery} />
<button onclick={async () => { memoryOpen = !memoryOpen; if (memoryOpen) memories = memoryQuery.trim() ? await searchMemories(memoryQuery) : await listMemories(); }}>
          {lang === "ar" ? "ذاكرة" : "Memory"}
        </button>
        {#if memoryOpen}
        {#each memories.slice(0, 5) as memory}
          <div class="memory-item"><span>{memory.content}</span><button onclick={async () => { const value = window.prompt("Edit memory", memory.content); if (value !== null && value.trim()) { await updateMemory(memory.id, value.trim(), memory.kind); memories = memoryQuery.trim() ? await searchMemories(memoryQuery) : await listMemories(); } }}>Edit</button><button onclick={async () => { await deleteMemory(memory.id); memories = memoryQuery.trim() ? await searchMemories(memoryQuery) : await listMemories(); }}>Delete</button></div>
        {/each}
        {/if}
        {#if transcript}
          <button onclick={async () => { await addMemory(transcript, "voice"); memories = await listMemories(); }}>
            {lang === "ar" ? "حفظ الكلام" : "Save transcript"}
          </button>
        {/if}
      </div>
      <button onclick={toggleThrough}>{clickThrough ? "Disable click-through" : "Enable click-through"}</button>
      <button onclick={() => void openSafety()}>{lang === "ar" ? "الأمان والصلاحيات" : "Safety & Permissions"}</button>
      {#if safetyOpen}
        <div class="safety-panel">
          <strong>{lang === "ar" ? "الصلاحيات" : "Permissions"}</strong>
          {#each permissions as permission}
            <label>
              <span>{permission.capability}</span>
              <select value={permission.mode} onchange={(e) => void changePermission(permission.capability, (e.currentTarget as HTMLSelectElement).value)}>
                <option value="ask">Ask</option><option value="session">Session</option><option value="always">Always</option><option value="deny">Deny</option>
              </select>
            </label>
          {/each}
          <button onclick={() => void stopAll()}>{lang === "ar" ? "إيقاف جميع عمليات الكمبيوتر" : "Emergency stop"}</button>
          {#if activity.length}
            <div class="activity-log">
              {#each activity.slice(0, 20) as item}
                <div><small>{item.createdAt}</small> — {item.action}: {item.detail}</div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>
<style>.stage{position:relative;width:100vw;height:100vh;overflow:visible;user-select:none}.stage.drag-over{outline:2px dashed rgba(100,180,255,.9);outline-offset:-4px}.error{padding:7px;border-radius:8px;background:rgba(180,40,40,.3);color:#ffd7d7}.capabilities{font-size:11px;opacity:.8}.animation-panel{display:flex;flex-direction:column;gap:5px}.loading{padding:6px;border-radius:8px;background:rgba(80,140,255,.18)}.loading.hidden{display:none}.quality-row{display:flex;justify-content:space-between;align-items:center;gap:8px}.quality-row label{display:flex;align-items:center;gap:5px}.bone-capabilities{display:grid;grid-template-columns:1fr 1fr;gap:2px;font-size:10px}.bone-capabilities span{opacity:.9}.bone-capabilities .missing{opacity:.45}.history{opacity:.65;word-break:break-word}.animation-panel small{font-size:10px;opacity:.7;word-break:break-word}.stage canvas{display:block;width:100%;height:100%}.bubble{position:absolute;left:50%;bottom:8px;transform:translateX(-50%);padding:4px 10px;border-radius:12px;background:rgba(20,25,35,.72);color:white;font:12px sans-serif;pointer-events:none}.menu{position:absolute;right:8px;top:8px;width:260px;max-height:90vh;overflow:auto;padding:10px;border-radius:14px;background:rgba(20,24,32,.94);color:white;font:13px sans-serif;display:flex;flex-direction:column;gap:7px}.menu button,.menu input,.menu select,.menu textarea{font:inherit;border-radius:8px;border:1px solid rgba(255,255,255,.18);padding:7px;box-sizing:border-box}.menu button{background:#2f3746;color:white}.menu input,.menu select,.menu textarea{width:100%;background:#151a22;color:white}.chat-panel{padding:7px;border-radius:10px;background:rgba(255,255,255,.06);display:flex;flex-direction:column;gap:6px}.safety-panel{padding:7px;border-radius:10px;background:rgba(255,255,255,.06);display:flex;flex-direction:column;gap:6px} .safety-panel label{display:flex;justify-content:space-between;gap:6px} .activity-log{max-height:140px;overflow:auto;padding:6px;background:rgba(0,0,0,.18);border-radius:8px}.chat-reply{white-space:pre-wrap;max-height:180px;overflow:auto;padding:7px;border-radius:8px;background:rgba(255,255,255,.08)}</style>
