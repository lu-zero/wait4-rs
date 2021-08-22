use std::io;
use std::process::ExitStatus;
use std::time::Duration;

#[derive(Debug)]
pub struct ResourceUsage {
    pub utime: Duration,
    pub stime: Duration,
    pub maxrss: u64,
}

#[derive(Debug)]
pub struct ResUse {
    pub status: ExitStatus,
    pub rusage: ResourceUsage,
}

pub trait Wait4 {
    fn wait4(&mut self) -> io::Result<ResUse>;
}

cfg_if::cfg_if! {
    if #[cfg(windows)] {
        mod windows;
        pub use windows::*;
    } else if #[cfg(unix)] {
        mod unix;
        pub use unix::*;
    } else {
        impl Wait4 for std::process::Child {
            fn wait4(&mut self) -> io::Result<ResUse> {
                Err(io::ErrorKind::Unsupported.into())
            }
        }
    }
}
