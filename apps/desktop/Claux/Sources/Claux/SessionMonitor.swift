import Foundation
import Combine

/// Watches `~/.claude/projects/` (or the directory chosen in Settings) and
/// keeps a live list of `ClaudeSession` objects up to date.
///
/// Changes are published on the **main queue**.
/// Uses an mtime cache so only modified JSONL files are re-parsed on each tick.
final class SessionMonitor: ObservableObject {

    @Published private(set) var sessions: [ClaudeSession] = []

    // MARK: – Private state
    private var dispatchSources: [DispatchSourceFileSystemObject] = []
    private var pollTimer: Timer?
    private let queue = DispatchQueue(label: "com.claux.monitor", qos: .utility)
    private var settingsObserver: NSObjectProtocol?

    /// mtime-keyed cache: only re-parse a file when its modification date changes.
    private var parseCache: [URL: (mtime: Date, session: ClaudeSession)] = [:]

    // MARK: – Init / deinit

    init() {
        startMonitoring()

        // Re-scan whenever any UserDefault changes (covers the monitored-dir preference).
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

    // MARK: – Public API

    /// Trigger an immediate re-scan (called from AppStore after resetting data).
    func refresh() {
        queue.async { self.loadSessions() }
    }

    /// Wipe the parse cache — forces a full re-parse on the next refresh.
    func invalidateCache() {
        queue.async { self.parseCache.removeAll() }
    }

    // MARK: – Lifecycle

    private var refreshIntervalSeconds: TimeInterval {
        let configured = UserDefaults.standard.integer(forKey: "autoRefreshInterval")
        let clamped = max(1, min(configured > 0 ? configured : 10, 300))
        return TimeInterval(clamped)
    }

    private func startMonitoring() {
        refresh()
        installWatchers()

        // Fallback poll (configurable) — catches JSONL updates inside project subdirs
        // that DispatchSource on the parent directory may miss.
        DispatchQueue.main.async { [weak self] in
            guard let self else { return }
            self.pollTimer = Timer.scheduledTimer(withTimeInterval: self.refreshIntervalSeconds, repeats: true) { [weak self] _ in
                self?.refresh()
            }
        }
    }

    private func stopMonitoring() {
        pollTimer?.invalidate()
        pollTimer = nil
        dispatchSources.forEach { $0.cancel() }
        dispatchSources.removeAll()
    }

    private func restartMonitoring() {
        parseCache.removeAll()
        stopMonitoring()
        startMonitoring()
    }

    // MARK: – File-system watchers

    private var projectsURL: URL {
        let base = NSString(string: watchDirectory).expandingTildeInPath
        return URL(fileURLWithPath: base).appendingPathComponent("projects")
    }

    private var sessionsURL: URL {
        let base = NSString(string: watchDirectory).expandingTildeInPath
        return URL(fileURLWithPath: base).appendingPathComponent("sessions")
    }

    private var watchDirectory: String {
        let stored = UserDefaults.standard.string(forKey: "monitoredDirectory") ?? "~/.claude"
        let expanded = NSString(string: stored).expandingTildeInPath
        let projectsPath = (expanded as NSString).appendingPathComponent("projects")
        if !FileManager.default.fileExists(atPath: projectsPath) {
            let fallback = "~/.claude"
            UserDefaults.standard.set(fallback, forKey: "monitoredDirectory")
            return fallback
        }
        return stored
    }

    private func installWatchers() {
        watchURL(projectsURL)

        let fm = FileManager.default
        guard let subdirs = try? fm.contentsOfDirectory(
            at: projectsURL, includingPropertiesForKeys: [.isDirectoryKey]
        ) else { return }

        for sub in subdirs {
            var isDir: ObjCBool = false
            if fm.fileExists(atPath: sub.path, isDirectory: &isDir), isDir.boolValue {
                watchURL(sub)
            }
        }
    }

    private func watchURL(_ url: URL) {
        let fd = open(url.path, O_EVTONLY)
        guard fd >= 0 else { return }

        let src = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: fd,
            eventMask: [.write, .rename, .delete, .attrib],
            queue: queue
        )
        src.setEventHandler  { [weak self] in self?.loadSessions() }
        src.setCancelHandler { close(fd) }
        src.resume()
        dispatchSources.append(src)
    }

    // MARK: – Session loading

    private func loadSessions() {
        let fm = FileManager.default

        // 1. Active session IDs from ~/.claude/sessions/*.json
        var activeIds = Set<String>()
        if let files = try? fm.contentsOfDirectory(at: sessionsURL, includingPropertiesForKeys: nil) {
            for f in files where f.pathExtension == "json" {
                guard let data = try? Data(contentsOf: f),
                      let obj  = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
                      let sid  = obj["sessionId"] as? String
                else { continue }
                activeIds.insert(sid)
            }
        }

        // 2. Enumerate project directories
        guard let projectDirs = try? fm.contentsOfDirectory(
            at: projectsURL, includingPropertiesForKeys: [.isDirectoryKey]
        ) else { return }

        var result: [ClaudeSession] = []
        var seenURLs = Set<URL>()

        for projectDir in projectDirs {
            var isDir: ObjCBool = false
            guard fm.fileExists(atPath: projectDir.path, isDirectory: &isDir),
                  isDir.boolValue else { continue }

            guard let jsonlFiles = try? fm.contentsOfDirectory(
                at: projectDir,
                includingPropertiesForKeys: [.contentModificationDateKey]
            ) else { continue }

            for jsonlURL in jsonlFiles where jsonlURL.pathExtension == "jsonl" {
                seenURLs.insert(jsonlURL)

                // Fetch modification date
                let mtime = (try? jsonlURL.resourceValues(forKeys: [.contentModificationDateKey]))?
                    .contentModificationDate

                // Cache hit: same mtime → reuse, just refresh the isActive flag
                if let mtime,
                   let cached = parseCache[jsonlURL],
                   cached.mtime == mtime {
                    var session = cached.session
                    let wasActive = session.isActive
                    session.isActive = activeIds.contains(session.id.uuidString)
                    if !session.isActive { session.endTime = session.endTime ?? cached.session.endTime }
                    // If liveness changed, force a re-parse to pick up new turns
                    if session.isActive == wasActive {
                        result.append(session)
                        continue
                    }
                }

                // Cache miss: parse the file
                if let session = SessionParser.parse(url: jsonlURL, activeSessionIds: activeIds) {
                    if let mtime {
                        parseCache[jsonlURL] = (mtime: mtime, session: session)
                    }
                    result.append(session)
                }
            }
        }

        // Evict stale cache entries for files that no longer exist
        parseCache = parseCache.filter { seenURLs.contains($0.key) }

        // 3. Sort: active first, then most-recent first
        result.sort {
            if $0.isActive != $1.isActive { return $0.isActive }
            return $0.startTime > $1.startTime
        }

        DispatchQueue.main.async { [weak self] in
            self?.sessions = result
        }
    }
}
