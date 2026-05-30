# Right-Click Global Fallback Update (2026-05-29)

## Issue
Right-click on Claux menu bar icon still did not show a menu after local/SwiftUI context-menu changes.

## Hypothesis
`MenuBarExtra` can swallow or reroute local right-click events before app-local handlers receive them.

## Changes Made

### 1) Added global right-click fallback monitor
- **What**: Added `NSEvent.addGlobalMonitorForEvents(matching: .rightMouseDown)`.
- **Why**: Captures right-clicks even when local monitor/action callbacks are skipped.
- **Where**: `Sources/Claux/ClauxApp.swift` (`StatusButtonHandler`).

### 2) Added status-button hit-test guard
- **What**: Implemented `isMouseOverStatusButton()` using button bounds converted to screen coordinates and `NSEvent.mouseLocation`.
- **Why**: Ensures global right-click only opens menu when click happened over Claux icon.
- **Where**: `Sources/Claux/ClauxApp.swift`.

### 3) Centralized menu popup behavior
- **What**: Added `showContextMenu(with:)` helper and reused across local/global paths.
- **Why**: One consistent path, less duplication, easier debugging.
- **Where**: `Sources/Claux/ClauxApp.swift`.

### 4) Monitor lifecycle cleanup
- **What**: Removes both local and global monitors in `deinit`.
- **Why**: Avoid leaks/duplicate listeners across re-installs.
- **Where**: `Sources/Claux/ClauxApp.swift`.

## Validation
- `swift build` passes.
- App rebuilt and relaunched for manual retest.
