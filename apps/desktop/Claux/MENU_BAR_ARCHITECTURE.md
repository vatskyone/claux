# Menu Bar Architecture (Claux)

## Current Design
Claux uses a direct AppKit status-item stack for menu bar interaction:
- `NSStatusItem` for the menu bar icon/button
- `NSPopover` for left-click UI (`PopoverView`)
- `NSMenu` for right-click context menu

Implementation lives in:
- `Sources/Claux/ClauxApp.swift`
  - `ClauxStatusAppDelegate`
  - `ClauxStatusItemController`

## Why This Architecture
The previous `MenuBarExtra` + custom event interception path was not reliable for right-click behavior on this environment. The AppKit path gives deterministic control of click routing and menu display.

## Click Behavior
- Left click: toggles popover.
- Right click or control-click: opens context menu.

## Context Menu Contract
The context menu must include:
- `Settings…`
- `Show in Menu Bar`
  - `Always`
  - `When session is active`
- `Quit Claux`

## Visibility Rules
`menuBarVisibility` (UserDefaults) controls status item presence:
- `always`: status item shown.
- `when_active`: status item shown only with an active Claude session.

## Window Routing
`clauxOpenWindow` is the bridge used by SwiftUI views and AppKit actions to open `settings` and `analytics` windows through `ClauxStatusItemController`.

## Maintenance Notes
- Prefer changing behavior in `ClauxStatusItemController` instead of reintroducing `MenuBarExtra` click interception.
- If UI gets desynced, first verify `updateVisibilityAndAppearance()` and `updateStatusButtonAppearance()` are triggered by observers.
