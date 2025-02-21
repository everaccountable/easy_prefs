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
//! - **Safety:** Writes are performed using a temporary file so that crashes won’t leave your data corrupted.
//! - **Performance:** Reading and writing are optimized for speed.
//! - **Easy Unit Testing:** The library is designed to play well with your unit tests.
//!
//! **Note:** This library is NOT intended to store large quantities of data. All data is cached in memory,
//! and the entire file is rewritten on each save. Use a full database for heavy data storage.
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
//!     let mut prefs = AppPreferences::load();
//!     println!("Notifications enabled: {}", prefs.get_notifications());
//!     prefs.save_notifications(false);
//! }
//! ```

pub use paste;  // Re-export paste so that users need not list it in their Cargo.toml.
pub use toml;


/// Macro to define an easy-to-use preferences struct with automatic serialization and file persistence.
///
/// # Overview
/// This macro generates a struct that is serializable (via TOML) and provides methods to load,
/// save, and edit preferences with a minimal API. It is designed to be thread safe and to protect
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
/// This library is not intended for large datasets. Use an actual database if you need to handle a lot of data.
///
/// # Example
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
            }

            impl Default for $name {
                fn default() -> Self {
                    $name {
                        $(
                            [<_ $field>]: $default,
                        )*
                        full_path: None,
                        temp_file: None,
                    }
                }
            }

            impl $name {
                const PREFERENCES_FILENAME: &'static str = concat!($preferences_filename, ".toml");

                /// Loads the preferences from the configuration file.
                ///
                /// If the file exists, it is deserialized; otherwise, the default values are used.
                /// Panics if it fails to read or deserialize the file.
                pub fn load() -> Self {
                    let project = directories::ProjectDirs::from("rs", "everaccountable", "").unwrap_or_else(|| {
                        panic!("Failed to get project directories");
                    });
                    let path = project.config_dir().join(Self::PREFERENCES_FILENAME);
                    println!("non-tauri Loading preferences from {:?}", path);
                    assert!(path.is_absolute(), "Path must be absolute: '{}'", path.display());
                    assert!(path.to_str().is_some(), "Path must be valid Unicode: '{:?}'", path);

                    let mut cfg: Self = {
                        if path.exists() {
                            let mut file = std::fs::File::open(&path).unwrap_or_else(|e| {
                                panic!("Failed to open preferences file: {}", e);
                            });
                            let mut contents = String::new();
                            std::io::Read::read_to_string(&mut file, &mut contents).unwrap_or_else(|e| {
                                panic!("Failed to read preferences file: {}", e);
                            });
                            let mut out = $crate::toml::from_str(&contents).unwrap_or_else(|e| {
                                eprintln!("Failed to deserialize preferences, using default: {}", e);
                                Self::default()
                            });
                            out.full_path = Some(path.to_path_buf());
                            out
                        } else {
                            let mut default = Self::default();
                            default.full_path = Some(path.to_path_buf());
                            default
                        }
                    };
                    debug_assert!(cfg.full_path.is_some(), "full_path must be set");
                    cfg
                }

                /// Loads preferences using a temporary file for testing purposes.
                ///
                /// This method creates a temporary file with the default preferences.
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
                    $crate::toml::to_string(self).expect("Unable to convert to toml")
                }

                /// Saves the current preferences to the file.
                ///
                /// This method writes to a temporary file first and then renames it to ensure the data
                /// is never left in a corrupt state.
                pub fn save(&self) {
                    debug_assert!(self.full_path.is_some(), "full_path must be set. Use ::load() to create preferences.");
                    debug_assert!(self.full_path.as_ref().unwrap().is_absolute(), "Path must be absolute: '{}'", self.full_path.as_ref().unwrap().display());
                    let parent_dir = self.full_path.as_ref().unwrap().parent().unwrap();
                    std::fs::create_dir_all(&parent_dir).unwrap();

                    let tmp_file = tempfile::NamedTempFile::new_in(parent_dir)
                        .unwrap_or_else(|e| panic!("Unable to create temporary file in '{}': {}", parent_dir.display(), e));

                    let mut file = std::fs::File::create(&tmp_file.path())
                        .expect("Unable to create temporary file");
                    std::io::Write::write_all(
                        &mut file,
                        $crate::toml::to_string(self).expect("Unable to convert to toml").as_bytes()
                    ).expect("Unable to write to temporary file");
                    std::fs::rename(tmp_file.path(), &self.full_path.as_ref().unwrap())
                        .expect("Unable to rename temporary file");
                }

                /// Returns the full path to the preferences file as a String.
                pub fn get_preferences_file_path(&self) -> String {
                    debug_assert!(self.full_path.is_some(), "full_path must be set");
                    self.full_path.as_ref().unwrap().to_str().unwrap().to_string()
                }

                $(
                    /// Returns a reference to the value of the field.
                    pub fn [<get_ $field>](&self) -> &$type {
                        &self.[<_ $field>]
                    }

                    /// Updates the field value and immediately saves the preferences file.
                    ///
                    /// If the new value is the same as the current one, no write is performed.
                    pub fn [<save_ $field>](&mut self, value: $type) {
                        if self.[<_ $field>] != value {
                            self.[<_ $field>] = value;
                            self.save();
                        }
                    }
                )*

                /// Returns an edit guard that allows multiple fields to be updated at once.
                ///
                /// All changes made through the edit guard are saved when it goes out of scope.
                ///
                /// # Example
                /// ```rust
                /// {
                ///     let mut edit_guard = prefs.edit();
                ///     edit_guard.set_bool_field(false);
                ///     edit_guard.set_string_field("new value".to_string());
                /// } // Preferences are saved here.
                /// ```
                pub fn edit(&mut self) -> [<EditGuard_ $name>] {
                    [<EditGuard_ $name>] {
                        preferences: self,
                        modified: false,
                        created: std::time::Instant::now()
                    }
                }
            }

            /// A guard that batches multiple changes to the preferences.
            ///
            /// When this guard is dropped, if any modifications were made, the preferences are saved.
            pub struct [<EditGuard_ $name>]<'a> {
                preferences: &'a mut $name,
                modified: bool,
                created: std::time::Instant,
            }

            impl<'a> [<EditGuard_ $name>]<'a> {
                $(
                    /// Sets the field value without immediately saving.
                    ///
                    /// The updated value will be persisted when the guard is dropped.
                    pub fn [<set_ $field>](&mut self, value: $type) {
                        if self.preferences.[<_ $field>] != value {
                            self.preferences.[<_ $field>] = value;
                            self.modified = true;
                        }
                    }

                    /// Returns a reference to the current value of the field.
                    pub fn [<get_ $field>](&self) -> &$type {
                        &self.preferences.[<_ $field>]
                    }
                )*
            }

            impl<'a> Drop for [<EditGuard_ $name>]<'a> {
                fn drop(&mut self) {
                    // In debug builds, assert that the guard is not held for too long.
                    // If the system is panicking, skip the check.
                    if cfg!(debug_assertions) && !std::thread::panicking() {
                        let duration = self.created.elapsed();
                        assert!(duration.as_millis() < 10, "Edit guard was held for too long ({:?}). Keep operations fast to avoid blocking other threads.", duration);
                        if self.modified {
                            self.preferences.save();
                        }
                    }
                }
            }
        }
    }
}


