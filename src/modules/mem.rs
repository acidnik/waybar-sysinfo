use std::fs;

use humansize::{DECIMAL, format_size};
use sysinfo::{RefreshKind, System};

use crate::{
    config::ConfigMem,
    measure::{Measure, Measures, SysinfoModule},
};

pub struct Mem {
    sysinfo: sysinfo::System,
    labels: Vec<String>,
}

impl Mem {
    pub fn new(config: &ConfigMem) -> Self {
        let sysinfo = System::new_with_specifics(
            RefreshKind::nothing().with_memory(sysinfo::MemoryRefreshKind::everything()),
        );
        let labels = config.show.clone();
        Self { sysinfo, labels }
    }
}

impl SysinfoModule for Mem {
    fn labels(&self) -> Vec<String> {
        self.labels.clone()
    }

    fn update(&mut self, measures: &mut Measures) {
        self.sysinfo.refresh_memory();

        // /proc/meminfo reports these values in KiB.
        let meminfo = fs::read_to_string("/proc/meminfo").ok();
        let meminfo_value = |key: &str| -> Option<u64> {
            meminfo.as_deref()?.lines().find_map(|line| {
                let (name, value) = line.split_once(':')?;
                if name != key {
                    return None;
                }
                value
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()?
                    .checked_mul(1024)
            })
        };
        let zswapped = meminfo_value("Zswapped");
        let zswap_pool = meminfo_value("Zswap");

        let measure = measures
            .entry("mem".to_owned())
            .or_insert(Measure::new("mem"));
        measure.value =
            self.sysinfo.used_memory() as f64 / self.sysinfo.total_memory() as f64 * 100.0;
        measure.tooltip = format!(
            "mem: {} used / {} free / {} total",
            format_size(self.sysinfo.used_memory(), DECIMAL),
            format_size(self.sysinfo.free_memory(), DECIMAL),
            format_size(self.sysinfo.total_memory(), DECIMAL)
        );

        let measure = measures
            .entry("swap".to_owned())
            .or_insert(Measure::new("swap"));
        let swap_used = self.sysinfo.used_swap();
        let swap_free = self.sysinfo.free_swap();
        let swap_total = self.sysinfo.total_swap();
        let backing_used = zswapped.map_or(swap_used, |zswapped| {
            swap_used.saturating_sub(zswapped)
        });

        if swap_total == 0 {
            measure.value = 0.0
        } else {
            measure.value = backing_used as f64 / swap_total as f64 * 100.0;
        }

        measure.tooltip = match (zswapped, zswap_pool) {
            (Some(zswapped), Some(zswap_pool)) => format!(
                "swap: {} allocated / {} free / {} total\nbacking: ~{} / zswap: {} uncompressed / pool: {} compressed",
                format_size(swap_used, DECIMAL),
                format_size(swap_free, DECIMAL),
                format_size(swap_total, DECIMAL),
                format_size(swap_used.saturating_sub(zswapped), DECIMAL),
                format_size(zswapped, DECIMAL),
                format_size(zswap_pool, DECIMAL)
            ),
            (Some(zswapped), None) => format!(
                "swap: {} allocated / {} free / {} total\nbacking: ~{} / zswap: {} uncompressed / pool: unavailable",
                format_size(swap_used, DECIMAL),
                format_size(swap_free, DECIMAL),
                format_size(swap_total, DECIMAL),
                format_size(swap_used.saturating_sub(zswapped), DECIMAL),
                format_size(zswapped, DECIMAL)
            ),
            (None, _) => format!(
                "swap: {} used / {} free / {} total",
                format_size(swap_used, DECIMAL),
                format_size(swap_free, DECIMAL),
                format_size(swap_total, DECIMAL)
            ),
        };
    }
}
