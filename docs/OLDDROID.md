# OldDroid hybrid design

OldDroid is the legacy compatibility profile for Android 1.6 through Android
5.0. It is a separate crate inside DroidCore, not a second application binary.

## Why hybrid

Old Android games are not all the same kind of program. A game can depend on
the Android framework and Dalvik/ART runtime, and it may also include native
ARM shared libraries. OldDroid models a hybrid stack with three layers:

1. **Android guest system** — a complete Android system image supplies the
   framework, services, package manager and runtime.
2. **Native-code path** — native libraries run in the guest when their ABI
   matches, or are routed to an ARM translation layer when an x86 guest is
   selected and the APK only contains ARM libraries.
3. **Graphics bridge** — an OpenGL ES compatibility/translation layer connects
   legacy game rendering to the host renderer.

Android 1.6 is API 4 and Android 5.0 is API 21. Guest releases before API 21
use Dalvik; API 21 uses ART. The release table and runtime selection live in
`crates/olddroid/src/lib.rs`.

## Current implementation

The crate currently builds a compatibility plan from an app descriptor:
minimum/target API levels, native ABIs, and the selected guest profile. The
emulator crate provides a backend-neutral lifecycle controller with explicit
states and backend capability checks.

**This is orchestration and planning code, not a bootable Android emulator yet.**
There is currently no Android system image, QEMU adapter, ARM translation
engine, Android framework, or OpenGL ES translator included. The plan reports
these missing pieces rather than pretending an APK can already run.

## First backend milestones

- Add a real full-system backend and boot a legally distributable Android
  guest image.
- Add APK manifest/ELF inspection to populate `LegacyAppDescriptor`
  automatically.
- Add ARM native-code translation for ARM-only games on x86 guests.
- Add GLES 1.x/2.0 compatibility and input/audio device bridges.
- Validate one test game per Android generation before publishing
  compatibility claims.
