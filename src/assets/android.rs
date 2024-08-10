use std::cell::OnceCell;
use std::ffi::{CStr, CString};
use std::io::Read;
use std::path::Path;
use std::sync::OnceLock;
use crate::assets::AssetManager;

pub static ANDROID_APP: OnceLock<winit::platform::android::activity::AndroidApp> = OnceLock::new();

pub struct AndroidManager {
    mgr: ndk::asset::AssetManager
}

impl AndroidManager {
    pub fn new() -> Self {
        let app = ANDROID_APP.get().unwrap();
        Self {mgr: app.asset_manager()}
    }
}

impl AssetManager for AndroidManager {
    fn open(&self, path: impl AsRef<Path>) -> Option<impl Read> {
        let path = Path::new("assets").join(path.as_ref());
        log::warn!("Attempting to load {}", path.display());
        let path = CString::new(path.to_str().unwrap()).unwrap();
        self.mgr.open(&path)
    }
}