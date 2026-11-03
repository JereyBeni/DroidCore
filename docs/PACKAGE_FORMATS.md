# APK, IPA and localized resources

DroidCore's package inspector reads APK and IPA files as ZIP archives. It
reports the platform, package or bundle identifier (when available), archive
size, entry count, and localized resources found inside the package.

## Android APK

Android locale folders are detected under resource directories such as:

- `res/values-es/`
- `res/values-es-rAR/`
- `res/values-b+zh+Hans+CN/`

Legacy Android resource qualifiers are normalized to language tags, for
example `es-rAR` becomes `es-AR`. BCP 47 resource qualifiers retain their
script and region subtags.

The inspector also lists DEX files and native `.so` files with their ABI
directory. Text XML manifests can provide the package identifier. Binary AXML
is detected but not decoded yet.

## Apple iOS IPA

An IPA is inspected for an app bundle at `Payload/*.app/Info.plist`.
For XML property lists, the inspector reads:

- `CFBundleIdentifier`
- `CFBundleName` or `CFBundleDisplayName`
- `CFBundleShortVersionString` or `CFBundleVersion`
- `MinimumOSVersion`
- `CFBundleDevelopmentRegion`

Localized folders such as `es.lproj`, `es-419.lproj`, and
`English.lproj` are detected. `Base.lproj` is excluded because it is a
development resource layout, not a language identifier.

Binary property lists are recognized, but not decoded yet. Mach-O CPU
architecture inspection and iOS execution are out of scope for this package
inspection stage. OldDroid remains an Android compatibility profile.

## Build targets

GitHub Actions builds and tests on:

- Windows x64, producing `droidcore.exe`.
- Ubuntu x64, producing the `droidcore` executable.

Successful runs publish both release executables as workflow artifacts for
14 days. These are native binaries, not installers. Ubuntu systems may need
the desktop libraries required by eframe/winit installed by the user.
