# Adam — Consolidated Master Gap Closure

This is the implementation target through Section 20. Phase 10 remains optional; Section 19 exclusions are not release blockers.

## Code-side requirements
- Complete Settings tabs and persistence.
- Complete 12-category permissions and per-app scopes.
- Required SQLite schema and migrations.
- Full local skills registry, Windows discovery/controls, Arabic/English date parsing, calendar recurrence and reminders.
- Provider abstraction, streaming/retries/cancellation, tool calling and usage.
- DPI-aware monitor/window/region capture, excluded-app enforcement, downscale/coordinate mapping.
- UI Automation tools, verified agent loop, limits, data-entry review/dry-run.
- Safety: confirmations, password-field detection, emergency stop, rate/scope limits, prompt-injection defenses.
- Privacy/security: CSP, single-instance, clean uninstall data choice, credential isolation.
- Release: online/offline installers, updater, signing docs, performance instrumentation, accessibility and RTL.
- Tests: unit, integration, safety and manual Windows matrix.

## Evidence that must be physically tested
Final character asset; clean Windows 10/11 install; DPI 100/125/150%; two monitors; no internet; no microphone; unplug microphone; sleep/resume; high contrast; 8-hour leak; actual CPU/RAM/start/voice latency; screen-reader/accessibility; signed installer/update and version upgrade.

A CI green result must not be used as evidence for these physical requirements.
