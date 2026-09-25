# Adam

Adam is a Windows desktop AI companion.

## Phase status
- Phase 0 Foundation: source + CI scaffold implemented.
- Phase 1 Character on desktop: source implemented; Windows CI builds the NSIS installer.
- Phase 3 Voice: bilingual English/Arabic speech recognition and speech synthesis are integrated through the WebView voice APIs, with microphone state and transcript display.\n- Phase 2 Animation: animation controller implemented; GLB/GLTF animation clips are detected and cross-faded, with procedural idle motion when no clip is active.

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

## Phase 2
The Three.js scene now indexes character bones, detects animation clips, supports animation playback with short cross-fades, and applies subtle breathing/head/arm idle motion when no imported animation is active. The implementation is designed to work with different GLB/GLTF rigs without requiring a fixed bone hierarchy.
