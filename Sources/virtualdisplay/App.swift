import Cocoa
import CoreGraphics
import Foundation
import DisplayDetectionKit

class AppDelegate: NSObject, NSApplicationDelegate {
    private var virtualDisplay: CGVirtualDisplay?
    private var nativeMode: CGDisplayMode?
    private var physicalDisplayID: CGDirectDisplayID = 0
    private var signalSources: [DispatchSourceSignal] = []
    private var isStopping = false
    private var pollTimer: Timer?

    func start() {
        for sig in [SIGTERM, SIGINT] {
            signal(sig, SIG_IGN)

            let src = DispatchSource.makeSignalSource(
                signal: sig,
                queue: .main
            )

            src.setEventHandler { [weak self] in
                self?.stop()
            }

            src.resume()
            signalSources.append(src)
        }

        setupVirtualDisplay()
    }


    private func setupVirtualDisplay() {
        guard let screen = NSScreen.main else { return }
        physicalDisplayID = CGMainDisplayID()
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

        pollTimer = Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { [weak self] _ in
            self?.pollForMirroring()
        }
    }

    private func pollForMirroring() {
        guard let vd = virtualDisplay else { pollTimer?.invalidate(); return }
        var count: CGDisplayCount = 0
        CGGetActiveDisplayList(0, nil, &count)
        var ids = [CGDirectDisplayID](repeating: 0, count: Int(count))
        CGGetActiveDisplayList(count, &ids, &count)
        guard ids.contains(vd.displayID) else { return }
        pollTimer?.invalidate()
        pollTimer = nil
        enableMirroring()
    }

    private func enableMirroring() {
        guard let vd = virtualDisplay else { return }
        nativeMode = CGDisplayCopyDisplayMode(physicalDisplayID)
        if CGDisplayIsInMirrorSet(physicalDisplayID) != 0 {
            var cfg: CGDisplayConfigRef?
            CGBeginDisplayConfiguration(&cfg)
            CGConfigureDisplayMirrorOfDisplay(cfg, physicalDisplayID, kCGNullDirectDisplay)
            CGCompleteDisplayConfiguration(cfg, CGConfigureOption(rawValue: 0))
        }
        var cfg: CGDisplayConfigRef?
        CGBeginDisplayConfiguration(&cfg)
        CGConfigureDisplayMirrorOfDisplay(cfg, physicalDisplayID, vd.displayID)
        CGCompleteDisplayConfiguration(cfg, CGConfigureOption(rawValue: 0))
    }

    private func disableMirroring() {
        guard CGDisplayIsInMirrorSet(physicalDisplayID) != 0 else { return }
        var cfg: CGDisplayConfigRef?
        CGBeginDisplayConfiguration(&cfg)
        CGConfigureDisplayMirrorOfDisplay(cfg, physicalDisplayID, kCGNullDirectDisplay)
        if let mode = nativeMode {
            CGConfigureDisplayWithDisplayMode(cfg, physicalDisplayID, mode, nil)
        }
        let rawOptions: UInt32 = 0
        CGCompleteDisplayConfiguration(cfg, CGConfigureOption(rawValue: rawOptions))
        nativeMode = nil
    }

    @objc func stop() {
        guard !isStopping else { return }
        isStopping = true
        pollTimer?.invalidate()
        pollTimer = nil
        disableMirroring()
        virtualDisplay = nil
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
