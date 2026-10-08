#!/usr/bin/env bash
# Installs or updates the WaspLauncher plain Linux binary (x86_64).
#
#   install:  curl -fsSL https://raw.githubusercontent.com/WaspScripts/wasp-launcher/main/scripts/install-linux.sh | bash
#   update:   wasp-launcher-update   (installed by this script, run it again any time)
#
# Options: --force  reinstall even if already on the latest version
set -euo pipefail

REPO="WaspScripts/wasp-launcher"
RAW="https://raw.githubusercontent.com/$REPO/main"
ASSET="wasp-launcher-linux-x86_64.tar.gz"

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons"
STATE_DIR="$HOME/.local/share/wasp-launcher"
VERSION_FILE="$STATE_DIR/version"

force=0
[ "${1:-}" = "--force" ] && force=1

die() { echo "error: $*" >&2; exit 1; }

[ "$(uname -m)" = "x86_64" ] || die "only x86_64 is supported (got $(uname -m))"
command -v curl >/dev/null || die "curl is required"
command -v tar >/dev/null || die "tar is required"

# GitHub redirects /releases/latest to /releases/tag/<tag>, so no API token or rate limit is involved.
latest=$(curl -fsSI -o /dev/null -w '%{url_effective}' -L "https://github.com/$REPO/releases/latest" | sed 's|.*/||')
[ -n "$latest" ] && [ "$latest" != "latest" ] || die "could not determine the latest release"

installed=$(cat "$VERSION_FILE" 2>/dev/null || true)
if [ "$installed" = "$latest" ] && [ -x "$BIN_DIR/wasp-launcher" ] && [ "$force" -eq 0 ]; then
  echo "Already up to date ($latest). Use --force to reinstall."
  exit 0
fi

if pgrep -x wasp-launcher >/dev/null; then
  echo "note: WaspLauncher is running. The update applies the next time you start it."
fi

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

echo "Downloading $latest..."
curl -fL --progress-bar "https://github.com/$REPO/releases/download/$latest/$ASSET" -o "$tmp/$ASSET"
tar -xzf "$tmp/$ASSET" -C "$tmp"
[ -f "$tmp/wasp-launcher" ] || die "wasp-launcher binary missing from $ASSET"

mkdir -p "$BIN_DIR" "$APP_DIR" "$ICON_DIR" "$STATE_DIR"

# Copy then rename: replacing a running binary in place fails with "text file busy", renaming doesn't.
install -m755 "$tmp/wasp-launcher" "$BIN_DIR/.wasp-launcher.new"
mv -f "$BIN_DIR/.wasp-launcher.new" "$BIN_DIR/wasp-launcher"

# The update command: the same script, so `wasp-launcher-update` works from anywhere.
if curl -fsSL "$RAW/scripts/install-linux.sh" -o "$tmp/update.sh"; then
  install -m755 "$tmp/update.sh" "$BIN_DIR/wasp-launcher-update"
fi

# Menu entry and icon (only created once, so local edits survive updates).
if [ ! -f "$ICON_DIR/wasp-launcher.png" ]; then
  curl -fsSL "$RAW/app-icon.png" -o "$ICON_DIR/wasp-launcher.png" || echo "warning: could not download the icon"
fi
if [ ! -f "$APP_DIR/wasp-launcher.desktop" ]; then
  cat > "$APP_DIR/wasp-launcher.desktop" <<DESKTOP
[Desktop Entry]
Type=Application
Name=WaspLauncher
Exec=$BIN_DIR/wasp-launcher
Icon=$ICON_DIR/wasp-launcher.png
Categories=Utility;
Terminal=false
DESKTOP
fi

echo "$latest" > "$VERSION_FILE"
echo "Installed $latest to $BIN_DIR/wasp-launcher"

case ":$PATH:" in
  *":$BIN_DIR:"*) ;;
  *) echo "note: $BIN_DIR is not in your PATH" ;;
esac
