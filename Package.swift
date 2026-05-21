// swift-tools-version: 5.8
import PackageDescription

let package = Package(
    name: "virtualdisplay",
    platforms: [.macOS(.v11)],
    targets: [
        .target(
            name: "CGVirtualDisplayPrivate",
            path: "Sources/CGVirtualDisplayPrivate",
            publicHeadersPath: ".",
        ),
        .executableTarget(
            name: "virtualdisplay",
            dependencies: ["CGVirtualDisplayPrivate"],
            path: "Sources/virtualdisplay"
        )
    ]
)
