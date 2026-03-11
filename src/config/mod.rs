pub mod commands;
pub mod editor;

use crate::error::{Error, Result, io_err};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DIR_NAME: &str = "assignment_packer";
const FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub name: Option<String>,
    #[serde(rename = "id", alias = "student_id")]
    pub student_id: Option<String>,
    pub output_dir: Option<PathBuf>,
    #[serde(alias = "autoDoc")]
    pub auto_doc: Option<bool>,
    #[serde(alias = "runCommand")]
    pub run_command: Option<String>,
    #[serde(alias = "runDisplayTemplate")]
    pub run_display_template: Option<String>,
    pub theme: Option<String>,
    pub editor: Option<String>,
    pub watermark: Option<bool>,
    pub input: Option<String>,
    pub timeout: Option<u64>,
}

pub fn config_path() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| Error::Validation("can't determine user config directory".into()))?;
    Ok(base.join(DIR_NAME).join(FILE_NAME))
}

pub fn load(path: &Path) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(path).map_err(|e| io_err("reading config", e))?;
    toml::from_str(&content).map_err(|e| Error::Validation(format!("bad config: {e}")))
}

pub fn save(path: &Path, cfg: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| io_err("creating config directory", e))?;
    }
    let content = toml::to_string_pretty(cfg)
        .map_err(|e| Error::Validation(format!("serializing config: {e}")))?;
    fs::write(path, content).map_err(|e| io_err("writing config", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trips() {
        let cfg = AppConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let parsed: AppConfig = toml::from_str(&s).unwrap();
        assert!(parsed.name.is_none());
    }

    #[test]
    fn config_with_values_round_trips() {
        let cfg = AppConfig {
            name: Some("Alice".into()),
            student_id: Some("12345".into()),
            auto_doc: Some(true),
            ..Default::default()
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let parsed: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed.name.as_deref(), Some("Alice"));
        assert_eq!(parsed.auto_doc, Some(true));
    }
}
