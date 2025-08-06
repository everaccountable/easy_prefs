use easy_prefs::easy_prefs;

easy_prefs! {
    struct TestDefaultPrefs {
        pub enabled: bool = true => "enabled",
        pub count: i32 = 42 => "count",
        pub name: String = "default".to_string() => "name",
    },
    "test-default-prefs"
}

// Combined test to avoid single-instance conflicts
#[test]
fn test_load_default_removal_and_new_api() {
    // Run tests sequentially to avoid global instance conflicts
    
    // Test 1: load() creates defaults when file missing
    test_load_creates_defaults_when_file_missing();
    
    // Test 2: load_default() panics
    test_load_default_panics();
    
    // Test 3: load_with_error() handles instance conflict
    test_load_with_error_handles_instance_conflict();
    
    // Test 4: load() panics on instance conflict
    test_load_panics_on_instance_conflict();
}

fn test_load_creates_defaults_when_file_missing() {
    // Create a unique directory for this test
    let test_dir = format!("/tmp/easy_prefs_test_{}", std::process::id());

    // Use load() on empty directory - should create with defaults
    let prefs = TestDefaultPrefs::load(&test_dir);

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
    
    // Drop the instance to release the lock
    drop(prefs);
}

fn test_load_default_panics() {
    let test_dir = format!("/tmp/easy_prefs_test_panic_{}", std::process::id());
    
    // This should panic with deprecation message
    let result = std::panic::catch_unwind(|| {
        #[allow(deprecated)]
        let _prefs = TestDefaultPrefs::load_default(&test_dir);
    });
    
    assert!(result.is_err());
    let panic_msg = result.unwrap_err();
    if let Some(msg) = panic_msg.downcast_ref::<String>() {
        assert!(msg.contains("load_default() has been removed in version 3.0.0"));
    } else if let Some(msg) = panic_msg.downcast_ref::<&str>() {
        assert!(msg.contains("load_default() has been removed in version 3.0.0"));
    }
}

fn test_load_with_error_handles_instance_conflict() {
    let test_dir = format!("/tmp/easy_prefs_test_conflict_{}", std::process::id());

    // First, load normally
    let _prefs1 = TestDefaultPrefs::load_with_error(&test_dir).expect("First load should succeed");

    // Try to load again - should return error
    let result = TestDefaultPrefs::load_with_error(&test_dir);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        easy_prefs::LoadError::InstanceAlreadyLoaded
    ));

    // Clean up by dropping the instance
    drop(_prefs1);
    let _ = std::fs::remove_dir_all(&test_dir);
}

fn test_load_panics_on_instance_conflict() {
    let test_dir = format!("/tmp/easy_prefs_test_panic_conflict_{}", std::process::id());

    // First, load normally
    let _prefs1 = TestDefaultPrefs::load(&test_dir);

    // Try to load again with load() - should panic
    let result = std::panic::catch_unwind(|| {
        let _prefs2 = TestDefaultPrefs::load(&test_dir);
    });
    
    assert!(result.is_err());
    let panic_msg = result.unwrap_err();
    if let Some(msg) = panic_msg.downcast_ref::<String>() {
        assert!(msg.contains("Failed to load preferences"));
    } else if let Some(msg) = panic_msg.downcast_ref::<&str>() {
        assert!(msg.contains("Failed to load preferences"));
    }
    
    // Clean up
    drop(_prefs1);
}
