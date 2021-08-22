use crate::*;

use std::time::Duration;
use std::{io, process::Child};

use libc::timeval;

#[allow(clippy::useless_conversion)]
fn timeval_to_duration(val: timeval) -> Duration {
    let v = i64::from(val.tv_sec) * 1_000_000 + i64::from(val.tv_usec);
    Duration::from_micros(v as u64)
}

cfg_if::cfg_if! {
    if #[cfg(any(target_os = "macos", target_os = "linux", target_os = "freebsd"))] {
        use std::os::unix::process::ExitStatusExt;

        impl Wait4 for Child {
            fn wait4(&mut self) -> io::Result<ResUse> {
                drop(self.stdin.take());
                let pid = self.id() as i32;
                let mut status = 0;
                let options = 0;
                let mut rusage = std::mem::MaybeUninit::zeroed();

                let r = unsafe { libc::wait4(pid, &mut status, options, rusage.as_mut_ptr()) };

                if r < 0 {
                    Err(io::Error::last_os_error())
                } else {
                    let rusage = unsafe { rusage.assume_init() };

                    let maxrss = if cfg!(target_os = "macos") || cfg!(target_os = "ios") {
                        rusage.ru_maxrss
                    } else {
                        rusage.ru_maxrss / 1024
                    };

                    Ok(ResUse {
                        status: ExitStatus::from_raw(status),
                        rusage: ResourceUsage {
                            utime: timeval_to_duration(rusage.ru_utime),
                            stime: timeval_to_duration(rusage.ru_stime),
                            maxrss: maxrss as u64,
                        },
                    })
                }
            }
        }
    } else {
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
                rusage.ru_maxrss / 1024
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
            fn wait4(&mut self) -> io::Result<ResUse> {
                self.wait().map(|status| {
                    let rusage = ResourceUsage::for_children();
                    ResUse { status, rusage }
                })
            }
        }
    }
}
