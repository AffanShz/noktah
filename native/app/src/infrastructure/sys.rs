//! Pembaca beban sistem, langsung dari Win32.
//!
//! CPU dihitung dari selisih waktu idle terhadap total antar pemanggilan,
//! jadi angka pertama selalu 0 — itu wajar, bukan bug.

use std::collections::VecDeque;

const HISTORY: usize = 60;

pub struct SysSampler {
    pub cpu: f32,
    pub ram_used: u64,
    pub ram_total: u64,
    pub uptime_secs: u64,
    pub cores: usize,
    pub history: VecDeque<f32>,
    last: Option<(u64, u64, u64)>,
}

impl Default for SysSampler {
    fn default() -> Self {
        Self {
            cpu: 0.0,
            ram_used: 0,
            ram_total: 0,
            uptime_secs: 0,
            cores: 1,
            history: VecDeque::with_capacity(HISTORY),
            last: None,
        }
    }
}

impl SysSampler {
    pub fn tick(&mut self) {
        #[cfg(windows)]
        self.tick_windows();

        self.history.push_back(self.cpu);
        if self.history.len() > HISTORY {
            self.history.pop_front();
        }
    }

    #[cfg(windows)]
    fn tick_windows(&mut self) {
        use windows::Win32::Foundation::FILETIME;
        use windows::Win32::System::SystemInformation::{
            GetSystemInfo, GetTickCount64, GlobalMemoryStatusEx, SYSTEM_INFO, MEMORYSTATUSEX,
        };
        use windows::Win32::System::Threading::GetSystemTimes;

        unsafe {
            let mut idle = FILETIME::default();
            let mut kernel = FILETIME::default();
            let mut user = FILETIME::default();

            if GetSystemTimes(Some(&mut idle), Some(&mut kernel), Some(&mut user)).is_ok() {
                let ke = |f: &FILETIME| ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64;
                let now = (ke(&idle), ke(&kernel), ke(&user));

                if let Some((pi, pk, pu)) = self.last {
                    let d_idle = now.0.saturating_sub(pi) as f32;
                    // kernel sudah termasuk idle
                    let d_total = (now.1.saturating_sub(pk) + now.2.saturating_sub(pu)) as f32;
                    if d_total > 0.0 {
                        self.cpu = (100.0 * (1.0 - d_idle / d_total)).clamp(0.0, 100.0);
                    }
                }
                self.last = Some(now);
            }

            let mut mem = MEMORYSTATUSEX {
                dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
                ..Default::default()
            };
            if GlobalMemoryStatusEx(&mut mem).is_ok() {
                self.ram_total = mem.ullTotalPhys;
                self.ram_used = mem.ullTotalPhys.saturating_sub(mem.ullAvailPhys);
            }

            self.uptime_secs = GetTickCount64() / 1000;

            let mut info = SYSTEM_INFO::default();
            GetSystemInfo(&mut info);
            self.cores = info.dwNumberOfProcessors.max(1) as usize;
        }
    }

    pub fn ram_fraction(&self) -> f32 {
        if self.ram_total == 0 {
            0.0
        } else {
            self.ram_used as f32 / self.ram_total as f32
        }
    }
}
