use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// A skill found in a source folder: a directory containing a SKILL.md.
#[derive(Debug)]
pub struct Skill {
    pub name: String,
    pub path: PathBuf,
}

/// Scan a folder (one level deep) for skill directories.
pub fn discover(source: &Path) -> Result<Vec<Skill>> {
    if !source.exists() {
        bail!("source folder not found: {}", source.display());
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
pub fn install(source: &Path, target: &Path, name: &str, force: bool) -> Result<usize> {
    let src = source.join(name);
    if !src.join("SKILL.md").is_file() {
        bail!("skill '{}' not found in {}", name, source.display());
    }
    let dst = target.join(name);
    if dst.exists() {
        if !force {
            bail!(
                "skill '{}' already installed at {}; use --force to overwrite",
                name,
                dst.display()
            );
        }
        fs::remove_dir_all(&dst)
            .with_context(|| format!("failed to remove old {}", dst.display()))?;
    }
    fs::create_dir_all(target).with_context(|| format!("failed to create {}", target.display()))?;
    copy_dir(&src, &dst)
}

/// Remove an installed skill from the target folder.
pub fn uninstall(target: &Path, name: &str) -> Result<()> {
    let dst = target.join(name);
    if !dst.exists() {
        bail!("skill '{}' is not installed in {}", name, target.display());
    }
    fs::remove_dir_all(&dst).with_context(|| format!("failed to remove {}", dst.display()))
}
