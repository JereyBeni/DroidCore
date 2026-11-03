# DroidCore

An original Android game emulator written in Rust, targeting Windows 11.

> **Status: early development.** DroidCore has a desktop APK inspector and
> initial emulator orchestration. It does not boot Android or run games yet.

## Goals

- Run Android games on Windows 11 through an original Rust implementation.
- Support ARM and ARM64 guest code, OpenGL ES graphics, audio, touch
  simulation, and landscape and portrait modes.
- Preserve support planning for legacy games from Android 1.6 through 5.0
  through the separate OldDroid compatibility crate.
- Stay optimized for modest hardware (Intel Core i5-2310, 8 GB DDR3,
  NVIDIA GeForce GT 710, Windows 11).

## Non-goals

- DroidCore is not a clone of BlueStacks, LDPlayer, or any other emulator, and
  does not copy their interfaces or proprietary code.

## Technology

| Area | Choice |
|---|---|
| Language | Rust (stable) |
| Build | Cargo workspace |
| GUI | egui |
| Graphics | wgpu (planned rendering backend) |
| Logging | tracing |
| Testing | Rust unit and integration tests |
| CI | GitHub Actions |

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
| `apk` | APK inspection and loading |
| `compatibility` | Android API compatibility |
| `frontend` | egui interface |

## Building

```sh
cargo build --workspace
cargo test --workspace
cargo run -p droidcore
```

## Documentation

- [Roadmap](docs/ROADMAP.md)
- [OldDroid hybrid design](docs/OLDDROID.md)
- [Technical feasibility notes](docs/FEASIBILITY.md) (pending)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option. Third-party components keep their own licenses; see the
documentation of each dependency.
