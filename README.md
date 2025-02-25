Below is a cleaned-up version of the markdown file with improved formatting and consistent styling:

---

# easy_preferences

A simple-to-use, safe, and lightweight way to save preferences (or really anything you would keep in a struct).

We asked: *"What is the simplest possible API to persist a struct in Rust?"*  
This is our answer. (It’s performant, testable, thread-safe, easy to migrate, and designed to never corrupt your data.)

*This library was created by Ever Accountable – an app that helps people quit compulsive porn use and become the best version of themselves. More info at [everaccountable.com](https://everaccountable.com).*

## Features

- **Easy to use!**  
  The overarching priority.
- **Define it all in one place:**  
  Define your preferences in a familiar struct format with types and default values. Anything serializable is allowed.
- **Lightweight API:**  
  Reading and writing are as simple as getting or setting a field on a struct.
- **Type Safety:**  
  Ensures that you are reading and writing the correct types.
- **Data Safety:**  
  Writes are performed carefully (using a temporary file then renaming) to avoid data corruption.
- **Migratable:**  
  Rename fields and specify old names so that legacy data isn’t lost.
- **Lightweight Performance:**  
  Uses TOML files under the hood and only writes to disk when something changes.
- **Thread Safety:**  
  Supports concurrent read and write from multiple threads.
- **Easy Unit Testing:**  
  Designed to play well with your tests.

## Example

```rust
use easy_prefs::easy_prefs;

// Define your preferences in a struct with default values.
easy_prefs! {
    pub struct AppPreferences {
        /// A boolean preference. Default is `true`, stored as "notifications".
        pub notifications: bool = true => "notifications",
        /// A string preference with default "guest", stored as "username".
        pub username: String = "guest".to_string() => "username",
    },
    "app-preferences"  // Filename (the directory is determined by the `directories` crate).
}

// Reading and writing example.
fn main() {
    // Load preferences from disk. Uses default values if the file doesn't exist.
    let mut prefs = AppPreferences::load("com.mycompany.myapp")
        .expect("Failed to load preferences");
    
    // Read a value.
    println!("Notifications enabled: {}", prefs.get_notifications());
    
    // Save a value (blocking operation).
    prefs.save_notifications(false)
        .expect("Failed to save notification preference");
    
    // Save multiple values using an edit guard.
    {
        let mut edit_guard = prefs.edit();
        edit_guard.set_notifications(true);
        edit_guard.set_username("Abe Lincoln".to_string());
    }
}

// Example unit test.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefs() {
        let mut prefs = AppPreferences::load_testing();
        assert_eq!(prefs.get_notifications(), true);
        prefs.save_notifications(false)
            .expect("Failed to save preference");
    }
}
```

## Limitations

- **Not for large data:**  
  All data is cached in memory, and the entire file is rewritten on each save. Use a full database for heavy storage.
- **Blocking Writes:**  
  Writes occur on the calling thread. (Future versions may use background threads.)

## Thread Safety

The library enforces a single-instance constraint for each preferences struct:

- **Single-Instance Constraint:**  
  The `load()` method ensures only one instance exists at a time using a static atomic flag.
- **Multiple Threads:**  
  Loading the same preferences struct on multiple threads will result in a `LoadError::InstanceAlreadyLoaded`.
- **Sharing Across Threads:**  
  Wrap your instance in an `Arc<Mutex<>>` to share it:

```rust
use std::sync::{Arc, Mutex};

// Load preferences once.
let prefs = AppPreferences::load("com.mycompany.myapp")
    .expect("Failed to load preferences");

// Share across threads.
let shared_prefs = Arc::new(Mutex::new(prefs));

// In thread 1.
let prefs_clone = Arc::clone(&shared_prefs);
let handle = std::thread::spawn(move || {
    let prefs = prefs_clone.lock().unwrap();
    println!("Username: {}", prefs.get_username());
});

// In main thread.
{
    let mut prefs = shared_prefs.lock().unwrap();
    prefs.save_notifications(false)
         .expect("Failed to save");
}

handle.join().unwrap();
```

## Edit Guards

Edit guards are designed for short-lived batch operations. In debug builds, holding an edit guard for more than 10ms triggers an assertion failure. This helps prevent accidental blocking and potential data corruption from concurrent writes.

## File Storage

Preferences are stored in the platform-specific configuration directory (determined by the [directories](https://crates.io/crates/directories) crate) based on the provided namespace. For example:
- **Linux:** `~/.config/app-preferences.toml`
- **Windows:** `C:\Users\Username\AppData\Roaming\app-preferences.toml`

## Origin Story

*By Tyler Patterson*

Common things should be simple! Writing and reading user preferences is a common task that has been surprisingly cumbersome in both Rust and Android.

While databases like sqlite and sled are powerful, they add unnecessary boilerplate for simple preferences. After trying several libraries, I wanted an API that felt as natural as setting and getting fields on a struct.

I went through about four iterations before settling on a macro-based approach that defines both the struct and default values in one place. Special thanks to ChatGPT, Claude, and Grok for their help!

## License

MIT License

```
Copyright (c) 2023 Ever Accountable

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
```

---

This version uses consistent heading levels, clear code blocks with Rust syntax highlighting, and improved spacing for readability.