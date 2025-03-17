use std::collections::HashMap;

use crate::{
    config::Config,
    modules::{cpu::Cpu, mem::Mem, net::Net, temp::Temp},
    widget::Widget,
};

#[derive(Debug)]
pub struct Measure {
    /// label that used in "show": ["..."]
    #[allow(dead_code)]
    pub label: String,
    /// tooltip with extended info for this measure
    pub tooltip: String,
    /// value between 0.0 and 1.0
    pub value: f64,
}

impl Measure {
    pub fn new(label: &str) -> Self {
        Measure {
            label: label.to_owned(),
            tooltip: String::new(),
            value: 0.0,
        }
    }
}

pub trait SysinfoModule {
    /// Labels (keys from `Measures`) to display on widget
    fn labels(&self) -> Vec<String>;
    /// Update info, insert or update data in `measures`
    fn update(&mut self, measures: &mut Measures);
}

pub struct MeasureCollector {
    pub modules: Vec<Box<dyn SysinfoModule>>,
    pub widgets: Vec<Widget>,
    measures: Measures,
}

impl MeasureCollector {
    pub fn new(config: &Config) -> Self {
        let mut modules: Vec<Box<dyn SysinfoModule>> = Vec::new();
        let measures = HashMap::new();
        let mut widgets = Vec::new();
        if let Some(ref cpu_config) = config.cpu {
            let module = Cpu::new(cpu_config);
            let labels = module.labels();
            modules.push(Box::new(module));

            let widget = Widget::new(&cpu_config.label, &labels);
            widgets.push(widget);
        }
        if let Some(ref mem_config) = config.mem {
            let module = Mem::new(mem_config);
            let labels = module.labels();
            modules.push(Box::new(module));

            let widget = Widget::new(&mem_config.label, &labels);
            widgets.push(widget);
        }
        if let Some(ref net_config) = config.net {
            let module = Net::new(net_config);
            let labels = module.labels();
            modules.push(Box::new(module));

            let widget = Widget::new(&net_config.label, &labels);
            widgets.push(widget);
        }
        if let Some(ref temp_config) = config.temp {
            let module = Temp::new(temp_config);
            let labels = module.labels();
            modules.push(Box::new(module));

            let widget = Widget::new(&temp_config.label, &labels);
            widgets.push(widget);
        }
        MeasureCollector {
            modules,
            widgets,
            measures,
        }
    }

    pub fn update(&mut self) {
        eprintln!("sysinfo: update started");
        for module in self.modules.iter_mut() {
            module.update(&mut self.measures);
        }

        // eprintln!("sysinfo: measures = {:?}", self.measures);

        for widget in self.widgets.iter_mut() {
            widget.update(&self.measures)
        }
    }
}

pub type Measures = HashMap<String, Measure>;
