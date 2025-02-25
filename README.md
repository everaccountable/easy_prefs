A simple to use, safe, and lightweight way to save preferences (or really anything you would keep in a struct).

We asked: "What is the simplest possible API to persist a struct in Rust?" This is our answer.
(Oh, and it is also performant, testable, thread safe, easy to migrate, and should never corrupt your data)

*This library was created by Ever Accountable – an app that helps people quit compulsive porn use
and become the best version of themselves. More info at [everaccountable.com](https://everaccountable.com).*

easy_preferences does these things well:

- **Easy to use!** This is the overarching priority.
- **Define it all in one place:** Define your preferences in a familiar struct format. Including the types and default values. Anything serializable is allowed.
- **Lightweight API:** Read and write operations are as simple as setting or getting a field on a struct.
- **Type safety:** The library will ensure that you are reading and writing the correct types.
- **Data Safety:** Writes are performed carefully so that an inopportune system crash won’t leave your data corrupted (we write to a temporary file and then rename it)
- **Migrate-able:** If you rename a field in the code, you can specify the old field name so that legacy data doesn't get lost.
- **Lightweight performance:** We use .toml files under the hood. No big memory or cpu requirements. It just saves to disk when something has changed.
- **Thread Safety:** Read and write from multiple threads without worry.
- **Easy Unit Testing:** The library is designed to play well with your unit tests.

## Example


use easy_prefs::easy_prefs;

/////////////////////////////////////////////////////////////
// Define your preferences in a struct. The default values are specified here.
easy_prefs! {
    pub struct AppPreferences {
        /// A boolean preference. Note the default value is true, and the field name in the file is "notifications"
        pub notifications: bool = true => "notifications",
        /// Note here that we've specified the name of the field in the file.
        pub username: String = "guest".to_string() => "username",
    },
    "app-preferences"  // This is the filename. The directory comes from the directories crate.
}

/////////////////////////////////////////////////////////////
// Example of reading and writing:
fn main() {
    // Load the preferences from disk. If the file doesn't exist, it will 
    // use default values. Note that the namespace is passed in as an argument.
    // load() returns a Result and can fail for various reasons.
    let mut prefs = AppPreferences::load("com.mycompany.myapp")
        .expect("Failed to load preferences");
    
    // Read a value
    println!("Notifications enabled: {}", prefs.get_notifications());
    
    // Save a value. If the value has changed then this will rewrite the file to disk.
    // Note that this is a blocking operation.
    prefs.save_notifications(false)
        .expect("Failed to save notification preference");
    
    // Save multiple values at the same time by creating an edit_guard. It gets saved
    // when the edit_guard goes out of scope:
    {
        let mut edit_guard = prefs.edit();
        edit_guard.set_notifications(true);
        edit_guard.set_username("Abe Lincoln".to_string());
    }
}

/////////////////////////////////////////////////////////////
// An example unit test. Notice that we call load_testing()
// so that the preferences file is temporary and gets cleaned 
// up after the test. Unlike load(), load_testing() will panic
// directly if there's an error.
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



### Limitations:
- NOT intended to store large quantities of data. All data is cached in memory,
and the entire file is rewritten on each save. Use a full database for heavy data storage.
- Writes happen on whatever thread you use when you save the data. This is a blocking operation. In the future we may make it save from a background thread.

## Thread Safety

The `easy_prefs` library enforces a single-instance constraint for each preferences struct:

- **Single-Instance Constraint**: The `load()` method ensures that only one instance of a preferences struct can exist at any time using a static atomic flag.

- **Multiple Threads**: It is an error to try to load the same preferences struct on multiple threads simultaneously. Attempting to do so will result in a `LoadError::InstanceAlreadyLoaded` error.

- **Sharing Across Threads**: If you need to access preferences from multiple threads, wrap your preferences instance in an `Arc<Mutex<>>`:

  
use std::sync::{Arc, Mutex};
  
// Load preferences once
let prefs = AppPreferences::load("com.mycompany.myapp")
    .expect("Failed to load preferences");
      
// Share across threads using Arc<Mutex<>>
let shared_prefs = Arc::new(Mutex::new(prefs));
  
// In thread 1
let prefs_clone = Arc::clone(&shared_prefs);
let handle = std::thread::spawn(move || {
    let prefs = prefs_clone.lock().unwrap();
    println!("Username: {}", prefs.get_username());
});
  
// In main thread
{
    let mut prefs = shared_prefs.lock().unwrap();
    prefs.save_notifications(false).expect("Failed to save");
}
  
handle.join().unwrap();

## Edit Guards

The edit guards are designed for short-lived batch operations. In debug builds,
holding an edit guard for more than 10ms will trigger an assertion failure. This
helps identify places where you might be accidentally holding the guard for too long,
which could block other operations.

The single-instance constraint prevents potential data corruption that could occur
if multiple instances were simultaneously reading from and writing to the same
preferences file. This design choice simplifies the API while ensuring data integrity.

## File Storage

Preferences are stored in the platform-specific config directory as determined by the
[directories](https://crates.io/crates/directories) crate, based on the namespace you
provide. For example, on Linux this might be `~/.config/app-preferences.toml`, while
on Windows it could be `C:\Users\Username\AppData\Roaming\app-preferences.toml`.


## Origin Story of easy_prefs, by Tyler Patterson
Common things should be simple! Writing and reading user preferences is a common task
that I have found to be surprisingly cumbersome both in Rust and in Android.

sqlite and sled seem like great databases, but using them for simple preferences seems like
overkill and has a lot of boilerplate. I've tried other preferences libraries and I never
liked how cumbersome things felt.

Reading and writing preferences should be as
close as possible to just setting and getting fields on a struct. That's it!
Define a struct, along with default values right there, set a name for the file,
and names for the fields in the file (for future-proofing).

I went through about 4 different iterations of this library before I landed on this one.

I settled on using a macro because I could define both the struct and and the default
values all in one place. And because the read and save functions can be made so simple.

ChatGPT, Claude, and Grok helped a bunch - thanks!

## License

MIT License

Copyright (c) 2023 Ever Accountable

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
