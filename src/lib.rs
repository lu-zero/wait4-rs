use std::io;
use std::process::ExitStatus;
use std::time::Duration;

/// Resources used by a process
#[derive(Debug)]
pub struct ResourceUsage {
    /// User CPU time used
    ///
    /// Time spent in user-mode
    pub utime: Duration,
    /// System CPU time used
    ///
    /// Time spent in kernel-mode
    pub stime: Duration,
    /// Maximum resident set size, in bytes.
    ///
    /// Zero if not available on the platform.
    pub maxrss: u64,
}

/// Resources used by a process and its exit status
#[derive(Debug)]
pub struct ResUse {
    /// Same as the one returned by [`wait`].
    ///
    /// [`wait`]: std::process::Child::wait
    pub status: ExitStatus,
    /// Resource used by the process and all its children
    pub rusage: ResourceUsage,
}

/// Add wait for a process and return the resources it used.
pub trait Wait4 {
    /// As for [`wait`], it waits for the child to exit completely,
    /// returning the status that it exited with and an estimate of
    /// the time and memory resources it used.
    ///
    /// The stdin handle to the child process, if any, will be
    /// closed before waiting, refer to [`wait`] for the rationale
    /// for it.
    ///
    /// Unlike [`wait`], `wait4` takes the child by value: the child has
    /// already been reaped by the time this returns, so [`wait`] or
    /// `wait4` cannot be called again on it. Reusing a process ID after
    /// it has already been reaped could otherwise end up waiting on an
    /// unrelated, recycled process.
    ///
    /// [`wait`]: std::process::Child::wait
    fn wait4(self) -> io::Result<ResUse>;
}

std::cfg_select! {
    windows => {
        mod windows;
        pub use windows::*;
    }
    unix => {
        mod unix;
        pub use unix::*;
    }
    _ => {
        impl Wait4 for std::process::Child {
            fn wait4(self) -> io::Result<ResUse> {
                Err(io::ErrorKind::Unsupported.into())
            }
        }
    }
}
