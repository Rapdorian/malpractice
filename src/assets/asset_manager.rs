use std::io::Read;
use std::path::Path;

/// Interface for loading assets from various sources
pub trait AssetManager {
    fn open(&self, path: impl AsRef<Path>) -> Option<impl Read>;

    fn read_string(&self, path: impl AsRef<Path>) -> Option<String> {
        let mut buffer = String::new();
        self.open(path)?.read_to_string(&mut buffer).ok()?;
            Some(buffer)
    }

    fn read_bytes(&self, path: impl AsRef<Path>) -> Option<Vec<u8>> {
        let mut buffer = Vec::new();
        self.open(path)?.read_to_end(&mut buffer).ok()?;
        Some(buffer)
    }
}
