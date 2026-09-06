#!/bin/bash
# skillbox installer — fetches the latest release binary from GitHub.
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

echo "downloading skillbox for ${target}..."
curl -fsSL "$url" -o "$tmp/skillbox.tar.gz"
tar xzf "$tmp/skillbox.tar.gz" -C "$tmp"

dest="${SKILLBOX_INSTALL_DIR:-$HOME/.local/bin}"
mkdir -p "$dest"
mv "$tmp/skillbox-${target}/skillbox" "$dest/skillbox"
chmod +x "$dest/skillbox"

echo "installed to $dest/skillbox"
case ":$PATH:" in
  *":$dest:"*) ;;
  *) echo "note: $dest is not in your PATH — add it to your shell profile" ;;
esac
echo
echo "next steps:"
echo "  1. skillbox init -a all          # self-install the skillbox skill"
echo "  2. git clone git@github.com:${REPO}.git ~/skills-repo   # if you want your skill repository here"
echo "  3. skillbox config --source ~/skills-repo/skills        # register it"
echo "  4. skillbox list && skillbox install <NAME>"
