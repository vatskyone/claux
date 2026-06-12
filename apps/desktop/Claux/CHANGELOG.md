# CLAUX Changelog

---

## [1.16.3] — 2026-06-12

### New Features
- **Multi-session switcher** (`AppStore.swift`, `Views/PopoverView.swift`) — when multiple Claude Code sessions are running simultaneously, the "Live" status label in the header becomes a tappable dropdown menu listing all active sessions. Each row shows the project folder name, current cost, and context window usage. Selecting a session switches the dashboard card to that session. The header label updates to "N Live" to indicate the session count. Single-session behaviour is unchanged.

## [1.16.2] — 2026-06-12

### Bug Fixes
- **Menu bar icon and badge text now correctly adapt to dark/light menu bar** (`ClauxApp.swift`) — the status-bar button's template image was being overridden by explicit `contentTintColor` values whose dark-mode detection was unreliable. The fix uses `isTemplate = true` with `contentTintColor = nil`, which lets the system render the icon in the correct colour for the actual menu bar context (white on dark, black on light). Badge text now uses `NSColor.labelColor`, a live dynamic colour that resolves correctly at draw time regardless of the in-app theme setting. Manual dark-mode detection and related appearance observers have been removed in favour of this native rendering approach.

## [1.16.1] — 2026-06-12

### Bug Fixes
- **Menu bar icon and text now visible in dark mode** (`ClauxApp.swift`) — the status item image was using `isTemplate = false`, which locked the icon to its original black rendering regardless of menu bar appearance. Switched to `isTemplate = true` so macOS automatically renders it white on dark menu bars and black on light ones. `contentTintColor` for the inactive state is now `nil` (fully automatic) instead of the forced `.labelColor`; the green tint for active sessions is preserved.

## [1.16.0] — 2026-06-10

### New Features
- **Saturday & Sunday in weekly recap day picker** (`NotificationManager.swift`, `Views/SettingsView.swift`) — the "Weekly recap day" setting now includes Saturday and Sunday options in addition to the existing Monday–Friday choices.
- **Hourly "Send at" options** (`Views/SettingsView.swift`) — the daily/weekly summary send-time picker now lists every hour of the day (12:00 am through 11:00 pm) instead of only four fixed slots.
- **$100 cost alert tier** (`Views/SettingsView.swift`) — added a `$100` option to the cost alert threshold picker, between the existing `$50` and the top of the range.
- **1 s auto-refresh option** (`Views/SettingsView.swift`, `SessionMonitor.swift`) — added a `1 s` option to the auto-refresh picker and lowered the minimum clamp in `SessionMonitor` from 5 s to 1 s so the preference is actually honoured.

### Improvements
- **"Erase All Data" preserves settings** (`AppStore.swift`, `Views/SettingsView.swift`) — the "Erase All Data…" button in Data Source now calls a new `eraseSessionData()` method that clears in-memory session arrays, the parse cache, and notification-tracking keys (`notificationSnoozedDayKey`, `weeklyRecapLastSentWeekKey`) without touching any user-configured settings. The confirmation copy and help text have been updated to reflect this. The "Reset all settings to default" footer button remains available for a full settings wipe.

## [1.15.1] — 2026-06-07

### Bug Fixes
- **TCC privacy guard for CLAUDE.md scanning** (`SessionParser.swift`) — added `isTCCProtectedHomePath` and `normalizedPath` helpers so the CLAUDE.md scorer never probes macOS privacy-protected home directories (`Desktop`, `Documents`, `Downloads`, `Movies`, `Music`, `Pictures`). Prevents unexpected TCC permission prompts (e.g. "Media & Apple Music") during session parsing. Scoring is also gated behind a new `allowExternalClaudeMdScoring` UserDefaults key and skips protected paths entirely.

### Improvements
- **Plan limits auto-refresh on session update** (`AppStore.swift`) — `rateLimitMonitor.refresh()` is now called automatically whenever the session monitor publishes new sessions, so plan-limit bars update without requiring a manual refresh or popover re-open.
- **Account section removed** (`Views/SettingsView.swift`) — the Account section (sign-in, sign-out, sync toggle, `SignInSheet`) has been removed from Settings. Clears unused state variables and simplifies the settings panel.
- **Feedback link updated** (`Views/SettingsView.swift`) — the "Open an issue" link now points to the correct `vatskyone/claux` GitHub repository.
- **Refresh button label clarified** (`Views/SettingsView.swift`) — "Refresh sessions now" renamed to "Refresh sessions and plan limits" to reflect that both datasets are updated on tap.

## [1.15.0] — 2026-06-04

### New Features
- **Actionable session notifications** (`NotificationManager.swift`, `ClauxApp.swift`, `Views/PopoverView.swift`, `AppStore.swift`) — session-ended, cost-threshold, and context-window alerts now carry richer session diagnostics plus notification actions to open the session, open the dashboard, or snooze the rest of the day.
- **Notification verbosity and scheduling controls** (`NotificationManager.swift`, `Views/SettingsView.swift`, `Design.swift`) — added `Minimal`, `Standard`, and `Detailed` notification modes together with quiet hours, weekday-only summaries, and weekly recap scheduling in Settings.
- **Weekly recap notification** (`NotificationManager.swift`, `AppStore.swift`, `Models.swift`) — Claux can now send a weekly recap for the last 7 completed days with top project/model and editing outcomes in addition to the existing daily summary.

## [1.14.0] — 2026-06-04

### New Features
- **Daily recap notification drill-down** (`NotificationManager.swift`, `AppStore.swift`, `Views/DailyRecapSheet.swift`, `Views/PopoverView.swift`, `ClauxApp.swift`) — daily summary notifications now include a richer spend/session digest and clicking the notification opens a new in-app daily recap sheet with top project, top model, accepted/rejected actions, and the day’s strongest sessions.

## [1.13.3] — 2026-06-03

