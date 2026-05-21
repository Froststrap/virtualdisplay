// swift-tools-version: 5.8
import PackageDescription

let package = Package(
    name: "virtualdisplay",
    platforms: [.macOS(.v11)],
    targets: [
        .executableTarget(
            name: "virtualdisplay",
            path: "Sources/virtualdisplay"
        )
    ]
)
