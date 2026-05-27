# CLAUX Changelog

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
