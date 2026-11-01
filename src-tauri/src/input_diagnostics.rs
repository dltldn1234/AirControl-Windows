use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static INPUT_EVENT_COUNT: AtomicU64 = AtomicU64::new(0);

#[cfg_attr(not(windows), allow(dead_code))]
pub fn record_input_events(count: usize) {
    INPUT_EVENT_COUNT.fetch_add(count as u64, Ordering::Relaxed);
}

pub fn input_event_count() -> u64 {
    INPUT_EVENT_COUNT.load(Ordering::Relaxed)
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ProcessUsage {
    pub cpu_percent: Option<f32>,
    pub memory_bytes: Option<u64>,
}

#[derive(Default)]
pub struct ProcessUsageSampler {
    previous_cpu_100ns: Option<u64>,
}

impl ProcessUsageSampler {
    pub fn sample(&mut self, elapsed: Duration) -> ProcessUsage {
        platform::sample(&mut self.previous_cpu_100ns, elapsed)
    }
}

#[cfg(windows)]
mod platform {
    use super::{Duration, ProcessUsage};
    use std::mem::size_of;
    use windows_sys::Win32::System::ProcessStatus::{
        K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
    };
    use windows_sys::Win32::System::SystemInformation::{GetSystemInfo, SYSTEM_INFO};
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

    pub fn sample(previous: &mut Option<u64>, elapsed: Duration) -> ProcessUsage {
        let process = unsafe { GetCurrentProcess() };
        let mut created = Default::default();
        let mut exited = Default::default();
        let mut kernel = Default::default();
        let mut user = Default::default();
        let cpu_percent = if unsafe {
            GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user)
        } != 0
        {
            let cpu = filetime_to_u64(kernel).saturating_add(filetime_to_u64(user));
            let percent = previous.replace(cpu).and_then(|old| {
                let elapsed_100ns = elapsed.as_nanos() / 100;
                let processor_count = processor_count().max(1) as u128;
                (elapsed_100ns > 0).then(|| {
                    ((cpu.saturating_sub(old) as u128 * 100) / (elapsed_100ns * processor_count))
                        .min(100) as f32
                })
            });
            percent
        } else {
            *previous = None;
            None
        };

        let mut counters = PROCESS_MEMORY_COUNTERS {
            cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            ..Default::default()
        };
        let memory_bytes = (unsafe {
            K32GetProcessMemoryInfo(
                process,
                &mut counters,
                size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
            )
        } != 0)
            .then_some(counters.WorkingSetSize as u64);

        ProcessUsage {
            cpu_percent,
            memory_bytes,
        }
    }

    fn filetime_to_u64(time: windows_sys::Win32::Foundation::FILETIME) -> u64 {
        (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)
    }

    fn processor_count() -> u32 {
        let mut info = SYSTEM_INFO::default();
        unsafe { GetSystemInfo(&mut info) };
        info.dwNumberOfProcessors
    }
}

#[cfg(not(windows))]
mod platform {
    use super::{Duration, ProcessUsage};

    pub fn sample(_previous: &mut Option<u64>, _elapsed: Duration) -> ProcessUsage {
        ProcessUsage::default()
    }
}

#[cfg(test)]
mod tests {
    use super::{ProcessUsageSampler, input_event_count, record_input_events};
    use std::time::Duration;

    #[test]
    fn system_input_count_records_only_explicitly_dispatched_events() {
        let initial = input_event_count();
        record_input_events(3);
        assert_eq!(input_event_count(), initial + 3);
    }

    #[test]
    fn process_metrics_are_optional_when_the_host_does_not_expose_them() {
        let mut sampler = ProcessUsageSampler::default();
        let usage = sampler.sample(Duration::from_secs(1));
        #[cfg(not(windows))]
        assert_eq!(usage.cpu_percent, None);
        #[cfg(not(windows))]
        assert_eq!(usage.memory_bytes, None);
        #[cfg(windows)]
        assert!(usage.memory_bytes.is_some());
    }
}
