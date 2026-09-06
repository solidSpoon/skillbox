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

    /// Target agent(s) for this call: name, repeatable, or "all"
    #[arg(short = 'a', long, global = true, value_delimiter = ',')]
    agent: Vec<String>,

    /// Override the install destination folder for this call
    #[arg(short, long, global = true)]
    target: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Command {
    /// Write the bundled skillbox skill into each target agent's skill folder
    Init {
        /// Also register this folder as the skill repository (persisted)
        #[arg(long)]
        source: Option<PathBuf>,
    },
    /// List skills available in the repository ("*" = installed)
    List,
    /// Install skills from the repository to the target agent(s)
    Install {
        /// Skill names (sub-folder names in the repository)
        names: Vec<String>,
        /// Overwrite already-installed skills
        #[arg(short, long)]
        force: bool,
    },
    /// Remove installed skills from the target agent(s)
    Uninstall { names: Vec<String> },
    /// List known agents, their skill folders, and the default
    Agents,
    /// Show or update configuration
    Config {
        /// Set the skill repository folder (persisted)
        #[arg(long)]
        source: Option<PathBuf>,
        /// Set the install destination folder (persisted)
        #[arg(long)]
        target: Option<PathBuf>,
        /// Set the agents used when --agent is omitted, comma-separated (persisted)
        #[arg(long, value_delimiter = ',')]
        agents: Vec<String>,
    },
    /// Print resolved paths (source, per-agent targets, or a specific skill)
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
        Command::Init { source } => {
            cmd_init(&mut cfg, source, &cli.agent, cli.source.as_deref(), cli.target.as_deref())
        }
        Command::List => cmd_list(&cfg, &cli.agent, cli.source.as_deref(), cli.target.as_deref()),
        Command::Install { names, force } => {
            cmd_install(&cfg, &names, force, &cli.agent, cli.source.as_deref(), cli.target.as_deref())
        }
        Command::Uninstall { names } => cmd_uninstall(&cfg, &names, &cli.agent, cli.target.as_deref()),
        Command::Agents => cmd_agents(&cfg),
        Command::Config { source, target, agents } => {
            cmd_config(&mut cfg, source, target, &agents)
        }
        Command::Path { name } => cmd_path(&cfg, name.as_deref(), &cli.agent, cli.source.as_deref(), cli.target.as_deref()),
    }
}

fn cmd_init(
    cfg: &mut config::Config,
    config_source: Option<PathBuf>,
    agents_flag: &[String],
    source_flag: Option<&Path>,
    target_flag: Option<&Path>,
) -> Result<()> {
    if let Some(source) = config_source.as_deref().or(source_flag) {
        cfg.source = Some(display_path(source));
    }
    config::save(cfg)?;

    let agents = config::resolve_agents(agents_flag, cfg)?;
    for agent in &agents {
        let target = config::target_for(agent, cfg, target_flag);
        let dst = target.join(EMBEDDED_SKILL_NAME);
        std::fs::create_dir_all(&dst)
            .with_context(|| format!("failed to create {}", dst.display()))?;
        let skill_md = dst.join("SKILL.md");
        std::fs::write(&skill_md, EMBEDDED_SKILL)
            .with_context(|| format!("failed to write {}", skill_md.display()))?;
        println!("installed skillbox skill -> {} ({agent})", skill_md.display());
    }
    println!(
        "source repo: {}",
        display_path(&config::resolve_source(source_flag, cfg))
    );
    println!("done. restart the agent session to pick it up.");
    Ok(())
}

fn cmd_list(
    cfg: &config::Config,
    agents_flag: &[String],
    source_flag: Option<&Path>,
    target_flag: Option<&Path>,
) -> Result<()> {
    let source = config::resolve_source(source_flag, cfg);
    let agents = config::resolve_agents(agents_flag, cfg)?;
    let skills = ops::discover(&source)?;
    if skills.is_empty() {
        println!("no skills found in {}", source.display());
        println!("hint: each skill is a sub-folder containing a SKILL.md");
        return Ok(());
    }

    println!("skills in {}:", display_path(&source));
    if agents.len() == 1 {
        let target = config::target_for(&agents[0], cfg, target_flag);
        for skill in &skills {
            let installed = target.join(&skill.name).is_dir();
            let mark = if installed { "*" } else { " " };
            println!(" {mark} {:<20} {}", skill.name, display_path(&skill.path));
        }
        println!("\n* = installed in {} ({})", display_path(&target), agents[0]);
    } else {
        let header: Vec<&str> = agents.iter().map(|s| s.as_str()).collect();
        println!(" {:<20} {}", "skill", header.join("  "));
        for skill in &skills {
            let marks: Vec<String> = agents
                .iter()
                .map(|a| {
                    if config::target_for(a, cfg, target_flag)
                        .join(&skill.name)
                        .is_dir()
                    {
                        "*".to_string()
                    } else {
                        "-".to_string()
                    }
                })
                .collect();
            println!(" {:<20} {}", skill.name, marks.join("  "));
        }
        println!("\n* = installed, - = not installed");
    }
    Ok(())
}

