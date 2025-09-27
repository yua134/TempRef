#![no_std]
//! This crate provides a type whose value remains unchanged even when accessed through a mutable reference.
//! Some functions are compatible with `no_std` environments.
//!
//! | Module        | Characteristics                          | Feature Flags               |
//! |---------------|-------------------------------------------|-----------------------------|
//! | `unsync`      | `!Sync`, `!Send` type<br>Supports `no_std`| `default`, `all`, `no_std`, `unsync` |
//! | `mutex`       | `Sync`, `Send` type using `std::sync::Mutex` | `default`, `all`, `mutex` |
//! | `rwlock`      | `Sync`, `Send` type using `std::sync::RwLock` | `default`, `all`, `rwlock` |

#[cfg(feature = "unsync")]
pub mod unsync;

#[cfg(feature = "mutex")]
pub mod mutex;

#[cfg(feature = "rwlock")]
pub mod rwlock;
