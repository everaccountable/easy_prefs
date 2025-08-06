use easy_prefs::easy_prefs;

easy_prefs! {
    struct TestDefaultPrefs {
        pub enabled: bool = true => "enabled",
        pub count: i32 = 42 => "count",
        pub name: String = "default".to_string() => "name",
    },
    "test-default-prefs"
}

#[test]
fn test_load_default() {
    // Create a unique directory for this test
    let test_dir = format!("/tmp/easy_prefs_test_{}", std::process::id());

    // Use load_default to create preferences with defaults
    let prefs = TestDefaultPrefs::load_default(&test_dir);

    // Verify default values
    assert_eq!(*prefs.get_enabled(), true);
    assert_eq!(*prefs.get_count(), 42);
    assert_eq!(prefs.get_name(), "default");

    // Verify the path is set correctly
    let path = prefs.get_preferences_file_path();
    assert!(path.contains(&test_dir));
    assert!(path.contains("test-default-prefs.toml"));

    // Clean up
    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_load_default_and_save() {
    let test_dir = format!("/tmp/easy_prefs_test_save_{}", std::process::id());

    // Create with defaults
    let mut prefs = TestDefaultPrefs::load_default(&test_dir);

    // Modify and save
    prefs.save_count(100).expect("Failed to save count");
    prefs
        .save_name("modified".to_string())
        .expect("Failed to save name");

    // Drop the instance
    drop(prefs);

    // Load normally and verify persistence
    let loaded = TestDefaultPrefs::load(&test_dir).expect("Failed to load");
    assert_eq!(*loaded.get_count(), 100);
    assert_eq!(loaded.get_name(), "modified");
    assert_eq!(*loaded.get_enabled(), true); // unchanged

    // Clean up
    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_load_default_bypasses_instance_check() {
    let test_dir = format!("/tmp/easy_prefs_test_bypass_{}", std::process::id());

    // First, load normally
    let _prefs1 = TestDefaultPrefs::load(&test_dir).expect("First load should succeed");

    // Try to load again normally - should fail
    let result = TestDefaultPrefs::load(&test_dir);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        easy_prefs::LoadError::InstanceAlreadyLoaded
    ));

    // But load_default should work even with an existing instance
    let _prefs2 = TestDefaultPrefs::load_default(&test_dir);

    // Clean up
    let _ = std::fs::remove_dir_all(&test_dir);
}
