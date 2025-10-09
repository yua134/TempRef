# Changelog

## [0.2.0] - 2025-10-09 **hotfix**

[Diff](https://github.com/yua134/TempRef/compare/v0.1.1...v0.2.0)

### Changed

#### **Breaking Changes**

- Unified return types of locking functions to `TempRef`:
  - `mutex::Temp::lock`
  - `mutex::Temp::try_lock`
  - `rwlock::Temp::write`
  - `rwlock::Temp::write_lock`
- These functions no longer return `MutexGuard` or `RwLockWriteGuard` directly.
  Callers must now handle `TempRef`, which ensures the `reset` closure is always executed on drop.


### Fixed

- Fixed an issue where the `reset` closure was not executed in some cases.

### Planned

- `TempRef::dismiss()` was considered for this release but has been deferred to the next update.

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
