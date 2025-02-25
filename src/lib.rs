//! # easy_prefs
//!
//! A simple, safe, and performant preferences library for Rust applications.
//!
//! Created by Ever Accountable – an app that helps people quit compulsive porn use
//! and become the best version of themselves. More information is available at [everaccountable.com](https://everaccountable.com).
//!
//! This library provides an easy-to-use API for reading and writing preferences, using a struct-like interface.
//! The design priorities are:
//!
//! - **Ease of use:** Read and write operations are as simple as setting or getting a struct field.
//! - **Safety:** Writes are performed using a temporary file so that crashes won't leave your data corrupted.
//! - **Performance:** Reading and writing are optimized for speed.
//! - **Easy Unit Testing:** The library is designed to play well with your unit tests.
//!
//! **Note:** This library is NOT intended to store large quantities of data. All data is cached in memory,
//! and the entire file is rewritten on each save. Use a full database for heavy data storage.
//!
//! **Error Handling:** This library uses panics for simplicity. For production use, consider wrapping calls
//! in error-handling logic if graceful failure is required.
//!
//! ## Example
//!
//! ```rust
//! use easy_prefs::easy_prefs;
//!
//! easy_prefs! {
//!     /// Application preferences.
//!     pub struct AppPreferences {
//!         /// Whether notifications are enabled.
//!         pub notifications: bool = true => "notifications",
//!         /// The default username.
//!         pub username: String = "guest".to_string() => "username",
//!     },
//!     "app-preferences"
//! }
//!
//! fn main() {
//!     let mut prefs = AppPreferences::load("com.example.App");
//!     println!("Notifications enabled: {}", prefs.get_notifications());
//!     prefs.save_notifications(false);
//! }
//! ```

// Re-export dependencies so users don't need to add them to their Cargo.toml
pub use paste;  // For macro pasting utilities
pub use toml;   // For TOML serialization/deserialization
pub use once_cell;  // For lazy static initialization

