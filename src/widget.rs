use waybar_cffi::gtk::{
    self as gtk, Orientation,
    prelude::{
        ContainerExt, CssProviderExt, OrientableExt, ProgressBarExt, StyleContextExt, WidgetExt,
    },
};

use crate::measure::Measures;

pub struct Widget {
    box_: gtk::Box,
    #[allow(dead_code)]
    label: gtk::Label,
    bar: Vec<gtk::ProgressBar>,
    show: Vec<String>,
}

const DEFAULT_CSS: &str = r#"
#sysinfo .sysinfo-bar {
    padding-left: 5px;
    padding-top: 5px;
    padding-bottom: 5px;
}

/* progress bar */
#sysinfo trough {
    min-height: 3px;
    min-width: 7px;
    border: none;
}

/* colored part of progress bar */
#sysinfo progress {
    border: none;
    min-width: 7px;
}

#sysinfo .cpu progress {
  background-color: #d20f39;
}

#sysinfo .mem progress {
  background-color: #40a02b;
}

#sysinfo .net progress {
  background-color: #1e66f5;
}

#sysinfo .temp progress {
  background-color: #df8e1d;
}
"#;

thread_local! {
    static BUTTON_CSS_PROVIDER: gtk::CssProvider = {
        let css = gtk::CssProvider::new();
        if let Err(e) = css.load_from_data(DEFAULT_CSS.as_bytes()) {
            eprintln!("css parse error: {e:?}");
        }

        css
    };
}

impl Widget {
    pub fn new(label_str: &str, show_str: &[String]) -> Self {
        let box_ = gtk::Box::new(gtk::Orientation::Horizontal, 0);

        BUTTON_CSS_PROVIDER.with(|provider| {
            box_.style_context()
                .add_provider(provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);

            box_.style_context().add_class(label_str);
            // TODO: use config.<module>.label
            let label = gtk::Label::new(Some(label_str));
            box_.add(&label);
            let mut bar = Vec::new();
            let mut show = Vec::new();
            for s in show_str.iter() {
                show.push(s.to_owned());
                let b = gtk::ProgressBar::new();
                b.set_orientation(Orientation::Vertical);
                b.set_inverted(true);
                b.style_context()
                    .add_provider(provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
                b.style_context().add_class(s);
                b.style_context().add_class("sysinfo-bar");
                bar.push(b);
                box_.add(&bar[bar.len() - 1]);
            }

            box_.show_all();

            Self {
                box_,
                label,
                bar,
                show,
            }
        })
    }

    pub fn widget(&self) -> &gtk::Box {
        &self.box_
    }

    /// take the measures we are displaying and update widget
    pub fn update(&mut self, measures: &Measures) {
        let mut tooltip = Vec::new();
        for (i, label) in self.show.iter().enumerate() {
            if let Some(measure) = measures.get(label) {
                if !measure.tooltip.is_empty() {
                    tooltip.push(measure.tooltip.clone());
                }
                let bar = &self.bar[i];
                bar.set_fraction(measure.value / 100.0);
            }
        }

        let tooltip: String = tooltip.join("\n");
        self.box_.set_tooltip_text(Some(&tooltip));
        // self.box_.show_all();
    }
}
