import Foundation
import Combine

/// Watches a local rate-limit snapshot file produced by Claude Code statusLine.
///
/// Expected file path:
///   <monitoredDirectory>/claux/rate_limits.json
/// Example default: ~/.claude/claux/rate_limits.json
final class RateLimitMonitor: ObservableObject {

    @Published private(set) var snapshot: PlanLimitsSnapshot = .empty
    @Published private(set) var diagnostics: PlanLimitsDiagnostics = .defaultState

    // MARK: – Private state
    private let queue = DispatchQueue(label: "com.claux.ratelimits", qos: .utility)
    private var pollTimer: Timer?
    private var directorySource: DispatchSourceFileSystemObject?
    private var settingsObserver: NSObjectProtocol?
    private var lastMTime: Date?

    init() {
        startMonitoring()

        settingsObserver = NotificationCenter.default.addObserver(
            forName: UserDefaults.didChangeNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.restartMonitoring()
        }
    }

    deinit {
        stopMonitoring()
        if let obs = settingsObserver {
            NotificationCenter.default.removeObserver(obs)
        }
    }

    func refresh() {
        queue.async { [weak self] in
            self?.loadSnapshotIfNeeded(force: true)
        }
    }

    // MARK: – Monitoring lifecycle

    private var refreshIntervalSeconds: TimeInterval {
        let configured = UserDefaults.standard.integer(forKey: "autoRefreshInterval")
        let clamped = max(5, min(configured > 0 ? configured : 10, 300))
        return TimeInterval(clamped)
    }

    private func startMonitoring() {
        queue.async { [weak self] in
            self?.loadSnapshotIfNeeded(force: true)
            self?.installDirectoryWatcher()
        }

        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.pollTimer = Timer.scheduledTimer(withTimeInterval: self.refreshIntervalSeconds, repeats: true) { [weak self] _ in
                self?.queue.async {
                    self?.loadSnapshotIfNeeded(force: false)
                }
            }
        }
    }

    private func stopMonitoring() {
        pollTimer?.invalidate()
        pollTimer = nil

        directorySource?.cancel()
        directorySource = nil
    }

    private func restartMonitoring() {
        lastMTime = nil
        stopMonitoring()
        startMonitoring()
    }

    // MARK: – Paths

    private var monitoredDirectory: String {
        UserDefaults.standard.string(forKey: "monitoredDirectory") ?? "~/.claude"
    }

    private var rateLimitsURL: URL {
        let base = NSString(string: monitoredDirectory).expandingTildeInPath
        return URL(fileURLWithPath: base)
            .appendingPathComponent("claux")
            .appendingPathComponent("rate_limits.json")
    }

    private var debugLogURL: URL {
        let base = NSString(string: monitoredDirectory).expandingTildeInPath
        return URL(fileURLWithPath: base)
            .appendingPathComponent("claux")
            .appendingPathComponent("statusline_debug.log")
    }

    // MARK: – Watcher

    private func installDirectoryWatcher() {
        let dirURL = rateLimitsURL.deletingLastPathComponent()
        let fd = open(dirURL.path, O_EVTONLY)
        guard fd >= 0 else { return }

        let src = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .rename, .delete, .attrib],
            queue: queue
        )
        src.setEventHandler { [weak self] in
            self?.loadSnapshotIfNeeded(force: false)
        }
        src.setCancelHandler {
            close(fd)
        }
        src.resume()
        directorySource = src
    }

    // MARK: – Loader + parser

    private func loadSnapshotIfNeeded(force: Bool) {
        let fm = FileManager.default
        let url = rateLimitsURL

        guard fm.fileExists(atPath: url.path) else {
            lastMTime = nil
            publish(snapshot: .empty, diagnostics: diagnosticsWithoutSnapshot())
            return
        }

        let mtime = (try? url.resourceValues(forKeys: [.contentModificationDateKey]))?.contentModificationDate
        if !force, let mtime, let lastMTime, mtime == lastMTime {
            return
        }

        guard let data = try? Data(contentsOf: url),
              let obj = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            publish(snapshot: .empty, diagnostics: diagnosticsWithoutSnapshot())
            return
        }

        let parsed = parseSnapshot(obj)
        lastMTime = mtime
        publish(snapshot: parsed, diagnostics: diagnosticsForSnapshot(parsed))
    }

    private func publish(snapshot: PlanLimitsSnapshot, diagnostics: PlanLimitsDiagnostics) {
        DispatchQueue.main.async { [weak self] in
            self?.snapshot = snapshot
            self?.diagnostics = diagnostics
        }
    }

    private func diagnosticsForSnapshot(_ snapshot: PlanLimitsSnapshot) -> PlanLimitsDiagnostics {
        if snapshot.hasAnyData {
            if snapshot.isStale {
                return PlanLimitsDiagnostics(
                    state: .staleData,
                    message: "Data is stale. Send a new Claude message to refresh limits."
                )
            }
            return PlanLimitsDiagnostics(
                state: .ready,
                message: "Plan limits active."
            )
        }
        return diagnosticsWithoutSnapshot()
    }

    private func diagnosticsWithoutSnapshot() -> PlanLimitsDiagnostics {
        guard let last = readLastDebugEntry() else {
            return PlanLimitsDiagnostics(
                state: .statusLineNotRunning,
                message: "No statusLine activity detected. Restart Claude and check statusLine trust."
            )
        }

        if last.isRecent(seconds: 120), last.kind == .ok, last.hasRateLimits == false {
            return PlanLimitsDiagnostics(
                state: .limitsUnavailableForSession,
                message: "This session has no subscription plan-limit payload (rate_limits absent)."
            )
        }

        if last.isRecent(seconds: 120), last.kind == .emptyStdin {
            return PlanLimitsDiagnostics(
                state: .waitingForFirstResponse,
                message: "Waiting for first API response in the current session…"
            )
        }

        if last.isRecent(seconds: 120), last.kind == .ok, last.hasRateLimits == true {
            return PlanLimitsDiagnostics(
                state: .waitingForFirstResponse,
                message: "StatusLine is active. Waiting for plan-limit snapshot write."
            )
        }

        return PlanLimitsDiagnostics(
            state: .statusLineNotRunning,
            message: "StatusLine appears inactive in this runtime."
        )
    }

    private func readLastDebugEntry() -> DebugEntry? {
        guard let raw = try? String(contentsOf: debugLogURL, encoding: .utf8) else {
            return nil
        }
        guard let line = raw.split(separator: "\n").last.map(String.init),
              !line.trimmingCharacters(in: .whitespaces).isEmpty else {
            return nil
        }
        return DebugEntry.parse(line: line)
    }

    private func parseSnapshot(_ obj: [String: Any]) -> PlanLimitsSnapshot {
        let payload: [String: Any]
        if let nested = obj["rate_limits"] as? [String: Any] {
            payload = nested
        } else {
            payload = obj
        }

        let fiveHour = parseWindow(payload["five_hour"])
        let sevenDay = parseWindow(payload["seven_day"])

        let updatedAt = parseDate(obj["updated_at"]) ?? parseDate(payload["updated_at"])

        return PlanLimitsSnapshot(
            fiveHour: fiveHour,
            sevenDay: sevenDay,
            updatedAt: updatedAt
        )
    }

    private func parseWindow(_ raw: Any?) -> PlanLimitWindow {
        guard let dict = raw as? [String: Any] else {
            return PlanLimitWindow()
        }

        let usedPercentage = parseDouble(dict["used_percentage"])
        let resetsAt = parseDate(dict["resets_at"])

        return PlanLimitWindow(
            usedPercentage: usedPercentage,
            resetsAt: resetsAt
        )
    }

    private func parseDouble(_ raw: Any?) -> Double? {
        switch raw {
        case let value as Double:
            return value
        case let value as Int:
            return Double(value)
        case let value as NSNumber:
            return value.doubleValue
        case let value as String:
            return Double(value)
        default:
            return nil
        }
    }

    private func parseDate(_ raw: Any?) -> Date? {
        switch raw {
        case let unix as Int:
            return Date(timeIntervalSince1970: TimeInterval(unix))
        case let unix as Double:
            return Date(timeIntervalSince1970: unix)
        case let unix as NSNumber:
            return Date(timeIntervalSince1970: unix.doubleValue)
        case let text as String:
            if let unix = Double(text) {
                return Date(timeIntervalSince1970: unix)
            }
            return parseISO(text)
        default:
            return nil
        }
    }
}

