# Right-Click Fix Update (2026-05-29)

## Issue Reported
Right-click on the menu bar icon was not opening the context menu.

## Root Cause (Likely)
The right-click monitor path in `StatusButtonHandler` was too strict and could ignore valid status-bar events, especially when `NSEvent.window` is nil or right-click delivery differs by macOS behavior.

## Changes Made

### 1) Broadened right-click events handled by the status button
- **What**: Updated `sendAction(on:)` to include `.rightMouseUp` in addition to `.rightMouseDown`.
- **Why**: Some macOS configurations deliver actionable right-clicks on mouse-up, not only mouse-down.
- **Where**: `Sources/Claux/ClauxApp.swift` (`StatusButtonHandler.init`).

### 2) Hardened local event monitor logic
- **What**:
  - Monitor now listens to `[.rightMouseDown, .rightMouseUp]`.
  - If `event.window` exists, it must match the status button window.
  - If `event.window` is nil, event is still considered valid for status-bar right-click handling.
  - Context menu opens on `.rightMouseDown`, and event is consumed (`return nil`).
- **Why**: Prevent dropping legitimate menu-bar right-click events due window-matching assumptions.
- **Where**: `Sources/Claux/ClauxApp.swift` (`rightClickMonitor` block).

### 3) Prevented double-firing between monitor and action callback
- **What**:
  - Added `didOpenContextMenuFromMonitor` flag.
  - `handleClick` exits early when monitor already opened the context menu.
- **Why**: Avoid duplicate menu opens / conflicting behavior from both pathways firing for the same gesture.
- **Where**: `Sources/Claux/ClauxApp.swift` (`StatusButtonHandler` state + `handleClick`).

## Validation
- `swift build` passes after changes.
- Relaunched app with `./build_app.sh run` for manual retest.

## Next Manual Test
1. Right-click the Claux menu bar icon.
2. Confirm context menu appears.
3. Confirm entries work:
   - `Settings…`
   - `Show in Menu Bar` -> `Always` / `When session is active`
   - `Quit Claux`
