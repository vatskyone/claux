#!/usr/bin/env bash
# build_dmg.sh — builds Claux.app and packages it as a drag-and-drop DMG.
# Usage:
#   ./build_dmg.sh          # release DMG
#   ./build_dmg.sh debug    # debug DMG
#   ./build_dmg.sh release  # release DMG

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
APP_NAME="Claux"
APP_DIR="$SCRIPT_DIR/${APP_NAME}.app"
VERSION="$(sed -nE 's/^[[:space:]]*static let current = "([^"]+)"/\1/p' "$SCRIPT_DIR/Sources/Claux/Design.swift" | head -n1)"
VERSION="${VERSION:-0.0.0}"
CONFIG="${1:-release}"
DIST_DIR="$SCRIPT_DIR/dist"
STAGING_DIR="$DIST_DIR/dmg-root"
DMG_NAME="${APP_NAME}-${VERSION}-${CONFIG}.dmg"
DMG_PATH="$DIST_DIR/$DMG_NAME"
VOL_NAME="${APP_NAME} ${VERSION}"

if [[ "$CONFIG" != "release" && "$CONFIG" != "debug" ]]; then
    echo "Usage: $0 [release|debug]" >&2
    exit 1
fi

echo "▶ Building app bundle for DMG (${CONFIG})…"
if [[ "$CONFIG" == "release" ]]; then
    bash "$SCRIPT_DIR/build_app.sh" release
else
    bash "$SCRIPT_DIR/build_app.sh"
fi

if [[ ! -d "$APP_DIR" ]]; then
    echo "❌ Missing app bundle at $APP_DIR" >&2
    exit 1
fi

echo "▶ Preparing DMG staging directory…"
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR"
rm -f "$DMG_PATH"

cp -R "$APP_DIR" "$STAGING_DIR/"
ln -s /Applications "$STAGING_DIR/Applications"

echo "▶ Creating DMG…"
hdiutil create \
    -volname "$VOL_NAME" \
    -srcfolder "$STAGING_DIR" \
    -ov \
    -format UDZO \
    "$DMG_PATH"

rm -rf "$STAGING_DIR"

echo "✅  DMG: $DMG_PATH"
