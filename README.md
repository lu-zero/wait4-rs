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

`try_wait4()` is the non-blocking counterpart (`WNOHANG` on Unix), left
open so a probe doesn't close pipes out from under a still-running child.
Once it returns `Ok(Some(_))` the child is reaped: do not `wait`, `wait4`,
`try_wait4`, or send it a signal (e.g. `kill`) again — the PID may already
be recycled. On Unix, `try_wait4_pid` does the same for a raw child pid
(useful when the handle is a tokio `Child`); it rejects a pid of `0`,
which `wait4(2)` would otherwise interpret as "any child in this process's
process group".

## Upgrading from 0.1

`wait4()` now takes `self` by value instead of `&mut self`, so it reaps
the child, closes stdin first, and cannot be called again — the fix for
[#3](https://github.com/lu-zero/wait4-rs/issues/3). Most call sites don't
need to change; drop any now-unnecessary `mut` on the `Child` binding.
`try_wait4`/`try_wait4_pid` are new in 0.2 for non-blocking polling loops.

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
