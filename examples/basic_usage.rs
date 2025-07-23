use easy_prefs::easy_prefs;

easy_prefs! {
    pub struct AppPreferences {
        pub notifications_enabled: bool = true => "notifications",
        pub username: String = "guest".to_string() => "username",
        pub theme: String = "light".to_string() => "theme",
        pub font_size: i32 = 14 => "font_size",
    },
    "app-preferences"
}

fn main() {
    // On native platforms, this will use the file system
    // On WASM, this will use localStorage
    let mut prefs = AppPreferences::load("com.example.myapp")
        .expect("Failed to load preferences");

    println!("Current preferences:");
    println!("  Notifications: {}", prefs.get_notifications_enabled());
    println!("  Username: {}", prefs.get_username());
    println!("  Theme: {}", prefs.get_theme());
    println!("  Font size: {}", prefs.get_font_size());

    // Update a single value
    prefs.save_username("Alice".to_string()).expect("Failed to save username");

    // Batch updates using edit guard
    {
        let mut edit = prefs.edit();
        edit.set_theme("dark".to_string());
        edit.set_font_size(16);
        // Saves automatically when edit guard is dropped
    }

    println!("\nUpdated preferences:");
    println!("  Username: {}", prefs.get_username());
    println!("  Theme: {}", prefs.get_theme());
    println!("  Font size: {}", prefs.get_font_size());
    
    println!("\nPreferences stored at: {}", prefs.get_preferences_file_path());
}
