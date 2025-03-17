use std::{
    collections::{BinaryHeap, HashMap},
    time::Instant,
};

use humansize::{DECIMAL, format_size};
use regex::Regex;

use crate::{
    config::ConfigNet,
    measure::{Measure, Measures, SysinfoModule},
};

#[derive(Default)]
struct AutoMax {
    inner: HashMap<String, BinaryHeap<u64>>,
}

impl AutoMax {
    fn update(&mut self, dev: &str, value: u64) -> u64 {
        let queue = self.inner.entry(dev.to_owned()).or_default();
        queue.push(value);

        // TODO config instead of hardcoded?
        let val = if queue.len() >= 10 {
            queue.pop().unwrap()
        } else {
            *queue.peek().unwrap()
        };
        if val < 5_000 { 5_000 } else { val }
    }
}

pub struct Net {
    networks: sysinfo::Networks,
    last_update: Instant,
    max_limit: AutoMax,
    labels: Vec<String>,
}

impl Net {
    pub fn new(config: &ConfigNet) -> Self {
        let networks = sysinfo::Networks::new_with_refreshed_list();
        let last_update = Instant::now();
        let max_limit = Default::default();

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
            measure.value = send_per_sec as f64 / max as f64 * 100.0;

            let dev_recv = dev.to_owned() + "_recv";
            let measure = measures
                .entry(dev_recv.clone())
                .or_insert(Measure::new(&dev_recv));
            measure.value = recv_per_sec as f64 / max as f64 * 100.0;

            measure.tooltip = format!(
                "{dev}: {} in / {} out",
                format_size(recv_per_sec, DECIMAL),
                format_size(send_per_sec, DECIMAL)
            );
        }
    }
}
