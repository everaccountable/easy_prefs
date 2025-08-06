use easy_prefs::easy_prefs;
use std::time::Duration;

easy_prefs! {
    pub struct EditGuardTestPrefs {
        pub value1: i32 = 0 => "value1",
        pub value2: String = String::new() => "value2",
        pub value3: bool = false => "value3",
    },
    "edit-guard-test"
}

#[test]
fn test_edit_guard_saves_on_drop() {
    let mut prefs = EditGuardTestPrefs::load_testing();

    // Use edit guard to make changes
    {
        let mut edit = prefs.edit();
        edit.set_value1(100);
        edit.set_value2("modified".to_string());
        edit.set_value3(true);
        // Guard drops here and should save
    }

    // Verify values were saved
    let file_path = prefs.get_preferences_file_path();

    // For native platforms, verify by reading the file
    #[cfg(not(target_arch = "wasm32"))]
    {
        let contents = std::fs::read_to_string(&file_path).unwrap();
        assert!(contents.contains("value1 = 100"));
        assert!(contents.contains("value2 = \"modified\""));
        assert!(contents.contains("value3 = true"));
    }
}

#[test]
fn test_edit_guard_no_save_without_changes() {
    let mut prefs = EditGuardTestPrefs::load_testing();

    // Set initial values
    prefs.save_value1(50).unwrap();

    // Get the file path before edit
    let file_path = prefs.get_preferences_file_path();

    // Get initial modification time
    #[cfg(not(target_arch = "wasm32"))]
    let initial_mtime = std::fs::metadata(&file_path).unwrap().modified().unwrap();

    // Use edit guard but don't change anything
    {
        let edit = prefs.edit();
        // Just read values
        let _ = edit.get_value1();
        let _ = edit.get_value2();
    }

    // On native platforms, verify file wasn't modified
    #[cfg(not(target_arch = "wasm32"))]
    {
        let new_mtime = std::fs::metadata(&file_path).unwrap().modified().unwrap();
        assert_eq!(
            initial_mtime, new_mtime,
            "File should not be modified without changes"
        );
    }
}

#[test]
fn test_edit_guard_only_saves_on_actual_changes() {
    let mut prefs = EditGuardTestPrefs::load_testing();

    // Set initial value
    prefs.save_value1(42).unwrap();

    // Use edit guard to set same value
    {
        let mut edit = prefs.edit();
        edit.set_value1(42); // Same value
                             // Should not mark as modified
    }

    // Now make an actual change
    {
        let mut edit = prefs.edit();
        edit.set_value1(43); // Different value
                             // Should mark as modified and save
    }

    // Verify the change
    assert_eq!(*prefs.get_value1(), 43);
}

#[test]
#[cfg(debug_assertions)]
fn test_edit_guard_warning_on_long_hold() {
    let mut prefs = EditGuardTestPrefs::load_testing();

    // This should print a warning in debug mode
    {
        let mut edit = prefs.edit();
        edit.set_value1(100);

        // Hold the guard for more than 1 second
        std::thread::sleep(Duration::from_millis(1100));

        // The warning should be printed when the guard drops
    }

    // Verify the value was still saved
    assert_eq!(*prefs.get_value1(), 100);
}
