# Changelog

## [3.0.0] - 2024-01-XX

### Breaking Changes

- **Removed `load_default()`**: This method has been removed as it bypassed the single-instance constraint, which could lead to data corruption and race conditions.
- **Changed `load()` behavior**: The `load()` method now always succeeds instead of returning a `Result`:
  - In release builds: Returns defaults on errors (except instance conflicts which always panic)
  - In debug/test builds: Panics on errors to catch issues early
  - Always panics if another instance is already loaded
  - This provides a simpler API that gracefully handles errors
- **Added `load_with_error()`**: New method that returns `Result<Self, LoadError>` for explicit error handling, replacing the old `load()` behavior.

### Migration Guide

```rust
// Old (v2.x) - handling errors with load_default()
let prefs = AppPreferences::load(dir).unwrap_or_else(|e| {
    log::error!("Failed to load: {}", e);
    AppPreferences::load_default(dir)
});

// New (v3.0) - Simple approach (panics on error)
let prefs = AppPreferences::load(dir);

// New (v3.0) - With explicit error handling
let prefs = match AppPreferences::load_with_error(dir) {
    Ok(p) => p,
    Err(e) => {
        log::error!("Failed to load: {}", e);
        return; // Handle error appropriately
    }
};
```

### Why These Changes?

The previous `load_default()` method violated the library's core safety principle by allowing multiple instances to exist simultaneously. This could lead to:
- Data corruption when multiple instances write to the same file
- Race conditions in concurrent environments
- Inconsistent application state

The new API design:
- Maintains the single-instance guarantee
- Provides a simpler default path with `load()` that "just works"
- Offers explicit error handling when needed via `load_with_error()`
- Helps catch configuration issues early in development

### Other Changes

- Updated documentation to reflect the new API
- Updated all examples to use the new methods
- Added comprehensive tests for the new behavior
