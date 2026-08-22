use crate::*;

use std::time::Duration;
use std::{io, process::Child};

use libc::timeval;

#[allow(clippy::useless_conversion)]
fn timeval_to_duration(val: timeval) -> Duration {
    let v = i64::from(val.tv_sec) * 1_000_000 + i64::from(val.tv_usec);
    Duration::from_micros(v as u64)
}

std::cfg_select! {
    any(target_os = "macos", target_os = "linux", target_os = "freebsd") => {
        use std::os::unix::process::ExitStatusExt;

        fn maxrss_bytes(maxrss: i64) -> u64 {
            let v = maxrss.max(0) as u64;
            if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
                v
            } else {
                v.saturating_mul(1024)
            }
        }

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
        /// is returned the pid has been reaped; do not wait on it again.
        pub fn try_wait4_pid(pid: u32) -> io::Result<Option<ResUse>> {
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
        fn get_resource_usage(for_children: bool) -> ResourceUsage {
            let who = if for_children {
                libc::RUSAGE_CHILDREN
            } else {
                libc::RUSAGE_SELF
            };

            let rusage = unsafe {
                let mut rusage = std::mem::MaybeUninit::zeroed();
                libc::getrusage(who, rusage.as_mut_ptr());
                rusage.assume_init()
            };
            let maxrss = if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
                rusage.ru_maxrss
            } else {
                rusage.ru_maxrss * 1024
            };
            ResourceUsage {
                utime: timeval_to_duration(rusage.ru_utime),
                stime: timeval_to_duration(rusage.ru_stime),
                maxrss: maxrss as u64,
            }
        }

        impl ResourceUsage {
            /// Return resource usage statistics for all children of the calling process
            /// that have terminated and been waited for.
            /// These statistics will include the resources used by grandchildren,
            /// and further removed descendants, if all of the intervening descendants waited
            /// on their terminated children.
            fn for_children() -> ResourceUsage {
                get_resource_usage(true)
            }
            /*
            /// Return resource usage statistics for the calling process, which is the sum
            /// of resources used by all threads in the process.
            fn for_self() -> ResourceUsage {
                get_resource_usage(false)
            }
            */
        }

        impl Wait4 for Child {
            fn wait4(mut self) -> io::Result<ResUse> {
                self.wait().map(|status| {
                    let rusage = ResourceUsage::for_children();
                    ResUse { status, rusage }
                })
            }

            fn try_wait4(&mut self) -> io::Result<Option<ResUse>> {
                self.try_wait().map(|status| {
                    status.map(|status| ResUse {
                        status,
                        rusage: ResourceUsage::for_children(),
                    })
                })
            }
        }

        /// No native `wait4`; fall back to [`Child::try_wait`] + `RUSAGE_CHILDREN`.
        pub fn try_wait4_pid(_pid: u32) -> io::Result<Option<ResUse>> {
            Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "try_wait4_pid requires wait4(2)",
            ))
        }
    }
}
