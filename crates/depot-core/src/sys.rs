#[cfg(windows)]
pub mod windows {
    use std::path::Path;
    use windows_sys::Win32::Foundation::INVALID_HANDLE_VALUE;
    use windows_sys::Win32::System::Console::{
        GetConsoleMode, GetStdHandle, SetConsoleMode, ENABLE_VIRTUAL_TERMINAL_PROCESSING,
        STD_ERROR_HANDLE, STD_OUTPUT_HANDLE,
    };
    use windows_sys::Win32::System::Performance::{
        QueryPerformanceCounter, QueryPerformanceFrequency,
    };
    use windows_sys::Win32::System::SystemInformation::{
        GetSystemInfo, GlobalMemoryStatusEx, MEMORYSTATUSEX, SYSTEM_INFO,
    };
    use windows_sys::Win32::Storage::FileSystem::CreateHardLinkW;

    /// Enables Virtual Terminal (VT) ANSI escape sequence processing on Windows stdout and stderr.
    pub fn enable_vt_processing() -> bool {
        unsafe {
            let mut ok = true;
            for handle_id in [STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
                let handle = GetStdHandle(handle_id);
                if handle != INVALID_HANDLE_VALUE && !handle.is_null() {
                    let mut mode: u32 = 0;
                    if GetConsoleMode(handle, &mut mode) != 0 {
                        let new_mode = mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING;
                        if SetConsoleMode(handle, new_mode) == 0 {
                            ok = false;
                        }
                    }
                }
            }
            ok
        }
    }

    /// Queries system physical memory (total_bytes, available_bytes) via GlobalMemoryStatusEx.
    pub fn get_memory_status() -> Option<(u64, u64)> {
        unsafe {
            let mut mem: MEMORYSTATUSEX = std::mem::zeroed();
            mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
            if GlobalMemoryStatusEx(&mut mem) != 0 {
                Some((mem.ullTotalPhys, mem.ullAvailPhys))
            } else {
                None
            }
        }
    }

    /// Queries processor count and architecture via GetSystemInfo.
    pub fn get_cpu_info() -> (u32, u16) {
        unsafe {
            let mut info: SYSTEM_INFO = std::mem::zeroed();
            GetSystemInfo(&mut info);
            let arch = info.Anonymous.Anonymous.wProcessorArchitecture;
            (info.dwNumberOfProcessors, arch)
        }
    }

    /// Creates an NTFS hard link using Win32 CreateHardLinkW.
    pub fn create_hard_link(link: &Path, target: &Path) -> std::io::Result<()> {
        use std::os::windows::ffi::OsStrExt;
        let wide_link: Vec<u16> = link.as_os_str().encode_wide().chain(Some(0)).collect();
        let wide_target: Vec<u16> = target.as_os_str().encode_wide().chain(Some(0)).collect();
        unsafe {
            if CreateHardLinkW(wide_link.as_ptr(), wide_target.as_ptr(), std::ptr::null()) != 0 {
                Ok(())
            } else {
                Err(std::io::Error::last_os_error())
            }
        }
    }

    /// Queries high-resolution performance counter via QueryPerformanceCounter.
    pub fn query_perf_counter() -> u64 {
        unsafe {
            let mut count: i64 = 0;
            QueryPerformanceCounter(&mut count);
            count as u64
        }
    }

    /// Queries high-resolution performance frequency via QueryPerformanceFrequency.
    pub fn query_perf_frequency() -> u64 {
        unsafe {
            let mut freq: i64 = 0;
            QueryPerformanceFrequency(&mut freq);
            freq as u64
        }
    }
}

#[cfg(not(windows))]
pub mod fallback {
    use std::path::Path;

    pub fn enable_vt_processing() -> bool {
        true
    }

    pub fn get_memory_status() -> Option<(u64, u64)> {
        None
    }

    pub fn get_cpu_info() -> (u32, u16) {
        let count = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1);
        (count, 0)
    }

    pub fn create_hard_link(link: &Path, target: &Path) -> std::io::Result<()> {
        std::fs::hard_link(target, link)
    }

    pub fn query_perf_counter() -> u64 {
        0
    }

    pub fn query_perf_frequency() -> u64 {
        1_000_000_000
    }
}

#[cfg(windows)]
pub use windows::*;

#[cfg(not(windows))]
pub use fallback::*;
