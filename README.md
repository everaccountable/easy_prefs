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

```rust
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
    // just use default values
    let mut prefs = AppPreferences::load();
    
    // Read a value
    println!("Notifications enabled: {}", prefs.get_notifications());
    
    // Save a value. If the value has changed then this will rewrite the file to disk.
    // Note that this is a blocking operation.
    prefs.save_notifications(false);
    
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
// up after the test.
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefs() {
        let mut prefs = AppPreferences::load_testing();
        assert_eq!(prefs.get_notifications(), true);
        prefs.save_notifications(false);
        assert_eq!(prefs.get_notifications(), false);
    }
}
```


### Limitations:
- NOT intended to store large quantities of data. All data is cached in memory,
  and the entire file is rewritten on each save. Use a full database for heavy data storage.
- Writes happen on whatever thread you use when you save the data. This is a blocking operation. In the future we may make it save from a background thread.
- Currently we don't make any guarantee about which version will be written when multiple threads write at the same time.


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

ChatGPT and Grok helped a bunch - thanks!

## Zero Guarantees
We did our best to make this great and keep your data safe, but it comes with
ZERO GUARANTEES! Use at your own risk... And if you find problems please help us fix them!
