use crate::*;

use std::os::unix::process::ExitStatusExt;
use std::time::Duration;
use std::{io, process::Child};

use libc::timeval;

fn timeval_to_duration(val: timeval) -> Duration {
    let v = i64::from(val.tv_sec) * 1000_000 + i64::from(val.tv_usec);
    Duration::from_micros(v as u64)
}

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
