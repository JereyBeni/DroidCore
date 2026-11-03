# DroidCore Development Roadmap

This roadmap is built incrementally. Each stage must be complete, tested, and
honest about its limits before the next one starts. No stage may introduce fake
emulation or claim Android compatibility it does not have.

## Stage 1 — Workspace and project structure (current)

- Cargo workspace with all crates (`droidcore`, `emulator`, `cpu`, `memory`,
  `android_runtime`, `graphics`, `input`, `audio`, `apk`, `compatibility`,
  `frontend`).
- README, LICENSE (MIT and Apache-2.0), CONTRIBUTING, `.gitignore`.
- GitHub Actions CI on Windows and Linux: format, clippy, build, test.
- **Done when:** `cargo build --workspace` and `cargo test --workspace` pass in
  CI.

## Stage 2 — Original egui desktop interface

- Main window with an original layout (library, settings, performance panel).
- Landscape and portrait preview switch (UI only).
- Configurable controls screen (stored with serde, no emulation behind it).
- **Done when:** the app opens, navigates between views, and persists settings.

## Stage 3 — APK parser and inspector

- Read the APK as a ZIP archive and parse `AndroidManifest.xml` (binary XML).
- List package name, version, minimum and target SDK, permissions, and
  native libraries.
- Detect ABIs (`lib/armeabi-v7a`, `lib/arm64-v8a`).
- **Done when:** the inspector reads a set of test APKs (created in-repo or
  with permission) and reports correct metadata, verified by tests.

## Stage 4 — CPU and architecture detection

- Identify the guest architecture from the APK and native libraries (ARM and
  ARM64 ELF headers).
- Report unsupported architectures explicitly.
- **Done when:** detection is covered by unit tests using real ELF headers.

## Stage 5 — Emulator foundation

- Guest memory model with address translation (`memory`).
- Emulation orchestration skeleton (`emulator`) that starts and stops cleanly.
- Instruction decoding for a small, tested ARM subset, verified against
  reference test vectors.
- **Done when:** the emulator runs synthetic code sequences with correct
  results. No Android app is expected to run yet.

## Stage 6 — Graphics backend

- wgpu rendering surface integrated into the egui window.
- OpenGL ES command translation (`graphics`) tested with synthetic draw calls.
- Performance monitor (frame time, GPU memory) for the GT 710.
- **Done when:** synthetic OpenGL ES triangles render correctly on the target.

## Stage 7 — Android runtime integration

- Process and loader model (`android_runtime`).
- Minimal Android API compatibility surface (`compatibility`) driven by real
  failing calls, not guesses.
- Input (`input`) for touch simulation, and audio (`audio`) output.
- **Done when:** a simple, legally distributable test application starts and
  reaches its first frame.

## Stage 8 — Actual APK execution

- Run real applications, one at a time, and record results.
- Document each application's status honestly (works, partial, fails) with
  logs.
- Compatibility claims are published only after testing.
- **Done when:** at least one real application runs end to end and its result
  is documented with reproducible steps.

## Ongoing

- Hardware target: Intel Core i5-2310, 8 GB DDR3, NVIDIA GT 710, Windows 11.
- Performance budgets are checked on the target hardware at each stage.
