use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    /// Skill repository folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Install destination folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

pub fn config_path() -> Result<PathBuf> {
    let dir = dirs::config_dir()
        .context("cannot determine user config directory")?
        .join("skillbox");
    Ok(dir.join("config.toml"))
}

pub fn load() -> Result<Config> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(Config::default());
    }
    let raw = fs::read_to_string(&path)
        .with_context(|| format!("failed to read {}", path.display()))?;
    toml::from_str(&raw).with_context(|| format!("failed to parse {}", path.display()))
}

pub fn save(config: &Config) -> Result<()> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let raw = toml::to_string_pretty(config)?;
    fs::write(&path, raw).with_context(|| format!("failed to write {}", path.display()))
}

pub fn default_target() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".agents")
        .join("skills")
}

pub fn default_source() -> PathBuf {
    PathBuf::from("skills")
}

/// Expand a leading `~` to the user's home directory.
pub fn expand_tilde(path: &str) -> PathBuf {
    if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    } else if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

/// Resolve the effective source folder: flag > config > default.
pub fn resolve_source(flag: Option<&Path>, config: &Config) -> PathBuf {
    if let Some(p) = flag {
        return p.to_path_buf();
    }
    if let Some(s) = &config.source {
        return expand_tilde(s);
    }
    default_source()
}

/// Resolve the effective target folder: flag > config > ~/.agents/skills.
pub fn resolve_target(flag: Option<&Path>, config: &Config) -> PathBuf {
    if let Some(p) = flag {
        return p.to_path_buf();
    }
    if let Some(s) = &config.target {
        return expand_tilde(s);
    }
    default_target()
}
