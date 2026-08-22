# Rust idiomatic binding to wait4
[![crates.io](https://img.shields.io/crates/v/wait4.svg)](https://crates.io/crates/wait4)
[![docs.rs](https://docs.rs/mio/badge.svg)](https://docs.rs/wait4)

``` rust
use std::process::Command;
use wait4::Wait4;

let cmd = Command::new(command);

// ...
let child = cmd.spawn().unwrap();

let r = child.wait4().unwrap();
```

`wait4()` takes the child by value: it reaps the process, so unlike
[`Child::wait`](https://doc.rust-lang.org/std/process/struct.Child.html#method.wait)
it cannot be called more than once on the same child.

## Status

- [x] Unix-like using `libc::wait4` or `libc::getrusage`
- [x] Windows using `windows-sys`' `GetProcessTimes` and `GetProcessMemoryInfo`.
  This only accounts for the spawned process itself, not further descendants
  it may spawn (unlike Unix's `RUSAGE_CHILDREN`); see
  [#1](https://github.com/lu-zero/wait4-rs/issues/1) for why that isn't
  easily fixable without an upstream `std` change.
- [x] Proper documentation

MSRV: 1.95 (needed for `std::cfg_select!`).

## License

[MIT](https://spdx.org/licenses/MIT), the windows code is partially from [hyperfine](https://github.com/sharkdp/hyperfine).
