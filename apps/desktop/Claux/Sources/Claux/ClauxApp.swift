import SwiftUI
import AppKit

// Captured from SwiftUI's @Environment(\.openWindow) when MenuBarLabel first appears.
// Used by the AppKit right-click handler to open SwiftUI windows.
var clauxOpenWindow: ((String) -> Void)?

@main
struct ClauxApp: App {
    @StateObject private var store = AppStore()

    init() {
        if UserDefaults.standard.string(forKey: "menuBarVisibility") == "never" {
            UserDefaults.standard.set("always", forKey: "menuBarVisibility")
        }

        UserDefaults.standard.register(defaults: [
            "enableNotifications":  true,
            "costAlertThreshold":   5.0,
            "contextHealthAlert":   80.0,
            "alertOnSessionEnd":    false,
            "claudemdAlertEnabled": true,
            "claudemdThreshold":    50,
            "showCostInMenuBar":    false,
            "showModelInMenuBar":   false,
            "sessionRetentionDays": 30,
            "autoRefreshInterval":  10,
            "costProjectionPeriod": "monthly",
            "appTheme":             "auto",
            "includeCacheCost":     true,
            "monthlyBudget":        0.0,
            "dailySummaryEnabled":  false,
            "dailySummaryHour":     18,
            "menuBarVisibility":    "always",
        ])

        DispatchQueue.main.asyncAfter(deadline: .now() + 1.0) {
            NotificationManager.shared.requestPermission()
        }
    }

    var body: some Scene {
        MenuBarExtra {
            PopoverView()
                .environmentObject(store)
                .appThemed()
        } label: {
            MenuBarLabel(store: store)
        }
        .menuBarExtraStyle(.window)

        WindowGroup("Settings", id: "settings") {
            SettingsView()
                .environmentObject(store)
                .appThemed()
        }
        .windowResizability(.contentSize)
        .defaultPosition(.center)

        WindowGroup("Analytics", id: "analytics") {
            AnalyticsView()
                .environmentObject(store)
                .appThemed()
        }
        .windowResizability(.contentSize)
        .defaultPosition(.center)
    }
}

// MARK: – Menu bar icon label

struct MenuBarLabel: View {
    @ObservedObject var store: AppStore
    @State private var pulse = false

    @AppStorage("showCostInMenuBar")  private var showCost:  Bool = false
    @AppStorage("showModelInMenuBar") private var showModel: Bool = false

    @Environment(\.openWindow) private var openWindow

    private var isActive: Bool { store.activeSession != nil }

    private var displayCost: Double {
        if let s = store.activeSession { return s.totalCost }
        return store.spendSummary.today
    }

    private var displayModel: String? {
        guard showModel, let s = store.activeSession else { return nil }
        return ModelInfo.shortName(s.model)
    }

    var body: some View {
        HStack(spacing: 4) {
            ZStack {
                if isActive {
                    Circle()
                        .fill(Color(nsColor: .systemGreen).opacity(0.4))
                        .frame(width: 20, height: 20)
                        .scaleEffect(pulse ? 2.0 : 1.0)
                        .opacity(pulse ? 0 : 0.7)
                        .animation(
                            .easeOut(duration: 1.6).repeatForever(autoreverses: false),
                            value: pulse
                        )
                }
                Image(systemName: "c.circle.fill")
                    .resizable()
                    .frame(width: 16, height: 16)
                    .foregroundStyle(
                        isActive
                            ? Color(nsColor: .systemGreen)
                            : Color(nsColor: .controlTextColor)
                    )
            }

            if showCost {
                Text(Format.cost(displayCost))
                    .font(.system(size: 12, weight: .medium, design: .rounded))
                    .monospacedDigit()
                    .foregroundStyle(Color(nsColor: .controlTextColor))
            }

            if let model = displayModel {
                Text(model)
                    .font(.system(size: 10, weight: .semibold))
                    .foregroundStyle(ModelInfo.color(store.activeSession?.model ?? ""))
                    .padding(.horizontal, 4)
                    .padding(.vertical, 1)
                    .background(
                        ModelInfo.color(store.activeSession?.model ?? "").opacity(0.15)
                            .clipShape(Capsule())
                    )
            }
        }
        .onAppear {
            pulse = true
            clauxOpenWindow = { id in openWindow(id: id) }
        }
        // Zero-size overlay embedded in the SwiftUI label tree.
        // Once it's added to the window, it walks up via superview to find
        // NSStatusBarButton and installs the transparent right-click catcher.
        .overlay(RightClickInstaller().frame(width: 0, height: 0))
    }
}

// MARK: – Zero-size installer: walks superview chain → NSStatusBarButton → intercepts clicks

private struct RightClickInstaller: NSViewRepresentable {
    func makeNSView(context: Context) -> InstallerView { InstallerView() }
    func updateNSView(_ nsView: InstallerView, context: Context) {}
}

