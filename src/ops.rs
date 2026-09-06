use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A skill found in a source folder: a directory containing a SKILL.md.
#[derive(Debug)]
pub struct Skill {
    pub name: String,
    pub path: PathBuf,
}

/// Scan a folder (one level deep) for skill directories.
pub fn discover(source: &Path) -> Result<Vec<Skill>> {
    if !source.exists() {
        bail!(
            "source folder not found: {}\nfix it with:\n  skillbox config --source <PATH-TO-SKILLS>\n  # or clone the public repo:\n  git clone --depth 1 https://github.com/solidSpoon/skillbox.git ~/.skillbox/repo",
            source.display()
        );
    }
    let mut skills = Vec::new();
    for entry in fs::read_dir(source).with_context(|| format!("cannot read {}", source.display()))? {
        let entry = entry.with_context(|| format!("cannot read {}", source.display()))?;
        let path = entry.path();
        if path.is_dir() && path.join("SKILL.md").is_file() {
            skills.push(Skill {
                name: path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .into_owned(),
                path,
            });
        }
    }
    skills.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(skills)
}

/// Recursively copy a directory. Returns the number of files copied.
pub fn copy_dir(src: &Path, dst: &Path) -> Result<usize> {
    fs::create_dir_all(dst)
        .with_context(|| format!("failed to create {}", dst.display()))?;
    let mut copied = 0;
    for entry in fs::read_dir(src).with_context(|| format!("cannot read {}", src.display()))? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copied += copy_dir(&from, &to)?;
        } else {
            fs::copy(&from, &to)
                .with_context(|| format!("failed to copy {} -> {}", from.display(), to.display()))?;
            copied += 1;
        }
    }
    Ok(copied)
}

/// Install a skill (by directory name) from source to target.
/// Always overwrites an existing installation.
pub fn install(source: &Path, target: &Path, name: &str) -> Result<usize> {
    let src = source.join(name);
    if !src.join("SKILL.md").is_file() {
        bail!("skill '{}' not found in {}", name, source.display());
    }
    let dst = target.join(name);
    if dst.exists() {
        fs::remove_dir_all(&dst)
            .with_context(|| format!("failed to remove old {}", dst.display()))?;
    }
    fs::create_dir_all(target).with_context(|| format!("failed to create {}", target.display()))?;
    copy_dir(&src, &dst)
}

/// If the source lives inside the canonical git clone (~/.skillbox/repo),
/// refresh it from the remote (fetch + hard reset). Local changes are
/// intentionally discarded — the clone is treated as a read-only cache.
/// Returns Ok(Some(summary)) when HEAD moved, Ok(None) otherwise.
pub fn update_source(source: &Path) -> Result<Option<String>> {
    let repo = match crate::config::canonical_repo_skills().parent() {
        Some(p) => p.to_path_buf(),
        None => return Ok(None),
    };
    if !source.starts_with(&repo) || !repo.join(".git").exists() {
        return Ok(None);
    }
    let head_before = git(&repo, &["rev-parse", "HEAD"]).ok();
    git(&repo, &["fetch", "origin", "--prune"])?;
    git(&repo, &["reset", "--hard", "origin/HEAD"])?;
    let head_after = git(&repo, &["rev-parse", "HEAD"]).ok();
    match (head_before, head_after) {
        (Some(a), Some(b)) if a != b => {
            Ok(Some(format!("repo updated: {} -> {}", &a[..7], &b[..7])))
        }
        _ => Ok(None),
    }
}

fn git(repo: &Path, args: &[&str]) -> Result<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .context("failed to run git")?;
    if !out.status.success() {
        bail!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&out.stderr).trim()
        );
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Remove an installed skill from the target folder.
/// Returns Ok(false) when the skill was not installed (idempotent).
pub fn uninstall(target: &Path, name: &str) -> Result<bool> {
    let dst = target.join(name);
    if !dst.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(&dst).with_context(|| format!("failed to remove {}", dst.display()))?;
    Ok(true)
}
