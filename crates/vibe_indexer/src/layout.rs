use std::path::{Path, PathBuf};

pub const PRIMARY_SOURCE_EXT: &str = "yb";
pub const LEGACY_SOURCE_EXT: &str = "vibe";
pub const SUPPORTED_SOURCE_EXTS: &[&str] = &[PRIMARY_SOURCE_EXT, LEGACY_SOURCE_EXT];

pub const PRIMARY_METADATA_DIR: &str = ".yb";
pub const LEGACY_METADATA_DIR: &str = ".vibe";

pub fn is_supported_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(is_supported_source_ext)
}

pub fn is_supported_source_ext(ext: &str) -> bool {
    SUPPORTED_SOURCE_EXTS.contains(&ext)
}

pub fn metadata_root_for(base: &Path) -> PathBuf {
    base.join(PRIMARY_METADATA_DIR)
}

pub fn legacy_metadata_root_for(base: &Path) -> PathBuf {
    base.join(LEGACY_METADATA_DIR)
}