### Improvements
- **Refined app icon artwork** (`Sources/Claux/Resources/app-icon.png`) — updated the bundled Claux icon asset with the latest user-provided artwork so newly built app bundles use the revised branding in Finder, Applications, and DMG installs.

## [1.13.2] — 2026-06-03

### Improvements
- **Updated branded app icon** (`Sources/Claux/Resources/app-icon.png`) — refreshed the bundled Claux app icon with the latest rounded-corner artwork and color treatment so the installed app and DMG present the final branding consistently.

## [1.13.1] — 2026-06-03

### New Features
- **Custom app icon packaging** (`build_app.sh`, `Sources/Claux/Resources/app-icon.png`) — the macOS app bundle now generates and embeds a proper `.icns` file from the shipped source icon so `Claux.app` and DMG installs display the branded app icon in Finder, Launchpad, and Applications.

### Bug Fixes
- **Bundle icon metadata compatibility** (`build_app.sh`) — switched the app bundle plist to reference the icon by basename (`Claux`) instead of the full filename so Finder correctly resolves the bundled `.icns` during install and DMG mounting.

## [1.13.0] — 2026-06-03

### New Features
- **Drag-and-drop DMG packaging** (`build_dmg.sh`) — added a dedicated macOS DMG build script that packages `Claux.app` alongside an `Applications` shortcut so users can install the app with the standard drag-and-drop flow.

### Improvements
- **Ad-hoc app signing in bundle build** (`build_app.sh`) — the app bundle build now runs `codesign --sign -` so local `.app` and DMG artifacts match the documented packaging flow more closely.
- **Install/build docs updated** (`README.md`) — added DMG build instructions and documented the output location in `apps/desktop/Claux/dist/`.

## [1.12.0] — 2026-06-03

### New Features
- **Additional accessible state palettes** (`Design.swift`, `Views/SettingsView.swift`) — added `High Contrast`, `Colorblind Safe`, and `Soft Contrast` options to the in-app `State colors` selector so users can choose a more legible blue/green/orange/red palette for badges, warnings, progress bars, and session stats.

## [1.11.0] — 2026-06-03

### New Features
- **State color palette switcher** (`Views/SettingsView.swift`, `Design.swift`) — added an in-app `State colors` setting with `System` and `Vivid` palettes plus a live blue/green/orange/red preview so state and stat colors can be reviewed without code changes.

### Improvements
- **Normalized state palette** (`Design.swift`) — introduced shared semantic blue/green/orange/red helpers and a consistent warning-orange ramp for usage, warnings, badges, and status states.
- **State/stat views migrated to semantic colors** (`Views/ActiveSessionCard.swift`, `Views/AnalyticsView.swift`, `Views/ContextHealthBar.swift`, `Views/OnboardingView.swift`, `Views/PlanLimitsCard.swift`, `Views/PopoverView.swift`, `Views/SessionDetailSheet.swift`, `Views/SpendSummaryView.swift`, `Views/TokenBreakdownView.swift`) — the key session, analytics, onboarding, and dashboard surfaces now respect the selected palette instead of mixing direct macOS system colors.

## [1.10.9] — 2026-06-03

### Bug Fixes
- **First-launch onboarding now appears from the menu bar popover** (`ClauxApp.swift`) — launching Claux no longer auto-opens the Settings window on fresh install. The app now stays menu-bar-first and automatically presents the popover onboarding when setup has not been completed.

## [1.10.8] — 2026-06-03

### Improvements
- **Fresh-install activation guidance** (`RateLimitMonitor.swift`, `Models.swift`, `Views/PlanLimitsCard.swift`) — when Claude integration is installed but has never been invoked yet, the Dashboard now tells users to restart Claude Code, accept the trust prompt, and send one message instead of showing the generic `statusLine source not running` state.

## [1.10.7] — 2026-06-03

### Bug Fixes
- **Bundled statusline wrapper lookup fixed** (`ClaudeStatusLineManager.swift`) — the Claude integration installer now loads `statusline_wrapper.py` from the SwiftPM resource bundle (`Bundle.module`) instead of only checking `Bundle.main`, fixing the “Bundled statusline wrapper is missing from the app.” error in the packaged app.

## [1.10.6] — 2026-06-03

### New Features
- **Managed Claude statusLine installer** (`ClaudeStatusLineManager.swift`, `Resources/statusline_wrapper.py`) — Claux can now install and repair its own Claude `statusLine` integration, writing a managed wrapper that collects plan limits while preserving any existing custom command.
- **Onboarding integration step** (`Views/OnboardingView.swift`) — first-run onboarding now includes a dedicated Claude integration step so new users can enable plan-limit collection in one flow instead of editing `~/.claude/settings.json` manually.

### Improvements
- **Settings repair controls** (`Views/SettingsView.swift`) — added install/repair controls and live integration status in Settings so existing users can migrate or recover the Claude integration without manual file edits.
- **Plan-limit diagnostics for unmanaged setups** (`Models.swift`, `RateLimitMonitor.swift`, `Views/PlanLimitsCard.swift`) — the Dashboard now distinguishes between missing integration, custom unmanaged statusLine commands, broken managed installs, invalid statusLine JSON, and runtime inactivity.

### Bug Fixes
- **Onboarding no longer auto-skips integration** (`Views/PopoverView.swift`) — removed the old popover heuristic that could dismiss onboarding before the new integration step had a chance to run.

## [1.10.5] — 2026-06-02

### Improvements
- **Weekly plan-limit label spacing** (`Views/PlanLimitsCard.swift`) — added a separator dot between `7-day window` and `All models` to improve readability and spacing in the Dashboard header row.

## [1.10.4] — 2026-06-02

### Improvements
- **7-day plan-limit label clarified** (`Views/PlanLimitsCard.swift`) — added an `All models` label next to the 7-day window title in the Dashboard so the scope of the weekly limit is explicit.

## [1.10.3] — 2026-06-02

