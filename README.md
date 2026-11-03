# DroidCore

An original Android game emulator project written in Rust, targeting Windows 11 and Ubuntu Linux.

> **Status: early development.** DroidCore has a desktop package inspector and initial emulator orchestration. It does not boot Android or run games yet. IPA support is inspection-only and does not emulate iOS.

## Goals

- Build native desktop releases for Windows x64 and Ubuntu x64.
- Inspect Android APK and Apple IPA package archives.
- Detect localized resources bundled in Android and iOS packages.
- Run Android games on Windows through an original Rust implementation.
- Support ARM and ARM64 guest code, OpenGL ES graphics, audio, touch simulation, and landscape and portrait modes.
- Preserve support planning for legacy games from Android 1.6 through 5.0 through the separate OldDroid compatibility crate.
- Stay optimized for modest hardware (Intel Core i5-2310, 8 GB DDR3, NVIDIA GeForce GT 710, Windows 11).

## Package inspection

The package inspector accepts:

| Format | Metadata inspected |
|---|---|
| Android `.apk` | Android manifest type, package ID when text XML is available, DEX files, native library ABIs, localized `res/values-*` resources |
| Apple `.ipa` | App bundle Info.plist when XML, bundle ID, display name, version, minimum iOS version, localized `.lproj` resources |

Binary Android manifests and binary Apple property lists are detected, but their metadata is not decoded yet. iOS apps are not executed or emulated.

## Non-goals

- DroidCore is not a clone of BlueStacks, LDPlayer, or any other emulator, and does not copy their interfaces or proprietary code.
- IPA inspection does not imply an iOS emulator.

## Technology

| Area | Choice |
|---|---|
| Language | Rust (stable) |
| Build | Cargo workspace |
| GUI | egui |
| Graphics | wgpu (planned rendering backend) |
| Logging | tracing |
| Testing | Rust unit and integration tests |
| CI | GitHub Actions: Windows and Ubuntu |

## Workspace layout

| Crate | Responsibility |
|---|---|
| `droidcore` | Main application binary |
| `emulator` | Backend-neutral emulator lifecycle and configuration |
| `olddroid` | Hybrid compatibility planning for Android 1.6-5.0 |
| `cpu` | CPU architecture support |
| `memory` | Guest memory management |
| `android_runtime` | Android execution environment |
| `graphics` | Rendering backend and OpenGL ES translation |
| `input` | Keyboard, mouse and touch input |
| `audio` | Audio subsystem |
| `apk` | APK and IPA package inspection |
| `compatibility` | Android API compatibility |
| `frontend` | egui interface |

## Building locally

```sh
cargo build --workspace
cargo test --workspace
cargo run -p droidcore
```

Release builds:

```sh
cargo build -p droidcore --release
```

## Download CI builds

Every push to `main`, pull request, or manual workflow run builds Windows x64 and Ubuntu x64. Open the repository's [Actions](https://github.com/JereyBeni/DroidCore/actions) page, select a successful run, and download either `DroidCore-Windows-x64` or `DroidCore-Ubuntu-x64` from its artifacts.

## Documentation

- [Roadmap](docs/ROADMAP.md)
- [OldDroid hybrid design](docs/OLDDROID.md)
- [Package formats and locale detection](docs/PACKAGE_FORMATS.md)
- [Technical feasibility notes](docs/FEASIBILITY.md) (pending)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option. Third-party components keep their own licenses; see the documentation of each dependency.
