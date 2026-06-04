import SwiftUI
import AppKit
import Combine

// Shared window-opening bridge used by SwiftUI views and AppKit menu actions.
var clauxOpenWindow: ((String) -> Void)?

@main
struct ClauxApp: App {
    @StateObject private var store = AppStore()
    @NSApplicationDelegateAdaptor(ClauxStatusAppDelegate.self) private var appDelegate

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
            "stateColorPreset":     "system",
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
        let _ = appDelegate.configureIfNeeded(store: store)

        Settings {
            EmptyView()
        }
    }
}

final class ClauxStatusAppDelegate: NSObject, NSApplicationDelegate {
    private var statusController: ClauxStatusItemController?

    func configureIfNeeded(store: AppStore) {
        guard statusController == nil else { return }
        statusController = ClauxStatusItemController(store: store)
    }
}

final class ClauxPanel: NSPanel {
    override var canBecomeKey: Bool { true }
}

final class ClauxStatusItemController: NSObject {
    private let store: AppStore
    private var statusItem: NSStatusItem?
    private var panel: ClauxPanel?
    private var panelDismissMonitor: Any?
    private var cancellables = Set<AnyCancellable>()
    private var settingsWindowController: NSWindowController?
    private var analyticsWindowController: NSWindowController?

    init(store: AppStore) {
        self.store = store
        super.init()
        configurePanel()
        configureObservers()
        updateVisibilityAndAppearance()
        presentOnboardingIfNeeded()
        clauxOpenWindow = { [weak self] id in
            self?.openWindow(id: id)
        }
    }

    deinit {
        if let monitor = panelDismissMonitor {
            NSEvent.removeMonitor(monitor)
        }
    }

    private func configurePanel() {
        let host = NSHostingController(
            rootView: PopoverView()
                .environmentObject(store)
                .appThemed()
        )
        let panel = ClauxPanel(
            contentRect: NSRect(x: 0, y: 0, width: 340, height: 420),
            styleMask: [.nonactivatingPanel, .fullSizeContentView],
            backing: .buffered,
            defer: false
        )
        panel.contentViewController = host
        panel.level = .floating
        panel.isFloatingPanel = true
        panel.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary, .transient]
        panel.hidesOnDeactivate = false
        panel.backgroundColor = .clear
        panel.isOpaque = false
        panel.hasShadow = true
        panel.titleVisibility = .hidden
        panel.titlebarAppearsTransparent = true
        panel.contentView?.wantsLayer = true
        panel.contentView?.layer?.cornerRadius = 11
        panel.contentView?.layer?.masksToBounds = true
        panel.isReleasedWhenClosed = false
        panel.standardWindowButton(.closeButton)?.isHidden = true
        panel.standardWindowButton(.miniaturizeButton)?.isHidden = true
        panel.standardWindowButton(.zoomButton)?.isHidden = true
        panel.setContentSize(NSSize(width: 340, height: 420))
        self.panel = panel

