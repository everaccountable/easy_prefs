//! Example of using easy_prefs in a WASM environment (e.g., Safari extension)
//!
//! To build for WASM:
//! cargo build --example wasm_usage --target wasm32-unknown-unknown
//!

#[cfg(target_arch = "wasm32")]
mod wasm {
    use easy_prefs::easy_prefs;
    use wasm_bindgen::prelude::*;

    easy_prefs! {
        pub struct ExtensionSettings {
            pub enabled: bool = true => "enabled",
            pub api_key: String = String::new() => "api_key",
            pub block_list: String = String::new() => "block_list",
            pub last_sync: String = String::new() => "last_sync",
        },
        "safari-extension-settings"
    }

    #[wasm_bindgen]
    pub struct ExtensionPrefs {
        prefs: ExtensionSettings,
    }

    #[wasm_bindgen]
    impl ExtensionPrefs {
        #[wasm_bindgen(constructor)]
        pub fn new() -> Result<ExtensionPrefs, JsValue> {
            let prefs = ExtensionSettings::load_with_error("com.example.safari-extension")
                .map_err(|e| JsValue::from_str(&format!("Failed to load preferences: {}", e)))?;
            Ok(ExtensionPrefs { prefs })
        }

        #[wasm_bindgen(getter)]
        pub fn enabled(&self) -> bool {
            *self.prefs.get_enabled()
        }

        #[wasm_bindgen(setter)]
        pub fn set_enabled(&mut self, value: bool) -> Result<(), JsValue> {
            self.prefs
                .save_enabled(value)
                .map_err(|e| JsValue::from_str(&format!("Failed to save: {}", e)))
        }

        #[wasm_bindgen(getter)]
        pub fn api_key(&self) -> String {
            self.prefs.get_api_key().clone()
        }

        #[wasm_bindgen(setter)]
        pub fn set_api_key(&mut self, value: String) -> Result<(), JsValue> {
            self.prefs
                .save_api_key(value)
                .map_err(|e| JsValue::from_str(&format!("Failed to save: {}", e)))
        }

        pub fn update_multiple(&mut self, enabled: bool, api_key: String) -> Result<(), JsValue> {
            {
                let mut edit = self.prefs.edit();
                edit.set_enabled(enabled);
                edit.set_api_key(api_key);
            }
            Ok(())
        }
    }

    #[wasm_bindgen(start)]
    pub fn init() {
        console_error_panic_hook::set_once();
    }
}

fn main() {
    // No-op main for non-wasm targets
}
