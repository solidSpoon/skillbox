---
name: skillbox
description: >
  Manage the user's agent skills with the `skillbox` CLI: list skills in the
  skill repository, install/uninstall skills to/from the local agent skill
  folder, and inspect or change skillbox configuration. Use when the user
  wants to install a skill, remove a skill, see available skills, or ask
  about skillbox itself.
---

# skillbox

`skillbox` is a small CLI that manages agent skills.

- **Source** (skill repository): a folder of skills, one sub-folder per skill,
  each containing a `SKILL.md`. Default: `./skills`, configurable.
- **Target** (install destination): the local agent skill folder, e.g.
  `~/.agents/skills/`. Each installed skill is a sub-folder there.

## Commands

```bash
# One-time setup: write the skillbox skill into the agent skill folder,
# optionally registering the skill repository location.
skillbox init [--source <PATH>]

# List skills available in the source repository ("*" = already installed).
skillbox list

# Install one or more skills from the repository to the target folder.
skillbox install <NAME>... [--force]

# Remove installed skills from the target folder.
skillbox uninstall <NAME>...

# Show or update configuration (stored in ~/.config/skillbox/config.toml).
skillbox config [--source <PATH>] [--target <PATH>]

# Print resolved paths: target folder, source folder, or one skill's path.
skillbox path [NAME]
```

Global overrides (work on every subcommand):

- `-s, --source <PATH>` — use a different skill repository just for this call
- `-t, --target <PATH>` — install into a different folder just for this call

## Typical agent workflows

- "看看有哪些 skill 可以装" → `skillbox list`, then present the result.
- "把 X 装到电脑上" → `skillbox install X`, then tell the user to restart
  their agent session so the new skill is picked up.
- "这个 skill 不用了" → `skillbox uninstall X`.
- "我的 skills 仓库在 /path/to/repo" → `skillbox config --source /path/to/repo`
  (persists it) or use `-s` for a one-off.

## Notes

- `install` refuses to overwrite an existing skill unless `--force` is given.
- After installing or uninstalling, the agent usually needs a session restart
  (or a reload) to pick up the skill list change.
