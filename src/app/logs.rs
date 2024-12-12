use std::{io::ErrorKind, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::error::Category;

use crate::types::json_log::LogEntry;

#[derive(Serialize, Deserialize, Debug)]
pub struct Logs {
    entries: Vec<LogEntry>,
}

impl Logs {
    pub fn new() -> std::io::Result<Self> {
        let v = std::fs::read(logs_file())?;
        let logs = match serde_json::from_slice(&v) {
            Ok(logs) => logs,
            Err(e) if e.classify() == Category::Eof => {
                let logs = Logs {
                    entries: Vec::new(),
                };
                logs.store()?;
                logs
            }
            Err(e) => return Err(std::io::Error::new(ErrorKind::Other, e)),
        };
        Ok(logs)
    }

    pub fn store(&self) -> std::io::Result<()> {
        let s =
            serde_json::to_string(&self).map_err(|e| std::io::Error::new(ErrorKind::Other, e))?;
        std::fs::write(logs_file(), s)
    }

    pub fn append(&mut self, entry: LogEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }
}

impl Default for Logs {
    fn default() -> Self {
        Logs::new().unwrap()
    }
}

fn logs_file() -> PathBuf {
    let path = logs_dir().join("logs.json");
    if !std::fs::exists(&path).expect("Can't check existence of config dir") {
        let _ = std::fs::File::create_new(&path);
    }
    path
}

fn logs_dir() -> PathBuf {
    let local_config_dir = dirs::config_local_dir().expect("Failed to get config dir!");
    let dir = local_config_dir.join("bucklog");
    if !std::fs::exists(&dir).expect("Can't check existence of config dir") {
        std::fs::create_dir(&dir).expect("Failed to create config dir");
    }
    dir
}