        panelDismissMonitor = NSEvent.addGlobalMonitorForEvents(matching: [.leftMouseDown, .rightMouseDown]) { [weak self] _ in
            self?.dismissPanelIfNeeded()
        }
    }

    private func configureObservers() {
        store.$activeSession
            .receive(on: RunLoop.main)
            .sink { [weak self] _ in
                self?.updateVisibilityAndAppearance()
            }
            .store(in: &cancellables)

        store.$spendSummary
            .receive(on: RunLoop.main)
            .sink { [weak self] _ in
                self?.updateStatusButtonAppearance()
            }
            .store(in: &cancellables)

        NotificationCenter.default.publisher(for: UserDefaults.didChangeNotification)
            .receive(on: RunLoop.main)
            .sink { [weak self] _ in
                self?.updateVisibilityAndAppearance()
            }
            .store(in: &cancellables)

        NotificationCenter.default.publisher(for: .clauxOpenDailyRecap)
            .receive(on: RunLoop.main)
            .sink { [weak self] _ in
                self?.showPanelForNotification()
            }
            .store(in: &cancellables)
    }

    private func shouldShowStatusItem() -> Bool {
        let visibility = UserDefaults.standard.string(forKey: "menuBarVisibility") ?? "always"
        switch visibility {
        case "when_active":
            return store.activeSession != nil
        case "never":
            return false
        default:
            return true
        }
    }

    private func updateVisibilityAndAppearance() {
        if shouldShowStatusItem() {
            ensureStatusItem()
            updateStatusButtonAppearance()
        } else {
            removeStatusItem()
        }
    }

    private func presentOnboardingIfNeeded() {
        let onboardingCompleted = UserDefaults.standard.bool(forKey: "onboardingCompleted")
        guard !onboardingCompleted else { return }
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.35) { [weak self] in
            self?.showPanelForOnboarding()
        }
    }

    private func ensureStatusItem() {
        guard statusItem == nil else { return }
        let item = NSStatusBar.system.statusItem(withLength: NSStatusItem.variableLength)
        guard let button = item.button else { return }
        button.target = self
        button.action = #selector(handleStatusItemClick(_:))
        button.sendAction(on: [.leftMouseUp, .rightMouseUp])
        button.imagePosition = .imageLeft
        statusItem = item
    }

    private func removeStatusItem() {
        closePanel()
        guard let item = statusItem else { return }
        NSStatusBar.system.removeStatusItem(item)
        statusItem = nil
    }

    private func updateStatusButtonAppearance() {
        guard let button = statusItem?.button else { return }

        let isActive = store.activeSession != nil
        let showCost = (UserDefaults.standard.object(forKey: "showCostInMenuBar") as? Bool) ?? false
        let showModel = (UserDefaults.standard.object(forKey: "showModelInMenuBar") as? Bool) ?? false

        let image = NSImage(systemSymbolName: "c.circle.fill", accessibilityDescription: "Claux")
        image?.isTemplate = false
        button.image = image
        button.contentTintColor = isActive ? .systemGreen : .labelColor

        var suffix: [String] = []
        if showCost {
            let cost = store.activeSession?.totalCost ?? store.spendSummary.today
            suffix.append(Format.cost(cost))
        }
        if showModel, let model = store.activeSession?.model {
            suffix.append(ModelInfo.shortName(model))
        }
        button.title = suffix.isEmpty ? "" : " " + suffix.joined(separator: " ")
        button.toolTip = isActive ? "Claux (active session)" : "Claux"
    }

    @objc private func handleStatusItemClick(_ sender: NSStatusBarButton) {
        guard let event = NSApp.currentEvent else {
            togglePanel(from: sender)
            return
        }

        let isRightClick = event.type == .rightMouseUp || event.type == .rightMouseDown
        let isControlClick = (event.type == .leftMouseUp || event.type == .leftMouseDown) &&
            event.modifierFlags.contains(.control)

        if isRightClick || isControlClick {
            showContextMenu(from: sender)
        } else {
            togglePanel(from: sender)
        }
    }

    private func togglePanel(from button: NSStatusBarButton) {
        guard let panel else { return }
        if panel.isVisible {
            closePanel()
        } else {
            alignPanelToMenuBar(from: button)
            NSApp.activate(ignoringOtherApps: true)
            panel.makeKeyAndOrderFront(nil)
        }
    }

    private func closePanel() {
        panel?.orderOut(nil)
    }

    private func showPanelForOnboarding() {
        guard let panel, !panel.isVisible, let button = statusItem?.button else { return }
        alignPanelToMenuBar(from: button)
        NSApp.activate(ignoringOtherApps: true)
        panel.makeKeyAndOrderFront(nil)
    }

    private func showPanelForNotification() {
        guard let panel else { return }
        if let button = statusItem?.button {
            alignPanelToMenuBar(from: button)
        } else {
            panel.center()
        }
        NSApp.activate(ignoringOtherApps: true)
        panel.makeKeyAndOrderFront(nil)
    }

    private func alignPanelToMenuBar(from button: NSStatusBarButton) {
        guard let panel, let buttonWindow = button.window else { return }

        let buttonRectInScreen = buttonWindow.convertToScreen(button.convert(button.bounds, to: nil))
        var frame = panel.frame
        frame.origin.x = buttonRectInScreen.minX
        if let screen = buttonWindow.screen {
            frame.origin.y = screen.visibleFrame.maxY - frame.height
        }
        panel.setFrame(frame, display: true)
    }

    private func dismissPanelIfNeeded() {
        guard let panel, panel.isVisible else { return }
        let mouse = NSEvent.mouseLocation
        if panel.frame.contains(mouse) { return }
        if let statusRect = statusButtonFrameInScreen(), statusRect.contains(mouse) { return }
        closePanel()
    }

    private func statusButtonFrameInScreen() -> NSRect? {
        guard let button = statusItem?.button, let buttonWindow = button.window else { return nil }
        return buttonWindow.convertToScreen(button.convert(button.bounds, to: nil))
    }

    private func showContextMenu(from button: NSStatusBarButton) {
        closePanel()
        let menu = NSMenu()
        menu.autoenablesItems = false

        let settingsItem = NSMenuItem(title: "Settings…", action: #selector(contextOpenSettings), keyEquivalent: "")
        settingsItem.target = self
        menu.addItem(settingsItem)
        menu.addItem(.separator())

        let visibility = UserDefaults.standard.string(forKey: "menuBarVisibility") ?? "always"
        let visibilityItem = NSMenuItem(title: "Show in Menu Bar", action: nil, keyEquivalent: "")
        let visibilityMenu = NSMenu()

        let always = NSMenuItem(title: "Always", action: #selector(contextSetAlways), keyEquivalent: "")
        always.target = self
        always.state = visibility == "always" ? .on : .off
        visibilityMenu.addItem(always)

        let whenActive = NSMenuItem(title: "When session is active", action: #selector(contextSetWhenActive), keyEquivalent: "")
        whenActive.target = self
        whenActive.state = visibility == "when_active" ? .on : .off
        visibilityMenu.addItem(whenActive)

        visibilityItem.submenu = visibilityMenu
        menu.addItem(visibilityItem)
        menu.addItem(.separator())

        let quitItem = NSMenuItem(title: "Quit Claux", action: #selector(contextQuit), keyEquivalent: "")
        quitItem.target = self
        menu.addItem(quitItem)

        menu.popUp(positioning: nil, at: NSPoint(x: -1, y: button.bounds.maxY + 4), in: button)
    }

    @objc private func contextOpenSettings() {
        openWindow(id: "settings")
    }

    @objc private func contextSetAlways() {
        UserDefaults.standard.set("always", forKey: "menuBarVisibility")
    }

    @objc private func contextSetWhenActive() {
        UserDefaults.standard.set("when_active", forKey: "menuBarVisibility")
    }

    @objc private func contextQuit() {
        NSApp.terminate(nil)
    }

    private func openWindow(id: String) {
        switch id {
        case "settings":
            showSettingsWindow()
        case "analytics":
            showAnalyticsWindow()
        default:
            break
        }
    }

    private func showSettingsWindow() {
        let controller = settingsWindowController ?? makeSettingsWindowController()
        settingsWindowController = controller
        NSApp.activate(ignoringOtherApps: true)
        controller.showWindow(nil)
        controller.window?.makeKeyAndOrderFront(nil)
    }

    private func showAnalyticsWindow() {
        let controller = analyticsWindowController ?? makeAnalyticsWindowController()
        analyticsWindowController = controller
        NSApp.activate(ignoringOtherApps: true)
        controller.showWindow(nil)
        controller.window?.makeKeyAndOrderFront(nil)
    }

    private func makeSettingsWindowController() -> NSWindowController {
        let root = SettingsView()
            .environmentObject(store)
            .appThemed()
        let host = NSHostingController(rootView: root)
        let window = NSWindow(contentViewController: host)
        window.title = "Settings"
        window.styleMask = [.titled, .closable, .miniaturizable]
        window.setContentSize(NSSize(width: 560, height: 620))
        window.center()
        window.isReleasedWhenClosed = false
        return NSWindowController(window: window)
    }

    private func makeAnalyticsWindowController() -> NSWindowController {
        let root = AnalyticsView()
            .environmentObject(store)
            .appThemed()
        let host = NSHostingController(rootView: root)
        let window = NSWindow(contentViewController: host)
        window.title = "Analytics"
        window.styleMask = [.titled, .closable, .miniaturizable, .resizable]
        window.setContentSize(NSSize(width: 820, height: 640))
        window.center()
        window.isReleasedWhenClosed = false
        return NSWindowController(window: window)
    }
}
