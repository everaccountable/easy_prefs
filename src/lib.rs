//! # easy_prefs
//!
//! A simple, safe, and performant preferences library for Rust applications.
//!
//! Created by Ever Accountable – an app that helps people overcome compulsive porn use
//! and become their best selves. Visit [everaccountable.com](https://everaccountable.com) for more details.
//!
//! This library provides an intuitive API for managing preferences using a struct-like interface.
//! Its key design goals are:
//!
//! - **Ease of Use**: Read/write preferences as easily as struct fields.
//! - **Safety**: Uses temporary files for writes to prevent data corruption.
//! - **Performance**: Optimized for fast operations.
//! - **Testability**: Integrates seamlessly with unit tests.
//! - **Cross-Platform**: Works on native platforms and WebAssembly (WASM).
//!
//! **Limitation**: Not suited for large datasets. All data is held in memory, and the entire file
//! is rewritten on save. For substantial data, use a database instead.
//!
//! ## Single-Instance Constraint
//!
//! The `load()` method enforces that only one instance of a preferences struct exists at a time,
//! using a static atomic flag. This prevents data races in production but can cause issues in
//! parallel test execution. Tests using `load()` are combined into a single test to avoid conflicts.
//!
//! ## Error Handling
//!
//! The `load()` function returns a `Result` with `LoadError` variants instead of panicking.
//! Errors include existing instances, directory issues, or file operation failures. See
//! [`load()`](#method.load) for details.
//!
//! ## WASM Support
//!
//! This library supports WebAssembly targets for use in browser extensions and web applications.
//! When compiled to WASM, preferences are stored in localStorage instead of the file system.

pub mod storage;

// Re-export dependencies for convenience
pub use once_cell;
pub use paste; // Macro utilities
pub use toml; // TOML serialization // Lazy statics

/// Errors that can occur when loading preferences.
#[derive(Debug)]
pub enum LoadError {
    /// Another instance is already loaded (due to single-instance constraint).
    InstanceAlreadyLoaded,
    /// Failed to deserialize TOML data.
    DeserializationError(String, toml::de::Error),
    /// Storage operation failed
    StorageError(std::io::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InstanceAlreadyLoaded => {
                write!(f, "another preferences instance is already loaded")
            }
            Self::DeserializationError(location, e) => {
                write!(f, "deserialization error: {e} at {location}")
            }
            Self::StorageError(e) => write!(f, "storage error: {e}"),
        }
    }
}

