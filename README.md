# DroidCore

An original Android game emulator written in Rust, targeting Windows 11.

> **Status: early development (stage 1 — workspace skeleton).**
> DroidCore does not run Android applications yet. Nothing in this repository
> emulates Android today, and no compatibility is claimed for any app.

## Goals

- Run Android games on Windows 11 through an original Rust implementation.
- Support ARM and ARM64 guest code, OpenGL ES graphics, audio, touch
  simulation, and landscape and portrait modes.
- Stay optimized for modest hardware (Intel Core i5-2310, 8 GB DDR3,
  NVIDIA GeForce GT 710, Windows 11).

## Non-goals

- DroidCore is not a clone of BlueStacks, LDPlayer, or any other emulator, and
  does not copy their interfaces or proprietary code.

## Technology

| Area            | Choice                          |
|-----------------|---------------------------------|
| Language        | Rust (stable)                   |
| Build           | Cargo workspace                 |
| GUI             | egui                            |
| Graphics        | wgpu                            |
| Serialization   | serde                           |
| Logging         | tracing                         |
| Testing         | Rust unit and integration tests |
| CI              | GitHub Actions                  |

## Workspace layout

| Crate            | Responsibility                                  |
|------------------|-------------------------------------------------|
| `droidcore`      | Main application binary                         |
| `emulator`       | Emulation orchestration                         |
| `cpu`            | CPU architecture support                        |
| `memory`         | Memory management                               |
| `android_runtime`| Android execution environment                   |
| `graphics`       | Rendering backend                               |
| `input`          | Keyboard, mouse and touch input                 |
| `audio`          | Audio subsystem                                 |
| `apk`            | APK inspection and loading                      |
| `compatibility`  | Android API compatibility                       |
| `frontend`       | egui interface                                  |

## Building

```sh
cargo build --workspace
cargo test --workspace
cargo run -p droidcore
```

## Documentation

- [Roadmap](docs/ROADMAP.md)
- [Technical feasibility notes](docs/FEASIBILITY.md) (pending)

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at
your option. Third-party components keep their own licenses; see the
documentation of each dependency.
