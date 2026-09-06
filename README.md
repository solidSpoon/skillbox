# skillbox

A tiny Rust CLI to manage your agent skills across machines and agents (Codex, Pi).

## Concept

- **Source** — a skill repository: one sub-folder per skill, each containing a `SKILL.md` (this repo's `skills/` is one).
- **Agents** — install destinations. Registry: `codex` → `~/.codex/skills` (default), `pi` → `~/.agents/skills`.

## Install

```bash
# macOS / Linux — no Rust needed. Installs the binary, clones this repo to
# ~/.skillbox/repo, and registers the source.
curl -fsSL https://raw.githubusercontent.com/solidSpoon/skillbox/master/install.sh | bash

# Optional: also inject the skillbox skill into your agents in one go
SKILLBOX_AGENTS="pi,codex" curl -fsSL https://raw.githubusercontent.com/solidSpoon/skillbox/master/install.sh | bash
```

If no agents are configured yet (and SKILLBOX_AGENTS is not given), the installer
prints the two next steps instead of writing anything:

```bash
skillbox config --agents <LIST>   # e.g. pi,codex
skillbox init
```

Then verify and pick skills:

```bash
skillbox list
skillbox install <NAME> -a all
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
| `skillbox agents` | Show known agents, configured set, dirs |
| `skillbox config [--source] [--target] [--agents LIST]` | Show/update config |
| `skillbox path [NAME]` | Print resolved paths |

Global flags: `-s/--source`, `-a/--agent` (repeatable, comma-separated, or `all`), `-t/--target`.

Without `-a`, the configured agent set is used: config `agents` > `codex`. Set it with `skillbox config --agents pi,codex`.

Source resolution order: `-s` flag > config > `~/.skillbox/repo/skills` > `./skills`.

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
