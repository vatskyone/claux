# Right-Click Native Context Menu Update (2026-05-29)

## Problem
Right-click on Claux menu bar icon still did not show a context menu, while other menu bar apps worked.

## Likely Issues
- Low-level AppKit interception (`NSStatusBarButton` hook + event monitor) can be fragile with `MenuBarExtra` internals.
- Status-item event routing differs across macOS versions/input methods, so custom right-click hooks may never receive expected events.

## Change Made
Switched to a native SwiftUI `contextMenu` directly on the `MenuBarLabel` view, and removed dependency on the custom overlay-based right-click path for user-facing behavior.

## Where
- `Sources/Claux/ClauxApp.swift`
  - `MenuBarLabel` now includes `.contextMenu { ... }` with:
    - `Settings…`
    - `Show in Menu Bar` -> `Always` / `When session is active` (checkmarked)
    - `Quit Claux`
  - Removed the `RightClickInstaller` overlay from label composition.

## Why
Using the native SwiftUI context-menu pathway is generally more stable with `MenuBarExtra` than custom event interception.

## Validation
- `swift build` passes.
- App relaunched for manual retest.
