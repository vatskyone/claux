#!/usr/bin/env bash
# build_app.sh — builds Claux and packages it as a proper macOS .app bundle.
# Usage:
#   ./build_app.sh          # debug build (fast)
#   ./build_app.sh release  # release build (optimised)
#   ./build_app.sh run      # debug build + launch immediately

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BINARY_NAME="Claux"
APP_NAME="Claux"
BUNDLE_ID="com.claux.app"
VERSION="0.9.4"
CONFIG="${1:-debug}"
LAUNCH="${1:-}"

# ── 1. Build ──────────────────────────────────────────────────────────────────
if [[ "$CONFIG" == "release" ]]; then
    echo "▶ Building (release)…"
    swift build -c release --package-path "$SCRIPT_DIR" 2>&1
    BUILT_BINARY="$SCRIPT_DIR/.build/release/$BINARY_NAME"
else
    echo "▶ Building (debug)…"
    swift build --package-path "$SCRIPT_DIR" 2>&1
    BUILT_BINARY="$SCRIPT_DIR/.build/debug/$BINARY_NAME"
fi

# ── 2. Assemble .app bundle ───────────────────────────────────────────────────
APP_DIR="$SCRIPT_DIR/${APP_NAME}.app"
pkill -x "$BINARY_NAME" 2>/dev/null || true
sleep 0.2

rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"

cp "$BUILT_BINARY" "$APP_DIR/Contents/MacOS/$BINARY_NAME"

# ── 3. Write Info.plist ───────────────────────────────────────────────────────
cat > "$APP_DIR/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>       <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>             <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>      <string>$APP_NAME</string>
    <key>CFBundleExecutable</key>       <string>$BINARY_NAME</string>
    <key>CFBundleVersion</key>          <string>1</string>
    <key>CFBundleShortVersionString</key><string>$VERSION</string>
    <key>CFBundlePackageType</key>      <string>APPL</string>
    <key>LSMinimumSystemVersion</key>   <string>13.0</string>
    <!-- Hide from Dock — menu bar only -->
    <key>LSUIElement</key>              <true/>
    <key>NSHighResolutionCapable</key>  <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict>
</plist>
PLIST

echo "✅  Bundle: $APP_DIR"

# ── 4. Launch if requested ────────────────────────────────────────────────────
if [[ "$LAUNCH" == "run" || "$CONFIG" == "run" ]]; then
    echo "▶ Launching $APP_NAME…"
    open "$APP_DIR"
fi