fn cmd_install(
    cfg: &config::Config,
    names: &[String],
    force: bool,
    agents_flag: &[String],
    source_flag: Option<&Path>,
    target_flag: Option<&Path>,
) -> Result<()> {
    if names.is_empty() {
        bail!("no skill names given. usage: skillbox install <NAME>...");
    }
    let source = config::resolve_source(source_flag, cfg);
    let agents = config::resolve_agents(agents_flag, cfg)?;
    let mut failed = false;
    for agent in &agents {
        let target = config::target_for(agent, cfg, target_flag);
        for name in names {
            match ops::install(&source, &target, name, force) {
                Ok(files) => println!(
                    "installed {name} -> {} ({agent}, {files} files)",
                    target.join(name).display()
                ),
                Err(err) => {
                    failed = true;
                    eprintln!("error: {err:#}");
                }
            }
        }
    }
    if failed {
        bail!("some installs failed");
    }
    println!("done. restart the agent session to pick up new skills.");
    Ok(())
}

fn cmd_uninstall(
    cfg: &config::Config,
    names: &[String],
    agents_flag: &[String],
    target_flag: Option<&Path>,
) -> Result<()> {
    if names.is_empty() {
        bail!("no skill names given. usage: skillbox uninstall <NAME>...");
    }
    let agents = config::resolve_agents(agents_flag, cfg)?;
    let mut failed = false;
    for agent in &agents {
        let target = config::target_for(agent, cfg, target_flag);
        for name in names {
            match ops::uninstall(&target, name) {
                Ok(true) => println!("removed {name} from {} ({agent})", target.display()),
                Ok(false) => println!("skipped {name} ({agent}): not installed"),
                Err(err) => {
                    failed = true;
                    eprintln!("error: {err:#}");
                }
            }
        }
    }
    if failed {
        bail!("some uninstalls failed");
    }
    Ok(())
}

fn cmd_agents(cfg: &config::Config) -> Result<()> {
    let configured = config::resolve_agents(&[], cfg)?;
    println!("agents (configured set: {}):", configured.join(", "));
    for (name, _) in config::AGENTS {
        let dir = config::agent_dir(name).expect("known agent");
        let detected = if dir.is_dir() { "detected" } else { "not detected" };
        let mark = if configured.iter().any(|a| a == name) { " *" } else { "" };
        println!("  {name:<8} {}  [{detected}]{mark}", display_path(&dir));
    }
    println!("\n* = operated on when --agent is omitted (change with: skillbox config --agents <LIST>)");
    Ok(())
}

fn cmd_config(
    cfg: &mut config::Config,
    source: Option<PathBuf>,
    target: Option<PathBuf>,
    agents: &[String],
) -> Result<()> {
    for agent in agents {
        if !config::is_known_agent(agent) {
            bail!("unknown agent '{}'. known agents: {}", agent, config::known_agents());
        }
    }
    if source.is_none() && target.is_none() && agents.is_empty() {
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
                .unwrap_or_else(|| "(unset, uses agent registry)".to_string())
        );
        println!(
            "agents: {}",
            if cfg.agents.is_empty() {
                format!("(unset, default: {})", config::DEFAULT_AGENT)
            } else {
                cfg.agents.join(", ")
            }
        );
        return Ok(());
    }
    if let Some(source) = source {
        cfg.source = Some(display_path(&source));
    }
    if let Some(target) = target {
        cfg.target = Some(display_path(&target));
    }
    if !agents.is_empty() {
        cfg.agents = agents.to_vec();
    }
    config::save(cfg)?;
    println!("saved to {}", config::config_path()?.display());
    Ok(())
}

fn cmd_path(
    cfg: &config::Config,
    name: Option<&str>,
    agents_flag: &[String],
    source_flag: Option<&Path>,
    target_flag: Option<&Path>,
) -> Result<()> {
    let source = config::resolve_source(source_flag, cfg);
    let agents = config::resolve_agents(agents_flag, cfg)?;
    match name {
        None => {
            println!("source: {}", source.display());
            for agent in &agents {
                let target = config::target_for(agent, cfg, target_flag);
                println!("target ({agent}): {}", target.display());
            }
        }
        Some(name) => {
            for agent in &agents {
                let target = config::target_for(agent, cfg, target_flag);
                let in_source = source.join(name);
                let in_target = target.join(name);
                if in_source.join("SKILL.md").is_file() {
                    println!("({agent}) {}", in_source.display());
                } else if in_target.is_dir() {
                    println!("({agent}) {}", in_target.display());
                } else {
                    bail!(
                        "skill '{}' not found in {} or {}",
                        name,
                        source.display(),
                        target.display()
                    );
                }
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
