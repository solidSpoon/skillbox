#!/bin/bash
# skillbox installer — fetches the latest release binary from GitHub and sets up everything.
# Usage: curl -fsSL https://raw.githubusercontent.com/solidSpoon/skillbox/master/install.sh | bash
set -euo pipefail

REPO="solidSpoon/skillbox"

case "$(uname -s)" in
  Darwin) os="apple-darwin" ;;
  Linux) os="unknown-linux-musl" ;;
  *) echo "unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac

case "$(uname -m)" in
  arm64 | aarch64) arch="aarch64" ;;
  x86_64) arch="x86_64" ;;
  *) echo "unsupported arch: $(uname -m)" >&2; exit 1 ;;
esac

target="${arch}-${os}"
url="https://github.com/${REPO}/releases/latest/download/skillbox-${target}.tar.gz"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "==> downloading skillbox for ${target}..."
curl -fsSL "$url" -o "$tmp/skillbox.tar.gz"
tar xzf "$tmp/skillbox.tar.gz" -C "$tmp"

dest="${SKILLBOX_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$dest"
mv "$tmp/skillbox-${target}/skillbox" "$dest/skillbox"
chmod +x "$dest/skillbox"
echo "==> installed to $dest/skillbox"

case ":$PATH:" in
  *":$dest:"*) ;;
  *) echo "==> warning: $dest is not in your PATH — add it to your shell profile" ;;
esac

# Clone the public skill repository so install/list work out of the box.
repo_dir="${SKILLBOX_REPO_DIR:-$HOME/.skillbox/repo}"
if [ -d "$repo_dir" ]; then
  echo "==> skill repo already at $repo_dir"
elif command -v git >/dev/null 2>&1; then
  echo "==> cloning skill repository to $repo_dir..."
  git clone --depth 1 "https://github.com/${REPO}.git" "$repo_dir" || echo "==> warning: git clone failed — set the source manually later"
else
  echo "==> git not found — skip cloning (set source manually: skillbox config --source <PATH>)"
fi

if [ -d "$repo_dir/skills" ]; then
  "$dest/skillbox" config --source "$repo_dir/skills"
fi

# Inject the skillbox skill — only into agents the user actually asked for.
# Optional: SKILLBOX_AGENTS="pi,codex" curl -fsSL ... | bash
agents_line="$("$dest/skillbox" config 2>/dev/null | grep '^agents:' || true)"
if [ -n "${SKILLBOX_AGENTS:-}" ]; then
  "$dest/skillbox" config --agents "$SKILLBOX_AGENTS"
  "$dest/skillbox" init
elif echo "$agents_line" | grep -qv 'unset'; then
  "$dest/skillbox" init
else
  echo "==> skillbox skill not injected yet — pick your agents first:"
  echo "     skillbox config --agents <LIST>   # e.g. pi,codex"
  echo "     skillbox init"
fi

echo
echo "==> all set. try:"
echo "     skillbox list"
echo "     skillbox install <NAME> -a all"
echo "   restart your agent sessions to pick up new skills."
