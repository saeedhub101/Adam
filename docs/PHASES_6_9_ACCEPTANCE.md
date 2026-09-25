# Adam Phase 6–9 Production Completion Contract

This document turns the remaining production gaps into executable acceptance gates. Phases 0–5 remain functional foundations; phases 6–9 are the release, security, recovery and observability hardening layer.

## Phase 6 — Release & Update Readiness
- deterministic Windows x64 production build
- NSIS and MSI installers
- install, launch, upgrade, uninstall and data-cleanup smoke coverage
- taskbar/tray integration
- model-driven window sizing and monitor clamping
- version/build metadata exposed to diagnostics
- update manifest contract documented
- unsigned builds are explicitly marked as development/release-candidate builds; production signing requires the owner's certificate

## Phase 7 — Security & Privacy
- API keys remain in Windows Credential Manager/keyring
- secrets are redacted from diagnostics and logs
- local SQLite remains application-data scoped
- computer automation stays behind explicit permission/exclusion controls
- cloud requests are explicit and cancellable
- imported character assets are treated as untrusted input and validated before use
- no credential, token, or raw provider response is written to diagnostics

## Phase 8 — Recovery & Reliability
- WebView2/media permission failures are recoverable
- WebGL/render failure must not terminate the desktop shell
- database initialization/migration is idempotent
- reminder worker is isolated from the UI path
- cloud cancellation and timeout paths are isolated per request
- window position is clamped to an available monitor
- installer uninstall must remove application-owned runtime data

## Phase 9 — Observability & Acceptance
- runtime diagnostics command exposes only non-secret health data
- CI checks every phase gate
- Rust unit tests cover production safety helpers
- frontend and Rust builds are required before packaging
- installer artifacts are produced only after all gates pass
- final release QA must still be executed on real Windows 10 22H2 and Windows 11 machines, including DPI and multi-monitor permutations

## Definition of 100%

"100%" means all repository-verifiable requirements are implemented and green in CI. Hardware-, OS-, certificate-, microphone-device-, and final-character-specific acceptance must be verified on the target Windows machines; CI cannot honestly substitute for those physical tests.
