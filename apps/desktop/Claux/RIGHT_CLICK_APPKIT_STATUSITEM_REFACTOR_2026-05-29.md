# Right-Click AppKit StatusItem Refactor (2026-05-29)

## Problem
Right-click menu on the Claux menu bar icon still did not appear after multiple event-hook attempts.

## Root Cause Direction
`MenuBarExtra` click routing and status-item internals were not behaving reliably on this machine for right-click/context-menu display.

## Fix Strategy
Move away from `MenuBarExtra` click handling entirely and use a deterministic AppKit implementation:
- `NSStatusItem` for icon/button
- `NSPopover` for left-click popover
- `NSMenu` for right-click context menu

## What Changed

### 1) Replaced menu bar interaction layer
- **What**: Introduced `ClauxStatusAppDelegate` + `ClauxStatusItemController`.
- **Why**: AppKit `NSStatusItem` gives direct control over left/right clicks and popup menu behavior.
- **Where**: `Sources/Claux/ClauxApp.swift`.

### 2) Removed reliance on `MenuBarExtra` for icon clicks
- **What**: `ClauxApp` no longer declares a `MenuBarExtra` scene; it now configures the AppKit status controller via `@NSApplicationDelegateAdaptor`.
- **Why**: Avoid SwiftUI status-item event ambiguity.
- **Where**: `Sources/Claux/ClauxApp.swift` (`ClauxApp.body`).

### 3) Added deterministic click handling
- **Left click**: toggles `NSPopover` anchored to the status button.
- **Right click / control-click**: shows `NSMenu` anchored to the status button.
- **Where**: `ClauxStatusItemController.handleStatusItemClick`, `togglePopover`, `showContextMenu`.

### 4) Context menu now explicitly contains required actions
- `Settings…`
- `Show in Menu Bar` -> `Always` / `When session is active`
- `Quit Claux`
- **Where**: `ClauxStatusItemController.showContextMenu`.

### 5) Status item visibility + appearance wired to app state
- Visibility follows `menuBarVisibility` (`always` / `when_active`).
- Button display updates on session/defaults changes.
- **Where**: `configureObservers`, `updateVisibilityAndAppearance`, `updateStatusButtonAppearance`.

### 6) Added AppKit-backed settings/analytics window openers
- Implemented direct AppKit window creation for `SettingsView` and `AnalyticsView` via `NSWindowController`.
- Also wired `clauxOpenWindow` bridge so existing code paths can open these windows by id.
- **Where**: `ClauxStatusItemController.openWindow`, `showSettingsWindow`, `showAnalyticsWindow`.

### 7) Popover button fallbacks updated
- In `PopoverView`, actions that used `openWindow(id:)` now prefer `clauxOpenWindow` fallback for AppKit-hosted context.
- **Where**: `Sources/Claux/Views/PopoverView.swift`.

## Validation
- `swift build` passes.
- App rebuilt and relaunched via `./build_app.sh run`.

## Expected Outcome
Right-click on the Claux menu bar icon should now show the context menu reliably because it is handled by a direct `NSStatusItem`/`NSMenu` path, not `MenuBarExtra` interception.