/// Macro to define an easy-to-use preferences struct with automatic serialization and file persistence.
///
/// # Overview
/// This macro generates a struct that is serializable (via TOML) and provides methods to load,
/// save, and edit preferences with a minimal API. It is designed to be thread-safe and to protect
/// against file corruption by writing data to a temporary file before renaming it into place.
///
/// # Parameters
/// - A list of fields including their types, default values, and the keys to use when serializing.
/// - The base name for the preferences file (a `.toml` file is created in the configuration directory).
///
/// # Design Priorities
/// - **Ease of Use:** Reading and writing should be as natural as accessing struct properties.
/// - **Idiomatic Rust:** Defining the struct with defaults should be simple and clear.
/// - **Safety:** Ensures that if an app crashes during a write, the file will not be corrupted.
/// - **Performance:** Optimized for fast read and write operations.
///
/// # Limitations
/// - Not intended for large datasets. Use a database for heavy data storage.
/// - Uses `namespace` as the qualifier in `ProjectDirs::from(namespace, "", "")`. For standard paths,
///   provide a namespace like `"com.example.App"`.
///
/// # Example
///
/// ```rust
/// use easy_prefs::easy_prefs;
///
/// easy_prefs! {
///     /// Example preferences for an application.
///     pub struct AppPreferences {
///         /// Whether dark mode is enabled.
///         pub dark_mode: bool = false => "dark_mode",
///         /// Preferred language.
///         pub language: String = "en".to_string() => "language",
///     },
///     "app-preferences"
/// }
/// ```
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
            // Static atomic flag to enforce single-instance constraint per struct type
            static [<$name _INSTANCE_EXISTS>]: $crate::once_cell::sync::Lazy<std::sync::atomic::AtomicBool> =
                $crate::once_cell::sync::Lazy::new(|| std::sync::atomic::AtomicBool::new(false));

            // Guard struct to release the instance flag when dropped, with Debug for compatibility
            $vis #[derive(Debug)] struct [<$name InstanceGuard>];

            impl Drop for [<$name InstanceGuard>] {
                fn drop(&mut self) {
                    [<$name _INSTANCE_EXISTS>].store(false, std::sync::atomic::Ordering::Release);
                }
            }

            $(#[$outer])*
            #[derive(serde::Serialize, serde::Deserialize, Debug)]
            #[serde(default)]  // Apply default values to newly added fields
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
                    $name {
                        $(
                            [<_ $field>]: $default,
                        )*
                        full_path: None,
                        temp_file: None,
                        _instance_guard: None,
                    }
                }
            }

            impl $name {
                const PREFERENCES_FILENAME: &'static str = concat!($preferences_filename, ".toml");

                /// Loads preferences from the configuration file.
                /// Panics if another instance exists or if file operations fail.
                pub fn load(namespace: &str) -> Self {
                    let was_free = [<$name _INSTANCE_EXISTS>].compare_exchange(
                        false, true,
                        std::sync::atomic::Ordering::Acquire,
                        std::sync::atomic::Ordering::Relaxed
                    );

                    if was_free.is_err() {
                        panic!("Another instance of {} is already loaded.", stringify!($name));
                    }

                    let guard = [<$name InstanceGuard>];
                    let project = directories::ProjectDirs::from(namespace, "", "").unwrap_or_else(|| {
                        panic!("Failed to get project directories");
                    });
                    let path = project.config_dir().join(Self::PREFERENCES_FILENAME);
                    println!("non-tauri Loading preferences from {:?}", path);
                    assert!(path.is_absolute(), "Path must be absolute: '{}'", path.display());
                    assert!(path.to_str().is_some(), "Path must be valid Unicode: '{:?}'", path);

                    let mut cfg: Self = if path.exists() {
                        match std::fs::File::open(&path) {
                            Ok(mut file) => {
                                let mut contents = String::new();
                                std::io::Read::read_to_string(&mut file, &mut contents)
                                    .unwrap_or_else(|e| panic!("Failed to read preferences file: {}", e));
                                match $crate::toml::from_str::<Self>(&contents) {
                                    Ok(mut out) => {
                                        out.full_path = Some(path.to_path_buf());
                                        out
                                    },
                                    Err(e) => {
                                        eprintln!("Failed to deserialize preferences, using default: {}", e);
                                        let mut default = Self::default();
                                        default.full_path = Some(path.to_path_buf());
                                        default
                                    }
                                }
                            },
                            Err(e) => panic!("Failed to open preferences file: {}", e),
                        }
                    } else {
                        let mut default = Self::default();
                        default.full_path = Some(path.to_path_buf());
                        default
                    };
                    debug_assert!(cfg.full_path.is_some(), "full_path must be set");
                    cfg._instance_guard = Some(guard);
                    cfg
                }

                /// Loads preferences using a temporary file for testing, bypassing the instance flag.
                pub fn load_testing() -> Self {
                    let tmp_file = tempfile::NamedTempFile::with_prefix(Self::PREFERENCES_FILENAME)
                        .expect("Unable to create temporary file");
                    let preferences_path = tmp_file.path();
                    let mut cfg: Self = {
                        let mut file = std::fs::File::create(&preferences_path)
                            .expect("Unable to create temporary file");
                        std::io::Write::write_all(
                            &mut file,
                            $crate::toml::to_string(&Self::default()).unwrap().as_bytes()
                        ).expect("Unable to write default preferences to file");
                        Self::default()
                    };
                    cfg.full_path = Some(preferences_path.to_path_buf());
                    cfg.temp_file = Some(tmp_file);
                    debug_assert!(cfg.full_path.is_some(), "full_path must be set");
                    cfg
                }

                /// Returns a TOML-formatted string representation of the preferences.
                pub fn to_string(&self) -> String {
                    $crate::toml::to_string(self).expect("Unable to convert to TOML")
                }

                /// Saves preferences to the file using a temporary file for safety.
                pub fn save(&self) {
                    debug_assert!(self.full_path.is_some(), "full_path must be set. Use ::load() to create preferences.");
                    let full_path = self.full_path.as_ref().unwrap();
                    debug_assert!(full_path.is_absolute(), "Path must be absolute: '{}'", full_path.display());
                    let parent_dir = full_path.parent().unwrap();
                    std::fs::create_dir_all(&parent_dir).unwrap();

                    let tmp_file = tempfile::NamedTempFile::new_in(parent_dir)
                        .unwrap_or_else(|e| panic!("Unable to create temp file in '{}': {}", parent_dir.display(), e));
                    let mut file = std::fs::File::create(tmp_file.path())
                        .expect("Unable to create temporary file");
                    std::io::Write::write_all(
                        &mut file,
                        $crate::toml::to_string(self).expect("Unable to convert to TOML").as_bytes()
                    ).expect("Unable to write to temporary file");
                    std::fs::rename(tmp_file.path(), full_path)
                        .expect("Unable to rename temporary file");
                }

                /// Returns the full path to the preferences file as a String.
                pub fn get_preferences_file_path(&self) -> String {
                    debug_assert!(self.full_path.is_some(), "full_path must be set");
                    self.full_path.as_ref().unwrap().to_str().unwrap().to_string()
                }

                $(
                    /// Returns a reference to the field value.
                    pub fn [<get_ $field>](&self) -> &$type {
                        &self.[<_ $field>]
                    }

                    /// Updates the field value and saves immediately.
                    pub fn [<save_ $field>](&mut self, value: $type) {
                        if self.[<_ $field>] != value {
                            self.[<_ $field>] = value;
                            self.save();
                        }
                    }
                )*

                /// Returns an edit guard for batching multiple updates, saved on drop.
                pub fn edit(&mut self) -> [<EditGuard_ $name>] {
                    [<EditGuard_ $name>] {
                        preferences: self,
                        modified: false,
                        created: std::time::Instant::now()
                    }
                }
            }

            /// Guard struct for batching changes, saves on drop if modified.
            pub struct [<EditGuard_ $name>]<'a> {
                preferences: &'a mut $name,
                modified: bool,
                created: std::time::Instant,
            }

            impl<'a> [<EditGuard_ $name>]<'a> {
                $(
                    /// Sets a field value without immediate save, marks as modified if changed.
                    pub fn [<set_ $field>](&mut self, value: $type) {
                        if self.preferences.[<_ $field>] != value {
                            self.preferences.[<_ $field>] = value;
                            self.modified = true;
                        }
                    }

                    /// Returns a reference to the current field value.
                    pub fn [<get_ $field>](&self) -> &$type {
                        &self.preferences.[<_ $field>]
                    }
                )*
            }

            impl<'a> Drop for [<EditGuard_ $name>]<'a> {
                fn drop(&mut self) {
                    if cfg!(debug_assertions) && !std::thread::panicking() {
                        let duration = self.created.elapsed();
                        assert!(duration.as_millis() < 10, "Edit guard held too long ({:?}).", duration);
                    }
                    if self.modified {
                        self.preferences.save();
                    }
                }
            }
        }
    }
}

