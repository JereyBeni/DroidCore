use droidcore_emulator::{EmulatorConfig, ExecutionMode, GuestArchitecture};

pub const MIN_ANDROID_API: u32 = 4;
pub const MAX_ANDROID_API: u32 = 21;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidRelease {
    Donut,
    Eclair,
    Froyo,
    Gingerbread,
    Honeycomb,
    IceCreamSandwich,
    JellyBean,
    KitKat,
    Lollipop,
}

impl AndroidRelease {
    pub fn api_level(self) -> u32 {
        match self {
            Self::Donut => 4,
            Self::Eclair => 7,
            Self::Froyo => 8,
            Self::Gingerbread => 10,
            Self::Honeycomb => 13,
            Self::IceCreamSandwich => 15,
            Self::JellyBean => 18,
            Self::KitKat => 19,
            Self::Lollipop => 21,
        }
    }

    pub fn from_api_level(api_level: u32) -> Option<Self> {
        match api_level {
            4 => Some(Self::Donut),
            5..=7 => Some(Self::Eclair),
            8 => Some(Self::Froyo),
            9..=10 => Some(Self::Gingerbread),
            11..=13 => Some(Self::Honeycomb),
            14..=15 => Some(Self::IceCreamSandwich),
            16..=18 => Some(Self::JellyBean),
            19..=20 => Some(Self::KitKat),
            21 => Some(Self::Lollipop),
            _ => None,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Donut => "Android 1.6 Donut",
            Self::Eclair => "Android 2.0-2.1 Eclair",
            Self::Froyo => "Android 2.2 Froyo",
            Self::Gingerbread => "Android 2.3 Gingerbread",
            Self::Honeycomb => "Android 3.x Honeycomb",
            Self::IceCreamSandwich => "Android 4.0 Ice Cream Sandwich",
            Self::JellyBean => "Android 4.1-4.3 Jelly Bean",
            Self::KitKat => "Android 4.4 KitKat",
            Self::Lollipop => "Android 5.0 Lollipop",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidRuntime {
    Dalvik,
    Art,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeAbi {
    Armeabi,
    ArmeabiV7a,
    Arm64V8a,
    X86,
    X86_64,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeCodePath {
    NoNativeLibraries,
    GuestNative,
    ArmTranslationRequired,
    UnsupportedAbi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanReadiness {
    RequiresAndroidSystemBackend,
    BlockedByApiLevel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompatibilityWarning {
    MissingMinSdk,
    TargetSdkAboveSupportedRange(u32),
    UnknownNativeAbi,
    ArmTranslationNotImplemented,
    GraphicsBridgeNotImplemented,
    AndroidSystemBackendNotInstalled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OldDroidConfig {
    pub guest_release: AndroidRelease,
    pub guest_architecture: GuestArchitecture,
    pub memory_mib: u32,
    pub vcpus: u8,
}

impl Default for OldDroidConfig {
    fn default() -> Self {
        Self {
            guest_release: AndroidRelease::KitKat,
            guest_architecture: GuestArchitecture::X86,
            memory_mib: 512,
            vcpus: 1,
        }
    }
}

impl OldDroidConfig {
    pub fn emulator_config(&self) -> EmulatorConfig {
        EmulatorConfig {
            architecture: self.guest_architecture,
            mode: ExecutionMode::System,
            memory_mib: self.memory_mib,
            vcpus: self.vcpus,
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        self.emulator_config()
            .validate()
            .map_err(|error| error.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegacyAppDescriptor {
    pub package_name: String,
    pub min_sdk: Option<u32>,
    pub target_sdk: Option<u32>,
    pub native_abis: Vec<NativeAbi>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityPlan {
    pub package_name: String,
    pub guest_release: AndroidRelease,
    pub guest_api_level: u32,
    pub runtime: AndroidRuntime,
    pub guest_architecture: GuestArchitecture,
    pub native_code_path: NativeCodePath,
    pub readiness: PlanReadiness,
    pub emulator: EmulatorConfig,
    pub warnings: Vec<CompatibilityWarning>,
}

pub struct OldDroid {
    config: OldDroidConfig,
}

impl OldDroid {
    pub fn new(config: OldDroidConfig) -> Result<Self, String> {
        config.validate()?;
        Ok(Self { config })
    }

    pub fn config(&self) -> &OldDroidConfig {
        &self.config
    }

    pub fn plan(&self, app: &LegacyAppDescriptor) -> CompatibilityPlan {
        let guest_api = self.config.guest_release.api_level();
        let runtime = if guest_api >= 21 {
            AndroidRuntime::Art
        } else {
            AndroidRuntime::Dalvik
        };
        let mut warnings = vec![];
        let mut readiness = PlanReadiness::RequiresAndroidSystemBackend;

        if app.min_sdk.is_none() {
            warnings.push(CompatibilityWarning::MissingMinSdk);
        }

        if let Some(min_sdk) = app.min_sdk {
            if min_sdk > MAX_ANDROID_API || min_sdk > guest_api {
                readiness = PlanReadiness::BlockedByApiLevel;
            }
        }

        if let Some(target_sdk) = app.target_sdk {
            if target_sdk > MAX_ANDROID_API {
                warnings.push(CompatibilityWarning::TargetSdkAboveSupportedRange(
                    target_sdk,
                ));
            }
        }

        let native_code_path = self.select_native_code_path(&app.native_abis, &mut warnings);

        warnings.push(CompatibilityWarning::GraphicsBridgeNotImplemented);
        warnings.push(CompatibilityWarning::AndroidSystemBackendNotInstalled);

        CompatibilityPlan {
            package_name: app.package_name.clone(),
            guest_release: self.config.guest_release,
            guest_api_level: guest_api,
            runtime,
            guest_architecture: self.config.guest_architecture,
            native_code_path,
            readiness,
            emulator: self.config.emulator_config(),
            warnings,
        }
    }

    fn select_native_code_path(
        &self,
        abis: &[NativeAbi],
        warnings: &mut Vec<CompatibilityWarning>,
    ) -> NativeCodePath {
        if abis.is_empty() {
            return NativeCodePath::NoNativeLibraries;
        }

        if abis.iter().any(|abi| self.abi_matches_guest(*abi)) {
            return NativeCodePath::GuestNative;
        }

        if abis
            .iter()
            .any(|abi| matches!(*abi, NativeAbi::Armeabi | NativeAbi::ArmeabiV7a))
            && matches!(
                self.config.guest_architecture,
                GuestArchitecture::X86 | GuestArchitecture::X86_64
            )
        {
            warnings.push(CompatibilityWarning::ArmTranslationNotImplemented);
            return NativeCodePath::ArmTranslationRequired;
        }

        if abis.contains(&NativeAbi::Unknown) {
            warnings.push(CompatibilityWarning::UnknownNativeAbi);
        }

        NativeCodePath::UnsupportedAbi
    }

    fn abi_matches_guest(&self, abi: NativeAbi) -> bool {
        match self.config.guest_architecture {
            GuestArchitecture::Arm32 => {
                matches!(abi, NativeAbi::Armeabi | NativeAbi::ArmeabiV7a)
            }
            GuestArchitecture::Arm64 => abi == NativeAbi::Arm64V8a,
            GuestArchitecture::X86 => abi == NativeAbi::X86,
            GuestArchitecture::X86_64 => abi == NativeAbi::X86_64,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app(min_sdk: u32, abis: Vec<NativeAbi>) -> LegacyAppDescriptor {
        LegacyAppDescriptor {
            package_name: "org.example.legacygame".to_owned(),
            min_sdk: Some(min_sdk),
            target_sdk: Some(min_sdk),
            native_abis: abis,
        }
    }

    #[test]
    fn maps_supported_android_api_levels() {
        assert_eq!(
            AndroidRelease::from_api_level(4),
            Some(AndroidRelease::Donut)
        );
        assert_eq!(
            AndroidRelease::from_api_level(8),
            Some(AndroidRelease::Froyo)
        );
        assert_eq!(
            AndroidRelease::from_api_level(19),
            Some(AndroidRelease::KitKat)
        );
        assert_eq!(
            AndroidRelease::from_api_level(21),
            Some(AndroidRelease::Lollipop)
        );
        assert_eq!(AndroidRelease::from_api_level(22), None);
    }

    #[test]
    fn selects_dalvik_for_pre_lollipop_guests() {
        let old_droid = OldDroid::new(OldDroidConfig::default()).unwrap();
        let plan = old_droid.plan(&app(4, vec![]));

        assert_eq!(plan.runtime, AndroidRuntime::Dalvik);
        assert_eq!(plan.guest_api_level, 19);
        assert_eq!(plan.readiness, PlanReadiness::RequiresAndroidSystemBackend);
    }

    #[test]
    fn selects_art_for_lollipop_guest() {
        let config = OldDroidConfig {
            guest_release: AndroidRelease::Lollipop,
            ..OldDroidConfig::default()
        };
        let old_droid = OldDroid::new(config).unwrap();
        let plan = old_droid.plan(&app(21, vec![]));

        assert_eq!(plan.runtime, AndroidRuntime::Art);
        assert_eq!(plan.guest_api_level, 21);
    }

    #[test]
    fn marks_arm_libraries_for_translation_on_x86_guest() {
        let old_droid = OldDroid::new(OldDroidConfig::default()).unwrap();
        let plan = old_droid.plan(&app(10, vec![NativeAbi::ArmeabiV7a]));

        assert_eq!(
            plan.native_code_path,
            NativeCodePath::ArmTranslationRequired
        );
        assert!(plan
            .warnings
            .contains(&CompatibilityWarning::ArmTranslationNotImplemented));
    }

    #[test]
    fn accepts_native_x86_libraries() {
        let old_droid = OldDroid::new(OldDroidConfig::default()).unwrap();
        let plan = old_droid.plan(&app(10, vec![NativeAbi::X86]));

        assert_eq!(plan.native_code_path, NativeCodePath::GuestNative);
    }

    #[test]
    fn blocks_apps_that_require_a_newer_android_api() {
        let old_droid = OldDroid::new(OldDroidConfig::default()).unwrap();
        let plan = old_droid.plan(&app(22, vec![]));

        assert_eq!(plan.readiness, PlanReadiness::BlockedByApiLevel);
    }
}
