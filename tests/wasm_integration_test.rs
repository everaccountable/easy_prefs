#![cfg(target_arch = "wasm32")]

use easy_prefs::easy_prefs;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

easy_prefs! {
    struct WasmTestPrefs {
        pub enabled: bool = true => "enabled",
        pub count: i32 = 0 => "count",
    },
    "wasm-test-prefs"
}

#[wasm_bindgen_test]
fn test_wasm_basic_functionality() {
    // Test that we can create preferences in WASM
    let mut prefs = WasmTestPrefs::load_testing();

    // Test defaults
    assert_eq!(*prefs.get_enabled(), true);
    assert_eq!(*prefs.get_count(), 0);

    // Test saving
    prefs.save_enabled(false).expect("Failed to save enabled");
    prefs.save_count(42).expect("Failed to save count");

    // Verify values were updated
    assert_eq!(*prefs.get_enabled(), false);
    assert_eq!(*prefs.get_count(), 42);
}

#[wasm_bindgen_test]
fn test_wasm_edit_guard() {
    let mut prefs = WasmTestPrefs::load_testing();

    {
        let mut guard = prefs.edit();
        guard.set_enabled(false);
        guard.set_count(100);
    }

    assert_eq!(*prefs.get_enabled(), false);
    assert_eq!(*prefs.get_count(), 100);
}

#[wasm_bindgen_test]
fn test_wasm_storage_path() {
    let prefs = WasmTestPrefs::load_testing();
    let path = prefs.get_preferences_file_path();

    // In WASM, the path should indicate localStorage
    assert!(path.starts_with("localStorage::"));
}