private struct DebugEntry {
    enum Kind {
        case ok
        case emptyStdin
        case invalidJSON
        case other
    }

    let timestamp: Date?
    let kind: Kind
    let hasRateLimits: Bool?

    func isRecent(seconds: TimeInterval) -> Bool {
        guard let timestamp else { return false }
        return Date().timeIntervalSince(timestamp) <= seconds
    }

    static func parse(line: String) -> DebugEntry {
        let parts = line.split(separator: " ", maxSplits: 1, omittingEmptySubsequences: true)
        let ts = parts.first.map(String.init).flatMap(parseDebugTimestamp)
        let body = parts.count > 1 ? String(parts[1]) : line

        if body.contains("empty-stdin") {
            return DebugEntry(timestamp: ts, kind: .emptyStdin, hasRateLimits: nil)
        }
        if body.contains("invalid-json") {
            return DebugEntry(timestamp: ts, kind: .invalidJSON, hasRateLimits: nil)
        }
        if body.contains("ok ") || body.hasPrefix("ok") {
            if body.contains("has_rate_limits=True") {
                return DebugEntry(timestamp: ts, kind: .ok, hasRateLimits: true)
            }
            if body.contains("has_rate_limits=False") {
                return DebugEntry(timestamp: ts, kind: .ok, hasRateLimits: false)
            }
            return DebugEntry(timestamp: ts, kind: .ok, hasRateLimits: nil)
        }
        return DebugEntry(timestamp: ts, kind: .other, hasRateLimits: nil)
    }
}

private let debugTsFormatter: DateFormatter = {
    let f = DateFormatter()
    f.locale = Locale(identifier: "en_US_POSIX")
    f.timeZone = .current
    f.dateFormat = "yyyy-MM-dd'T'HH:mm:ss"
    return f
}()

private func parseDebugTimestamp(_ value: String) -> Date? {
    debugTsFormatter.date(from: value)
}

// ISO parser for rate-limit timestamps.
private let rateLimitISOFull: ISO8601DateFormatter = {
    let f = ISO8601DateFormatter()
    f.formatOptions = [.withInternetDateTime, .withFractionalSeconds]
    return f
}()

private let rateLimitISOBasic = ISO8601DateFormatter()

private func parseISO(_ value: String) -> Date? {
    rateLimitISOFull.date(from: value) ?? rateLimitISOBasic.date(from: value)
}
