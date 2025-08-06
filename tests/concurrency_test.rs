use easy_prefs::easy_prefs;
use std::sync::{Arc, Mutex};
use std::thread;

easy_prefs! {
    struct ConcurrentPrefs {
        pub counter: i32 = 0 => "counter",
        pub name: String = String::new() => "name",
    },
    "concurrent-prefs"
}

#[test]
fn test_concurrent_access_with_mutex() {
    let prefs = Arc::new(Mutex::new(ConcurrentPrefs::load_testing()));
    let mut handles = vec![];

    // Spawn 10 threads that increment the counter
    for i in 0..10 {
        let prefs_clone = Arc::clone(&prefs);
        let handle = thread::spawn(move || {
            let mut prefs = prefs_clone.lock().unwrap();
            let current = *prefs.get_counter();
            prefs
                .save_counter(current + 1)
                .expect("Save should succeed");

            // Also update name
            prefs
                .save_name(format!("thread-{}", i))
                .expect("Save should succeed");
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Verify final state
    let prefs = prefs.lock().unwrap();
    assert_eq!(*prefs.get_counter(), 10);
    // Name should be from one of the threads (we can't predict which)
    assert!(prefs.get_name().starts_with("thread-"));
}

#[test]
fn test_edit_guard_thread_safety() {
    let mut prefs = ConcurrentPrefs::load_testing();

    // Create an edit guard
    let mut edit = prefs.edit();

    // Verify we can't move prefs to another thread while edit guard exists
    // This is a compile-time check, but we can test the behavior
    edit.set_counter(42);
    edit.set_name("test".to_string());

    // Drop the guard
    drop(edit);

    // Verify changes were saved
    assert_eq!(*prefs.get_counter(), 42);
    assert_eq!(prefs.get_name(), "test");
}

#[test]
fn test_load_default_multiple_instances() {
    let test_dir = format!("/tmp/easy_prefs_multi_{}", std::process::id());

    // load_default should allow multiple instances
    let prefs1 = ConcurrentPrefs::load_default(&test_dir);
    let prefs2 = ConcurrentPrefs::load_default(&test_dir);
    let prefs3 = ConcurrentPrefs::load_default(&test_dir);

    // All should be independent instances with default values
    assert_eq!(*prefs1.get_counter(), 0);
    assert_eq!(*prefs2.get_counter(), 0);
    assert_eq!(*prefs3.get_counter(), 0);

    // Clean up
    let _ = std::fs::remove_dir_all(&test_dir);
}
