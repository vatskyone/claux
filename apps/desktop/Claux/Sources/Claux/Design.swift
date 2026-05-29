import SwiftUI
import AppKit

// MARK: – App version (single source of truth)
// Update this every time a file is modified or created, then add an entry to CHANGELOG.md.
enum AppVersion {
    static let current = "1.6.1"
}

// MARK: – Semantic system colours
// NSColor-backed → auto-adapts to light / dark mode.
// Accent is pinned to systemBlue to match macOS menu chrome (Wi-Fi toggle, etc.).
extension Color {

    // Backgrounds
    static let clauxBackground = Color(nsColor: .windowBackgroundColor)
    static let clauxSurface    = Color(nsColor: .controlBackgroundColor)

    // Borders / separators
    static let clauxBorder     = Color(nsColor: .separatorColor)

    // Text hierarchy
    static let clauxPrimary    = Color(nsColor: .labelColor)
    static let clauxSecondary  = Color(nsColor: .secondaryLabelColor)
    static let clauxTertiary   = Color(nsColor: .tertiaryLabelColor)

    // Accent — system blue, matching macOS interactive chrome (Wi-Fi toggle, etc.)
    static let clauxAccent     = Color(nsColor: .systemBlue)
    // Legacy alias kept for source compatibility
    static var clauxGold: Color { .clauxAccent }

    // Semantic status colours
    static let clauxGreen  = Color.green
    static let clauxYellow = Color.yellow
    static let clauxRed    = Color(nsColor: .systemRed)

    // Model badges
    static let clauxOpusColor   = Color(nsColor: .systemPurple)
    static let clauxSonnetColor = Color(nsColor: .systemBlue)
    static let clauxHaikuColor  = Color(nsColor: .systemGreen)
}

// MARK: – Context-health colour helper
// Healthy = blue (matches macOS progress/toggle blue), warning = yellow, critical = red
extension Color {
    static func contextHealthColor(_ fraction: Double) -> Color {
        if fraction < 0.70 { return .clauxAccent }
        if fraction < 0.90 { return .clauxYellow }
        return .clauxRed
    }
}

// MARK: – Model helpers
enum ModelInfo {
    /// Returns a display name with the version number extracted from the model ID.
    ///
    /// Handles both naming schemes Claude has used:
    ///   New: `claude-sonnet-4-6`          → "Sonnet 4.6"
    ///   New: `claude-haiku-4-5-20251001`   → "Haiku 4.5"
    ///   Old: `claude-3-5-sonnet-20241022`  → "Sonnet 3.5"
    ///   Old: `claude-3-opus-20240229`      → "Opus 3"
    static func shortName(_ model: String) -> String {
        let lower = model.lowercased()
        let parts = lower.components(separatedBy: "-")

        // Identify family
        let family: String
        if      lower.contains("opus")   { family = "Opus"   }
        else if lower.contains("sonnet") { family = "Sonnet" }
        else if lower.contains("haiku")  { family = "Haiku"  }
        else { return model }

        guard let familyIdx = parts.firstIndex(of: family.lowercased()) else {
            return family
        }

        // Version numbers are 1-2 digit integers (dates like 20241022 are > 99).
        func versionInts(_ slice: ArraySlice<String>) -> [Int] {
            slice.compactMap { Int($0) }.filter { $0 > 0 && $0 <= 99 }
        }

        // New format: family comes before the version  →  claude-sonnet-4-6
        let afterFamily = versionInts(parts.dropFirst(familyIdx + 1).prefix(3))
        if !afterFamily.isEmpty {
            return afterFamily.count >= 2
                ? "\(family) \(afterFamily[0]).\(afterFamily[1])"
                : "\(family) \(afterFamily[0])"
        }

        // Old format: version comes before the family  →  claude-3-5-sonnet-20241022
        let beforeFamily = versionInts(parts[1..<familyIdx])
        if !beforeFamily.isEmpty {
            return beforeFamily.count >= 2
                ? "\(family) \(beforeFamily[0]).\(beforeFamily[1])"
                : "\(family) \(beforeFamily[0])"
        }

        return family
    }

    static func color(_ model: String) -> Color {
        if model.lowercased().contains("opus")   { return .clauxOpusColor }
        if model.lowercased().contains("sonnet") { return .clauxSonnetColor }
        if model.lowercased().contains("haiku")  { return .clauxHaikuColor }
        return .clauxSecondary
    }
}

// MARK: – Formatting helpers
enum Format {
    static func cost(_ value: Double) -> String {
        String(format: "$%.2f", value)
    }

    static func tokens(_ count: Int) -> String {
        if count >= 1_000_000 { return String(format: "%.1fM", Double(count) / 1_000_000) }
        if count >= 1_000     { return String(format: "%.1fK", Double(count) / 1_000) }
        return "\(count)"
    }

