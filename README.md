# Rust idiomatic binding to wait4

``` rust
use std::process::Command;
use wait4::Wait4;

let cmd = Command::new(command);

// ...
let mut child = cmd.spawn().unwrap();

let r = child.wait4().unwrap();
```

## Status

- [x] Unix-like using `libc::wait4` or `libc::getrusage`
- [x] Windows using `winapi::um::processthreadsapi::GetProcessTimes` and `winapi::um::psapi::GetProcessMemoryInfo`.
- [ ] Proper documentation

## License

[MIT](https://spdx.org/licenses/MIT), the windows code is partially from [hyperfine](https://github.com/sharkdp/hyperfine).
