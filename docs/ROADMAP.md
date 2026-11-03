# DroidCore Development Roadmap

This roadmap is built incrementally. Each stage must be complete, tested, and
honest about its limits before the next one starts. No stage may introduce fake
emulation or claim Android compatibility it does not have.

## Stage 1 — Workspace and project structure

- Cargo workspace, project documentation, dual MIT/Apache-2.0 license.
- GitHub Actions CI on Windows and Linux.
- **Done when:** workspace formatting, build and tests pass in CI.

## Stage 2 — Desktop interface and APK inspection (in progress)

- Original egui desktop window.
- Select an APK and inspect its ZIP entries, DEX files and native libraries.
- Parse binary Android XML and extract SDK/package metadata.
- **Done when:** APK metadata is tested against known fixtures.

## Stage 3 — Emulator orchestration (implemented, backend pending)

- Backend-neutral emulator configuration with memory and vCPU validation.
- Explicit lifecycle: created, starting, running, paused, stopping, stopped
  and failed.
- Backend capability checks and error propagation.
- **Done when:** lifecycle tests pass. This does not yet emulate CPU
  instructions or boot Android.

## Stage 4 — OldDroid legacy hybrid profile (planner implemented)

- Cover Android API 4 through API 21 (Android 1.6 through 5.0).
- Select Dalvik for API 4-20 and ART for API 21.
- Plan the Android guest system, native ABI path and OpenGL ES bridge.
- Mark ARM translation and the full-system backend as unavailable until they
  are implemented.
- **Done when:** API selection and ABI routing are tested, then real legacy
  guest images boot in a full-system backend.

## Stage 5 — CPU, memory and native translation

- Guest memory model with address translation.
- ARM/ARM64 instruction decoding and execution with reference test vectors.
- ARM native-code translation for ARM-only games on x86 Android guests.
- **Done when:** deterministic CPU/memory tests pass and a legal guest userspace
  runs without host escape.

## Stage 6 — Graphics, input and audio

- wgpu rendering surface integrated into egui.
- OpenGL ES 1.x/2.0 compatibility for the legacy profile.
- Touch, keyboard, mouse and audio device bridges.
- **Done when:** synthetic graphics and input/audio tests pass on the target.

## Stage 7 — Android runtime integration

- Full Android system image boot and process/loader model.
- Dalvik and ART guest runtime profiles.
- Android API compatibility driven by real failing calls.
- **Done when:** a legally distributable test application starts and reaches
  its first frame.

## Stage 8 — Actual APK execution

- Run real applications, one at a time, and record results.
- Document each application's status (works, partial, fails) with logs.
- Publish compatibility claims only after reproducible testing.

## Ongoing

- Hardware target: Intel Core i5-2310, 8 GB DDR3, NVIDIA GT 710, Windows 11.
- Performance budgets are checked on the target hardware at each stage.
