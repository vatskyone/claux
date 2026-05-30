# Right-Click Anchored Popup Fix (2026-05-29)

## Problem
Right-click menu still did not appear.

## Likely Cause
`NSMenu.popUpContextMenu(menu, with:event, for:view)` can fail silently when event context does not match expectations (common with status-item event routing and monitor-delivered events).

## Fix Applied
- Replaced event-dependent popup call with deterministic anchored popup:
  - `menu.popUp(positioning:nil, at:..., in: statusButton)`
- Expanded monitors to also handle control-click (`leftMouseDown + .control`) as secondary-click equivalent.
- Unified all right-click/control-click paths to call the same `showContextMenu()` helper.

## File
- `Sources/Claux/ClauxApp.swift`

## Validation
- `swift build` passes.
- App rebuilt and relaunched for manual retest.
