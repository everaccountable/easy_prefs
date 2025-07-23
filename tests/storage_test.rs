#[cfg(not(target_arch = "wasm32"))]
mod native_storage_tests {
    use easy_prefs::storage::{Storage, create_storage};
    use std::fs;
    
    #[test]
    fn test_file_storage_read_write() {
        let test_dir = format!("/tmp/easy_prefs_storage_test_{}", std::process::id());
        let storage = create_storage(&test_dir);
        
        // Test write
        storage.write("test.toml", "content = \"test\"").expect("Write should succeed");
        
        // Test read
        let content = storage.read("test.toml").expect("Read should succeed");
        assert_eq!(content, Some("content = \"test\"".to_string()));
        
        // Test get_path
        let path = storage.get_path("test.toml");
        assert!(path.contains(&test_dir));
        assert!(path.contains("test.toml"));
        
        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }
    
    #[test]
    fn test_file_storage_read_nonexistent() {
        let test_dir = format!("/tmp/easy_prefs_storage_test_ne_{}", std::process::id());
        let storage = create_storage(&test_dir);
        
        // Reading non-existent file should return None
        let content = storage.read("nonexistent.toml").expect("Read should succeed");
        assert_eq!(content, None);
    }
    
    #[test]
    fn test_file_storage_creates_directories() {
        let test_dir = format!("/tmp/easy_prefs_storage_test_dir_{}/nested/deep", std::process::id());
        let storage = create_storage(&test_dir);
        
        // Write should create parent directories
        storage.write("test.toml", "data").expect("Write should create directories");
        
        // Verify file exists
        let content = storage.read("test.toml").expect("Read should succeed");
        assert_eq!(content, Some("data".to_string()));
        
        // Clean up
        let base_dir = format!("/tmp/easy_prefs_storage_test_dir_{}", std::process::id());
        let _ = fs::remove_dir_all(&base_dir);
    }
    
    #[test]
    fn test_storage_trait_object() {
        // Verify we can use Storage as a trait object
        let test_dir = format!("/tmp/easy_prefs_trait_test_{}", std::process::id());
        let storage: Box<dyn Storage> = create_storage(&test_dir);
        
        storage.write("test.toml", "data").expect("Write should succeed");
        let _ = storage.read("test.toml").expect("Read should succeed");
        
        // Clean up
        let _ = fs::remove_dir_all(&test_dir);
    }
}