impl std::error::Error for LoadError {}
/// Macro to define a preferences struct with persistence.
///
/// Generates a struct with methods for loading, saving, and editing preferences.
/// Enforces a single instance (except in test mode) using a static flag.
///
/// # Example
///
/// ```rust
/// use easy_prefs::easy_prefs;
///
/// easy_prefs! {
///     pub struct AppPrefs {
///         pub dark_mode: bool = false => "dark_mode",
///         pub font_size: i32 = 14 => "font_size",
///     },
///     "app-settings"
/// }
/// ```
///
/// # Platform Behavior
///
/// - **Native**: Stores preferences as TOML files in the specified directory
/// - **WASM**: Stores preferences in browser localStorage
#[macro_export]
macro_rules! easy_prefs {
    (
        $(#[$outer:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$inner:meta])*
                $field_vis:vis $field:ident: $type:ty = $default:expr => $saved_name:expr,
            )*
        },
        $preferences_filename:expr
    ) => {
        $crate::paste::paste!{
            // Static flag to enforce single instance.
            static [<$name:upper _INSTANCE_EXISTS>]: $crate::once_cell::sync::Lazy<std::sync::atomic::AtomicBool> =
                $crate::once_cell::sync::Lazy::new(|| std::sync::atomic::AtomicBool::new(false));

            // Guard that resets the instance flag on drop.
            #[derive(Debug)]
            struct [<$name InstanceGuard>];
            impl Drop for [<$name InstanceGuard>] {
                fn drop(&mut self) {
                    [<$name:upper _INSTANCE_EXISTS>].store(false, std::sync::atomic::Ordering::Release);
                }
            }

            $(#[$outer])*
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            #[serde(default)]  // Use defaults for missing fields.
            $vis struct $name {
                $(
                    $(#[$inner])*
                    #[serde(rename = $saved_name)]
                    $field_vis [<_ $field>]: $type,
                )*
                #[serde(skip_serializing, skip_deserializing)]
                storage: Option<Box<dyn $crate::storage::Storage>>,
                #[serde(skip_serializing, skip_deserializing)]
                storage_key: Option<String>,
                #[serde(skip_serializing, skip_deserializing)]
                #[cfg(not(target_arch = "wasm32"))]
                temp_file: Option<tempfile::NamedTempFile>,
                #[serde(skip_serializing, skip_deserializing)]
                _instance_guard: Option<[<$name InstanceGuard>]>,
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $( [<_ $field>]: $default, )*
                        storage: None,
                        storage_key: None,
                        #[cfg(not(target_arch = "wasm32"))]
                        temp_file: None,
                        _instance_guard: None,
                    }
                }
            }

            impl $name {
                pub const PREFERENCES_FILENAME: &'static str = concat!($preferences_filename, ".toml");

                /// Loads preferences from a file, enforcing the single-instance constraint.
                ///
                /// Deserializes from file if it exists; otherwise uses defaults.
                /// Only one instance can exist at a time (tracked by a static flag).
                ///
                /// # Arguments
                ///
                /// * `directory` - The directory path (native) or app ID (WASM) where preferences are stored.
                ///
                /// # Errors
                ///
                /// Returns a `LoadError` if:
                /// - Another instance is already loaded.
                /// - Storage operations fail.
                /// - TOML deserialization fails.
                pub fn load(directory: &str) -> Result<Self, $crate::LoadError> {

                    {
                        // Runtime duplicate check for field_names. We don't want duplicates!
                        use std::collections::HashSet;
                        let keys = [ $( ($saved_name, stringify!($field) ), )* ];
                        let mut seen = HashSet::new();
                        for (key, field_name) in keys.iter() {
                            if !seen.insert(*key) {
                                panic!("Duplicate saved_name '{}' found for field '{}'", key, field_name);
                            }
                        }
                    }

                    let was_free = [<$name:upper _INSTANCE_EXISTS>].compare_exchange(
                        false, true, std::sync::atomic::Ordering::Acquire, std::sync::atomic::Ordering::Relaxed
                    );
                    if was_free.is_err() {
                        return Err($crate::LoadError::InstanceAlreadyLoaded);
                    }

                    let guard = [<$name InstanceGuard>];
                    let storage = $crate::storage::create_storage(directory);
                    let storage_key = Self::PREFERENCES_FILENAME;

                    let mut cfg = match storage.read(storage_key).map_err($crate::LoadError::StorageError)? {
                        Some(contents) => {
                            $crate::toml::from_str::<Self>(&contents)
                                .map_err(|e| $crate::LoadError::DeserializationError(
                                    storage.get_path(storage_key), e
                                ))?
                        }
                        None => Self::default(),
                    };

                    cfg.storage = Some(storage);
                    cfg.storage_key = Some(storage_key.to_string());
                    cfg._instance_guard = Some(guard);
                    Ok(cfg)
                }

                /// Creates a preferences instance with default values without loading from storage.
                ///
                /// This method bypasses the single-instance constraint and doesn't attempt to read
                /// from storage. The preferences will be saved to the specified directory when
                /// save() is called.
                ///
                /// # Arguments
                ///
                /// * `directory_or_app_id` - The directory path (native) or app ID (WASM)
                pub fn load_default(directory_or_app_id: &str) -> Self {
                    // Don't take the instance guard to allow multiple instances
                    let storage = $crate::storage::create_storage(directory_or_app_id);
                    let storage_key = Self::PREFERENCES_FILENAME;

                    let mut default = Self::default();
                    default.storage = Some(storage);
                    default.storage_key = Some(storage_key.to_string());
                    default._instance_guard = None; // No guard = bypasses single-instance constraint
                    default
                }

                /// Loads preferences into a temporary location for testing (ignores the single-instance constraint).
                #[cfg(not(target_arch = "wasm32"))]
                pub fn load_testing() -> Self {
                    let tmp_file = tempfile::NamedTempFile::with_prefix(Self::PREFERENCES_FILENAME)
                        .expect("Failed to create temporary file for testing preferences");
                    let tmp_dir = tmp_file.path().parent().unwrap().to_str().unwrap();
                    let storage = $crate::storage::create_storage(tmp_dir);
                    let storage_key = tmp_file.path().file_name().unwrap().to_str().unwrap();

                    let mut cfg = Self::default();
                    let serialized = $crate::toml::to_string(&cfg).unwrap();
                    storage.write(storage_key, &serialized)
                        .expect("Failed to write preferences data to temporary file");

                    cfg.storage = Some(storage);
                    cfg.storage_key = Some(storage_key.to_string());
                    cfg.temp_file = Some(tmp_file);
                    cfg
                }

                /// Loads preferences into a temporary location for testing (ignores the single-instance constraint).
                #[cfg(target_arch = "wasm32")]
                pub fn load_testing() -> Self {
                    let test_id = format!("test_{}", std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis());
                    let storage = $crate::storage::create_storage(&test_id);
                    let storage_key = Self::PREFERENCES_FILENAME;

                    let mut cfg = Self::default();
                    cfg.storage = Some(storage);
                    cfg.storage_key = Some(storage_key.to_string());
                    cfg
                }

                /// Serializes preferences to a TOML string.
                pub fn to_string(&self) -> String {
                    $crate::toml::to_string(self).expect("Serialization failed")
                }

                /// Save the preferences data to storage.
                ///
                /// This function serializes the preferences data to TOML format and writes it to storage.
                /// On native platforms, it uses atomic writes via temporary files. On WASM, it writes to localStorage.
                ///
                /// # Errors
                ///
                /// Returns an error if:
                /// - Storage is not initialized
                /// - Serialization fails
                /// - Storage write operation fails
                pub fn save(&self) -> Result<(), std::io::Error> {
                    // Ensure storage is initialized
                    let storage = self.storage.as_ref().ok_or_else(|| std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "storage not initialized"
                    ))?;

                    let storage_key = self.storage_key.as_ref().ok_or_else(|| std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "storage key not set"
                    ))?;

                    // Serialize the preferences data to TOML
                    let serialized = $crate::toml::to_string(self).map_err(|e| std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("serialization failed: {}", e)
                    ))?;

                    // Write to storage
                    storage.write(storage_key, &serialized)?;

                    Ok(())
                }

                /// Returns the storage path/key as a string.
                pub fn get_preferences_file_path(&self) -> String {
                    match (&self.storage, &self.storage_key) {
                        (Some(storage), Some(key)) => storage.get_path(key),
                        _ => panic!("storage not initialized"),
                    }
                }

                $(
                    /// Gets the value of the field.
                    pub fn [<get_ $field>](&self) -> &$type {
                        &self.[<_ $field>]
                    }

                    /// Sets the field's value and immediately saves.
                    pub fn [<save_ $field>](&mut self, value: $type) -> Result<(), std::io::Error> {
                        if self.[<_ $field>] != value {
                            self.[<_ $field>] = value;
                            self.save()
                        } else {
                            Ok(())
                        }
                    }
                )*

                /// Creates an edit guard for batching updates (saves on drop).
                pub fn edit(&mut self) -> [<$name EditGuard>] {
                    [<$name EditGuard>] {
                        preferences: self,
                        modified: false,
                        created: std::time::Instant::now()
                    }
                }
            }

            /// Guard for batch editing; saves changes on drop if any fields were modified.
            $vis struct [<$name EditGuard>]<'a> {
                preferences: &'a mut $name,
                modified: bool,
                created: std::time::Instant,
            }

            impl<'a> [<$name EditGuard>]<'a> {
                $(
                    /// Sets the field's value (save is deferred until the guard is dropped).
                    pub fn [<set_ $field>](&mut self, value: $type) {
                        if self.preferences.[<_ $field>] != value {
                            self.preferences.[<_ $field>] = value;
                            self.modified = true;
                        }
                    }

                    /// Gets the current value of the field.
                    pub fn [<get_ $field>](&self) -> &$type {
                        &self.preferences.[<_ $field>]
                    }
                )*
            }

            impl<'a> Drop for [<$name EditGuard>]<'a> {
                fn drop(&mut self) {
                    if cfg!(debug_assertions) && !std::thread::panicking() {
                        let duration = self.created.elapsed();
                        // Warn if edit guard is held for more than 1 second in debug mode
                        if duration.as_secs() >= 1 {
                            eprintln!("Warning: Edit guard held for {:?} - consider reducing the scope", duration);
                        }
                    }
                    if self.modified {
                        if let Err(e) = self.preferences.save() {
                            eprintln!("Failed to save: {}", e);
                        }
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier, Mutex};
    use std::thread;
    use std::time::Duration;

    #[cfg(debug_assertions)]
    easy_prefs! {
        /// Original test preferences.
        struct TestEasyPreferences {
            pub bool1_default_true: bool = true => "bool1_default_true",
            pub bool2_default_true: bool = true => "bool2_default_true",
            pub bool3_initial_default_false: bool = false => "bool3_initial_default_false",
            pub string1: String = String::new() => "string1",
            pub int1: i32 = 42 => "int1",
        }, "test-easy-prefs"
    }

    #[cfg(debug_assertions)]
    easy_prefs! {
        /// Updated test preferences for schema evolution.
        pub struct TestEasyPreferencesUpdated {
            pub bool2_default_true_renamed: bool = true => "bool2_default_true",
            pub bool3_initial_default_false: bool = true => "bool3_initial_default_false",
            pub bool4_default_true: bool = true => "bool4_default_true",
            pub string1: String = "ea".to_string() => "string1",
            pub string2: String = "new default value".to_string() => "string2",
        }, "test-easy-prefs"
    }

    /// Tests loading and saving using `load_testing()` (ignores the single-instance constraint).
    #[test]
    fn test_load_save_preferences_with_macro() {
        let mut prefs = TestEasyPreferences::load_testing();
        assert_eq!(prefs.get_bool1_default_true(), &true);
        assert_eq!(prefs.get_int1(), &42);

        prefs
            .save_bool1_default_true(false)
            .expect("Failed to save bool1");
        prefs
            .save_string1("hi".to_string())
            .expect("Failed to save string1");

        // Verify the values were saved
        let file_path = prefs.get_preferences_file_path();
        assert!(file_path.contains("test-easy-prefs"));
        // For native platforms, we can verify the file contents
        #[cfg(not(target_arch = "wasm32"))]
        {
            let contents = std::fs::read_to_string(&file_path).expect("Failed to read file");
            assert!(contents.contains("bool1_default_true = false"));
            assert!(contents.contains("string1 = \"hi\""));
        }
    }

    /// Tests the edit guard batching and save-on-drop functionality.
    #[test]
    fn test_edit_guard() {
        let mut prefs = TestEasyPreferences::load_testing();
        {
            let mut guard = prefs.edit();
            guard.set_bool1_default_true(false);
            guard.set_int1(43);
        }
        assert_eq!(prefs.get_bool1_default_true(), &false);
        assert_eq!(prefs.get_int1(), &43);

        // Verify the values were saved
        #[cfg(not(target_arch = "wasm32"))]
        {
            let contents = std::fs::read_to_string(prefs.get_preferences_file_path())
                .expect("Failed to read file");
            assert!(contents.contains("bool1_default_true = false"));
            assert!(contents.contains("int1 = 43"));
        }
    }

    /// Tests multithreading with Arc/Mutex using `load_testing()`.
    #[test]
    fn test_with_arc_mutex() {
        let prefs = Arc::new(Mutex::new(TestEasyPreferences::load_testing()));
        {
            let prefs = prefs.lock().unwrap();
            assert_eq!(prefs.get_int1(), &42);
        }
        {
            let mut prefs = prefs.lock().unwrap();
            prefs.save_int1(100).expect("Failed to save int1");
        }
        {
            let prefs = prefs.lock().unwrap();
            assert_eq!(prefs.get_int1(), &100);
        }
    }

    /// Combined test for real file operations and the single-instance constraint.
    ///
    /// Running these tests sequentially avoids conflicts caused by the single-instance flag.
    #[test]
    fn test_real_preferences_and_single_instance() {
        // --- Part 1: Test persistence and schema upgrades ---
        let path = {
            let prefs = TestEasyPreferences::load("/tmp/tests/").expect("Failed to load");
            prefs.get_preferences_file_path()
        };
        let _ = std::fs::remove_file(&path); // Clean up any previous run

        // Save some values.
        {
            let mut prefs = TestEasyPreferences::load("/tmp/tests/").expect("Failed to load");
            prefs
                .save_bool1_default_true(false)
                .expect("Failed to save bool1");
            prefs.edit().set_string1("test1".to_string());
        }
        // Verify persistence.
        {
            let prefs = TestEasyPreferences::load("/tmp/tests/").expect("Failed to load");
            assert_eq!(prefs.get_bool1_default_true(), &false);
            assert_eq!(prefs.get_string1(), "test1");
        }
        // Test schema evolution.
        {
            let prefs =
                TestEasyPreferencesUpdated::load("/tmp/tests/").expect("Failed to load updated");
            assert_eq!(prefs.get_bool2_default_true_renamed(), &true); // Default (not saved earlier)
            assert_eq!(prefs.get_string1(), "test1");
            assert_eq!(prefs.get_string2(), "new default value");
        }

        // --- Part 2: Test the single-instance constraint ---
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();

        let handle = thread::spawn(move || {
            let prefs = TestEasyPreferences::load("/tmp/tests/").expect("Failed to load");
            barrier_clone.wait(); // Hold instance until main thread tries to load.
            thread::sleep(Duration::from_millis(100));
            drop(prefs); // Release instance.
            true
        });

        barrier.wait(); // Synchronize with spawned thread.
        let result = TestEasyPreferences::load("com.example.app");
        assert!(matches!(result, Err(LoadError::InstanceAlreadyLoaded)));

        handle.join().unwrap(); // Wait for thread to finish.

        // Verify instance can be loaded after release.
        let _prefs =
            TestEasyPreferences::load("com.example.app").expect("Failed to load after drop");

        // Verify that `load_testing()` ignores the single-instance constraint.
        let _test1 = TestEasyPreferences::load_testing();
        let _test2 = TestEasyPreferences::load_testing();
    }
}
