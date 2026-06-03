import Foundation
import Combine

enum ClaudeStatusLineSetupState: Equatable {
    case managedReady
    case managedNeedsRepair
    case notInstalled
    case customCommand(String)
    case invalidSettings
}

struct ClaudeStatusLinePaths {
    let baseURL: URL
    let settingsURL: URL
    let clauxDirectoryURL: URL
    let wrapperURL: URL
    let bridgeURL: URL
}

struct ClaudeStatusLineInspection {
    let state: ClaudeStatusLineSetupState
    let message: String
}

final class ClaudeStatusLineManager: ObservableObject {
    static let shared = ClaudeStatusLineManager()

    @Published private(set) var inspection: ClaudeStatusLineInspection
    @Published private(set) var isInstalling = false
    @Published private(set) var lastOperationMessage: String?

    private var settingsObserver: NSObjectProtocol?

    private init() {
        inspection = Self.inspect(monitoredDirectory: Self.monitoredDirectory)
        settingsObserver = NotificationCenter.default.addObserver(
            forName: UserDefaults.didChangeNotification,
            object: nil,
            queue: .main
        ) { [weak self] _ in
            self?.refresh()
        }
    }

    deinit {
        if let settingsObserver {
            NotificationCenter.default.removeObserver(settingsObserver)
        }
    }

    func refresh() {
        inspection = Self.inspect(monitoredDirectory: Self.monitoredDirectory)
    }

    @discardableResult
    func installOrRepair() -> Bool {
        isInstalling = true
        defer { isInstalling = false }

        do {
            try Self.installManagedWrapper(monitoredDirectory: Self.monitoredDirectory)
            inspection = Self.inspect(monitoredDirectory: Self.monitoredDirectory)
            lastOperationMessage = "Claude integration installed."
            return true
        } catch {
            inspection = Self.inspect(monitoredDirectory: Self.monitoredDirectory)
            lastOperationMessage = error.localizedDescription
            return false
        }
    }

    static func inspect(monitoredDirectory: String) -> ClaudeStatusLineInspection {
        let paths = paths(for: monitoredDirectory)
        let fm = FileManager.default

        guard fm.fileExists(atPath: paths.settingsURL.path) else {
            return ClaudeStatusLineInspection(
                state: .notInstalled,
                message: "Install Claude integration to enable plan-limit collection."
            )
        }

        guard let data = try? Data(contentsOf: paths.settingsURL),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return ClaudeStatusLineInspection(
                state: .invalidSettings,
                message: "Claude settings.json could not be parsed. Repair the file, then reinstall integration."
            )
        }

        guard let statusLine = json["statusLine"] as? [String: Any],
              let command = (statusLine["command"] as? String)?.trimmingCharacters(in: .whitespacesAndNewlines),
              !command.isEmpty else {
            return ClaudeStatusLineInspection(
                state: .notInstalled,
                message: "Claude statusLine is not configured. Install Claux integration."
            )
        }

        if command.contains(paths.wrapperURL.path) {
            let wrapperExists = fm.fileExists(atPath: paths.wrapperURL.path)
            let bridgeExists = fm.fileExists(atPath: paths.bridgeURL.path)
            if wrapperExists && bridgeExists {
                return ClaudeStatusLineInspection(
                    state: .managedReady,
                    message: "Claux manages Claude statusLine and preserves any existing custom command."
                )
            }
            return ClaudeStatusLineInspection(
                state: .managedNeedsRepair,
                message: "Claux integration is partially installed. Repair it to restore plan limits."
            )
        }

