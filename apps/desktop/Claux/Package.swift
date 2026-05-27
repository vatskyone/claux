// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Claux",
    platforms: [.macOS(.v13)],
    targets: [
        .executableTarget(
            name: "Claux",
            path: "Sources/Claux"
        )
    ]
)