### Improvements
- **Refresh action wording clarified** (`Views/PopoverView.swift`) — updated the Dashboard refresh button tooltip to explicitly say it refreshes both session data and plan-limit stats, matching the existing `AppStore.refreshNow()` behavior.

## [1.10.2] — 2026-06-02

### Improvements
- **Forced 24-hour reset timestamps** (`Views/PlanLimitsCard.swift`) — the Dashboard plan-limit reset labels now use explicit 24-hour formatting (`HH:mm` and `DD Mon HH:mm`) instead of locale-dependent time output.

## [1.10.1] — 2026-06-02

### Improvements
- **Exact plan-limit reset timestamps** (`Views/PlanLimitsCard.swift`) — the Dashboard now shows both the countdown and the exact local reset moment for each plan window: `HH:mm` for the 5-hour window and `DD Mon HH:mm` for the 7-day window.

## [1.10.0] — 2026-06-02

### New Features
- **Active-session quality shortcut** (`Views/PopoverView.swift`) — added a new dashboard/header icon beside refresh that opens the same per-session quality and acceptance detail sheet for the current live session, so users can inspect session quality without switching to History first.

## [1.9.0] — 2026-06-02

### New Features
- **Per-session acceptance and quality metrics** (`SessionParser.swift`, `Models.swift`) — each parsed session now tracks accepted edits, rejected actions, successful tool results, completed agents, touched files, and a derived session quality score from the local Claude Code JSONL.
- **History session quality panel** (`Views/SessionDetailSheet.swift`) — tapping a session in the History tab now opens a richer detail sheet with a quality score bar, acceptance breakdown, rejected-action counts, agent outcome summary, permission mode badge, and touched-file context.

### Improvements
- **Scrollable session detail sheet** (`Views/SessionDetailSheet.swift`) — the session detail overlay now scrolls within the popover so expanded per-session diagnostics remain usable without clipping.

## [1.8.1] — 2026-06-01

### Improvements
- **Plan-limits empty-state diagnostics** (`RateLimitMonitor.swift`, `Models.swift`, `AppStore.swift`, `Views/PlanLimitsCard.swift`, `Views/PopoverView.swift`) — Dashboard now explains *why* plan-limit bars are blank with explicit states: waiting for first API response, statusLine source not running, plan limits unavailable in current session payload, or stale data.
- **Statusline debug-signal integration** (`RateLimitMonitor.swift`) — the rate-limit monitor now reads the local statusline debug signal (`statusline_debug.log`) to classify missing-data causes instead of showing a generic placeholder.

---

## [1.8.0] — 2026-06-01

### New Features
- **Plan limits dashboard card** (`Views/PlanLimitsCard.swift`, `Views/PopoverView.swift`) — added a new Dashboard card that shows Claude subscription usage for the **5-hour** and **7-day** windows with progress bars, used percentage, and reset countdowns.
- **Plan-limit data model + monitor** (`Models.swift`, `RateLimitMonitor.swift`, `AppStore.swift`) — introduced `PlanLimitsSnapshot` / `PlanLimitWindow` and a local `RateLimitMonitor` that watches `<monitoredDirectory>/claux/rate_limits.json`, parses usage windows, and publishes updates into app state.

### Improvements
- **Manual refresh now includes plan limits** (`AppStore.swift`) — pressing refresh (or opening the popover) now updates both session data and plan-limit data.
- **Dashboard fallback state for missing plan-limit data** (`Views/PlanLimitsCard.swift`) — when no snapshot is present, the card renders an explicit “No plan-limit data yet” state with the expected local file path.

---

## [1.7.2] — 2026-05-30

### Bug Fixes
- **Popover re-centered under icon** (`ClauxApp.swift`) — reverted the left-anchor positioning introduced in v1.7.1; the popover now drops down centered beneath the menu bar icon, matching v1.6.4 behavior.

---

## [1.7.1] — 2026-05-30

### Improvements
- **Analytics default range 7D** (`AnalyticsView.swift`) — both the full Analytics window and the compact popover Analytics tab now open on the 7-day view instead of 30 days.
- **"Open full Analytics window" link in blue** (`PopoverView.swift`) — the link at the bottom of the Analytics tab now uses `systemBlue` to match other interactive text in the UI, replacing the previous tertiary-label grey.
- **Popover anchored to left of menu bar icon** (`ClauxApp.swift`) — the popover now positions its right edge flush with the left edge of the status item button, so it drops down to the left of the icon rather than centred beneath it.

---

## [1.7.0] — 2026-05-30

### New Features
- **AppKit menu bar architecture** (`ClauxApp.swift`) — Full architectural refactor: replaced `MenuBarExtra` with a native `NSStatusItem` + `NSPopover` + `NSMenu` stack implemented via `ClauxStatusAppDelegate` and `ClauxStatusItemController`. Left click deterministically toggles the popover; right click (and control-click) deterministically shows the native context menu. Eliminates the event-routing ambiguity inherent to `MenuBarExtra`.
- **Right-click context menu** (`ClauxApp.swift`) — Right-clicking the Claux menu bar icon now reliably shows the context menu on all tested macOS versions. Menu items: **Settings…**, **Show in Menu Bar** (Always / When session is active), **Quit Claux**.
- **`clauxOpenWindow` bridge** (`ClauxApp.swift`, `PopoverView.swift`) — New global bridge allows SwiftUI views hosted inside the AppKit popover to open Settings and Analytics windows through `ClauxStatusItemController`, replacing the broken `openWindow(id:)` SwiftUI environment action in the AppKit-hosted context.

### Bug Fixes
- **`includeCacheCost` now applied to cost totals** (`SessionParser.swift`) — The setting existed in UI but was not actually excluding cache token charges from cost calculations. Now correctly omits cache read/write costs from per-turn totals when disabled.
- **`autoRefreshInterval` wired into polling timer** (`SessionMonitor.swift`) — Session monitor was always polling every 10 s regardless of the configured refresh interval. Now reads the stored preference (clamped 5–300 s) and restarts the timer when settings change.
- **Menu bar visibility now enforced** (`ClauxApp.swift`) — `menuBarVisibility` preference (`always` / `when_active`) now actually controls status item presence. `when_active` hides the icon when no Claude session is active; `always` keeps it visible at all times.

