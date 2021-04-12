// Code partially from https://github.com/sharkdp/hyperfine
//
// Copyright (c) 2018-2020 The hyperfine developers
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use crate::*;

use std::mem;
use std::os::windows::io::AsRawHandle;
use std::time::Duration;
use std::{io, process::Child};

use winapi::um::processthreadsapi::GetProcessTimes;
use winapi::um::psapi::GetProcessMemoryInfo;
use winapi::um::psapi::PROCESS_MEMORY_COUNTERS;
use winapi::um::winnt::HANDLE;

impl Wait4 for Child {
    fn wait4(&mut self) -> io::Result<ResUse> {
        let handle = self.as_raw_handle();
        let status = self.wait()?;

        let (utime_nsec, stime_nsec) = unsafe {
            let mut _ctime = mem::zeroed();
            let mut _etime = mem::zeroed();
            let mut kernel_time = mem::zeroed();
            let mut user_time = mem::zeroed();
            let res = GetProcessTimes(
                handle as HANDLE,
                &mut _ctime,
                &mut _etime,
                &mut kernel_time,
                &mut user_time,
            );

            // GetProcessTimes will exit with non-zero if success as per:
            // https://msdn.microsoft.com/en-us/library/windows/desktop/ms683223(v=vs.85).aspx
            if res != 0 {
                let user = (((user_time.dwHighDateTime as i64) << 32)
                    + user_time.dwLowDateTime as i64)
                    * 100;
                let kernel = (((kernel_time.dwHighDateTime as i64) << 32)
                    + kernel_time.dwLowDateTime as i64)
                    * 100;
                (user as u64, kernel as u64)
            } else {
                (0, 0)
            }
        };

        let maxrss = unsafe {
            let mut pmc = mem::zeroed();
            let res = GetProcessMemoryInfo(
                handle as HANDLE,
                &mut pmc,
                std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            );
            if res != 0 {
                pmc.PeakWorkingSetSize as u64
            } else {
                0
            }
        };

        Ok(ResUse {
            status,
            rusage: ResourceUsage {
                utime: Duration::from_nanos(utime_nsec),
                stime: Duration::from_nanos(stime_nsec),
                maxrss,
            },
        })
    }
}