/// The following test structs simulate usage of the preferences API in debug builds.
#[cfg(debug_assertions)]
easy_prefs! {
    /// Test preferences for verifying the API functionality.
    struct TestEasyPreferences {
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
    }, "test-easy-preferences-file"
}

/// Simulates an upgraded preferences schema where a field is removed, renamed, or its default changed.
#[cfg(debug_assertions)]
easy_prefs! {
    /// Updated test preferences simulating an app upgrade.
    struct TestEasyPreferencesUpdated {
        // pub bool1_default_true: bool = true,  // This field was removed.
        /// This field was renamed from `bool2_default_true`.
        pub bool2_default_true_renamed: bool = true => "bool2_default_true",
        /// The default for this field has changed.
        pub bool3_initial_default_false: bool = true => "bool3_initial_default_false",
        /// A new boolean field added with a default of true.
        pub bool4_default_true: bool = true => "bool4_default_true",

        /// The default value for this field was updated.
        pub string1: String = "ea".to_string() => "string1",
        /// A new string field with its default value.
        pub string2: String = "new default value".to_string() => "string2",
    }, "test-easy-preferences-file"  // Same filename as the previous one.
}


#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};
    use super::*;

    /// Tests loading and saving preferences via the macro-generated API.
    #[test]
    fn test_load_save_preferences_with_macro() {
        let mut preferences = TestEasyPreferences::load_testing();

        // Verify default values.
        assert_eq!(preferences.get_bool1_default_true(), &true);
        assert_eq!(preferences.get_bool2_default_true(), &true);
        assert_eq!(preferences.get_bool3_initial_default_false(), &false);
        assert_eq!(preferences.get_string1(), &"");
        assert_eq!(preferences.get_int1(), &42);

        // Change values and save them.
        preferences.save_bool1_default_true(false);
        preferences.save_bool2_default_true(false);
        preferences.save_bool3_initial_default_false(true);
        preferences.save_string1("hi".to_string());

        // Check that the in-memory values were updated.
        assert_eq!(preferences.get_bool1_default_true(), &false);
        assert_eq!(preferences.get_bool2_default_true(), &false);
        assert_eq!(preferences.get_bool3_initial_default_false(), &true);
        assert_eq!(preferences.get_string1(), &"hi");

        // Read the file contents to ensure data was persisted.
        let mut file = std::fs::File::open(preferences.get_preferences_file_path())
            .expect("Unable to open preferences file");
        let mut contents = String::new();
        std::io::Read::read_to_string(&mut file, &mut contents)
            .expect("Unable to read preferences file");
        assert!(contents.contains("bool1_default_true"));
    }

    /// Verifies that the edit guard batches changes and persists them on drop.
    #[test]
    fn test_edit_guard() {
        let mut preferences = TestEasyPreferences::load_testing();
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

    /// Tests the usage of the preferences API in a multithreaded context.
    #[test]
    fn test_with_arc_mutex() {
        let preferences = Arc::new(Mutex::new(TestEasyPreferences::load_testing()));
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

    /// Tests persistence across loads and simulates an upgrade of the preferences schema.
    #[test]
    fn test_real_preferences() {
        // Delete any existing preferences file.
        let file_path = {
            let preferences = TestEasyPreferences::load();
            preferences.get_preferences_file_path()
        };
        let _ = std::fs::remove_file(&file_path);

        // Test 1: Write and reload preferences.
        {
            let mut preferences = TestEasyPreferences::load();
            preferences.save_bool1_default_true(false);
            {
                let mut edit_guard = preferences.edit();
                edit_guard.set_bool2_default_true(false);
                edit_guard.set_string1("test1".to_string());
            }
        }
        {
            let preferences = TestEasyPreferences::load();
            assert_eq!(preferences.get_bool1_default_true(), &false);
        }

        // Test 2: Simulate upgrading the preferences schema.
        {
            let preferences = TestEasyPreferencesUpdated::load();
            // The renamed field should preserve its value.
            assert_eq!(preferences.get_bool2_default_true_renamed(), &false);
            // The unchanged field should retain its saved value.
            assert_eq!(preferences.get_bool3_initial_default_false(), &false);
            // New field uses its default.
            assert_eq!(preferences.get_bool4_default_true(), &true);
            // String fields should reflect previously saved values or new defaults.
            assert_eq!(preferences.get_string1(), "test1");
            assert_eq!(preferences.get_string2(), "new default value");
        }
    }
}
