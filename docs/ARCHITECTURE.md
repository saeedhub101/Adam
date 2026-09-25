# Adam Architecture

## Phase 0 — Foundation

Adam's foundation is a Windows desktop application built around:

- **Tauri v2** for the desktop shell and native bridge.
- **Rust** for native commands, persistence and protected credential access.
- **WebView2** for the Windows web runtime.
- **Svelte 5 + TypeScript + Vite** for the UI.
- **Three.js / WebGL** for the character rendering layer.
- **SQLite** for local application data.
- **Windows Credential Manager** through the Rust keyring integration for cloud API keys.

### Supported target

- Windows 10 22H2 x64
- Windows 11 x64

### Native persistence

The Rust backend creates the application data directory and initializes the SQLite database. Current foundation data includes memory, calendar and permissions tables. Window position is persisted separately and is clamped to the active monitor bounds.

### Native command boundary

The Tauri command layer is the only frontend-to-native bridge. Phase 0/1 intentionally does not provide unrestricted shell execution, administrator elevation or computer-control commands.

### Build acceptance

The Windows CI workflow performs, in order:

1. Foundation-file verification.
2. `npm install`.
3. `npm run check`.
4. `npm run build`.
5. `cargo test --manifest-path src-tauri/Cargo.toml`.
6. `npm run tauri build`.
7. Explicit verification that at least one NSIS `.exe` and one MSI `.msi` exist.
8. Artifact upload with `if-no-files-found: error`.

This separates source validation from actual installer production and prevents a green workflow from hiding missing installer artifacts.

### Runtime verification boundary

CI source/build checks do not by themselves prove that a Windows installer was interactively installed and launched. A real Windows GUI smoke test must be reported separately when executed.

### Reproducibility boundary

The repository commits `package-lock.json`, and Windows CI uses `npm ci` for lockfile-reproducible dependency installation.

### Release signing boundary

Windows code signing and publisher identity are intentionally outside the current Phase 0 acceptance gate. Current CI produces unsigned installers.

## Phase 1 — Character on desktop

Phase 1 adds the desktop character surface:

- transparent always-on-top window
- Three.js/WebGL renderer
- default GLB asset slot with safe fallback avatar
- GLB/GLTF picker
- size control
- drag positioning
- click-through API
- persisted position
- multi-monitor position clamping

The final `adam.glb` asset is not required for the current foundation acceptance pass.

## Phase 2 — Animation

The animation layer:

- detects imported animation clips
- cross-fades between clips
- indexes common character bones
- applies procedural breathing/head/body motion and full limb fallback motion when an imported locomotion clip is unavailable
- keeps procedural offsets separate from AnimationMixer updates to avoid frame-to-frame bone drift/conflicts
- supports one-shot gesture fallback and automatic recovery to idle
- supports generic GLB/GLTF/FBX rigs without requiring one fixed bone hierarchy

## Phase 3 — Voice

Voice capabilities include bilingual English/Arabic recognition and speech synthesis, microphone state, transcript handling, local Whisper model management, VAD, speech interruption behavior, and character talking/viseme-driven mouth animation. Model assets are cache/remote managed by Transformers.js; fully bundled offline model distribution remains a packaging-size decision rather than a build blocker.

## Phase 4 — Local Brain & Memory

The local path uses SQLite-backed memory, reminders and calendar data. The intended routing is:

`User → Local Brain → execute locally when understood → otherwise Cloud Brain → offline fallback if cloud is unavailable.`

Local execution is implemented through explicit native commands rather than generic text-only responses. The router covers local memory, notes, reminders, calendar, time/greetings and permission-gated application opening, with cloud routing and offline fallback handled in the UI.

## Phase 5 — Cloud Brain

The cloud layer provides an OpenAI-compatible provider abstraction, protected API-key storage, streaming, cancellable generation, connectivity testing, persona context, provider/model configuration, conversation history, relevant-memory context and offline fallback. The local integration also includes background reminder due events, relevance-ranked memory search, and a Windows WebView2 media-permission bridge for microphone capture.

## Safety boundaries

Phase 0/1 do not include:

- unrestricted computer control
- arbitrary shell execution
- administrator elevation
- automatic destructive actions

Permissions and controlled computer operations belong to later phases.

## Asset boundary

The production `adam.glb` character asset is intentionally excluded from the Phase 0 foundation gate so that missing art assets cannot mask infrastructure/build failures.
