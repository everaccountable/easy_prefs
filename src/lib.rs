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


// Re-export dependencies for convenience
pub use paste;       // Macro utilities
pub use toml;        // TOML serialization
pub use once_cell;   // Lazy statics

// IMPORTANT: Don't use these because the macro won't be able to see them.
// Instead, use fully qualified names wherever needed.
// use std::fmt;
// use std::io::Write;
// use std::path::PathBuf;
// use std::sync::atomic::{AtomicBool, Ordering};
// use once_cell::sync::Lazy;
// use directories::ProjectDirs;
// use tempfile::NamedTempFile;

/// Errors that can occur when loading preferences.
#[derive(Debug)]
pub enum LoadError {
    /// Another instance is already loaded (due to single-instance constraint).
    InstanceAlreadyLoaded,
    /// Failed to determine project directories (e.g., invalid namespace).
    ProjectDirsError(String),
    /// Failed to open the preferences file.
    FileOpenError(std::io::Error),
    /// Failed to read/write the file.
    FileReadError(std::io::Error),
    /// Failed to deserialize TOML data.
    DeserializationError(std::path::PathBuf, toml::de::Error),
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InstanceAlreadyLoaded => write!(f, "another preferences instance is already loaded"),
            Self::ProjectDirsError(msg) => write!(f, "project directories error: {}", msg),
            Self::FileOpenError(e) => write!(f, "file open error: {}", e),
            Self::FileReadError(e) => write!(f, "file read/write error: {}", e),
            Self::DeserializationError(path, e) => write!(f, "error: {}, file: {:?}", e, path),
        }
    }
}

