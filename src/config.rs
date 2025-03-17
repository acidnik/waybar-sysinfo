use serde::Deserialize;
use serde_inline_default::serde_inline_default;

// TODO remove serde_inline_default after https://github.com/serde-rs/serde/issues/368 is done
#[serde_inline_default]
#[derive(Deserialize)]
pub struct Config {
    #[serde_inline_default(5000)]
    pub interval_ms: u64,

    pub cpu: Option<ConfigCpu>,
    pub mem: Option<ConfigMem>,
    pub net: Option<ConfigNet>,
    pub temp: Option<ConfigTemp>,
}

#[serde_inline_default]
#[derive(Deserialize)]
pub struct ConfigCpu {
    #[serde_inline_default("cpu".to_string())]
    pub label: String,
    pub show: Vec<String>,
}

#[serde_inline_default]
#[derive(Deserialize)]
pub struct ConfigMem {
    #[serde_inline_default("mem".to_string())]
    pub label: String,
    pub show: Vec<String>,
}

#[serde_inline_default]
#[derive(Deserialize)]
pub struct ConfigNet {
    #[serde_inline_default("net".to_string())]
    pub label: String,
    pub show: Vec<String>,
}

#[serde_inline_default]
#[derive(Deserialize)]
pub struct ConfigTemp {
    #[serde_inline_default("temp".to_string())]
    pub label: String,
    pub show: Option<Vec<String>>,
    pub show_max: Option<Vec<String>>,
}
