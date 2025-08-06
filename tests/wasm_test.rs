#![cfg(target_arch = "wasm32")]

use easy_prefs::easy_prefs;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

easy_prefs! {
    struct TestWasmPrefs {
        pub enabled: bool = true => "enabled",
        pub count: i32 = 0 => "count",
    },
    "test-wasm-prefs"
}

#[wasm_bindgen_test]
fn test_wasm_storage() {
    // Create preferences with a unique ID to avoid conflicts
    let test_id = format!("test_{}", js_sys::Date::now());
    let mut prefs = TestWasmPrefs::load(&test_id).expect("Failed to load prefs");

    // Test default values
    assert_eq!(*prefs.get_enabled(), true);
    assert_eq!(*prefs.get_count(), 0);

    // Test saving values
    prefs.save_enabled(false).expect("Failed to save enabled");
    prefs.save_count(42).expect("Failed to save count");

    // Reload and verify persistence
    drop(prefs);
    let prefs2 = TestWasmPrefs::load(&test_id).expect("Failed to reload prefs");
    assert_eq!(*prefs2.get_enabled(), false);
    assert_eq!(*prefs2.get_count(), 42);
}

#[wasm_bindgen_test]
fn test_wasm_edit_guard() {
    let test_id = format!("test_edit_{}", js_sys::Date::now());
    let mut prefs = TestWasmPrefs::load(&test_id).expect("Failed to load prefs");

    {
        let mut edit = prefs.edit();
        edit.set_enabled(false);
        edit.set_count(100);
    }

    assert_eq!(*prefs.get_enabled(), false);
    assert_eq!(*prefs.get_count(), 100);
}
