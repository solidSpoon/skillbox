# skillbox

A tiny Rust CLI to manage your agent skills across machines and agents (Codex, Pi).

## Concept

- **Source** — a skill repository: one sub-folder per skill, each containing a `SKILL.md` (this repo's `skills/` is one).
- **Agents** — install destinations. Registry: `codex` → `~/.codex/skills` (default), `pi` → `~/.agents/skills`.

## Install

```bash
# macOS / Linux — no Rust needed
curl -fsSL https://raw.githubusercontent.com/solidSpoon/skillbox/master/install.sh | bash
```

Setup on a new machine:

```bash
skillbox init -a all                       # install the skillbox skill into every agent
git clone git@github.com:solidSpoon/skillbox.git ~/skills-repo
skillbox config --source ~/skills-repo/skills
skillbox list
```

From source (requires Rust):

```bash
cargo install --git https://github.com/solidSpoon/skillbox.git
```

## Commands

| Command | Description |
|---------|-------------|
| `skillbox init [-a all] [--source PATH]` | Self-install the bundled skillbox skill |
| `skillbox list` | List repo skills (`*` = installed; `-a all` shows per-agent columns) |
| `skillbox install <NAME>... [--force]` | Install skill(s) to target agent(s) |
| `skillbox uninstall <NAME>...` | Remove skill(s) (idempotent, skips missing) |
| `skillbox agents` | Show known agents, dirs, default |
| `skillbox config [--source] [--target] [--default-agent]` | Show/update config |
| `skillbox path [NAME]` | Print resolved paths |

Global flags: `-s/--source`, `-a/--agent` (repeatable, comma-separated, or `all`), `-t/--target`.

Config: `~/.config/skillbox/config.toml`.

## Adding a skill

```bash
mkdir skills/<name> && $EDITOR skills/<name>/SKILL.md
skillbox install <name> -a all
git commit -am "add <name> skill" && git push
```

## Releasing

Push a tag and GitHub Actions builds binaries for macOS (arm64/x86_64) and Linux (x86_64/aarch64, static musl):

```bash
git tag v0.1.1 && git push origin v0.1.1
```

## License

Personal project, no license.
