---
name: skillbox
description: Manage the user's agent skills with the `skillbox` CLI — list skills in the repository, install/uninstall skills to/from local agents (Codex, Pi), inspect configuration. Use when the user asks to install a skill, remove a skill, see available skills, sync skills, or mentions skillbox.
---

# skillbox

`skillbox` manages agent skills. Two concepts:

- **Source** — the skill repository: a folder of skills, one sub-folder per skill, each with a `SKILL.md`. Resolution order: `-s` flag > config > `~/.skillbox/repo/skills` (the canonical clone made by install.sh) > `./skills`.
- **Agents** — install destinations. Registry: `codex` → `~/.codex/skills` (default), `pi` → `~/.agents/skills`.

## Help first (IMPORTANT)

**When unsure about a command, flag, or agent name, run help instead of guessing.** One help query beats guess-fail-retry loops.

```bash
skillbox --help            # all commands + global flags
skillbox install --help
skillbox agents            # registry, per-agent dirs, current default
```

## Quick Start

Fresh machine (installs binary, clones the repo, registers source, self-installs):

```bash
curl -fsSL https://raw.githubusercontent.com/solidSpoon/skillbox/master/install.sh | bash
```

Everyday use:

```bash
skillbox list              # what's available; "*" = already installed
skillbox install foo       # install skill "foo" to the default agent
skillbox uninstall foo     # remove it
```

## Commands

```bash
skillbox init [--source <PATH>]       # self-install the skillbox skill into agent(s)
skillbox list                         # list repo skills; "*" = installed
skillbox install <NAME>... [--force]  # install to target agent(s)
skillbox uninstall <NAME>...          # remove from target agent(s)
skillbox agents                       # show agents, dirs, default
skillbox config [--source P] [--target P] [--default-agent A]
skillbox path [NAME]                  # print source/target/skill paths
```

## Agents

`-a/--agent` is a **global flag** (works on every subcommand): repeatable, comma-separated, or `all`.

```bash
skillbox init -a all        # install skillbox skill into every known agent
skillbox install foo -a pi  # only Pi
skillbox list -a all        # one column per agent, */- marks
skillbox uninstall foo -a all
```

Without `-a`, the configured agent set is used (default: `codex`). Persist it:

```bash
skillbox config --agents pi,codex   # now bare install/init target BOTH
skillbox agents                     # show the configured set (* marks)
```

## Typical workflows

- "看看有哪些 skill 可以装" → `skillbox list`, present the result.
- "把 X 装到电脑上" → `skillbox install X`; if the user mainly uses another agent, add `-a <agent>` or confirm first.
- "这个 skill 不用了" → `skillbox uninstall X` (add `-a all` if it may be installed in several agents).
- "我的 skills 仓库在 /path/to/repo" → `skillbox config --source /path/to/repo` (persists) or `-s` for one-off.
- After any install/uninstall, tell the user to restart the agent session so the change takes effect.

## Common Pitfalls

| Pitfall | Correct Approach |
|---------|-----------------|
| `install` fails with "already installed" | Add `--force` to overwrite |
| `uninstall` fails with "not installed" | The skill lives in another agent; retry with `-a all` |
| Bare commands hit the wrong agents | Check the configured set: `skillbox agents`; change it: `skillbox config --agents <LIST>` |
| Guessing an agent name | Run `skillbox agents` — known: `codex`, `pi` |
| Guessing skill names | Run `skillbox list` — names are the sub-folder names in the repo |
| `source folder not found` | Set it: `skillbox config --source <PATH>` (or clone `https://github.com/solidSpoon/skillbox.git ~/.skillbox/repo`) |
| Expecting changes to apply live | Agent reads the skill list at session start — restart required |
| Setting `--target` casually | It overrides the agent registry for ALL agents; prefer `-a` and the registry |

## Notes

- `init` embeds the skillbox skill in the binary, so it works from any directory.
- `-t/--target` forces one specific destination folder (rarely needed; useful for testing).
- Config lives in `~/.config/skillbox/config.toml` — inspect with `skillbox config`.