        return ClaudeStatusLineInspection(
            state: .customCommand(command),
            message: "Claude is using a custom statusLine command. Claux can wrap it automatically."
        )
    }

    static func paths(for monitoredDirectory: String) -> ClaudeStatusLinePaths {
        let basePath = NSString(string: monitoredDirectory).expandingTildeInPath
        let baseURL = URL(fileURLWithPath: basePath, isDirectory: true)
        let clauxDirectoryURL = baseURL.appendingPathComponent("claux", isDirectory: true)
        return ClaudeStatusLinePaths(
            baseURL: baseURL,
            settingsURL: baseURL.appendingPathComponent("settings.json"),
            clauxDirectoryURL: clauxDirectoryURL,
            wrapperURL: clauxDirectoryURL.appendingPathComponent("statusline-wrapper.py"),
            bridgeURL: clauxDirectoryURL.appendingPathComponent("statusline_bridge.json")
        )
    }

    private static var monitoredDirectory: String {
        UserDefaults.standard.string(forKey: "monitoredDirectory") ?? "~/.claude"
    }

    private static func installManagedWrapper(monitoredDirectory: String) throws {
        let fm = FileManager.default
        let paths = paths(for: monitoredDirectory)

        let bundledWrapper =
            Bundle.module.url(forResource: "statusline_wrapper", withExtension: "py") ??
            Bundle.main.url(forResource: "statusline_wrapper", withExtension: "py")

        guard let bundledWrapper else {
            throw NSError(
                domain: "ClauxIntegration",
                code: 1,
                userInfo: [NSLocalizedDescriptionKey: "Bundled statusline wrapper is missing from the app."]
            )
        }

        try fm.createDirectory(at: paths.clauxDirectoryURL, withIntermediateDirectories: true, attributes: nil)

        let wrapperData = try Data(contentsOf: bundledWrapper)
        try wrapperData.write(to: paths.wrapperURL, options: .atomic)
        try fm.setAttributes([.posixPermissions: 0o755], ofItemAtPath: paths.wrapperURL.path)

        var root: [String: Any] = [:]
        if fm.fileExists(atPath: paths.settingsURL.path) {
            let data = try Data(contentsOf: paths.settingsURL)
            guard let parsed = try JSONSerialization.jsonObject(with: data) as? [String: Any] else {
                throw NSError(
                    domain: "ClauxIntegration",
                    code: 2,
                    userInfo: [NSLocalizedDescriptionKey: "Claude settings.json is not a JSON object."]
                )
            }
            root = parsed
        } else {
            try fm.createDirectory(at: paths.baseURL, withIntermediateDirectories: true, attributes: nil)
        }

        var statusLine = root["statusLine"] as? [String: Any] ?? [:]
        let currentCommand = (statusLine["command"] as? String)?
            .trimmingCharacters(in: .whitespacesAndNewlines)
        let previousBridge = readBridge(at: paths.bridgeURL)

        let downstreamCommand: String?
        if let currentCommand, !currentCommand.isEmpty, !currentCommand.contains(paths.wrapperURL.path) {
            downstreamCommand = currentCommand
        } else {
            downstreamCommand = previousBridge["downstream_command"] as? String
        }

        let managedCommand = shellQuoted(paths.wrapperURL.path)
        statusLine["type"] = "command"
        statusLine["command"] = managedCommand
        statusLine["refreshInterval"] = statusLine["refreshInterval"] as? Int ?? 5
        root["statusLine"] = statusLine

        let bridge: [String: Any?] = [
            "managed_command": managedCommand,
            "downstream_command": downstreamCommand,
            "installed_at": ISO8601DateFormatter().string(from: Date())
        ]

        let bridgeData = try JSONSerialization.data(withJSONObject: bridge.compactMapValues { $0 }, options: [.prettyPrinted, .sortedKeys])
        try bridgeData.write(to: paths.bridgeURL, options: .atomic)

        let settingsData = try JSONSerialization.data(withJSONObject: root, options: [.prettyPrinted, .sortedKeys])
        try settingsData.write(to: paths.settingsURL, options: .atomic)
    }

    private static func readBridge(at url: URL) -> [String: Any] {
        guard let data = try? Data(contentsOf: url),
              let json = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else {
            return [:]
        }
        return json
    }

    private static func shellQuoted(_ text: String) -> String {
        "'" + text.replacingOccurrences(of: "'", with: "'\\''") + "'"
    }
}
