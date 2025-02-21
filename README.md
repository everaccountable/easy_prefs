A simple, safe, and performant preferences library for Rust applications.

Created by Ever Accountable – an app that helps people quit compulsive porn use
and become the best version of themselves. More information is available at [everaccountable.com](https://everaccountable.com).

This library provides an easy-to-use API for reading and writing preferences, using a struct-like interface.
The design priorities are:

- **Ease of use:** Read and write operations are (almost) as simple as setting or getting a struct field.
- **Idiomatic Rust:** Define your preferences with clear defaults in an idiomatic way.
- **Safety:** Writes are performed using a temporary file so that crashes won’t leave your data corrupted.
- **Performance:** Reading and writing are optimized for speed.

**Note:** This library is NOT intended to store large quantities of data. All data is cached in memory,
and the entire file is rewritten on each save. Use a full database for heavy data storage.

## Example

```rust
use easy_prefs::easy_prefs;

easy_prefs! {
    /// Application preferences.
    pub struct AppPreferences {
        /// Whether notifications are enabled.
        pub notifications: bool = true => "notifications",
        /// The default username.
        pub username: String = "guest".to_string() => "username",
    },
    "app-preferences"
}

fn main() {
    let mut prefs = AppPreferences::load();
    println!("Notifications enabled: {}", prefs.get_notifications());
    prefs.save_notifications(false);
}
```
