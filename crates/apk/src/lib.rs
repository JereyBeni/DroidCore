use std::{
    collections::BTreeSet,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
};

use zip::ZipArchive;

const MAX_METADATA_SIZE: u64 = 4 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLibrary {
    pub path: String,
    pub abi: String,
    pub file_name: String,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagePlatform {
    Android,
    AppleIos,
}

impl std::fmt::Display for PackagePlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Android => f.write_str("Android"),
            Self::AppleIos => f.write_str("iOS"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    pub path: PathBuf,
    pub platform: PackagePlatform,
    pub file_size: u64,
    pub package_name: Option<String>,
    pub display_name: Option<String>,
    pub version: Option<String>,
    pub minimum_os_version: Option<String>,
    pub manifest_kind: ManifestKind,
    pub dex_files: Vec<String>,
    pub native_libraries: Vec<NativeLibrary>,
    pub languages: Vec<String>,
    pub asset_count: usize,
    pub total_uncompressed_size: u64,
    pub entries: usize,
}

pub type ApkInfo = PackageInfo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestKind {
    TextXml,
    BinaryAndroidXml,
    XmlPlist,
    BinaryPlist,
    Missing,
}

impl std::fmt::Display for ManifestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TextXml => f.write_str("Text XML"),
            Self::BinaryAndroidXml => f.write_str("Binary Android XML (AXML)"),
            Self::XmlPlist => f.write_str("XML property list"),
            Self::BinaryPlist => f.write_str("Binary property list"),
            Self::Missing => f.write_str("Missing"),
        }
    }
}

#[derive(Debug)]
pub enum ApkError {
    Io(io::Error),
    InvalidArchive(zip::result::ZipError),
    NotApk,
    NotIpa,
    UnsupportedPackage,
}

impl std::fmt::Display for ApkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Could not read file: {e}"),
            Self::InvalidArchive(e) => write!(f, "Invalid package archive: {e}"),
            Self::NotApk => f.write_str("Expected an .apk Android package"),
            Self::NotIpa => f.write_str("Expected an .ipa iOS package"),
            Self::UnsupportedPackage => f.write_str("Only .apk and .ipa packages are supported"),
        }
    }
}

impl std::error::Error for ApkError {}

impl From<io::Error> for ApkError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<zip::result::ZipError> for ApkError {
    fn from(value: zip::result::ZipError) -> Self {
        Self::InvalidArchive(value)
    }
}

pub fn inspect_apk(path: impl AsRef<Path>) -> Result<PackageInfo, ApkError> {
    inspect_archive(path.as_ref(), PackagePlatform::Android)
}

pub fn inspect_ipa(path: impl AsRef<Path>) -> Result<PackageInfo, ApkError> {
    inspect_archive(path.as_ref(), PackagePlatform::AppleIos)
}

pub fn inspect_package(path: impl AsRef<Path>) -> Result<PackageInfo, ApkError> {
    let path = path.as_ref();
    match path.extension().and_then(|ext| ext.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("apk") => inspect_apk(path),
        Some(ext) if ext.eq_ignore_ascii_case("ipa") => inspect_ipa(path),
        _ => Err(ApkError::UnsupportedPackage),
    }
}