### Improvements
- **Visibility option cleanup** (`ClauxApp.swift`) — Removed the `Never` option from the "Show in Menu Bar" submenu (avoids trapping users with a hidden icon). Renamed `When Claude Code is running` → `When session is active` for clarity. Existing `never` preferences are automatically migrated to `always` on first launch.

### Internal
- **Build version unified** (`build_app.sh`, `Design.swift`) — `build_app.sh` now extracts `AppVersion.current` from `Design.swift` and writes it to `CFBundleShortVersionString` in the packaged `Info.plist`, preventing version drift between the UI and the app bundle.
- `ClauxStatusAppDelegate` + `ClauxStatusItemController` introduced; `MenuBarExtra` scene removed from `ClauxApp.body`.
- `updateVisibilityAndAppearance()` and `updateStatusButtonAppearance()` are the single choke-points for all icon/visibility state changes.

---

## [1.6.4] — 2026-05-29

### Internal
- Removed CLAUX logo watermark from Dashboard tab bottom area.

---

## [1.6.3] — 2026-05-29

### Improvements
- **Real CLAUX logo** (`PopoverView.swift`, `Package.swift`) — replaced ASCII pixel-art watermark with the actual `ascii-art-text.png` logo image, bundled as a Swift package resource. Uses multiply blend (light mode) and invert + screen blend (dark mode) so the white-background PNG renders cleanly on any macOS appearance.

---

## [1.6.2] — 2026-05-29

### New Features
- **CLAUX pixel-art logo** (`PopoverView.swift`) — 5×5 monospace block-art watermark of "CLAUX" displayed at the bottom of the Dashboard tab in the blank space below the active session card; very subtle (quaternary label color).

### Improvements
- **History tab — sticky header** (`PopoverView.swift`) — "Recent" label, session count badge, and search bar are now pinned and do not scroll; only the session rows scroll, matching standard list UX.
- **Analytics tab — no duplicate sparkline** (`PopoverView.swift`) — 7-day sparkline removed from the SpendSummaryView card in the Analytics tab (the chart below already covers spend history).
- **Analytics tab — tighter layout** (`PopoverView.swift`) — reduced vertical padding between the spend-totals card / divider and the analytics chart for a cleaner, denser layout.

---

## [1.6.1] — 2026-05-29

### Bug Fixes
- **Right-click context menu** (`ClauxApp.swift`) — fixed on macOS 14/15 (Sequoia) by adding a `NSEvent.addLocalMonitorForEvents(.rightMouseDown)` path as the primary handler; the OS was swallowing `rightMouseUp` before it reached the button's action target. Also catches control-click as an alias.

### Improvements
- **Larger header icons** (`PopoverView.swift`) — refresh and settings buttons increased from size 10 → 13, tap target from 22 × 22 → 28 × 28.
- **Dashboard — session card only** (`PopoverView.swift`) — `SpendSummaryView` moved to the Analytics tab; the Dashboard tab now shows only the active session card for a leaner default view.
- **Fixed popover height** (`PopoverView.swift`) — tab content area is pinned to 340 pt; switching between Dashboard, Analytics, and History no longer resizes the popover window.
- **Analytics tab — spend summary at top** (`PopoverView.swift`) — Today / This week / This month spend cards now appear at the top of the Analytics tab before the chart and breakdowns.

---

## [1.6.0] — 2026-05-29

### New Features
- **History tab** (`PopoverView.swift`) — sessions list and search bar moved out of the main view into a dedicated "History" tab, keeping the dashboard lean. Switch to it with the tab bar at the bottom of the popover.
- **Analytics tab** (`PopoverView.swift`, `AnalyticsView.swift`) — compact inline analytics (spend chart, by-project, by-model) now lives directly in the popover. An "Open full Analytics window" link opens the existing large window for the full view.
- **Three-tab segmented control** (`PopoverView.swift`) — native macOS segmented picker at the bottom of the popover replaces the old bottom bar: **Dashboard · Analytics · History**.

### Improvements
- **Settings moved to header** (`PopoverView.swift`) — gear icon added to the top-right header alongside the refresh button; bottom bar is now the tab control only.
- **Dashboard view** (`PopoverView.swift`) — now shows only the active session card and spend summary, with no sessions list cluttering the default view.
- **CompactAnalyticsView** (`AnalyticsView.swift`) — new struct optimised for the 340 pt popover width: adaptive 2-row project rows (no fixed-width truncation issues) and the full interactive spend chart.

### Internal
- `PopoverTab` enum added (`PopoverView.swift`): `.dashboard`, `.analytics`, `.history`.
- Old `bottomBar` (Settings + Analytics buttons) removed from `PopoverView`.

---

## [1.5.4] — 2026-05-27

### Improvements
- **SessionDetailSheet native blur** (`SessionDetailSheet.swift`) — replaced `.thickMaterial` with `VisualEffectView(.withinWindow)` so the detail overlay frosts the popover content behind it for a true layered-glass look.
- **Settings form transparent rows** (`SettingsView.swift`) — applied `.listRowBackground(Color.clear)` to all form sections so the sidebar blur shows through every row, eliminating the white opaque row backgrounds.
- **Session row: removed status dot** (`SessionRowView.swift`) — dropped the colored `Circle()` indicator from session rows to reclaim space and left-align text flush with the other list items.
- **Centered capsule search bar** (`RecentSessionsView.swift`) — replaced `NSSearchField` wrapper with a native SwiftUI capsule search bar: magnifier icon + placeholder centered when idle, slides left on focus with a clear button; fully rounded corners for a smoother look.
- **Active session timer color** (`ActiveSessionCard.swift`) — session duration at top-right now uses `systemGray` instead of `tertiaryLabel` for better legibility over the blur background.

