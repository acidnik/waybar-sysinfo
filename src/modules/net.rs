use std::{
    collections::{HashMap, VecDeque},
    time::Instant,
};

use humansize::{DECIMAL, format_size};
use regex::Regex;

use crate::{
    config::{ConfigNet, ConfigNetScaling},
    measure::{Measure, Measures, SysinfoModule},
};

impl ConfigNetScaling {
    fn scale(&self, value: f64, max: f64) -> f64 {
        if max <= 0.0 {
            return if value <= 0.0 { 0.0 } else { 1.0 };
        }
        let scaled = match self {
            ConfigNetScaling::Linear => value / max,
            ConfigNetScaling::Power { exponent } => (value / max).powf(*exponent),
            ConfigNetScaling::LogPower { exponent } => {
                ((value + 1.0).ln() / (max + 1.0).ln()).powf(*exponent)
            }
        };
        return scaled.clamp(0.0, 1.0);
    }
}

#[derive(Default)]
struct AutoMax {
    inner: HashMap<String, VecDeque<u64>>,
    floor: u64,
}

impl AutoMax {
    fn new(floor: u64) -> Self {
        Self {
            inner: Default::default(),
            floor,
        }
    }

    fn update(&mut self, dev: &str, value: u64) -> u64 {
        let queue = self.inner.entry(dev.to_owned()).or_default();
        queue.push_back(value);

        // TODO config instead of hardcoded?
        if queue.len() > 10 {
            queue.pop_front();
        }
        // can we do better than O(N)?
        (*queue.iter().max().unwrap_or(&self.floor)).max(self.floor)
    }
}

pub struct Net {
    networks: sysinfo::Networks,
    last_update: Instant,
    max_limit: AutoMax,
    labels: Vec<String>,
    scaling: ConfigNetScaling,
}

impl Net {
    pub fn new(config: &ConfigNet) -> Self {
        let networks = sysinfo::Networks::new_with_refreshed_list();
        let last_update = Instant::now();
        let max_limit = AutoMax::new(config.floor.unwrap_or(5000));

        // config.show is a vec of regex; transform it into list of dev1_send, dev1_recv, ...
        let labels = config
            .show
            .iter()
            .map(|r| Regex::new(r).unwrap())
            .flat_map(|r| {
                networks
                    .list()
                    .keys()
                    .filter_map(|n| {
                        r.is_match(n)
                            .then_some(vec![n.to_owned() + "_recv", n.to_owned() + "_send"])
                    })
                    .collect::<Vec<Vec<String>>>()
            })
            .flatten()
            .collect::<Vec<String>>();

        Self {
            networks,
            last_update,
            max_limit,
            labels,
            scaling: config.scaling.clone(),
        }
    }
}

impl SysinfoModule for Net {
    fn labels(&self) -> Vec<String> {
        self.labels.clone()
    }

    fn update(&mut self, measures: &mut Measures) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update);
        if dt.as_secs() == 0 {
            return;
        }
        self.networks.refresh(true);
        self.last_update = now;

        for (dev, net) in self.networks.list() {
            let recv_per_sec = net.received() / dt.as_secs();
            let send_per_sec = net.transmitted() / dt.as_secs();
            let max = self
                .max_limit
                .update(dev, recv_per_sec)
                .max(self.max_limit.update(dev, send_per_sec));
            if max == 0 {
                continue;
            }

            let dev_send = dev.to_owned() + "_send";
            let measure = measures
                .entry(dev_send.clone())
                .or_insert(Measure::new(&dev_send));
            measure.value = self.scaling.scale(send_per_sec as f64, max as f64) * 100.0;

            let dev_recv = dev.to_owned() + "_recv";
            let measure = measures
                .entry(dev_recv.clone())
                .or_insert(Measure::new(&dev_recv));
            measure.value = self.scaling.scale(recv_per_sec as f64, max as f64) * 100.0;

            measure.tooltip = format!(
                "{dev}: {} in / {} out",
                format_size(recv_per_sec, DECIMAL),
                format_size(send_per_sec, DECIMAL)
            );
        }
    }
}
