# easy_prefs

A simple, safe, and performant preferences library for Rust applications that makes storing and retrieving settings as easy as reading and writing struct fields.

This macro-based library lets you define your preferences—including default values and custom storage keys—and persist them to disk using TOML. It emphasizes data safety by using atomic writes via temporary files and enforces a single-instance rule to prevent race conditions.

**Now with WebAssembly support!** Use the same API in browser extensions, web apps, and native applications. When compiled to WASM, preferences are stored in localStorage instead of the file system.

*Created by Ever Accountable – an app dedicated to helping people overcome compulsive porn use and become their best selves. More info at [everaccountable.com](https://everaccountable.com).*

## Quick Start

### 1. Add Dependencies

In your `Cargo.toml`, add:

```toml
[dependencies]
easy_prefs = "x.y"  # Use the latest version
serde = { version = "1.0", features = ["derive"] }
```

*(The library re-exports `paste`, `toml`, and `once_cell` so you don’t need to add them separately.)*

### 2. Define Your Preferences

Create a preferences struct with default values and customizable storage keys:

```rust
use easy_prefs::easy_prefs;

easy_prefs! {
    pub struct AppPreferences {
        /// Boolean preference with default `true`, stored as "notifications"
        pub notifications: bool = true => "notifications",
        /// String preference with default "guest", stored as "username"
        pub username: String = "guest".to_string() => "username",
    },
    "app-preferences"  // This defines the filename (stored in the platform-specific config directory)
}
```

### 3. Load and Use Preferences

```rust
fn main() {
    // Load preferences; defaults are used if the file doesn't exist.
    let mut prefs = AppPreferences::load("./com.mycompany.myapp")
        .expect("Failed to load preferences");

    println!("Notifications: {}", prefs.get_notifications());

    // Update a value (this write is blocking).
    prefs.save_notifications(false).expect("Save failed");

    // Batch updates using an edit guard (auto-saves on drop).
    {
        let mut guard = prefs.edit();
        guard.set_notifications(true);
        guard.set_username("Abe Lincoln".to_string());
    }
}
```

## WebAssembly Support

easy_prefs works seamlessly in WebAssembly environments like Safari extensions and web applications. When compiled to WASM, it automatically uses localStorage instead of the file system.

### Enabling WASM Support

Add the `wasm` feature to your dependency:

```toml
[dependencies]
easy_prefs = { version = "x.y", features = ["wasm"] }
```

### Building for WASM

```bash
cargo build --target wasm32-unknown-unknown --features wasm
```

### Usage in Safari Extensions

```rust
use easy_prefs::easy_prefs;
use wasm_bindgen::prelude::*;

easy_prefs! {
    pub struct ExtensionSettings {
        pub enabled: bool = true => "enabled",
        pub api_key: String = String::new() => "api_key",
    },
    "safari-extension"
}

#[wasm_bindgen]
pub fn init_extension() -> Result<(), JsValue> {
    // The "directory" parameter becomes the localStorage key prefix
    let mut settings = ExtensionSettings::load("com.mycompany.extension")
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    
    // Use the same API as native
    settings.save_enabled(true)?;
    Ok(())
}
```

### Storage Locations

- **Native platforms**: Files stored in the specified directory
- **WASM/Browser**: Data stored in localStorage with keys prefixed by your app ID

## Detailed Information

### Error Handling

- **LoadError Enum:**  
  The library defines a `LoadError` enum with these variants:
  - **InstanceAlreadyLoaded:** Only one instance can be loaded at a time.
  - **ProjectDirsError:** Issues with determining the configuration directory.
  - **FileOpenError / FileReadError:** Problems during file I/O.
  - **DeserializationError:** Errors while parsing TOML data.


### Use Across Threads

Use `Arc<Mutex<>>` to share the preferences struct between threads.
Trying to call `load()` on the same struct from multiple threads simultaneously will return an error.

### Temporary Files & Atomic Writes

To ensure data integrity, writes are performed as follows:
- Data is first written to a temporary file.
- The temporary file is renamed to the final file, ensuring the preferences file is never left in a partially written state.

### Testing with `load_testing()`

For unit tests, use `load_testing()`, which:
- Creates a temporary file (cleaned up after the test).
- Bypasses the single-instance constraint, making testing simpler.

### Edit Guards and Debug Checks

When batching updates with an edit guard:
- An assertion (active only in debug builds) ensures the guard isn’t held for more than 10ms to prevent blocking.
- This safety check helps catch long-held locks during development.

### Utility Methods

- **get_preferences_file_path():**  
  Returns the full path of the preferences file as a string, useful for debugging.

### Customizable Storage Keys

The macro’s syntax (`=> "field_name"`) lets you define a stored key that differs from the struct field name. This is helpful when renaming fields or preserving legacy data formats.

### Dependencies & Serialization

The macro requires [Serde](https://serde.rs) for serialization/deserialization and re-exports helpful crates like `paste`, `toml`, and `once_cell` to manage lazy statics and code generation.

## Limitations

- **Not for Large Data:**  
  All data is kept in memory and the entire file is rewritten on every save. Use a full database if you need to handle large datasets.
- **Blocking Writes:**  
  File writes happen on the calling thread, so be mindful of performance in critical sections.

## License

MIT License

```
Copyright (c) 2023 Ever Accountable

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
  
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
  
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```