    static func duration(_ seconds: TimeInterval) -> String {
        let h = Int(seconds) / 3600
        let m = (Int(seconds) % 3600) / 60
        if h > 0 { return "\(h)h \(String(format: "%02d", m))m" }
        return "\(m)m"
    }

    static func relativeTime(_ date: Date) -> String {
        let s = -date.timeIntervalSinceNow
        if s < 3600   { return "\(Int(s / 60))m ago" }
        if s < 86400  { return "\(Int(s / 3600))h ago" }
        return "\(Int(s / 86400))d ago"
    }

    static func projectPath(_ raw: String) -> String {
        let parts = raw.components(separatedBy: "/")
        if parts.count >= 3, parts[1] == "Users" {
            return "~/" + parts.dropFirst(3).joined(separator: "/")
        }
        return raw
    }
}

// MARK: – App-wide theme modifier
// Apply .appThemed() to every scene's root view.
// Two-pronged approach:
//   1. .preferredColorScheme() — keeps SwiftUI views in sync
//   2. NSApp.appearance — applies to ALL NSWindow/NSPanel instances,
//      including the MenuBarExtra NSPanel which ignores preferredColorScheme.
// Any open window with .appThemed() triggers the global change immediately.
struct AppThemeModifier: ViewModifier {
    @AppStorage("appTheme") private var appTheme: String = "auto"

    private var colorScheme: ColorScheme? {
        switch appTheme {
        case "light": return .light
        case "dark":  return .dark
        default:      return nil   // follows macOS system appearance
        }
    }

    private func applyNSAppearance() {
        switch appTheme {
        case "light": NSApp.appearance = NSAppearance(named: .aqua)
        case "dark":  NSApp.appearance = NSAppearance(named: .darkAqua)
        default:      NSApp.appearance = nil   // follow macOS system appearance
        }
    }

    func body(content: Content) -> some View {
        content
            .preferredColorScheme(colorScheme)
            .onAppear    { applyNSAppearance() }
            .onChange(of: appTheme) { _ in applyNSAppearance() }
    }
}

extension View {
    /// Force the app-wide theme preference on this view hierarchy.
    func appThemed() -> some View { modifier(AppThemeModifier()) }
}

// MARK: – Native blur background (NSVisualEffectView)
// Wraps NSVisualEffectView so SwiftUI views get the system vibrancy/blur.
// Use .behindWindow to blur the desktop / other apps behind the panel.
struct VisualEffectView: NSViewRepresentable {
    var material:     NSVisualEffectView.Material
    var blendingMode: NSVisualEffectView.BlendingMode

    init(material:     NSVisualEffectView.Material     = .menu,
         blendingMode: NSVisualEffectView.BlendingMode = .behindWindow) {
        self.material     = material
        self.blendingMode = blendingMode
    }

    func makeNSView(context: Context) -> NSVisualEffectView {
        let v = NSVisualEffectView()
        v.material     = material
        v.blendingMode = blendingMode
        v.state        = .active
        return v
    }
    func updateNSView(_ v: NSVisualEffectView, context: Context) {
        v.material     = material
        v.blendingMode = blendingMode
    }
}

// Makes the host NSPanel / NSWindow transparent so the blur shows through.
// Uses viewDidMoveToWindow for reliable detection (the window isn't available
// inside makeNSView).
final class _TransparentWindowSetupView: NSView {
    override func viewDidMoveToWindow() {
        super.viewDidMoveToWindow()
        guard let win = window else { return }
        DispatchQueue.main.async {
            win.backgroundColor = .clear
            win.isOpaque        = false
        }
    }
}

struct WindowBlurInstaller: NSViewRepresentable {
    func makeNSView(context: Context) -> _TransparentWindowSetupView { _TransparentWindowSetupView() }
    func updateNSView(_ v: _TransparentWindowSetupView, context: Context) {}
}

extension View {
    /// Apply native macOS vibrancy blur to this view's host window.
    /// - `material`: controls the tint/density of the blur (default `.menu` for panels, `.sidebar` for windows).
    func nativeBlurBackground(material: NSVisualEffectView.Material = .menu) -> some View {
        self
            .background(VisualEffectView(material: material, blendingMode: .behindWindow))
            .overlay(WindowBlurInstaller().frame(width: 0, height: 0), alignment: .topLeading)
    }
}

// MARK: – Card style
// On a blurred background, cards use .regularMaterial (.withinWindow blending) so they
// appear as a slightly frosted panel floating above the main blur — the standard macOS
// layered-glass look seen in Spotlight, Control Centre, etc.
struct CardStyle: ViewModifier {
    var borderColor: Color = .clauxBorder
    func body(content: Content) -> some View {
        content
            .background(.regularMaterial)
            .clipShape(RoundedRectangle(cornerRadius: 8))
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .stroke(borderColor.opacity(0.35), lineWidth: 0.5)
            )
    }
}

extension View {
    func cardStyle(borderColor: Color = .clauxBorder) -> some View {
        modifier(CardStyle(borderColor: borderColor))
    }
}
