# Adam Architecture

Phase 0 establishes Tauri v2 + Rust + WebView2 + Svelte/TypeScript/Vite.

Phase 1 adds the desktop character surface:
- transparent always-on-top window
- Three.js/WebGL renderer
- default GLB asset slot with safe fallback avatar
- GLB/GLTF picker
- size control
- drag positioning
- click-through API
- persisted position
- tray/close wiring is reserved for the next integration pass if the target environment lacks tray support

Safety boundaries for Phase 0/1:
- no computer-control tools
- no microphone
- no cloud API
- no shell execution
- no admin elevation

Later phases add voice, local memory/calendar, cloud routing, permissions and computer control.
