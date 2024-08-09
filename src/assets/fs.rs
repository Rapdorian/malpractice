use crate::assets::AssetManager;
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct FsManager {
    root: PathBuf,
}

impl FsManager {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        let root = root.into();
        log::info!(
            "Creating a Filesystem Asset Manager at: `{}`",
            root.display()
        );
        Self { root }
    }
}

impl AssetManager for FsManager {
    fn open(&self, path: impl AsRef<Path>) -> Option<impl Read> {
        let full_path = self.root.clone();
        let full_path = full_path.join(&path);
        match std::fs::File::open(&full_path) {
            Ok(asset) => Some(asset),
            Err(_) => {
                log::error!(
                    "Failed to load asset: {} from {}",
                    path.as_ref().display(),
                    full_path.display()
                );
                None
            }
        }
    }
}
