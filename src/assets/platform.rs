use crate::assets::AssetManager;
use std::env::current_exe;
use std::ffi::OsStr;
use std::path::PathBuf;

use crate::assets;
use crate::assets::fs::FsManager;

fn fs_data_manager() -> FsManager {
    FsManager::new(
        dirs::data_dir()
            .map(|p| p.join(env!("CARGO_PKG_NAME")))
            .unwrap_or(PathBuf::from("assets/")),
    )
}

fn cargo_dir() -> Option<FsManager> {
    // try to find the root directory
    let mut path = current_exe().unwrap_or(PathBuf::default());
    while !(path.is_dir() && path.file_name() == Some(OsStr::new("target"))) {
        path.pop();
    }
    path.parent()
        .map(|p| {
            env!("CARGO_MANIFEST_DIR")
                .contains(p.to_str().unwrap())
                .then_some(env!("CARGO_MANIFEST_DIR"))
        }).flatten()
        .map(|p| FsManager::new(p))
}

#[cfg(not(target_os = "android"))]
#[cfg(any(windows, unix, target_os = "macos"))]
pub fn os_asset_manager() -> impl AssetManager {
    cargo_dir().unwrap_or_else(|| fs_data_manager())
}

#[cfg(target_os = "android")]
pub fn os_asset_manager() -> impl AssetManager {
    use crate::assets::android::AndroidManager;
    AndroidManager::new()
}
