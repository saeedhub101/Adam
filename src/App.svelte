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