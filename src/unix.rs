use crate::*;

use std::time::Duration;
use std::{io, process::Child};

use libc::timeval;

#[allow(clippy::useless_conversion)]
fn timeval_to_duration(val: timeval) -> Duration {
    let v = i64::from(val.tv_sec) * 1_000_000 + i64::from(val.tv_usec);
    Duration::from_micros(v as u64)
}

/// `ru_maxrss` is KiB on Linux/Android/*BSD, but already bytes on Apple's
/// kernels (macOS/iOS/tvOS/watchOS all deviate from the rest of the BSD
/// family here).
fn maxrss_bytes(maxrss: i64) -> u64 {
    let v = maxrss.max(0) as u64;
    if cfg!(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos"
    )) {
        v
    } else {
        v.saturating_mul(1024)
    }
}

std::cfg_select! {
    any(
        target_os = "linux",
        target_os = "android",
        target_os = "macos",
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "netbsd",
        target_os = "openbsd",
    ) => {
        use std::os::unix::process::ExitStatusExt;

        fn resuse_from(status: libc::c_int, rusage: libc::rusage) -> ResUse {
            ResUse {
                status: ExitStatus::from_raw(status),
                rusage: ResourceUsage {
                    utime: timeval_to_duration(rusage.ru_utime),
                    stime: timeval_to_duration(rusage.ru_stime),
                    maxrss: {
                        // `ru_maxrss` is `c_long` here and `i32` on some libc targets.
                        #[allow(clippy::unnecessary_cast)]
                        maxrss_bytes(rusage.ru_maxrss as i64)
                    },
                },
            }
        }

        fn wait4_pid(pid: i32, options: libc::c_int) -> io::Result<Option<ResUse>> {
            loop {
                let mut status = 0;
                let mut rusage = std::mem::MaybeUninit::zeroed();
                let r = unsafe { libc::wait4(pid, &mut status, options, rusage.as_mut_ptr()) };
                if r < 0 {
                    let err = io::Error::last_os_error();
                    if err.raw_os_error() == Some(libc::EINTR) {
                        continue;
                    }
                    return Err(err);
                }
                if r == 0 {
                    return Ok(None);
                }
                let rusage = unsafe { rusage.assume_init() };
                return Ok(Some(resuse_from(status, rusage)));
            }
        }

        /// Non-blocking `wait4` for a child pid of this process.
        ///
        /// `Ok(None)` means the child is still running. Once `Ok(Some(_))`
        /// is returned the pid has been reaped: do not `wait`, `wait4`,
        /// `try_wait4`, `try_wait4_pid`, or send it a signal (e.g. `kill`)
        /// again. The pid may already have been recycled by the OS onto an
        /// unrelated process.
        ///
        /// `pid` must identify a specific child of this process; `0` is
        /// rejected rather than silently waiting for *any* child in this
        /// process's process group, which is what `wait4(2)` does with a
        /// pid of `0`.
        pub fn try_wait4_pid(pid: u32) -> io::Result<Option<ResUse>> {
            if pid == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "try_wait4_pid: pid 0 would wait for any child in this process's \
                     process group, not one specific child",
                ));
            }
            wait4_pid(pid as i32, libc::WNOHANG)
        }

        impl Wait4 for Child {
            fn wait4(mut self) -> io::Result<ResUse> {
                drop(self.stdin.take());
                match wait4_pid(self.id() as i32, 0)? {
                    Some(r) => Ok(r),
                    None => Err(io::Error::other("wait4 returned 0 without WNOHANG")),
                }
            }

            fn try_wait4(&mut self) -> io::Result<Option<ResUse>> {
                try_wait4_pid(self.id())
            }
        }
    }
    _ => {
        // No native wait4(2) binding on this target: fall back to
        // Child::wait/try_wait plus getrusage(RUSAGE_CHILDREN). Unlike the
        // wait4(2) path above, RUSAGE_CHILDREN is *not* scoped to this one
        // Child: it is the sum of every child this whole process has ever
        // waited on. Concurrent children on this fallback will see mixed
        // totals, not per-child figures.
        fn resource_usage_children() -> io::Result<ResourceUsage> {
            let rusage = unsafe {
                let mut rusage = std::mem::MaybeUninit::zeroed();
                if libc::getrusage(libc::RUSAGE_CHILDREN, rusage.as_mut_ptr()) < 0 {
                    return Err(io::Error::last_os_error());
                }
                rusage.assume_init()
            };
            Ok(ResourceUsage {
                utime: timeval_to_duration(rusage.ru_utime),
                stime: timeval_to_duration(rusage.ru_stime),
                #[allow(clippy::unnecessary_cast)]
                maxrss: maxrss_bytes(rusage.ru_maxrss as i64),
            })
        }

        impl Wait4 for Child {
            fn wait4(mut self) -> io::Result<ResUse> {
                let status = self.wait()?;
                let rusage = resource_usage_children()?;
                Ok(ResUse { status, rusage })
            }

            fn try_wait4(&mut self) -> io::Result<Option<ResUse>> {
                match self.try_wait()? {
                    None => Ok(None),
                    Some(status) => {
                        let rusage = resource_usage_children()?;
                        Ok(Some(ResUse { status, rusage }))
                    }
                }
            }
        }

        /// No native `wait4` binding on this target, and `RUSAGE_CHILDREN`
        /// cannot be scoped to an arbitrary pid at all (it is process-wide,
        /// not per-child), so there is no honest value to return here.
        pub fn try_wait4_pid(_pid: u32) -> io::Result<Option<ResUse>> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "try_wait4_pid requires wait4(2)",
            ))
        }
    }
}
