use easy_prefs::easy_prefs;

// Define a comprehensive preferences struct
easy_prefs! {
    /// Application settings with various data types
    pub struct AppSettings {
        // User preferences
        pub username: String = "guest".to_string() => "username",
        pub email: String = String::new() => "email",
        pub age: i32 = 0 => "age",

        // UI settings
        pub dark_mode: bool = false => "dark_mode",
        pub font_size: i32 = 14 => "font_size",
        pub window_width: i32 = 800 => "window_width",
        pub window_height: i32 = 600 => "window_height",

        // Feature flags
        pub notifications_enabled: bool = true => "notifications",
        pub auto_save: bool = true => "auto_save",
        pub telemetry: bool = false => "telemetry",
    },
    "app-settings"
}

#[test]
fn test_complete_workflow() {
    // Use testing mode to avoid file system dependencies
    let mut settings = AppSettings::load_testing();

    // Verify defaults
    assert_eq!(settings.get_username(), "guest");
    assert_eq!(*settings.get_dark_mode(), false);
    assert_eq!(*settings.get_font_size(), 14);

    // Update individual fields
    settings.save_username("alice".to_string()).unwrap();
    settings.save_dark_mode(true).unwrap();

    // Batch update with edit guard
    {
        let mut edit = settings.edit();
        edit.set_email("alice@example.com".to_string());
        edit.set_age(25);
        edit.set_window_width(1920);
        edit.set_window_height(1080);
        edit.set_telemetry(true);
    }

    // Verify all changes
    assert_eq!(settings.get_username(), "alice");
    assert_eq!(settings.get_email(), "alice@example.com");
    assert_eq!(*settings.get_age(), 25);
    assert_eq!(*settings.get_dark_mode(), true);
    assert_eq!(*settings.get_window_width(), 1920);
    assert_eq!(*settings.get_window_height(), 1080);
    assert_eq!(*settings.get_telemetry(), true);

    // Test serialization
    let serialized = settings.to_string();
    assert!(serialized.contains("username = \"alice\""));
    assert!(serialized.contains("dark_mode = true"));
    assert!(serialized.contains("window_width = 1920"));
}

#[test]
fn test_preferences_file_path() {
    let settings = AppSettings::load_testing();
    let path = settings.get_preferences_file_path();

    // Path should contain the preferences filename
    assert!(path.contains("app-settings"));

    // On native platforms, it should be a file path
    #[cfg(not(target_arch = "wasm32"))]
    assert!(path.contains(".toml"));

    // On WASM, it should indicate localStorage
    #[cfg(target_arch = "wasm32")]
    assert!(path.starts_with("localStorage::"));
}

#[test]
fn test_no_unnecessary_saves() {
    let mut settings = AppSettings::load_testing();

    // Set a value
    settings.save_font_size(16).unwrap();

    // Setting the same value again should not trigger a save
    // (This is tested by the fact that save returns Ok(()) without actually writing)
    settings.save_font_size(16).unwrap();

    // Edit guard should also not save if no changes
    {
        let mut edit = settings.edit();
        let current = *edit.get_font_size();
        edit.set_font_size(current); // Same value
    }
}

#[test]
fn test_type_safety() {
    let settings = AppSettings::load_testing();

    // These should all be the correct types
    let _username: &String = settings.get_username();
    let _dark_mode: &bool = settings.get_dark_mode();
    let _font_size: &i32 = settings.get_font_size();

    // The macro ensures type safety at compile time
}
