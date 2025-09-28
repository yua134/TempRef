# TempRef

[![Crates.io](https://img.shields.io/crates/v/tempref)](https://crates.io/crates/tempref)
[![Docs.rs](https://docs.rs/tempref/badge.svg)](https://docs.rs/tempref)
[![Downloads](https://img.shields.io/crates/d/tempref.svg)](https://crates.io/crates/tempref)

## overview

This crate provides a type whose value remains unchanged even when accessed through a mutable reference.

## features

- Automatically reset when the mutable reference is dropped
- Works in both single-threaded and multi-threaded contexts
- no_std compatible (only the unsync module)

## feature flags

| Module        | Characteristics                          | Feature Flags               |
|---------------|-------------------------------------------|-----------------------------|
| `unsync`      | `!Sync`, `!Send` type supports `no_std`| `default`, `all`, `no_std`, `unsync` |
| `mutex`       | `Sync`, `Send` type using `std::sync::Mutex` | `default`, `all`, `mutex` |
| `rwlock`      | `Sync`, `Send` type using `std::sync::RwLock` | `default`, `all`, `rwlock` |

## usage

```rust
use tempref::unsync::Temp;

let data = vec![0;128];
let workspace = Temp::new(data, |d| {d.fill(0);});

assert_eq!(*workspace.borrow(), vec![0;128]);

{
    // vec.clone() is unnecessary, so repeated allocations are avoided (as long as it’s not reallocated).
    // This helps keep your program lightweight.
    let mut guard = workspace.borrow_mut();
    guard.fill(1);
    assert_eq!(*guard, vec![1;128]);
} // d.fill(0) is called here.

assert_eq!(*workspace.borrow(), vec![0;128]);
```

## crate info

- License: MIT OR Apache-2.0
- Crate: [crates.io](https://crates.io/crates/tempref)
- Docs: [docs.rs](https://docs.rs/tempref)
- Repository: [GitHub](https://github.com/yua134/tempref)
