// swift-tools-version: 5.8
import PackageDescription

let package = Package(
    name: "virtualdisplay",
    platforms: [.macOS(.v11)],
    products: [
        .library(name: "swvirtualdisplay", type: .static, targets: ["virtualdisplay"])
    ],
    targets: [
        .target(
            name: "DisplayDetectionKit"
        ),
        .target(
            name: "virtualdisplay",
            dependencies: [
                "DisplayDetectionKit",
            ]
        ),
    ]
)
