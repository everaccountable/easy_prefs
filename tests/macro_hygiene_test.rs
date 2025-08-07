//! Test to verify that the easy_prefs macro works properly in downstream crates
//! without requiring them to import web_time themselves (macro hygiene test).

use easy_prefs::easy_prefs;

// This would previously fail with "use of unresolved module or unlinked crate `web_time`"
// because the macro contained unqualified web_time references
easy_prefs! {
    pub struct MacroHygieneTest {
        pub test_field: bool = false => "test_field",
    },
    "macro-hygiene-test"
}

#[test]
fn test_macro_hygiene() {
    // Test that the macro expands correctly and we can create edit guards
    // (which use web_time::Instant internally)
    let mut prefs = MacroHygieneTest::load_testing();
    
    // Test edit guard creation (this internally uses web_time::Instant::now())
    {
        let mut edit = prefs.edit();
        edit.set_test_field(true);
        // Guard drops here, testing web_time::Instant usage
    }
    
    assert_eq!(*prefs.get_test_field(), true);
}

#[cfg(target_arch = "wasm32")]
#[test] 
fn test_macro_hygiene_wasm() {
    // Test that load_testing works on WASM (uses web_time::SystemTime internally)
    let mut prefs = MacroHygieneTest::load_testing();
    
    {
        let mut edit = prefs.edit();
        edit.set_test_field(true);
    }
    
    assert_eq!(*prefs.get_test_field(), true);
}
