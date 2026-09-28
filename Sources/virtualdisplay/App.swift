import Cocoa
import CoreGraphics
import Foundation
import DisplayDetectionKit

final class AppDelegate {
    public var virtualDisplay: CGVirtualDisplay?

    func start() {
        setupVirtualDisplay()
    }

    func stop() {
        virtualDisplay = nil
    }

    private func setupVirtualDisplay() {
        guard let screen = NSScreen.main else { return }
        let displayManager = NSDisplayManager()

        let scale = Int(screen.backingScaleFactor)
        let physical = displayManager.getMode(displayManager.getPrimaryDisplay())
        let width = physical.width / scale
        let height = physical.height / scale

        var descriptor = CGVirtualDisplayDescriptor()
        descriptor.setDispatchQueue(.main)
        descriptor.name = "Virtual 240Hz"
        descriptor.maxPixelsWide = UInt32(physical.width)
        descriptor.maxPixelsHigh = UInt32(physical.height)
        descriptor.sizeInMillimeters = screen.physicalSizeInMillimeters
        descriptor.productID = 0x1234
        descriptor.vendorID = 0x3456
        descriptor.serialNum = 0x0002
        descriptor.terminationHandler = { [weak self] _, _ in self?.virtualDisplay = nil }

        let display = CGVirtualDisplay(descriptor: descriptor)
        guard display.handle != nil else { return }
        var settings = CGVirtualDisplaySettings()
        settings.hiDPI = scale > 1 ? 1 : 0
        settings.modes = [
            CGVirtualDisplayMode(width: UInt(width), height: UInt(height), refreshRate: 240),
            CGVirtualDisplayMode(width: UInt(width), height: UInt(height), refreshRate: 60),
        ]
    
        _ = display.applySettings(settings)
        virtualDisplay = display
    }
}

private extension NSScreen {
    var physicalSizeInMillimeters: CGSize {
        guard let dpi = deviceDescription[NSDeviceDescriptionKey("NSDeviceResolution")] as? NSValue else {
            return CGSize(width: 600, height: 340)
        }
        let d = dpi.sizeValue
        guard d.width > 0, d.height > 0 else { return CGSize(width: 600, height: 340) }
        return CGSize(width: frame.width / d.width * 25.4, height: frame.height / d.height * 25.4)
    }
}