final class InstallerView: NSView {
    private var didInstall = false
    // Retain the handler so it isn't deallocated.
    private var buttonHandler: StatusButtonHandler?

    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        guard !didInstall, window != nil else { return }
        // Defer one run-loop tick so the full hierarchy is settled.
        DispatchQueue.main.async { [weak self] in self?.install() }
    }

    private func install() {
        guard !didInstall else { return }
        var view: NSView? = self
        while let current = view {
            if let button = current as? NSStatusBarButton {
                buttonHandler = StatusButtonHandler(button: button)
                didInstall = true
                return
            }
            view = current.superview
        }
    }
}

// Captures NSStatusBarButton's original action/target (SwiftUI's popover toggle),
// then routes left-clicks back to it and right-clicks to the context menu.
final class StatusButtonHandler: NSObject {
    private weak var button: NSStatusBarButton?
    private let originalAction: Selector?
    private weak var originalTarget: AnyObject?

    init(button: NSStatusBarButton) {
        self.button         = button
        self.originalAction = button.action
        self.originalTarget = button.target as AnyObject?
        super.init()
        button.target = self
        button.action = #selector(handleClick(_:))
        // Fire the action on both mouse-up events so we can distinguish them.
        button.sendAction(on: [.leftMouseUp, .rightMouseUp])
    }

    @objc private func handleClick(_ sender: NSStatusBarButton) {
        guard let event = NSApp.currentEvent else { return }
        if event.type == .rightMouseUp {
            // Build and pop the context menu anchored to the button.
            let menu = MenuBarContextMenu.build()
            NSMenu.popUpContextMenu(menu, with: event, for: sender)
        } else {
            // Forward left-click to SwiftUI's handler → toggles the popover.
            if let action = originalAction {
                NSApp.sendAction(action, to: originalTarget, from: sender)
            }
        }
    }
}

// MARK: – Context menu

final class MenuBarContextMenu: NSObject {
    static let shared = MenuBarContextMenu()

    /// Build the menu (items target MenuBarContextMenu.shared).
    static func build() -> NSMenu { shared.buildMenu() }

    private func buildMenu() -> NSMenu {
        let visibility = UserDefaults.standard.string(forKey: "menuBarVisibility") ?? "always"

        let menu = NSMenu()
        menu.autoenablesItems = false

        menu.addItem(make("Open Claux Dashboard",  #selector(openDashboard)))
        menu.addItem(.separator())
        menu.addItem(make("Settings…",             #selector(openSettings)))
        menu.addItem(.separator())
        menu.addItem(make("Check for Updates…",    #selector(checkForUpdates)))
        menu.addItem(.separator())
        menu.addItem(visibilitySubmenu(current: visibility))
        menu.addItem(.separator())
        menu.addItem(make("Quit Claux",            #selector(quitApp)))

        return menu
    }

    private func make(_ title: String, _ sel: Selector) -> NSMenuItem {
        let i = NSMenuItem(title: title, action: sel, keyEquivalent: "")
        i.target = self; i.isEnabled = true
        return i
    }

    private func visibilitySubmenu(current: String) -> NSMenuItem {
        let sub  = NSMenu()
        let opts: [(String, String, Selector)] = [
            ("Always",                      "always",      #selector(setAlways)),
            ("When Claude Code is running", "when_active", #selector(setWhenActive)),
            ("Never",                       "never",       #selector(setNever)),
        ]
        for (title, tag, sel) in opts {
            let mi = NSMenuItem(title: title, action: sel, keyEquivalent: "")
            mi.target = self; mi.isEnabled = true
            mi.state  = current == tag ? .on : .off
            sub.addItem(mi)
        }
        let parent = NSMenuItem(title: "Show in Menu Bar", action: nil, keyEquivalent: "")
        parent.submenu = sub
        return parent
    }

    @objc private func openDashboard()  { NSWorkspace.shared.open(URL(string: "https://claux.app/dashboard")!) }
    @objc private func openSettings()   { NSApp.activate(ignoringOtherApps: true); clauxOpenWindow?("settings") }
    @objc private func checkForUpdates() {
        let a = NSAlert()
        a.messageText     = "Claux is up to date"
        a.informativeText = "You're running version \(AppVersion.current), which is the latest."
        a.alertStyle      = .informational
        a.addButton(withTitle: "OK")
        NSApp.activate(ignoringOtherApps: true)
        a.runModal()
    }
    @objc private func setAlways()     { save("always") }
    @objc private func setWhenActive() { save("when_active") }
    @objc private func setNever()      { save("never") }
    @objc private func quitApp()       { NSApp.terminate(nil) }

    private func save(_ value: String) { UserDefaults.standard.set(value, forKey: "menuBarVisibility") }
}
