# Adam

Adam is a Windows desktop AI companion.

## Supported foundation

- Windows 10 22H2 and Windows 11 x64
- Tauri v2 + Rust backend
- WebView2 desktop runtime
- Svelte 5 + TypeScript + Vite
- Three.js / WebGL
- SQLite local persistence
- Windows Credential Manager integration for cloud keys
- NSIS and MSI Windows installers

## Phase 0–2 batch

The current main branch contains the consolidated Phase 0–2 hardening batch. The character engine now has unified GLB/GLTF/FBX loading, persistent character-package support, rig/capability detection, blink and facial-emotion layers, gaze targeting, gesture queueing, one-shot gesture recovery, procedural locomotion, render-context recovery, and desktop safety controls. The build must be treated as release-ready only after the single CI run for this batch passes all frontend, Rust, packaging, installer, and smoke-test gates.

Code signing still requires the release certificate/secret and cannot be produced from source code alone. The final Adam production GLB is also an external project asset and is not invented by the repository.

## Requirements

Windows 10 22H2 or Windows 11 x64, Node.js LTS, Rust stable, Microsoft C++ Build Tools, and WebView2.

## Development

```bash
npm install
npm run tauri dev
```

## Production build

```bash
npm run tauri build
```

Installer output:

- `src-tauri/target/release/bundle/nsis`
- `src-tauri/target/release/bundle/msi`

## Release boundary

The CI result is the acceptance authority for this consolidated batch. If it fails, the failure must be fixed in the same batch before another user-facing build is requested. The repository must not be considered complete merely because TypeScript compiles: the Windows installer and runtime gates must also pass.
