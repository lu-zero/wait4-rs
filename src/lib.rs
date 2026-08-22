use std::io;
use std::process::ExitStatus;
use std::time::Duration;

/// Resources used by a process
#[derive(Clone, Copy, Debug)]
pub struct ResourceUsage {
    /// User CPU time used
    ///
    /// Time spent in user-mode. On Windows, zero if the underlying
    /// `GetProcessTimes` query failed.
    pub utime: Duration,
    /// System CPU time used
    ///
    /// Time spent in kernel-mode. On Windows, zero if the underlying
    /// `GetProcessTimes` query failed.
    pub stime: Duration,
    /// Maximum resident set size, in bytes.
    ///
    /// Zero if not available on the platform, or on Windows if the
    /// underlying `GetProcessMemoryInfo` query failed.
    pub maxrss: u64,
}

/// Resources used by a process and its exit status
#[derive(Clone, Copy, Debug)]
pub struct ResUse {
    /// Same as the one returned by [`wait`].
    ///
    /// [`wait`]: std::process::Child::wait
    pub status: ExitStatus,
    /// Resources used by the process.
    ///
    /// On Unix targets with a native `wait4(2)` (Linux, Android, macOS,
    /// iOS/tvOS/watchOS, FreeBSD, DragonFly, NetBSD, OpenBSD) this is
    /// `wait4(2)`'s own rusage for this specific child, which includes
    /// descendants the child itself waited on before exiting. On other
    /// Unix targets it falls back to `getrusage(RUSAGE_CHILDREN)`, which is
    /// *not* scoped to this child: it is the sum of every child this whole
    /// process has ever waited on, so concurrent children will see mixed
    /// totals rather than per-child figures. On Windows this covers only
    /// the process itself, not further descendants it may spawn — see
    /// [issue #1](https://github.com/lu-zero/wait4-rs/issues/1).
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

    /// As for [`try_wait`], does not block. `Ok(None)` means the child is
    /// still running. Unlike [`wait4`](Self::wait4), stdin is left open:
    /// a non-blocking probe should not close pipes out from under a still-running child.
    ///
    /// Once this returns `Ok(Some(_))` the child has been reaped. Do not
    /// call [`wait`], [`Self::wait4`], `try_wait4` again, or send the child
    /// a signal (e.g. [`kill`]) on the same `Child`: the PID may already
    /// have been recycled onto an unrelated process, and `Child`'s own
    /// bookkeeping has no way to learn it was reaped through this call.
    ///
    /// [`try_wait`]: std::process::Child::try_wait
    /// [`wait`]: std::process::Child::wait
    /// [`kill`]: std::process::Child::kill
    fn try_wait4(&mut self) -> io::Result<Option<ResUse>>;
}

std::cfg_select! {
    windows => {
        mod windows;
    }
    unix => {
        mod unix;
        pub use unix::try_wait4_pid;
    }
    _ => {
        impl Wait4 for std::process::Child {
            fn wait4(self) -> io::Result<ResUse> {
                Err(io::ErrorKind::Unsupported.into())
            }

            fn try_wait4(&mut self) -> io::Result<Option<ResUse>> {
                Err(io::ErrorKind::Unsupported.into())
            }
        }
    }
}