impl std::error::Error for LoadError {}
/// Macro to define a preferences struct with file persistence.
///
/// Generates a struct with methods for loading, saving, and editing preferences.
/// Enforces a single instance (except in test mode) using a static flag.
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
            static [<$name _INSTANCE_EXISTS>]: $crate::once_cell::sync::Lazy<std::sync::atomic::AtomicBool> =
                $crate::once_cell::sync::Lazy::new(|| std::sync::atomic::AtomicBool::new(false));

            // Guard that resets the instance flag on drop.
            #[derive(Debug)]
            struct [<$name InstanceGuard>];
            impl Drop for [<$name InstanceGuard>] {
                fn drop(&mut self) {
                    [<$name _INSTANCE_EXISTS>].store(false, std::sync::atomic::Ordering::Release);
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
                full_path: Option<std::path::PathBuf>,
                #[serde(skip_serializing, skip_deserializing)]
                temp_file: Option<tempfile::NamedTempFile>,
                #[serde(skip_serializing, skip_deserializing)]
                _instance_guard: Option<[<$name InstanceGuard>]>,
            }

            impl Default for $name {
                fn default() -> Self {
                    Self {
                        $( [<_ $field>]: $default, )*
                        full_path: None,
                        temp_file: None,
                        _instance_guard: None,
                    }
                }
            }

            impl $name {
                const PREFERENCES_FILENAME: &'static str = concat!($preferences_filename, ".toml");

                /// Loads preferences from a file, enforcing the single-instance constraint.
                ///
                /// Deserializes from file if it exists; otherwise uses defaults.
                /// Only one instance can exist at a time (tracked by a static flag).
                ///
                /// # Arguments
                ///
                /// * `path` - The full path to the preferences file.
                ///
                /// # Errors
                ///
                /// Returns a `LoadError` if:
                /// - Another instance is already loaded.
                /// - The project directory cannot be determined.
                /// - File operations fail.
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

                    let was_free = [<$name _INSTANCE_EXISTS>].compare_exchange(
                        false, true, std::sync::atomic::Ordering::Acquire, std::sync::atomic::Ordering::Relaxed
                    );
                    if was_free.is_err() {
                        return Err($crate::LoadError::InstanceAlreadyLoaded);
                    }

                    let guard = [<$name InstanceGuard>];
                    let path = std::path::PathBuf::from(directory).join(Self::PREFERENCES_FILENAME);

                    let mut cfg = if path.exists() {
                        let mut file = std::fs::File::open(&path)
                            .map_err($crate::LoadError::FileOpenError)?;
                        let mut contents = String::new();
                        std::io::Read::read_to_string(&mut file, &mut contents)
                            .map_err($crate::LoadError::FileReadError)?;
                            match $crate::toml::from_str::<Self>(&contents) {
                                Ok(mut out) => { out.full_path = Some(path); out },
                                Err(e) => {
                                    return Err($crate::LoadError::DeserializationError(path.clone(), e));
                                }
                            }
                    } else {
                        let mut default = Self::default();
                        default.full_path = Some(path);
                        default
                    };
                    cfg._instance_guard = Some(guard);
                    Ok(cfg)
                }

                /// Loads preferences into a temporary file for testing (ignores the single-instance constraint).
                pub fn load_testing() -> Self {
                    let tmp_file = tempfile::NamedTempFile::with_prefix(Self::PREFERENCES_FILENAME)
                        .expect("Failed to create temporary file for testing preferences");
                    let path = tmp_file.path().to_path_buf();
                    let mut cfg = Self::default();
                    let mut file = std::fs::File::create(&path)
                        .expect("Failed to create preferences file for testing");
                    std::io::Write::write_all(&mut file, $crate::toml::to_string(&cfg).unwrap().as_bytes())
                        .expect("Failed to write preferences data to temporary file");
                    cfg.full_path = Some(path);
                    cfg.temp_file = Some(tmp_file);
                    cfg
                }

                /// Serializes preferences to a TOML string.
                pub fn to_string(&self) -> String {
                    $crate::toml::to_string(self).expect("Serialization failed")
                }

                /// Save the preferences data to the specified file path.
                ///
                /// This function serializes the preferences data to TOML format, writes it to a temporary file,
                /// and then persists that file to the final path. It ensures that parent directories exist
                /// and provides detailed error messages if any step fails.
                ///
                /// # Returns
                /// - `Ok(())` if the save operation succeeds.
                /// - `Err(std::io::Error)` if any step (e.g., directory creation, file writing, or persisting) fails.
                ///
                /// # Errors
                /// - Returns an error if the `full_path` is not set.
                /// - Returns an error if the parent directory cannot be determined or created.
                /// - Returns an error if serialization fails.
                /// - Returns an error if writing to the temporary file or persisting it fails.
                pub fn save(&self) -> Result<(), std::io::Error> {
                    // Ensure the full_path is set
                    let path = self.full_path.as_ref().ok_or_else(|| std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "path not set"
                    ))?;

                    // Get the parent directory and ensure it exists
                    let parent_dir = path.parent().ok_or_else(|| std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("no parent directory for '{}'", path.display())
                    ))?;
                    std::fs::create_dir_all(parent_dir)?;

                    // Create a temporary file in the parent directory
                    let mut tmp_file = tempfile::NamedTempFile::new_in(parent_dir)?;

                    // Serialize the preferences data to TOML
                    let serialized = $crate::toml::to_string(self).map_err(|e| std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("serialization failed: {}", e)
                    ))?;

                    // Write the serialized data to the temporary file
                    std::io::Write::write_all(&mut tmp_file, serialized.as_bytes())?;

                    // Persist the temporary file to the final path
                    tmp_file.persist(path).map_err(|persist_err| {
                        std::io::Error::new(
                            persist_err.error.kind(),
                            format!(
                                "failed to save '{}' (testing: {}): {}",
                                path.display(),
                                self.temp_file.is_some(),
                                persist_err.error
                            )
                        )
                    })?;

                    Ok(())
                }

                /// Returns the file path as a string.
                pub fn get_preferences_file_path(&self) -> String {
                    self.full_path.as_ref().expect("full_path must be set").to_str().unwrap().to_string()
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
                pub fn edit(&mut self) -> [<EditGuard_ $name>] {
                    [<EditGuard_ $name>] {
                        preferences: self,
                        modified: false,
                        created: std::time::Instant::now()
                    }
                }
            }

            /// Guard for batch editing; saves changes on drop if any fields were modified.
            struct [<EditGuard_ $name>]<'a> {
                preferences: &'a mut $name,
                modified: bool,
                created: std::time::Instant,
            }

            impl<'a> [<EditGuard_ $name>]<'a> {
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

            impl<'a> Drop for [<EditGuard_ $name>]<'a> {
                fn drop(&mut self) {
                    if cfg!(debug_assertions) && !std::thread::panicking() {
                        let duration = self.created.elapsed();
                        assert!(duration.as_millis() < 10, "Edit guard held too long ({:?})", duration);
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
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex, Barrier};
    use std::thread;
    use std::time::Duration;

    /// Tests loading and saving using `load_testing()` (ignores the single-instance constraint).
    #[test]
    fn test_load_save_preferences_with_macro() {
        let mut prefs = TestEasyPreferences::load_testing();
        assert_eq!(prefs.get_bool1_default_true(), &true);
        assert_eq!(prefs.get_int1(), &42);

        prefs.save_bool1_default_true(false).expect("Failed to save bool1");
        prefs.save_string1("hi".to_string()).expect("Failed to save string1");

        let contents = std::fs::read_to_string(prefs.get_preferences_file_path())
            .expect("Failed to read file");
        assert!(contents.contains("bool1_default_true = false"));
        assert!(contents.contains("string1 = \"hi\""));
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

        let contents = std::fs::read_to_string(prefs.get_preferences_file_path())
            .expect("Failed to read file");
        assert!(contents.contains("bool1_default_true = false"));
        assert!(contents.contains("int1 = 43"));
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
            prefs.save_bool1_default_true(false).expect("Failed to save bool1");
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
            let prefs = TestEasyPreferencesUpdated::load("/tmp/tests/").expect("Failed to load updated");
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
        let _prefs = TestEasyPreferences::load("com.example.app").expect("Failed to load after drop");

        // Verify that `load_testing()` ignores the single-instance constraint.
        let _test1 = TestEasyPreferences::load_testing();
        let _test2 = TestEasyPreferences::load_testing();
    }
}
