use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Logs {
    pub file_level: LevelFilter,
    pub file_timestamp: Timestamp,
    pub pane_level: LevelFilter,
    pub max_file_count: usize,
}

impl Default for Logs {
    fn default() -> Self {
        Self {
            file_level: LevelFilter::Debug,
            file_timestamp: Timestamp::default(),
            pane_level: LevelFilter::Info,
            max_file_count: 4,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<LevelFilter> for log::LevelFilter {
    fn from(level: LevelFilter) -> Self {
        match level {
            LevelFilter::Off => log::LevelFilter::Off,
            LevelFilter::Error => log::LevelFilter::Error,
            LevelFilter::Warn => log::LevelFilter::Warn,
            LevelFilter::Info => log::LevelFilter::Info,
            LevelFilter::Debug => log::LevelFilter::Debug,
            LevelFilter::Trace => log::LevelFilter::Trace,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Timestamp {
    #[default]
    Local,
    Utc,
}
