# Installation

## Requirements

- macOS 13 Ventura or later
- Apple Silicon or Intel Mac
- Swift 5.9+ (bundled with Xcode 15+) — only required if building from source

## Option 1: Download the DMG (recommended)

1. Download the latest release from GitHub:

   [**Download Claux v1.15.1**](https://github.com/vatskyone/claux/releases/download/v1.15.1/Claux-1.15.1-release.dmg)

2. Open the DMG file.

3. Drag **Claux.app** to your **Applications** folder.

4. Eject the DMG.

5. Open **Claux** from Applications or Spotlight.

### macOS Gatekeeper prompt

Because Claux is not yet notarized through Apple's developer program, macOS may show a warning the first time you open it. This is expected. To bypass it:

1. Right-click (or Control-click) `Claux.app` → **Open**
2. Click **Open** in the dialog that appears

macOS remembers your choice after the first approval. You will not see the prompt again.

Alternatively, remove the quarantine attribute before opening:

```bash
xattr -cr ~/Downloads/Claux-1.15.1-release.dmg
```

## Option 2: Build from source

Building from source produces an unsigned debug or release bundle.

```bash
git clone https://github.com/vatskyone/claux.git
cd claux/apps/desktop/Claux

# Build the .app bundle and launch immediately
bash build_app.sh run

# Build release .app only (no launch)
bash build_app.sh release

# Fast compile check — no bundle, notifications disabled
swift build
```

The `build_app.sh` script:

1. Compiles the Swift package
2. Assembles a proper `Claux.app` bundle with `Info.plist`
3. Generates and embeds a `.icns` icon from the bundled source PNG
4. Ad-hoc signs the bundle with `codesign --sign -`
5. Optionally launches the app

The output is `apps/desktop/Claux/Claux.app`.

## Option 3: Build a DMG installer

```bash
cd claux/apps/desktop/Claux
bash build_dmg.sh
```

Produces a drag-and-drop DMG at `apps/desktop/Claux/dist/Claux-<version>-release.dmg`.

## Verifying the installation

After launching, the Claux icon (`c` monogram) appears in your menu bar. If Claude Code is not running, the icon is static. Once you start a Claude Code session, the icon pulses green within 10 seconds.

## Uninstalling

1. Quit Claux (right-click the menu bar icon → Quit Claux).
2. Move `Claux.app` from Applications to Trash.
3. Optionally remove Claux's local data:

```bash
rm -rf ~/.claude/claux
```

This removes plan-limit data and the statusLine integration config stored by Claux. Claude Code's own session files in `~/.claude/projects/` are not affected.
