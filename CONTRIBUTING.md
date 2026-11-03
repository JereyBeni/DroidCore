# Contributing to DroidCore

Thank you for your interest in DroidCore. This document explains how to
contribute.

## Ground rules

1. **No fake emulation.** Do not add placeholder code that pretends to run
   Android applications. Unimplemented features must return explicit errors.
2. **No compatibility claims without tests.** Compatibility is claimed only for
   real applications that have been tested.
3. **Original work only.** Do not copy proprietary source code, and do not copy
   the interfaces of existing commercial emulators.
4. **Third-party licenses.** Any new dependency must be necessary, documented,
   and compatible with MIT and Apache-2.0.

## Development setup

- Rust stable (see `rust-version` in `Cargo.toml`).
- Windows 11 is the primary target; Linux should also build.

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Workflow

1. Open an issue describing the problem or proposal for non-trivial changes.
2. Create a branch from `main`.
3. Keep commits focused and write clear commit messages in English.
4. Make sure `fmt`, `clippy` and `test` pass locally.
5. Open a pull request describing what changed and how it was tested.

## Code style

- Code and documentation are written in English.
- Use `tracing` for logging, not `println!`.
- Public items need doc comments.
- Keep crate boundaries clean: see the workspace layout in the README.

## Reporting bugs

Include your Windows version, hardware, the steps to reproduce, and the logs
produced with `RUST_LOG=debug`.
