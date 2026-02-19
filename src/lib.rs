// // ignore some warnings in debug mode
// #![cfg_attr(
//     debug_assertions,
//     allow(
//         dead_code,
//         unused_imports,
//         unused_mut,
//         unused_variables,
//         unreachable_code
//     )
// )]

use std::time::Duration;

use config::Config;
use measure::MeasureCollector;
use waybar_cffi::{
    Module,
    gtk::{
        self, Orientation,
        glib::MainContext,
        traits::{ContainerExt, WidgetExt},
    },
    waybar_module,
};

mod config;
mod measure;
mod modules;
mod widget;

struct SysinfoModule {}

impl Module for SysinfoModule {
    type Config = Config;

    fn init(info: &waybar_cffi::InitInfo, config: Config) -> Self {
        let module = Self {};

        init(info, config);

        module
    }
}

waybar_module!(SysinfoModule);

fn init(info: &waybar_cffi::InitInfo, config: Config) {
    let root = info.get_root_widget();
    let container = gtk::Box::new(Orientation::Horizontal, 0);
    container.set_widget_name("sysinfo");
    root.add(&container);

    let context = MainContext::default();

    context.spawn_local(async move {
        eprintln!("sysinfo: module started");

        let mut collector = MeasureCollector::new(&config);
        for widget in &collector.widgets {
            container.add(widget.widget())
        }

        // first interval is very short, since we need 2 updates to measure stuff
        collector.update();
        smol::Timer::after(Duration::from_millis(1100)).await;

        loop {
            collector.update();
            smol::Timer::after(Duration::from_millis(config.interval_ms)).await;
            container.show_all();
        }
    });
}