fn inspect_archive(path: &Path, platform: PackagePlatform) -> Result<PackageInfo, ApkError> {
    let expected_extension = match platform {
        PackagePlatform::Android => "apk",
        PackagePlatform::AppleIos => "ipa",
    };

    if !path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case(expected_extension))
    {
        return Err(match platform {
            PackagePlatform::Android => ApkError::NotApk,
            PackagePlatform::AppleIos => ApkError::NotIpa,
        });
    }

    let file_size = std::fs::metadata(path)?.len();
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut dex_files = Vec::new();
    let mut native_libraries = Vec::new();
    let mut languages = BTreeSet::new();
    let mut asset_count = 0usize;
    let mut total_uncompressed_size = 0u64;
    let mut manifest_kind = ManifestKind::Missing;
    let mut package_name = None;
    let mut display_name = None;
    let mut version = None;
    let mut minimum_os_version = None;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let size = entry.size();
        total_uncompressed_size = total_uncompressed_size.saturating_add(size);

        match platform {
            PackagePlatform::Android => {
                if let Some(locale) = android_locale_from_path(&name) {
                    languages.insert(locale);
                }

                if name == "AndroidManifest.xml" {
                    let mut bytes = Vec::new();
                    entry.take(MAX_METADATA_SIZE).read_to_end(&mut bytes)?;
                    if bytes.starts_with(b"<?xml") || bytes.starts_with(b"<manifest") {
                        manifest_kind = ManifestKind::TextXml;
                        let xml = String::from_utf8_lossy(&bytes);
                        package_name = extract_xml_attribute(&xml, "manifest", "package");
                    } else if bytes.len() >= 8 {
                        manifest_kind = ManifestKind::BinaryAndroidXml;
                    }
                } else if name == "classes.dex"
                    || (name.starts_with("classes") && name.ends_with(".dex"))
                {
                    dex_files.push(name);
                } else if name.starts_with("lib/") && name.ends_with(".so") {
                    let parts: Vec<&str> = name.split('/').collect();
                    if parts.len() >= 3 {
                        native_libraries.push(NativeLibrary {
                            path: name.clone(),
                            abi: parts[1].to_owned(),
                            file_name: parts.last().unwrap_or(&"").to_string(),
                            size,
                        });
                    }
                } else if name.starts_with("assets/") {
                    asset_count += 1;
                }
            }
            PackagePlatform::AppleIos => {
                if let Some(locale) = apple_locale_from_path(&name) {
                    languages.insert(locale);
                }

                if is_app_info_plist(&name) {
                    let mut bytes = Vec::new();
                    entry.take(MAX_METADATA_SIZE).read_to_end(&mut bytes)?;
                    if bytes.starts_with(b"bplist00") {
                        manifest_kind = ManifestKind::BinaryPlist;
                    } else {
                        let plist = String::from_utf8_lossy(&bytes);
                        if plist.contains("<plist") {
                            manifest_kind = ManifestKind::XmlPlist;
                            package_name = extract_plist_string(&plist, "CFBundleIdentifier");
                            display_name = extract_plist_string(&plist, "CFBundleDisplayName")
                                .or_else(|| extract_plist_string(&plist, "CFBundleName"));
                            version = extract_plist_string(&plist, "CFBundleShortVersionString")
                                .or_else(|| extract_plist_string(&plist, "CFBundleVersion"));
                            minimum_os_version = extract_plist_string(&plist, "MinimumOSVersion");
                            if let Some(locale) =
                                extract_plist_string(&plist, "CFBundleDevelopmentRegion")
                            {
                                languages.insert(normalize_apple_locale(&locale));
                            }
                        }
                    }
                } else if name.starts_with("Payload/")
                    && name.contains(".app/")
                    && (name.ends_with(".strings")
                        || name.ends_with(".storyboardc")
                        || name.ends_with(".nib"))
                {
                    asset_count += 1;
                }
            }
        }
    }

    dex_files.sort();
    native_libraries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(PackageInfo {
        path: path.to_path_buf(),
        platform,
        file_size,
        package_name,
        display_name,
        version,
        minimum_os_version,
        manifest_kind,
        dex_files,
        native_libraries,
        languages: languages.into_iter().collect(),
        asset_count,
        total_uncompressed_size,
        entries: archive.len(),
    })
}

fn extract_xml_attribute(xml: &str, tag_name: &str, attribute: &str) -> Option<String> {
    let start = xml.find(&format!("<{tag_name}"))?;
    let tag = &xml[start..start + xml[start..].find('>')?];
    let marker = format!("{attribute}=");
    let value_start = tag.find(&marker)? + marker.len();
    let quote = *tag.as_bytes().get(value_start)?;
    if quote != 39 && quote != 34 {
        return None;
    }
    let value = &tag[value_start + 1..];
    let end = value.find(quote as char)?;
    Some(value[..end].to_owned())
}

