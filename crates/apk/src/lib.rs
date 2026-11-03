use std::{
    fs::File,
    io,
    path::{Path, PathBuf},
};

use zip::ZipArchive;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeLibrary {
    pub path: String,
    pub abi: String,
    pub file_name: String,
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApkInfo {
    pub path: PathBuf,
    pub file_size: u64,
    pub package_name: Option<String>,
    pub manifest_kind: ManifestKind,
    pub dex_files: Vec<String>,
    pub native_libraries: Vec<NativeLibrary>,
    pub asset_count: usize,
    pub total_uncompressed_size: u64,
    pub entries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestKind {
    TextXml,
    BinaryAndroidXml,
    Missing,
}

impl std::fmt::Display for ManifestKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TextXml => write!(f, "Text XML"),
            Self::BinaryAndroidXml => write!(f, "Binary Android XML (AXML)"),
            Self::Missing => write!(f, "Missing"),
        }
    }
}

#[derive(Debug)]
pub enum ApkError {
    Io(io::Error),
    InvalidArchive(zip::result::ZipError),
    NotApk,
}

impl std::fmt::Display for ApkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "Could not read file: {e}"),
            Self::InvalidArchive(e) => write!(f, "Invalid APK/ZIP archive: {e}"),
            Self::NotApk => write!(f, "The selected file does not have an .apk extension"),
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

pub fn inspect_apk(path: impl AsRef<Path>) -> Result<ApkInfo, ApkError> {
    let path = path.as_ref();
    if !path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("apk"))
    {
        return Err(ApkError::NotApk);
    }

    let file_size = std::fs::metadata(path)?.len();
    let file = File::open(path)?;
    let mut archive = ZipArchive::new(file)?;

    let mut dex_files = Vec::new();
    let mut native_libraries = Vec::new();
    let mut asset_count = 0usize;
    let mut total_uncompressed_size = 0u64;
    let mut manifest_kind = ManifestKind::Missing;
    let mut package_name = None;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let name = entry.name().to_owned();
        let size = entry.size();
        total_uncompressed_size = total_uncompressed_size.saturating_add(size);

        if name == "AndroidManifest.xml" {
            let mut bytes = Vec::new();
            use std::io::Read;
            entry.read_to_end(&mut bytes)?;
            if bytes.starts_with(b"<?xml") || bytes.starts_with(b"<manifest") {
                manifest_kind = ManifestKind::TextXml;
                let xml = String::from_utf8_lossy(&bytes);
                package_name = extract_package_attribute(&xml);
            } else if bytes.len() >= 8 {
                manifest_kind = ManifestKind::BinaryAndroidXml;
            }
        } else if name == "classes.dex" || (name.starts_with("classes") && name.ends_with(".dex")) {
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

    dex_files.sort();
    native_libraries.sort_by(|a, b| a.path.cmp(&b.path));

    Ok(ApkInfo {
        path: path.to_path_buf(),
        file_size,
        package_name,
        manifest_kind,
        dex_files,
        native_libraries,
        asset_count,
        total_uncompressed_size,
        entries: archive.len(),
    })
}

fn extract_package_attribute(xml: &str) -> Option<String> {
    let start = xml.find("<manifest")?;
    let tag = &xml[start..xml[start..].find('>')? + start];
    let marker = "package=";
    let value_start = tag.find(marker)? + marker.len();
    let quote = tag.as_bytes().get(value_start).copied()?;
    if quote != 39 && quote != 34 {
        return None;
    }
    let value = &tag[value_start + 1..];
    let end = value.find(quote as char)?;
    Some(value[..end].to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_package_from_text_manifest() {
        assert_eq!(
            extract_package_attribute(r#"<manifest package="com.example.game">"#),
            Some("com.example.game".to_owned())
        );
    }

    #[test]
    fn ignores_manifest_without_package() {
        assert_eq!(extract_package_attribute("<manifest></manifest>"), None);
    }
}
