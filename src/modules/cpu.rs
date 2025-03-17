use sysinfo::{CpuRefreshKind, RefreshKind, System};

use crate::{
    config::ConfigCpu,
    measure::{Measure, Measures, SysinfoModule},
};

pub struct Cpu {
    sysinfo: sysinfo::System,
    labels: Vec<String>,
    core_labels: Vec<String>,
}

impl Cpu {
    pub fn new(config: &ConfigCpu) -> Self {
        let mut sysinfo = System::new_with_specifics(
            RefreshKind::nothing().with_cpu(CpuRefreshKind::everything()),
        );
        sysinfo.refresh_cpu_all();
        let core_labels: Vec<String> = sysinfo
            .cpus()
            .iter()
            .enumerate()
            .map(|(i, _)| format!("cpu_core_{i}"))
            .collect();
        let mut labels = Vec::new();
        for s in &config.show {
            if s == "all_cores" {
                labels.extend(core_labels.clone());
            } else {
                labels.push(s.to_owned())
            }
        }
        Self {
            sysinfo,
            labels,
            core_labels,
        }
    }
}

impl SysinfoModule for Cpu {
    fn labels(&self) -> Vec<String> {
        self.labels.clone()
    }

    fn update(&mut self, measures: &mut Measures) {
        self.sysinfo.refresh_cpu_all();
        let mut cpu_total = 0.0f64;
        let mut cpu_max = (0.0, 0);
        for (i, (cpu, label)) in self
            .sysinfo
            .cpus()
            .iter()
            .zip(&self.core_labels)
            .enumerate()
        {
            // value is between 0.0 and 100.0 (%)
            let value = cpu.cpu_usage() as f64;
            let measure = measures.entry(label.clone()).or_insert(Measure::new(label));
            measure.value = value;
            measure.tooltip = format!("core_{i}: {value:.0}%");
            cpu_total += value;
            if value > cpu_max.0 {
                cpu_max = (value, i);
            }
        }
        let cpu_avg = cpu_total / self.sysinfo.cpus().len() as f64;
        let measure = measures
            .entry("avg_core".to_owned())
            .or_insert(Measure::new("avg_core"));
        measure.value = cpu_avg;
        measure.tooltip = format!("avg: {:.0}%", cpu_avg);

        let measure = measures
            .entry("max_core".to_owned())
            .or_insert(Measure::new("max_core"));
        measure.value = cpu_max.0;
        measure.tooltip = format!("Max core: core_{} ({:.0}%)", cpu_max.1, cpu_max.0);
    }
}
