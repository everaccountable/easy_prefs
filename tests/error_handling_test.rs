use easy_prefs::{easy_prefs, LoadError};

easy_prefs! {
    struct TestErrorPrefs {
        pub value: i32 = 0 => "value",
    },
    "test-error-prefs"
}

#[test]
fn test_instance_already_loaded_error() {
    let test_dir = format!("/tmp/easy_prefs_error_test_{}", std::process::id());
    
    // First load should succeed
    let _prefs1 = TestErrorPrefs::load(&test_dir).expect("First load should succeed");
    
    // Second load should fail with InstanceAlreadyLoaded
    let result = TestErrorPrefs::load(&test_dir);
    assert!(result.is_err());
    match result.unwrap_err() {
        LoadError::InstanceAlreadyLoaded => {
            // Expected error
        }
        other => panic!("Expected InstanceAlreadyLoaded, got {:?}", other),
    }
    
    // Clean up
    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_deserialization_error() {
    let test_dir = format!("/tmp/easy_prefs_deser_test_{}", std::process::id());
    std::fs::create_dir_all(&test_dir).unwrap();
    
    // Write invalid TOML
    let file_path = format!("{}/test-error-prefs.toml", test_dir);
    std::fs::write(&file_path, "value = \"not a number\"").unwrap();
    
    // Try to load - should fail with deserialization error
    let result = TestErrorPrefs::load(&test_dir);
    assert!(result.is_err());
    match result.unwrap_err() {
        LoadError::DeserializationError(location, _) => {
            assert!(location.contains("test-error-prefs.toml"));
        }
        other => panic!("Expected DeserializationError, got {:?}", other),
    }
    
    // Clean up
    let _ = std::fs::remove_dir_all(&test_dir);
}

#[test]
fn test_storage_error_display() {
    let io_error = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "test error");
    let storage_error = LoadError::StorageError(io_error);
    let display = format!("{}", storage_error);
    assert!(display.contains("storage error"));
    assert!(display.contains("test error"));
}

#[test]
fn test_error_trait_implementation() {
    let error = LoadError::InstanceAlreadyLoaded;
    // Verify it implements std::error::Error
    let _: &dyn std::error::Error = &error;
}
