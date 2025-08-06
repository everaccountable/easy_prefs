use std::fmt::Debug;

/// Storage abstraction trait for cross-platform preferences storage
pub trait Storage: Send + Sync + Debug {
    /// Read data from storage
    fn read(&self, key: &str) -> Result<Option<String>, std::io::Error>;

    /// Write data to storage
    fn write(&self, key: &str, data: &str) -> Result<(), std::io::Error>;

    /// Get the full path/key for display purposes
    fn get_path(&self, key: &str) -> String;
}

#[cfg(not(target_arch = "wasm32"))]
pub mod native {
    use super::Storage;
    use std::io::{Read, Write};
    use std::path::PathBuf;

    #[derive(Debug)]
    pub struct FileStorage {
        base_dir: PathBuf,
    }

    impl FileStorage {
        pub fn new(directory: &str) -> Self {
            Self {
                base_dir: PathBuf::from(directory),
            }
        }
    }

    impl Storage for FileStorage {
        fn read(&self, key: &str) -> Result<Option<String>, std::io::Error> {
            let path = self.base_dir.join(key);

            if !path.exists() {
                return Ok(None);
            }

            let mut file = std::fs::File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;
            Ok(Some(contents))
        }

        fn write(&self, key: &str, data: &str) -> Result<(), std::io::Error> {
            let path = self.base_dir.join(key);

            // Ensure parent directory exists
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            // Write to temporary file first
            let parent_dir = path.parent().unwrap_or(&self.base_dir);
            let mut tmp_file = tempfile::NamedTempFile::new_in(parent_dir)?;
            tmp_file.write_all(data.as_bytes())?;

            // Atomically move temp file to final location
            tmp_file.persist(&path).map_err(|e| e.error)?;

            Ok(())
        }

        fn get_path(&self, key: &str) -> String {
            self.base_dir.join(key).display().to_string()
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use super::Storage;
    use web_sys::{window, Storage as WebStorage};

    #[derive(Debug)]
    pub struct LocalStorage {
        prefix: String,
    }

    impl LocalStorage {
        pub fn new(app_id: &str) -> Self {
            Self {
                prefix: format!("easy_prefs_{}_", app_id.replace('/', "_").replace('.', "_")),
            }
        }

        fn get_storage() -> Result<WebStorage, std::io::Error> {
            window()
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "window not available")
                })?
                .local_storage()
                .map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::Other, "localStorage not available")
                })?
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::Other, "localStorage is null")
                })
        }

        fn full_key(&self, key: &str) -> String {
            format!("{}{}", self.prefix, key)
        }
    }

    impl Storage for LocalStorage {
        fn read(&self, key: &str) -> Result<Option<String>, std::io::Error> {
            let storage = Self::get_storage()?;
            let full_key = self.full_key(key);

            storage.get_item(&full_key).map_err(|_| {
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "failed to read from localStorage",
                )
            })
        }

        fn write(&self, key: &str, data: &str) -> Result<(), std::io::Error> {
            let storage = Self::get_storage()?;
            let full_key = self.full_key(key);

            storage.set_item(&full_key, data).map_err(|_| {
                std::io::Error::new(std::io::ErrorKind::Other, "failed to write to localStorage")
            })
        }

        fn get_path(&self, key: &str) -> String {
            format!("localStorage::{}", self.full_key(key))
        }
    }
}

/// Platform-specific storage factory
#[cfg(not(target_arch = "wasm32"))]
pub fn create_storage(directory: &str) -> Box<dyn Storage> {
    Box::new(native::FileStorage::new(directory))
}

#[cfg(target_arch = "wasm32")]
pub fn create_storage(app_id: &str) -> Box<dyn Storage> {
    Box::new(wasm::LocalStorage::new(app_id))
}
