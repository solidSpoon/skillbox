mod config;
mod ops;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use std::path::{Path, PathBuf};

/// The skillbox skill, embedded so `init` works from anywhere.
const EMBEDDED_SKILL: &str = include_str!("../skills/skillbox/SKILL.md");
const EMBEDDED_SKILL_NAME: &str = "skillbox";

#[derive(Parser)]
#[command(
    name = "skillbox",
    version,
    about = "Manage your agent skills: list, install, uninstall"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Override the skill repository folder for this call
    #[arg(short, long, global = true)]
    source: Option<PathBuf>,

    /// Override the install destination folder for this call
    #[arg(short, long, global = true)]
    target: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Write the bundled skillbox skill into the agent skill folder
    Init {
        /// Also register this folder as the skill repository (persisted)
        #[arg(long)]
        source: Option<PathBuf>,
    },
    /// List skills available in the repository ("*" = installed)
    List,
    /// Install skills from the repository to the target folder
    Install {
        /// Skill names (sub-folder names in the repository)
        names: Vec<String>,
        /// Overwrite already-installed skills
        #[arg(short, long)]
        force: bool,
    },
    /// Remove installed skills from the target folder
    Uninstall { names: Vec<String> },
    /// Show or update configuration
    Config {
        /// Set the skill repository folder (persisted)
        #[arg(long)]
        source: Option<PathBuf>,
        /// Set the install destination folder (persisted)
        #[arg(long)]
        target: Option<PathBuf>,
    },
    /// Print resolved paths (target, source, or a specific skill)
    Path { name: Option<String> },
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli) {
        eprintln!("error: {err:#}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<()> {
    let mut cfg = config::load()?;

    match cli.command {
        Command::Init { source } => cmd_init(&mut cfg, source, cli.source.as_deref(), cli.target.as_deref()),
        Command::List => cmd_list(&cfg, cli.source.as_deref(), cli.target.as_deref()),
        Command::Install { names, force } => cmd_install(&cfg, &names, force, cli.source.as_deref(), cli.target.as_deref()),
        Command::Uninstall { names } => cmd_uninstall(&cfg, &names, cli.target.as_deref()),
        Command::Config { source, target } => cmd_config(&mut cfg, source, target),
        Command::Path { name } => cmd_path(&cfg, name.as_deref(), cli.source.as_deref(), cli.target.as_deref()),
    }
}

fn cmd_init(
    cfg: &mut config::Config,
    config_source: Option<PathBuf>,
    source_flag: Option<&Path>,
    target_flag: Option<&Path>,
) -> Result<()> {
    if let Some(source) = config_source.as_deref().or(source_flag) {
        cfg.source = Some(display_path(source));
    }
    config::save(cfg)?;

    let target = config::resolve_target(target_flag, cfg);
    let dst = target.join(EMBEDDED_SKILL_NAME);
    std::fs::create_dir_all(&dst).with_context(|| format!("failed to create {}", dst.display()))?;
    let skill_md = dst.join("SKILL.md");
    std::fs::write(&skill_md, EMBEDDED_SKILL)
        .with_context(|| format!("failed to write {}", skill_md.display()))?;

    println!("installed skillbox skill -> {}", skill_md.display());
    println!(
        "source repo:             {}",
        display_path(&config::resolve_source(source_flag, cfg))
    );
    println!("done. restart your agent session to pick it up.");
    Ok(())
}

fn cmd_list(cfg: &config::Config, source_flag: Option<&Path>, target_flag: Option<&Path>) -> Result<()> {
    let source = config::resolve_source(source_flag, cfg);
    let target = config::resolve_target(target_flag, cfg);
    let skills = ops::discover(&source)?;
    if skills.is_empty() {
        println!("no skills found in {}", source.display());
        println!("hint: each skill is a sub-folder containing a SKILL.md");
        return Ok(());
    }
    println!("skills in {}:", display_path(&source));
    for skill in skills {
        let installed = target.join(&skill.name).is_dir();
        let mark = if installed { "*" } else { " " };
        println!(" {mark} {:<20} {}", skill.name, display_path(&skill.path));
    }
    println!("\n* = installed in {}", display_path(&target));
    Ok(())
}

fn cmd_install(
    cfg: &config::Config,
    names: &[String],
    force: bool,
    source_flag: Option<&Path>,
    target_flag: Option<&Path>,
) -> Result<()> {
    if names.is_empty() {
        bail!("no skill names given. usage: skillbox install <NAME>...");
    }
    let source = config::resolve_source(source_flag, cfg);
    let target = config::resolve_target(target_flag, cfg);
    for name in names {
        let files = ops::install(&source, &target, name, force)?;
        println!("installed {} -> {} ({} files)", name, target.join(name).display(), files);
    }
    println!("done. restart your agent session to pick up new skills.");
    Ok(())
}

fn cmd_uninstall(cfg: &config::Config, names: &[String], target_flag: Option<&Path>) -> Result<()> {
    if names.is_empty() {
        bail!("no skill names given. usage: skillbox uninstall <NAME>...");
    }
    let target = config::resolve_target(target_flag, cfg);
    for name in names {
        ops::uninstall(&target, name)?;
        println!("removed {} from {}", name, target.display());
    }
    Ok(())
}

fn cmd_config(cfg: &mut config::Config, source: Option<PathBuf>, target: Option<PathBuf>) -> Result<()> {
    if source.is_none() && target.is_none() {
        println!("config file: {}", config::config_path()?.display());
        println!(
            "source: {}",
            cfg.source
                .as_deref()
                .map(display_path_str)
                .unwrap_or_else(|| format!("(unset, default: {})", config::default_source().display()))
        );
        println!(
            "target: {}",
            cfg.target
                .as_deref()
                .map(display_path_str)
                .unwrap_or_else(|| format!("(unset, default: {})", config::default_target().display()))
        );
        return Ok(());
    }
    if let Some(source) = source {
        cfg.source = Some(display_path(&source));
    }
    if let Some(target) = target {
        cfg.target = Some(display_path(&target));
    }
    config::save(cfg)?;
    println!("saved to {}", config::config_path()?.display());
    cmd_config(cfg, None, None)
}

fn cmd_path(cfg: &config::Config, name: Option<&str>, source_flag: Option<&Path>, target_flag: Option<&Path>) -> Result<()> {
    let target = config::resolve_target(target_flag, cfg);
    match name {
        None => {
            let source = config::resolve_source(source_flag, cfg);
            println!("source: {}", source.display());
            println!("target: {}", target.display());
        }
        Some(name) => {
            let source = config::resolve_source(source_flag, cfg);
            let in_source = source.join(name);
            let in_target = target.join(name);
            if in_source.join("SKILL.md").is_file() {
                println!("{}", in_source.display());
            } else if in_target.is_dir() {
                println!("{}", in_target.display());
            } else {
                bail!("skill '{}' not found in {} or {}", name, source.display(), target.display());
            }
        }
    }
    Ok(())
}

fn display_path(path: &Path) -> String {
    if let Some(home) = dirs::home_dir() {
        if let Ok(rest) = path.strip_prefix(&home) {
            return format!("~/{}", rest.display());
        }
    }
    path.display().to_string()
}

fn display_path_str(path: &str) -> String {
    display_path(&config::expand_tilde(path))
}
