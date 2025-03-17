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
        measure.value = self.sysinfo.used_swap() as f64 / self.sysinfo.total_swap() as f64 * 100.0;
        measure.tooltip = format!(
            "swap: {} used / {} free / {} total",
            format_size(self.sysinfo.used_swap(), DECIMAL),
            format_size(self.sysinfo.free_swap(), DECIMAL),
            format_size(self.sysinfo.total_swap(), DECIMAL)
        );
    }
}
