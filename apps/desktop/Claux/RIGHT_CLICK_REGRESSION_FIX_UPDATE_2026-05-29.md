# Right-Click Regression Fix Update (2026-05-29)

## Issue
Right-click still did not work after multiple handler changes.

## Root Cause Found
`RightClickInstaller` was no longer mounted in `MenuBarLabel` after a previous refactor to SwiftUI `.contextMenu`.

Without this overlay, `StatusButtonHandler` was never attached to `NSStatusBarButton`, so local/global monitor logic in that handler could never execute.

## Fix Applied
- Restored `.overlay(RightClickInstaller().frame(width: 0, height: 0))` on `MenuBarLabel`.

## File
- `Sources/Claux/ClauxApp.swift`

## Validation
- `swift build` passes.
- App rebuilt and relaunched for manual retest.
