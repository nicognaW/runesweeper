#!/bin/sh
# Installs Runesweeper for the current user: ~/.local/bin, plus a menu entry and icon.
set -e
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$HOME/.local/bin" "$HOME/.local/share/applications" "$HOME/.local/share/icons/hicolor/256x256/apps"
install -m 755 "$here/runesweeper" "$HOME/.local/bin/runesweeper"
install -m 644 "$here/runesweeper.png" "$HOME/.local/share/icons/hicolor/256x256/apps/runesweeper.png"
sed "s|^Exec=.*|Exec=$HOME/.local/bin/runesweeper|" "$here/runesweeper.desktop" > "$HOME/.local/share/applications/runesweeper.desktop"
echo "Installed. Run 'runesweeper' or find Runesweeper in your app menu."
