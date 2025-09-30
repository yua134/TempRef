# Changelog

## [0.1.1] - 2025-09-30

[Diff](https://github.com/yua134/TempRef/compare/v0.1.0...v0.1.1)

### Added

- `.new_with()`, `.new_default()`, `.new_default_with()` added for each module

### Changed

#### **Breaking Changes**

- `mutex::Temp` / `rwlock::Temp`: closure trait bound Sync → Send
- `unsync::swap()`: parameter type: T → &RefCell\<T>

### Planed

- Considering adding a `TempRef::dismiss()` method in the next update

### Note

- No features were removed

## [0.1.0] - 2025-09-27

initial release