fn extract_plist_string(plist: &str, key: &str) -> Option<String> {
    let marker = format!("<key>{key}</key>");
    let key_start = plist.find(&marker)? + marker.len();
    let remainder = plist[key_start..].trim_start();
    let string_start = remainder.find("<string>")? + "<string>".len();
    let value = &remainder[string_start..];
    let end = value.find("</string>")?;
    let value = value[..end].trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn is_app_info_plist(path: &str) -> bool {
    path.starts_with("Payload/")
        && path.contains(".app/Info.plist")
        && path.ends_with(".app/Info.plist")
}

fn android_locale_from_path(path: &str) -> Option<String> {
    for component in path.split('/') {
        let Some(qualifier) = component.strip_prefix("values-") else {
            continue;
        };

        if let Some(bcp47) = qualifier.strip_prefix("b+") {
            return Some(bcp47.replace('+', "-"));
        }

        let mut parts = qualifier.split('-');
        let language = parts.next()?;
        if !(2..=3).contains(&language.len())
            || !language.chars().all(|ch| ch.is_ascii_alphabetic())
        {
            continue;
        }

        let mut locale = language.to_owned();
        for part in parts {
            if let Some(region) = part.strip_prefix('r') {
                if !region.is_empty() && region.chars().all(|ch| ch.is_ascii_alphanumeric()) {
                    locale.push('-');
                    locale.push_str(region);
                }
                break;
            }
        }
        return Some(locale);
    }

    None
}

fn apple_locale_from_path(path: &str) -> Option<String> {
    path.split('/')
        .find_map(|component| component.strip_suffix(".lproj"))
        .and_then(|locale| {
            if locale.eq_ignore_ascii_case("Base") {
                None
            } else {
                Some(normalize_apple_locale(locale))
            }
        })
}

fn normalize_apple_locale(locale: &str) -> String {
    let normalized = match locale {
        "English" => "en",
        "Spanish" => "es",
        "French" => "fr",
        "German" => "de",
        "Italian" => "it",
        "Japanese" => "ja",
        "Korean" => "ko",
        "Chinese" => "zh",
        "Portuguese" => "pt",
        "Russian" => "ru",
        "Arabic" => "ar",
        "Dutch" => "nl",
        "Swedish" => "sv",
        _ => locale,
    };
    normalized.replace('_', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_package_from_text_manifest() {
        assert_eq!(
            extract_xml_attribute(
                r#"<manifest package="com.example.game">"#,
                "manifest",
                "package"
            ),
            Some("com.example.game".to_owned())
        );
    }

    #[test]
    fn ignores_manifest_without_package() {
        assert_eq!(
            extract_xml_attribute("<manifest></manifest>", "manifest", "package"),
            None
        );
    }

    #[test]
    fn extracts_ipa_bundle_metadata_from_xml_plist() {
        let plist = r#"<plist><dict>
            <key>CFBundleIdentifier</key><string>org.example.game</string>
            <key>CFBundleName</key><string>Legacy Game</string>
            <key>CFBundleShortVersionString</key><string>1.2.3</string>
            <key>MinimumOSVersion</key><string>7.0</string>
        </dict></plist>"#;

        assert_eq!(
            extract_plist_string(plist, "CFBundleIdentifier"),
            Some("org.example.game".to_owned())
        );
        assert_eq!(
            extract_plist_string(plist, "CFBundleName"),
            Some("Legacy Game".to_owned())
        );
        assert_eq!(
            extract_plist_string(plist, "CFBundleShortVersionString"),
            Some("1.2.3".to_owned())
        );
        assert_eq!(
            extract_plist_string(plist, "MinimumOSVersion"),
            Some("7.0".to_owned())
        );
    }

    #[test]
    fn detects_android_resource_locales() {
        assert_eq!(
            android_locale_from_path("res/values-es-rAR/strings.xml"),
            Some("es-AR".to_owned())
        );
        assert_eq!(
            android_locale_from_path("res/values-b+zh+Hans+CN/strings.xml"),
            Some("zh-Hans-CN".to_owned())
        );
        assert_eq!(
            android_locale_from_path("res/values-night/styles.xml"),
            None
        );
    }

    #[test]
    fn detects_apple_localized_resource_bundles() {
        assert_eq!(
            apple_locale_from_path("Payload/Game.app/es-419.lproj/Localizable.strings"),
            Some("es-419".to_owned())
        );
        assert_eq!(
            apple_locale_from_path("Payload/Game.app/English.lproj/Main.strings"),
            Some("en".to_owned())
        );
        assert_eq!(
            apple_locale_from_path("Payload/Game.app/Base.lproj/Main.storyboard"),
            None
        );
    }
}
