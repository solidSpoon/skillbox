use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Known agents and their skill folders (relative to $HOME).
pub const AGENTS: &[(&str, &str)] = &[("codex", ".codex/skills"), ("pi", ".agents/skills")];
pub const DEFAULT_AGENT: &str = "codex";

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Config {
    /// Skill repository folder.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Optional non-standard install destination (overrides agent registry).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    /// Agents operated on when --agent is omitted (empty = DEFAULT_AGENT).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub agents: Vec<String>,
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
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let raw = toml::to_string_pretty(config)?;
    fs::write(&path, raw).with_context(|| format!("failed to write {}", path.display()))
}

pub fn known_agents() -> String {
    AGENTS.iter().map(|(n, _)| *n).collect::<Vec<_>>().join(", ")
}

pub fn agent_dir(name: &str) -> Option<PathBuf> {
    AGENTS.iter().find(|(n, _)| *n == name).map(|(_, rel)| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(rel)
    })
}

pub fn is_known_agent(name: &str) -> bool {
    AGENTS.iter().any(|(n, _)| *n == name)
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

pub fn default_source() -> PathBuf {
    PathBuf::from("skills")
}

/// Canonical clone location used by install.sh.
pub fn canonical_repo_skills() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".skillbox")
        .join("repo")
        .join("skills")
}

/// Resolve the effective source folder:
/// flag > config > ~/.skillbox/repo/skills > ./skills > ./skills (for the error).
pub fn resolve_source(flag: Option<&Path>, config: &Config) -> PathBuf {
    if let Some(p) = flag {
        return p.to_path_buf();
    }
    if let Some(s) = &config.source {
        return expand_tilde(s);
    }
    let canonical = canonical_repo_skills();
    if canonical.is_dir() {
        return canonical;
    }
    PathBuf::from("skills")
}

/// Resolve the selected agents:
/// flags ("all" expands) > config.agents > [DEFAULT_AGENT].
pub fn resolve_agents(flags: &[String], config: &Config) -> Result<Vec<String>> {
    let mut raw: Vec<String> = Vec::new();
    if flags.is_empty() {
        if config.agents.is_empty() {
            raw.push(DEFAULT_AGENT.to_string());
        } else {
            raw = config.agents.clone();
        }
    } else {
        for flag in flags {
            if flag == "all" {
                for (name, _) in AGENTS {
                    raw.push(name.to_string());
                }
            } else {
                raw.push(flag.clone());
            }
        }
    }
    let mut out: Vec<String> = Vec::new();
    for name in raw {
        if !is_known_agent(&name) {
            bail!("unknown agent '{}'. known agents: {}", name, known_agents());
        }
        if !out.contains(&name) {
            out.push(name);
        }
    }
    Ok(out)
}

/// Install destination for one agent: flag > config.target > agent registry dir.
pub fn target_for(agent: &str, config: &Config, flag: Option<&Path>) -> PathBuf {
    if let Some(p) = flag {
        return p.to_path_buf();
    }
    if let Some(s) = &config.target {
        return expand_tilde(s);
    }
    agent_dir(agent).unwrap_or_else(|| PathBuf::from("."))
}
