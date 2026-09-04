import CoreGraphics

public final class NSDisplayManager {
    public struct DisplayMode {
        public let width: Int
        public let height: Int
        public let pixelWidth: Int
        public let pixelHeight: Int
        public let refreshRate: Double
        public let bitsPerPixel: Int
    }

    public struct Display {
        public let id: CGDirectDisplayID
        public let isMain: Bool
        public let isOnline: Bool
        public let isBuiltin: Bool

        public let mode: DisplayMode
    }

    internal let internals = Internals()

    public init() {}
}
