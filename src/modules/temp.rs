use std::collections::HashSet;

use regex::Regex;

use crate::{
    config::ConfigTemp,
    measure::{Measure, Measures, SysinfoModule},
};

pub struct Temp {
    components: sysinfo::Components,
    // show_max: [core*, foo*] => [ [core1, core2], [foo1, foo2] ] => show max(core1, core2), max(foo1, foo2)
    max_temp: Vec<HashSet<String>>,
    // all labels to display
    labels: HashSet<String>,
}

impl Temp {
    pub fn new(config: &ConfigTemp) -> Self {
        let components = sysinfo::Components::new_with_refreshed_list();
        // for each regex from `show_max` build a vec of matched labels
        let max_temp: Vec<HashSet<String>> = config
            .show_max
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|r| Regex::new(r).unwrap())
            .map(|r| {
                components
                    .list()
                    .iter()
                    .filter_map(|c| r.is_match(c.label()).then_some(c.label().to_owned()))
                    .collect()
            })
            .filter(|x: &HashSet<String>| !x.is_empty())
            .collect();

        let labels: HashSet<String> = config
            .show
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|r| Regex::new(r).unwrap())
            .map(|r| {
                components
                    .list()
                    .iter()
                    .filter_map(|c| r.is_match(c.label()).then_some(c.label().to_owned()))
                    .collect()
            })
            .chain(
                max_temp
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("temp_max_{i}")),
            )
            .collect();

        Self {
            components,
            max_temp,
            labels,
        }
    }
}

impl SysinfoModule for Temp {
    fn labels(&self) -> Vec<String> {
        self.labels.clone().into_iter().collect()
    }

    fn update(&mut self, measures: &mut Measures) {
        self.components.refresh(true);
        let components: Vec<_> = self
            .components
            .list()
            .iter()
            .filter_map(|c| {
                if c.temperature().is_some() && c.critical().is_some() {
                    Some((
                        c.label().to_owned(),
                        c.temperature().unwrap(),
                        c.critical().unwrap(),
                    ))
                } else {
                    None
                }
            })
            .collect();

        // get max values
        for (i, ms) in self.max_temp.iter().enumerate() {
            let c = components
                .iter()
                .filter(|c| ms.contains(&c.0))
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
                .unwrap();
            let name = format!("temp_max_{i}");
            let measure = measures.entry(name.clone()).or_insert(Measure::new(&name));
            measure.value = (c.1 / c.2 * 100.0) as f64;
            measure.tooltip = format!("max: {} [ {}C / {}C ]", c.0, c.1, c.2);
        }

        // get just values
        for c in components {
            if self.labels.contains(&c.0) {
                let measure = measures.entry(c.0.clone()).or_insert(Measure::new(&c.0));
                measure.value = (c.1 / c.2 * 100.0) as f64;
                measure.tooltip = format!("{}: [ {}C / {}C ]", c.0, c.1, c.2)
            }
        }
    }
}
