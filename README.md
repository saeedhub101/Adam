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

## Phase status

- **Phase 0 — Foundation:** source and packaging gates implemented; CI validates foundation files, formatting, production build, NSIS/MSI packaging, installer launch/uninstall, persistence and data cleanup. Code signing remains a release-hardening step.
- **Phase 1 — Character on desktop:** transparent always-on-top surface, robust GLB/GLTF multi-file loading, fallback avatar, persistent size/position, multi-monitor clamping, click-through control, render-quality control, WebGL recovery handling, character history and capability inspection.
- **Phase 2 — Animation:** implemented with animation-clip detection, cross-fades, bone-aware procedural idle motion and fallback idle behavior.
- **Phase 3 — Voice:** bilingual English/Arabic speech recognition and speech synthesis are integrated through WebView voice APIs, with microphone state and transcript display.
- **Phase 4 — Local Brain & Memory:** local routing, SQLite memory/reminders/calendar and offline execution paths are implemented.
- **Phase 5 — Cloud Brain:** OpenAI-compatible cloud routing, credential storage, streaming, persona context and offline fallback are implemented.

The phase labels above describe source implementation. A phase is only considered release-ready after its automated checks and Windows runtime verification pass.

## Phase 0 acceptance

The Windows workflow at `.github/workflows/windows.yml` verifies:

1. Required foundation files exist.
2. Svelte/TypeScript validation passes.
3. The frontend production bundle builds.
4. Rust unit/integration tests pass.
5. The Tauri Windows production build succeeds.
6. Both NSIS (`.exe`) and MSI (`.msi`) installers are present.
7. Missing installer artifacts fail CI instead of being silently accepted.

A real Windows GUI launch/install smoke test remains a separate runtime gate; CI does not claim an interactive desktop launch was verified unless that test is actually executed.

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

## Phase 1

GLB/GLTF can be selected from the Change Character control. If no model is present, a built-in fallback character is shown. Position and size are stored through the Rust backend. A repository-provided production character asset remains optional: the built-in fallback is the guaranteed offline default until the final Adam model is supplied.

## Phase 2

The Three.js scene indexes character bones, detects animation clips, supports animation playback with short cross-fades, and applies subtle breathing/head/arm idle motion when no imported animation is active. The implementation is designed to work with different GLB/GLTF rigs without requiring a fixed bone hierarchy.

## Phase 3–5 boundaries

Voice, local memory/calendar, cloud routing, permissions and computer-control capabilities are developed in later phases. Phase 0 acceptance does not treat those later features as substitutes for foundation verification.

## Reproducibility note

CI now generates `package-lock.json` with `npm install --package-lock-only` and commits it when missing or changed, so subsequent dependency installation can be reproduced from the committed lockfile.

## Security and release note

Code signing / publisher identity is not part of the current Phase 0 acceptance gate. The Windows installers are currently unsigned; signing will be handled as a release-hardening step.
