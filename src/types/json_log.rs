use std::collections::HashMap;

use egui::Color32;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

#[derive(Serialize, Deserialize, Debug)]
pub enum Level {
    INFO,
    ERROR,
    TRACE,
    DEBUG,
    WARN,
}

impl Level {
    pub fn color(&self) -> Color32 {
        match self {
            Level::INFO => Color32::GREEN,
            Level::ERROR => Color32::RED,
            Level::TRACE => Color32::BLUE,
            Level::DEBUG => Color32::ORANGE,
            Level::WARN => Color32::YELLOW,
        }
    }
    pub fn to_string(&self) -> String {
        match self {
            Level::INFO => "INFO".to_string(),
            Level::ERROR => "ERROR".to_string(),
            Level::TRACE => "TRACE".to_string(),
            Level::DEBUG => "DEBUG".to_string(),
            Level::WARN => "WARN".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    pub target: Option<String>,
    pub function: Option<String>,
    pub file: String,
    pub line: usize,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LogEntry {
    #[serde(with = "log_time_format")]
    pub timestamp: OffsetDateTime,
    pub level: Level,
    pub message: String,
    pub fields: HashMap<String, serde_json::Value>,
    pub span: Option<HashMap<String, serde_json::Value>>,
    pub source: Source,
}

mod log_time_format {
    use serde::{Deserialize, Deserializer};
    use time::OffsetDateTime;

    use crate::default_time_format;

    pub fn deserialize<'de, D>(d: D) -> Result<OffsetDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        Ok(time::OffsetDateTime::parse(&s, default_time_format()).unwrap())
    }

    pub fn serialize<S>(dt: &OffsetDateTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = dt.format(default_time_format()).unwrap();
        serializer.serialize_str(&s)
    }
}
