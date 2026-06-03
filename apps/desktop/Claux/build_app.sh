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
ICON_SOURCE="$SCRIPT_DIR/Sources/Claux/Resources/app-icon.png"
ICON_NAME="${APP_NAME}.icns"
ICON_BASENAME="$APP_NAME"
VERSION="$(sed -nE 's/^[[:space:]]*static let current = "([^"]+)"/\1/p' "$SCRIPT_DIR/Sources/Claux/Design.swift" | head -n1)"
VERSION="${VERSION:-0.0.0}"
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
APP_STAGE_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/claux-app.XXXXXX")"
APP_STAGE_DIR="$APP_STAGE_ROOT/${APP_NAME}.app"
pkill -x "$BINARY_NAME" 2>/dev/null || true
sleep 0.2

rm -rf "$APP_DIR"
mkdir -p "$APP_STAGE_DIR/Contents/MacOS"
mkdir -p "$APP_STAGE_DIR/Contents/Resources"

cp "$BUILT_BINARY" "$APP_STAGE_DIR/Contents/MacOS/$BINARY_NAME"

if [[ ! -f "$ICON_SOURCE" ]]; then
    echo "❌ Missing app icon source: $ICON_SOURCE" >&2
    exit 1
fi

ICONSET_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/claux-icon.XXXXXX")"
ICONSET_DIR="$ICONSET_ROOT/${APP_NAME}.iconset"
mkdir -p "$ICONSET_DIR"

generate_icon() {
    local size="$1"
    local output="$2"
    sips -z "$size" "$size" "$ICON_SOURCE" --out "$ICONSET_DIR/$output" >/dev/null
}

generate_icon 16 "icon_16x16.png"
generate_icon 32 "icon_16x16@2x.png"
generate_icon 32 "icon_32x32.png"
generate_icon 64 "icon_32x32@2x.png"
generate_icon 128 "icon_128x128.png"
generate_icon 256 "icon_128x128@2x.png"
generate_icon 256 "icon_256x256.png"
generate_icon 512 "icon_256x256@2x.png"
generate_icon 512 "icon_512x512.png"
generate_icon 1024 "icon_512x512@2x.png"

iconutil -c icns "$ICONSET_DIR" -o "$APP_STAGE_DIR/Contents/Resources/$ICON_NAME"
rm -rf "$ICONSET_ROOT"

# ── 3. Write Info.plist ───────────────────────────────────────────────────────
cat > "$APP_STAGE_DIR/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN"
    "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleIdentifier</key>       <string>$BUNDLE_ID</string>
    <key>CFBundleName</key>             <string>$APP_NAME</string>
    <key>CFBundleDisplayName</key>      <string>$APP_NAME</string>
    <key>CFBundleExecutable</key>       <string>$BINARY_NAME</string>
    <key>CFBundleIconFile</key>         <string>$ICON_BASENAME</string>
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

# ── 4. Ad-hoc sign bundle ────────────────────────────────────────────────────
if command -v codesign >/dev/null 2>&1; then
    if command -v xattr >/dev/null 2>&1; then
        xattr -cr "$APP_STAGE_DIR" 2>/dev/null || true
    fi
    if ! codesign --force --deep --sign - "$APP_STAGE_DIR"; then
        echo "⚠️  Warning: ad-hoc signing failed; continuing with unsigned bundle." >&2
    fi
fi

if command -v ditto >/dev/null 2>&1; then
    ditto "$APP_STAGE_DIR" "$APP_DIR"
else
    cp -R "$APP_STAGE_DIR" "$APP_DIR"
fi
rm -rf "$APP_STAGE_ROOT"
touch "$APP_DIR"

echo "✅  Bundle: $APP_DIR"

# ── 5. Launch if requested ────────────────────────────────────────────────────
if [[ "$LAUNCH" == "run" || "$CONFIG" == "run" ]]; then
    echo "▶ Launching ${APP_NAME}..."
    open "$APP_DIR"
fi
