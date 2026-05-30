# Right-Click Context Menu Verification Update (2026-05-29)

## Scope
Verified whether the menu bar icon right-click context menu is implemented and aligned with the requested behavior:
- Open Settings
- Toggle menu bar visibility (`Always` / `When session is active`)
- Quit app

## Verification Results

### 1) Right-click interception is implemented
- **Status**: Implemented.
- **Where**: `Sources/Claux/ClauxApp.swift`
- **How it works**:
  - `MenuBarLabel` installs `RightClickInstaller` as a zero-size overlay.
  - `RightClickInstaller` locates `NSStatusBarButton` in the superview chain.
  - `StatusButtonHandler` replaces button target/action and installs a local monitor for `.rightMouseDown`.
  - Right-click (and control-click fallback) opens `MenuBarContextMenu`.

### 2) Context menu construction is implemented
- **Status**: Implemented.
- **Where**: `Sources/Claux/ClauxApp.swift` in `MenuBarContextMenu`.
- **Key actions present**:
  - `Settings…` -> opens settings window (`openWindow(id: "settings")`)
  - Visibility submenu under `Show in Menu Bar`
  - `Quit Claux` -> `NSApp.terminate(nil)`

## Changes Made

### A) Matched visibility options to requested set
- **What changed**:
  - Removed `Never` from visibility submenu.
  - Renamed `When Claude Code is running` to `When session is active`.
- **Why**:
  - Aligns menu options with requested behavior and avoids trapping users with hidden icon states.
- **Where**:
  - `Sources/Claux/ClauxApp.swift` (`visibilitySubmenu`, removed `setNever` action).

### B) Backward-compat migration for existing `never` preference
- **What changed**:
  - On app init, if stored `menuBarVisibility == "never"`, it is normalized to `"always"`.
- **Why**:
  - Prevents prior installs from staying in a hidden-icon mode after `Never` option removal.
- **Where**:
  - `Sources/Claux/ClauxApp.swift` (`init()`).

## Build Validation
- Ran `swift build`: **passes**.

## Notes
- This verification confirms code implementation and compile status.
- Runtime interaction was not UI-automated in this pass; if needed, do a manual smoke test by running the `.app` and right-clicking the menu bar icon.