---

## [1.5.3] — 2026-05-27

### Improvements
- **SwiftUI native green & yellow** (`Design.swift`) — `clauxGreen` now uses `Color.green` and `clauxYellow` uses `Color.yellow` (SwiftUI's built-in adaptive palette) instead of custom NSColor/RGB values. Colors now follow the system's dynamic rendering and match the SwiftUI kit consistently.

---

## [1.5.2] — 2026-05-27

### Improvements
- **Darker warning yellow** (`Design.swift`) — `clauxYellow` changed from `NSColor.systemYellow` (pale, washed out on vibrancy backgrounds) to a rich amber `rgb(0.82, 0.58, 0.02)`. Affects the context health bar warning state (70–90% fill) and any other UI that references `clauxYellow`.

---

## [1.5.1] — 2026-05-27

### Improvements
- **Settings blur now visible** (`SettingsView.swift`) — added `.scrollContentBackground(.hidden)` to the Form so SwiftUI's grouped-form scroll background no longer paints over the `NSVisualEffectView` blur; the blurred desktop now shows through behind the settings content.
- **Settings controls right-aligned** (`SettingsView.swift`) — all Picker rows converted from `LabeledContent { Picker(...).labelsHidden().frame(width:) }` to bare `Picker("label", selection:)` inside the Form. SwiftUI's grouped Form places bare Pickers the same way as Toggles — label on the left, popup button flush with the right edge — so all controls are now consistently aligned. The `System permission` and `Context window alert` rows had `.frame(maxWidth: .infinity, alignment: .trailing)` added to their HStack content for the same effect.
- **Settings footer centered** (`SettingsView.swift`) — "Reset all settings to default" text now has `.frame(maxWidth: .infinity, alignment: .center)` so it is horizontally centered in the footer bar regardless of window width.

---

## [1.5.0] — 2026-05-27

### Improvements
- **Native macOS vibrancy blur backgrounds** (`Design.swift`, `PopoverView.swift`, `SettingsView.swift`, `AnalyticsView.swift`, all card views) — all windows and the menu bar popover now use `NSVisualEffectView` instead of solid `windowBackgroundColor`. The popover uses `.menu` material with `.behindWindow` blending (blurs whatever is behind the panel — same look as Spotlight and Control Centre). Settings and Analytics use `.sidebar` material. Cards inside the popover use `.regularMaterial` (`.withinWindow` blending) so they appear as a frosted-glass panel floating above the main blur layer — the standard macOS layered-glass aesthetic. `WindowBlurInstaller` sets `window.backgroundColor = .clear` and `isOpaque = false` on the host panel via `viewDidMoveToWindow` to let the blur composite against the desktop. `CardStyle` updated from `Color.clauxSurface` to `.regularMaterial`. All inline `controlBackgroundColor` / `windowBackgroundColor` references replaced with `.regularMaterial` / `.thickMaterial` respectively.

---

## [1.4.1] — 2026-05-27

### Bug Fixes
- **Today's spend now resets correctly at midnight** (`SessionParser.swift`, `AppStore.swift`, `Models.swift`) — "Today" was always showing $0.00 for sessions that started before midnight (e.g. a long session that began yesterday and continued into today). Root cause: `computeSpend` bucketed each session's *entire* cost by `startTime`, so any session that started before today's midnight was excluded. Fix: the parser now builds a `dailyCosts: [Date: Double]` map as it processes each assistant turn, attributing that turn's cost to the calendar day of its UTC timestamp converted to local time. `AppStore.computeSpend` and `computeDailySpend` now iterate over `dailyCosts` instead of bucketing by `startTime`, so multi-day sessions split correctly across today / yesterday / week / month. The analytics chart also benefits from the per-day attribution.

---

## [1.4.0] — 2026-05-27

### New Features
- **Right-click context menu on menu bar icon** (`ClauxApp.swift`) — right-clicking the Claux icon now shows a native macOS context menu with: Open Claux Dashboard (opens claux.app/dashboard), Settings…, Check for Updates… (shows current version dialog), Show in Menu Bar selector (Always / When Claude Code is running / Never — preference stored, icon hide/show wired in next sprint), and Quit Claux.

### Improvements
- **Full-width card separators** (`ActiveSessionCard.swift`, `SpendSummaryView.swift`, `RecentSessionsView.swift`) — removed asymmetric `.padding(.leading, …)` from all `Divider()` calls inside white cards so separators now span the full card width, matching left and right edges equally.
- **"Reset all settings to default" footer** (`SettingsView.swift`) — a blue underlined link now appears centered at the very bottom of the Settings window. Triggers the same confirmation dialog as the existing "Erase All Data" button.

---

## [1.3.0] — 2026-05-26

### New Features
- **Session search / filter** (`RecentSessionsView.swift`) — a native `NSSearchField` now appears above the session list (shown when ≥ 2 sessions exist). Filters by AI-generated title or project path in real time. The count badge updates to show "matching/total" while a query is active. An "No results" empty state appears when nothing matches.
- **First-launch onboarding flow** (`OnboardingView.swift`, `PopoverView.swift`) — a 3-step overlay shown inside the popover on first open. Step 1: welcome + description. Step 2: session directory confirmation with a "Change…" option. Step 3: notification permission request with live status feedback. Animated progress capsules indicate the current step. Existing users are automatically skipped via a migration check on first popover open.

---

## [1.2.1] — 2026-05-26

### New Features
- **7-day spend sparkline** (`SpendSummaryView.swift`) — a compact bar chart now sits at the top of the spend card, showing daily costs for the last 7 days. Today's bar is full blue; prior days are dimmed. Hidden when there is no spend history. Uses SwiftUI Charts (already available in the project).
- **Monthly budget tracker** (`SpendSummaryView.swift`, `SettingsView.swift`, `AppStore.swift`, `ClauxApp.swift`) — set a monthly budget in Settings → General → Monthly budget (Off / $25 / $50 / $100 / $200 / $500 / $1 000). When set, a progress bar appears at the bottom of the spend card color-coded green (< 70%) → yellow (70–90%) → red (> 90%), with remaining amount or "Over budget" label.
- **Daily summary notification** (`NotificationManager.swift`, `SettingsView.swift`, `ClauxApp.swift`) — opt-in notification (Settings → Notifications → Daily summary) that fires once per day after a configurable hour (12 pm / 3 pm / 6 pm / 9 pm) with today's total spend and session count. Fires on the next session update after the configured time; uses a local date key to guarantee at most one delivery per calendar day.

### Internal
- `SpendSummary` now carries `yesterday`, `prevWeek`, `prevMonth` baseline fields (added in 1.1.0); `SpendSummaryView` now also takes a `sparkData: [DailySpend]` parameter passed from `PopoverView`.
- `AppStore.resetAllData()` now clears `monthlyBudget`, `dailySummaryEnabled`, `dailySummaryHour`.
- Deferred item G (global keyboard shortcut) — requires replacing SwiftUI `MenuBarExtra` with a manual `NSStatusItem` so the status bar button can be clicked programmatically. Planned for a future refactor sprint.

---

## [1.1.0] — 2026-05-26

### New Features
- **Spend trend indicators** (`SpendSummaryView.swift`, `Models.swift`, `AppStore.swift`) — each cell in the spend strip now shows a small ↑ / ↓ badge vs. the equivalent prior period: "Today" compares against yesterday, "This week" against the previous 7 days, "This month" against the previous 30 days. ↑ appears in orange (spending more), ↓ in green (spending less). The badge is hidden when there is no prior history or the change is under 5% (noise threshold). Added `yesterday`, `prevWeek`, `prevMonth` fields to `SpendSummary` (defaulted to 0, backward-compatible).
- **Right-click context menu on session rows** (`SessionRowView.swift`) — secondary-click (or two-finger tap) on any recent session row reveals: **Copy Path** (raw project path to clipboard), **Show in Finder** (opens the project directory), and **Copy Session ID** (JSONL UUID, useful for direct file inspection).

### Improvements
- **Spend data refreshes on popover open** (`PopoverView.swift`) — `store.refreshNow()` is now called whenever the popover window becomes key (i.e. every time the user clicks the menu bar icon). Previously, totals could be up to 10 seconds stale on re-open; now they are always current.

---

## [1.0.0] — 2026-05-26

### Bug Fixes
- **Theme now applies to all windows including the popover** (`Design.swift`) — `AppThemeModifier` previously called only `.preferredColorScheme()`, a SwiftUI hint that `MenuBarExtra` NSPanel windows ignore. It now also sets `NSApp.appearance` directly (aqua / darkAqua / nil) on `.onAppear` and `.onChange`, which applies immediately to every window in the process.
- **Notification "Allow" button now shows the system permission dialog** (`NotificationManager.swift`) — `requestAuthorization` was being called from a `getNotificationSettings` background callback. macOS silently drops the permission dialog when the calling app is not the frontmost process. Fixed by wrapping in `DispatchQueue.main.async { NSApp.activate(ignoringOtherApps: true); ... }`. Same fix applied to `sendTestNotification()`.
- **Notification permission row updates live after granting** (`SettingsView.swift`) — the `switch` was reading `NotificationManager.shared.authStatus` directly (bypassing SwiftUI's `@ObservedObject` dependency tracking), so the row never re-rendered when `authStatus` changed. Fixed by using `notifManager.authStatus` (the `@ObservedObject` instance). Added `.onAppear { notifManager.refreshAuthStatus() }` so the row shows the correct state every time Settings opens.

---

## [0.9.9] — 2026-05-26

### New Features
- **Session detail opens as in-popover overlay** (`PopoverView.swift`, `SessionRowView.swift`, `RecentSessionsView.swift`, `SessionDetailSheet.swift`) — tapping a recent session no longer opens a separate OS sheet window. Instead the detail slides up from the bottom of the 340 px popover as a full-width overlay with a dimmed backdrop. Dismiss by clicking the ✕ button _or_ by tapping anywhere on the dimmed area outside the card.
- **Notification permission status row** (`SettingsView.swift`) — Settings → Notifications now shows the live macOS authorization status: green "Enabled ✓", red "Denied ✗ + Open System Settings", or orange "Not requested + Allow button". The row updates in real time via `@ObservedObject` so clicking "Allow" immediately reflects the new state.

### Improvements
- **Notifications now fire reliably** (`NotificationManager.swift`, `ClauxApp.swift`) — `NotificationManager` is now `ObservableObject` with a `@Published authStatus` that Settings observes. `requestPermission()` replaces the old `requestPermissionIfNeeded()`; it handles all three states (`.notDetermined` → request dialog, `.denied` → open System Settings, `.authorized` → noop). The old `didNudge` rate-limit that permanently suppressed re-prompting is removed. `fire()` no longer silently drops notifications when `.denied`; it simply skips (no false positives).
- **Active session card cleaned up** (`ActiveSessionCard.swift`) — removed the green pulsing dot before "ACTIVE SESSION" label; removed the bolt ⚡ icon before "Cache efficiency"; removed the doc 📄 icon before "CLAUDE.md quality". Labels are plain text only, consistent with the rest of the card.

### Internal
- `SessionDetailSheet` now takes `onDismiss: () -> Void` instead of using `@Environment(\.dismiss)` — required for the overlay architecture.
- `SessionRowView` and `RecentSessionsView` now accept an `onSelect: (ClaudeSession) -> Void` callback instead of owning sheet state.
- `PopoverView` owns `@State private var selectedSession: ClaudeSession?` and renders the overlay via `ZStack`.
- `NotificationManager.shared` is `ObservableObject`; `SettingsView` holds it via `@ObservedObject`.

---

## [0.9.8] — 2026-05-26

### Bug Fixes
- **CLAUDE.md quality bar now appears** (`SessionParser.swift`) — the scorer previously only checked the exact session `cwd` for a CLAUDE.md file. It now walks **up** the directory tree (matching Claude Code's own search strategy) and, if not found, also searches **down** up to 4 levels (skipping `.git`, `node_modules`, `.build`, etc.). This means the bar appears whenever any relevant CLAUDE.md exists in the project hierarchy.
- **Session titles now display correctly** (`SessionParser.swift`) — fixed a bug where the `ai-title` JSONL field was being read as `"title"` but the actual key is `"aiTitle"` (camelCase). Session titles now appear in rows and the detail sheet.
- **Thinking tokens now tracked and displayed** (`SessionParser.swift`, `ActiveSessionCard.swift`) — the Claude Code JSONL doesn't expose a separate thinking token count in the usage object. Tokens are now estimated by reading `thinking` content blocks from each assistant message (`text.count / 4`). The "35% (12.5K) thinking" row in the active session card will show whenever thinking blocks are present in the session.
- **Notifications now fire correctly** (`ClauxApp.swift`, `NotificationManager.swift`) — root cause: `@AppStorage("enableNotifications")` defaults to `true` in the UI, but `UserDefaults.standard.bool(forKey:)` returns `false` when the key was never explicitly written. Fixed by calling `UserDefaults.standard.register(defaults:)` at launch for all settings keys. The permission request is also now slightly delayed (0.5 s) to ensure the bundle is fully loaded.
- **Notification permission handling improved** (`NotificationManager.swift`) — if authorization is `.denied`, the nudge dialog now opens System Settings → Notifications directly. The `nudgeForPermission()` method is now `internal` so it can be called from `sendTestNotification()`.

### New Features
- **Test notification button** (`SettingsView.swift`) — "Test notification" button added in Settings → About. Fires a visible "Claux Notifications Work ✓" banner immediately, letting users verify the notification pipeline end-to-end.

### Improvements
- **Theme picker moved to Settings** (`PopoverView.swift`, `SettingsView.swift`) — the 3-segment theme icon switcher has been removed from the popover header (reducing visual clutter). A standard "Appearance" picker (Light / Dark / Auto) is now in Settings → General, consistent with other pickers.

### Internal
- `ClauxApp.init()` now registers defaults for all 13 settings keys via `UserDefaults.standard.register(defaults:)`.
- `SessionParser.findClaudeMd(startingAt:)` — new private helper that walks up and down the directory tree.
- `SessionParser.skipDirs` — static set of directory names excluded from the downward CLAUDE.md search.

---

## [0.9.7] — 2026-05-26

### New Features
- **CLAUDE.md quality score** (`SessionParser.swift`, `ActiveSessionCard.swift`) — a new row appears below the context bar for any active session whose project folder contains a CLAUDE.md. Scores 0–100 across three dimensions: length (0–30 pts), structure — headers, code blocks, bullet lists (0–30 pts), and content coverage — 8 key topic categories (0–40 pts). Color-coded green ≥ 70 / yellow 40–69 / red < 40. A ⚠ warning icon and suggestion text appear when the score is below the configurable threshold.
- **CLAUDE.md alert threshold** (`SettingsView.swift`) — new setting in Notifications: "CLAUDE.md quality alert" toggle + "Quality threshold" picker (30 / 50 / 70 / 85). The toggle and threshold are stored in `@AppStorage` and read directly by `ActiveSessionCard`.
- **Cost projection period** (`ActiveSessionCard.swift`, `SettingsView.swift`) — the "proj." secondary label in the active session card is replaced by `$xx.xx/d`, `/w`, or `/m` based on the new "Cost projection" picker (Daily / Weekly / Monthly) in Settings → General. Value is computed from `burnRatePerHour × period hours`.

### Improvements
- **Theme segmented picker** (`PopoverView.swift`) — replaced the three individual icon buttons with a native `Picker(...).pickerStyle(.segmented)` with SF Symbol icons in each segment (☀ Light / 🌙 Dark / ◐ Auto). Matches the style of all other pickers in Settings.
- **Theme applies reliably to all open windows** (`Design.swift`, `ClauxApp.swift`) — added `AppThemeModifier` struct and `.appThemed()` extension. Applied to every scene's root view. Because `@AppStorage("appTheme")` is a tracked dependency inside the modifier, already-open Settings and Analytics windows react immediately when the user switches theme — no relaunch required.
- **Thinking token count** (`ActiveSessionCard.swift`) — tokens row now shows both percentage and actual count: `"35% (12.5K) thinking"` instead of just `"35% thinking"`.

### Internal
- `AppStore.resetAllData()` now also clears `appTheme`, `costProjectionPeriod`, `claudemdAlertEnabled`, `claudemdThreshold`.

---

## [0.9.6] — 2026-05-26

### Improvements
- **Analytics button moved next to Settings** (`PopoverView.swift`) — both buttons are now left-aligned in the bottom bar with the same `labelColor`, instead of Analytics being right-aligned in blue.
- **Chart tooltip follows cursor exactly** (`AnalyticsView.swift`) — fixed coordinate mapping: now uses `geo[proxy.plotAreaFrame].minX` to correctly offset the cursor X into plot-relative space before querying the chart proxy. Previously used `geo.frame(in: .local).minX` which is always 0, causing the selected day to lag behind the real cursor position.
- **Chart no longer moves on hover** (`AnalyticsView.swift`) — removed the `.annotation(position: .top)` from `RuleMark`. That annotation forced the chart to allocate height above the bars every time a day was selected, causing a visible jump. The tooltip is now rendered as a floating view inside `chartOverlay` using `.position()`, which sits on top of the chart without affecting its layout. Added `.chartYScale(domain:)` to lock the Y axis range.
- **Live / Idle status centered in header** (`PopoverView.swift`) — the "• Live" / "Idle" indicator now sits at true horizontal center of the popover using a `ZStack`, aligned with the window edges. The title (CLAUX + version) stays left-aligned and the controls stay right-aligned.
- **Theme switcher in header** (`PopoverView.swift`, `ClauxApp.swift`) — three icon buttons (☀ Light / 🌙 Dark / ◐ Auto) appear in the header right of the Live/Idle status. The selected theme is highlighted; "Auto" follows macOS system appearance. Theme is stored in `@AppStorage("appTheme")` and applied via `.preferredColorScheme()` to the popover, Settings window, and Analytics window.

---

All changes to the app are recorded here in reverse-chronological order.
Version format: `MAJOR.MINOR.PATCH`

---

## [0.9.5] — 2026-05-26

### New Features
- **Analytics window** (`AnalyticsView.swift`) — opens via "Analytics" button in popover bottom bar.
  - Daily cost bar chart (last 30 days, switchable to 7 days) with color gradient (blue → orange → red).
  - Per-project horizontal bar chart, sorted by total cost descending (up to 8 projects).
  - Per-model breakdown with model-tinted bars.
- **Session detail sheet** (`SessionDetailSheet.swift`) — tap any recent session row to open a sheet showing full stats: cost, duration, burn rate, projected cost, context fill %, cache hit %, token breakdown (input/output/cache read/cache write/thinking), AI-generated title, entrypoint badge, and copy-path button.
- **Cache efficiency row in active session card** (`ActiveSessionCard.swift`) — shows bolt ⚡ icon + percentage, color-coded green ≥ 60% / yellow 30–59% / red < 30%. Hidden when no cache activity.
- **Menu bar cost display** (`ClauxApp.swift`) — "Show cost in menu bar" toggle now works: displays active session cost (or today's total when idle) next to the icon.
- **Menu bar model badge** (`ClauxApp.swift`) — "Show model badge in menu bar" toggle now works: shows the model name as a tinted capsule when a session is live.

### Improvements
- **Incremental parsing** (`SessionMonitor.swift`) — mtime cache prevents re-parsing unchanged JSONL files on every 10-second tick. Only files with a newer modification date are re-parsed.
- **Richer session data** (`Models.swift`, `SessionParser.swift`) — sessions now carry: `title` (from `ai-title` JSONL entries), `entrypoint` (maps to "VS Code", "Terminal", etc.), `cacheHitRate`, `contextWindowTokens` (last-turn context fill).
- **Session row** (`SessionRowView.swift`) — shows AI-generated title when available; shows entrypoint label in subtitle; chevron hint appears on hover.
- **Model version names** (`Design.swift`) — model badges now show version numbers: "Sonnet 4.6", "Haiku 4.5", "Opus 3", etc. Handles both new (`claude-sonnet-4-6`) and old (`claude-3-5-sonnet-20241022`) naming schemes.
- **Launch at login** (`SettingsView.swift`) — wired to `SMAppService.mainApp.register() / unregister()`. Toggle reads actual system state, not just a UserDefaults bool.
- **Session retention** (`SettingsView.swift`) — changed from stepper to dropdown: 7 / 14 / 30 / 60 / 90 days / 1 year.
- **Cost alert** (`SettingsView.swift`) — changed from free-text field + stepper to dropdown: $0.50 / $1 / $2 / $5 / $10 / $20 / $50.
- **Version constant** (`Design.swift`) — `AppVersion.current` is now the single source of truth used by both the popover header and the Settings → About section.

### Internal
- Deleted `MockData.swift` — was unused, no references anywhere.
- Added `DailySpend`, `ProjectSpend`, `ModelSpend` structs to `Models.swift`.
- `AppStore` now publishes `dailySpend`, `projectBreakdown`, `modelBreakdown` for the analytics window.
- `AppStore.resetAllData()` calls `monitor.invalidateCache()` before refresh so the UI fully reloads.

---

## [0.9.4] — 2026-05-25

### New Features
- **Real session monitoring** — `SessionMonitor` + `SessionParser` read live JSONL files from `~/.claude/projects/`. Active sessions detected via `~/.claude/sessions/<pid>.json` with fallback to file mtime < 90 s.
- **Notifications** (`NotificationManager.swift`) — cost threshold alert, context window warning (configurable %) and critical alert, session-end notification. Guards against non-bundle contexts (raw SPM binary) where `UNUserNotificationCenter` is unavailable.
- **Settings view** (`SettingsView.swift`) — full settings panel: General (launch at login, menu bar toggles, retention, refresh interval), Notifications (master toggle + cost/context/session-end alerts), Data Source (monitored directory picker, cache cost toggle, erase data), Account (sign-in sheet with email, plan, sync toggle), About.
- **Erase All Data** — clears all UserDefaults keys and forces a full session re-scan.
- **Context window monitoring** — `contextHealthFraction` computed from last-turn token counts (not cumulative), displayed as animated fill bar in `ActiveSessionCard`.
- **`.app` bundle packaging** (`build_app.sh`) — wraps SPM binary in a proper `Claux.app` with `Info.plist` (`CFBundleIdentifier: com.claux.app`, `LSUIElement: true`) required for notifications and login items.
- **Menu bar header** — title changed to "CLAUX" with version badge.
- **Analytics window placeholder** — wired to `openWindow(id: "analytics")` (replaced Netlify URL stub).

### Improvements
- `SessionMonitor.watchDirectory` — auto-corrects misconfigured directory (validates `projects/` subdir exists, resets to `~/.claude` if not).
- Per-model pricing baked into `SessionParser.Rates`: Opus, Sonnet, Haiku with cache read/write rates.

---

## [0.9.3] and earlier

Initial prototype — SwiftUI menu bar skeleton with static mock data, placeholder views, and no real session parsing.