// Test structs for verifying the preferences API in debug builds
#[cfg(debug_assertions)]
easy_prefs! {
    /// Test preferences for verifying API functionality and real file operations.
    struct TestEasyPreferencesReal {
        /// A boolean field defaulting to true.
        pub bool1_default_true: bool = true => "bool1_default_true",
        /// Another boolean field defaulting to true.
        pub bool2_default_true: bool = true => "bool2_default_true",
        /// A boolean field initially false.
        pub bool3_initial_default_false: bool = false => "bool3_initial_default_false",
        /// A string field with an empty default.
        pub string1: String = String::new() => "string1",
        /// An integer field with a default value.
        pub int1: i32 = 42 => "int1",
    }, "test-easy-preferences-file-real"
}

#[cfg(debug_assertions)]
easy_prefs! {
    /// Updated test preferences simulating an app upgrade, sharing the same file as TestEasyPreferencesReal.
    struct TestEasyPreferencesUpdatedReal {
        /// Renamed from `bool2_default_true`.
        pub bool2_default_true_renamed: bool = true => "bool2_default_true",
        /// Default changed to true.
        pub bool3_initial_default_false: bool = true => "bool3_initial_default_false",
        /// New boolean field.
        pub bool4_default_true: bool = true => "bool4_default_true",
        /// Default updated.
        pub string1: String = "ea".to_string() => "string1",
        /// New string field.
        pub string2: String = "new default value".to_string() => "string2",
    }, "test-easy-preferences-file-real"
}

