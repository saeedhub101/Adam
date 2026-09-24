# Adam

Adam is a Windows desktop AI companion.

## Phase status
- Phase 0 Foundation: source + CI scaffold implemented.
- Phase 1 Character on desktop: source implemented; Windows CI builds the NSIS installer.
- Phase 2 Animation: not started.

## Requirements
Windows 10 22H2 or Windows 11 x64, Node.js LTS, Rust stable, Microsoft C++ Build Tools and WebView2.

## Run
`npm install`
`npm run tauri dev`

## Build
`npm run tauri build`

Installer output: `src-tauri/target/release/bundle/nsis`.

## Phase 1
GLB/GLTF can be selected from the Change Character control. If no model is present, a built-in fallback character is shown. Position and size are stored through the Rust backend.

Phase 2 starts only after approval.
