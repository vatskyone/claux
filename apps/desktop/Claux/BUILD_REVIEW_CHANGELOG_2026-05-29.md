# Claux Build Review Changes (2026-05-29)

## Scope
This document captures the fixes made after reviewing the Claux desktop app build and behavior.

## What Changed

### 1) Implemented `includeCacheCost` in cost calculation
- **What**: Session cost now conditionally includes cache read/write token charges based on the `includeCacheCost` setting.
- **Why**: The setting existed in UI but was not actually affecting totals.
- **Where**: `Sources/Claux/SessionParser.swift`
- **Detail**:
  - Reads `UserDefaults["includeCacheCost"]` with default `true`.
  - When disabled, cache token counts are still tracked for analytics/context, but excluded from per-turn cost.

### 2) Wired `autoRefreshInterval` into polling timer
- **What**: Session polling interval is now derived from `autoRefreshInterval` instead of being hardcoded.
- **Why**: Settings exposed refresh interval choices, but monitor always polled every 10 seconds.
- **Where**: `Sources/Claux/SessionMonitor.swift`
- **Detail**:
  - Added `refreshIntervalSeconds` computed value.
  - Uses configured value, fallback `10`, clamped to `5...300` seconds.
  - Existing settings observer already restarts monitoring on defaults changes, so interval updates apply automatically.

### 3) Implemented menu bar visibility behavior
- **What**: `menuBarVisibility` now controls whether the menu bar extra is inserted.
- **Why**: Context menu offered `Always / When Claude Code is running / Never`, but behavior was not enforced.
- **Where**: `Sources/Claux/ClauxApp.swift`
- **Detail**:
  - Added `@AppStorage("menuBarVisibility")` to app state.
  - Added `shouldShowMenuBarExtra`:
    - `always` => shown
    - `when_active` => shown only while an active session exists
    - `never` => hidden
  - Bound visibility with `MenuBarExtra(isInserted: .constant(shouldShowMenuBarExtra))`.
  - Removed old startup migration that forcibly changed `never` to `always`.

### 4) Unified app version used by bundle packaging
- **What**: `build_app.sh` now derives `CFBundleShortVersionString` from `AppVersion.current`.
- **Why**: Prevent drift between UI version and packaged app version.
- **Where**: `build_app.sh` and `Sources/Claux/Design.swift`
- **Detail**:
  - `build_app.sh` extracts `static let current = "..."` from `Design.swift`.
  - Falls back to `0.0.0` if parsing fails.

## Build Verification
- Ran `swift build` after changes: **passes**.

## Notes
- `swift test` still reports no tests found because no test target exists yet.