#[cfg(debug_assertions)]
easy_prefs! {
    /// Test preferences for verifying single-instance constraint behavior.
    struct TestEasyPreferencesSingle {
        pub bool1_default_true: bool = true => "bool1_default_true",
        pub bool2_default_true: bool = true => "bool2_default_true",
        pub bool3_initial_default_false: bool = false => "bool3_initial_default_false",
        pub string1: String = String::new() => "string1",
        pub int1: i32 = 42 => "int1",
    }, "test-easy-preferences-file-single"
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex, Barrier};
    use std::thread;
    use std::time::Duration;
    use std::io::Read;

    /// Tests loading and saving preferences via the macro-generated API.
    /// Uses `load_testing()` to avoid file-based conflicts with other tests.
    #[test]
    fn test_load_save_preferences_with_macro() {
        let mut preferences = TestEasyPreferencesReal::load_testing();
        assert_eq!(preferences.get_bool1_default_true(), &true);
        assert_eq!(preferences.get_bool2_default_true(), &true);
        assert_eq!(preferences.get_bool3_initial_default_false(), &false);
        assert_eq!(preferences.get_string1(), &"");
        assert_eq!(preferences.get_int1(), &42);

        preferences.save_bool1_default_true(false);
        preferences.save_bool2_default_true(false);
        preferences.save_bool3_initial_default_false(true);
        preferences.save_string1("hi".to_string());

        assert_eq!(preferences.get_bool1_default_true(), &false);
        assert_eq!(preferences.get_bool2_default_true(), &false);
        assert_eq!(preferences.get_bool3_initial_default_false(), &true);
        assert_eq!(preferences.get_string1(), &"hi");

        let mut file = std::fs::File::open(preferences.get_preferences_file_path())
            .expect("Unable to open preferences file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .expect("Unable to read preferences file");
        assert!(contents.contains("bool1_default_true"));
    }

    /// Verifies that the edit guard batches changes and persists them on drop.
    /// Uses `load_testing()` for in-memory testing, avoiding file system interference.
    #[test]
    fn test_edit_guard() {
        let mut preferences = TestEasyPreferencesReal::load_testing();
        {
            let mut edit_guard = preferences.edit();
            edit_guard.set_bool1_default_true(false);
            edit_guard.set_int1(43);
            edit_guard.set_bool3_initial_default_false(true);
            edit_guard.set_string1("hi".to_string());
        }
        assert_eq!(preferences.get_bool1_default_true(), &false);
        assert_eq!(preferences.get_int1(), &43);
        assert_eq!(preferences.get_bool3_initial_default_false(), &true);
        assert_eq!(preferences.get_string1(), &"hi");
    }

    /// Tests the preferences API in a multithreaded context using Arc and Mutex.
    /// Uses `load_testing()` to ensure no file contention occurs.
    #[test]
    fn test_with_arc_mutex() {
        let preferences = Arc::new(Mutex::new(TestEasyPreferencesReal::load_testing()));
        {
            let prefs = preferences.lock().unwrap();
            assert_eq!(prefs.get_int1(), &42i32);
        }
        {
            let mut prefs = preferences.lock().unwrap();
            prefs.save_bool2_default_true(false);
            prefs.save_bool3_initial_default_false(true);
            prefs.save_string1("hi".to_string());
        }
    }

    /// Combined test for real file operations and single-instance constraint.
    /// **Why Combined:** These tests both use `load()`, which relies on a static atomic flag to enforce a single instance.
    /// Running them separately in parallel could cause conflicts due to shared static state, leading to hangs or failures.
    /// By combining them into a single test and running sequentially, we ensure each section completes fully before the next begins,
    /// avoiding interference while still verifying all intended behavior.
    #[test]
    fn test_real_preferences_and_single_instance() {
        // --- Section 1: Test persistence across loads and schema upgrades ---
        // Clean up any existing file to start fresh
        let file_path = {
            let preferences = TestEasyPreferencesReal::load("com.example.app.real");
            preferences.get_preferences_file_path()
        };
        let _ = std::fs::remove_file(&file_path);

        // Write initial preferences
        {
            let mut preferences = TestEasyPreferencesReal::load("com.example.app.real");
            preferences.save_bool1_default_true(false);
            {
                let mut edit_guard = preferences.edit();
                edit_guard.set_bool2_default_true(false);
                edit_guard.set_string1("test1".to_string());
            }
        }

        // Verify persistence with the same struct
        {
            let preferences = TestEasyPreferencesReal::load("com.example.app.real");
            assert_eq!(preferences.get_bool1_default_true(), &false);
        }

        // Verify schema upgrade with updated struct
        {
            let preferences = TestEasyPreferencesUpdatedReal::load("com.example.app.real");
            assert_eq!(preferences.get_bool2_default_true_renamed(), &false);
            assert_eq!(preferences.get_bool3_initial_default_false(), &false);
            assert_eq!(preferences.get_bool4_default_true(), &true);
            assert_eq!(preferences.get_string1(), "test1");
            assert_eq!(preferences.get_string2(), "new default value");
        }

        // --- Section 2: Test single-instance constraint ---
        let barrier = Arc::new(Barrier::new(2));
        let barrier_clone = barrier.clone();
        let handle = thread::spawn(move || {
            let preferences = TestEasyPreferencesSingle::load("com.example.app.single");
            barrier_clone.wait(); // Wait until the main thread is ready
            thread::sleep(Duration::from_millis(100));
            drop(preferences);
            true
        });

        barrier.wait(); // Ensure the spawned thread has loaded the instance
        let second_result = std::panic::catch_unwind(|| {
            let _second_instance = TestEasyPreferencesSingle::load("com.example.app.single");
        });
        assert!(second_result.is_err(), "Expected panic when loading a second instance");

        assert_eq!(handle.join().unwrap(), true, "Thread should complete successfully");
        let _new_instance = TestEasyPreferencesSingle::load("com.example.app.single");

        // Verify that load_testing() allows multiple instances without interference
        let test_instance1 = TestEasyPreferencesSingle::load_testing();
        let test_instance2 = TestEasyPreferencesSingle::load_testing();
        assert!(
            test_instance1.get_preferences_file_path() != test_instance2.get_preferences_file_path(),
            "load_testing() should generate unique file paths"
        );
    }
}