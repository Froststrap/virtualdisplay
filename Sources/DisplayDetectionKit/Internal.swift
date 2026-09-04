import CoreGraphics

extension NSDisplayManager {
	internal final class Internals {
		public func getDisplays() -> [Display] {
		    var displayCount: UInt32 = 0

		    guard CGGetOnlineDisplayList(0, nil, &displayCount) == .success else {
		        return []
		    }

		    var displayIDs = Array(
		        repeating: CGDirectDisplayID(0),
		        count: Int(displayCount)
		    )

		    guard CGGetOnlineDisplayList(
		        displayCount,
		        &displayIDs,
		        &displayCount
		    ) == .success else {
		        return []
		    }

		    return displayIDs.compactMap {
		        getDisplayInfo($0)
		    }
		}

		public func getDisplayInfo(
		    _ displayID: CGDirectDisplayID
		) -> Display? {
		    let bounds = CGDisplayBounds(displayID)
		    let pixelWidth = CGDisplayPixelsWide(displayID)
		    let pixelHeight = CGDisplayPixelsHigh(displayID)
		    let displayMode = CGDisplayCopyDisplayMode(displayID)
		    let refreshRate = displayMode?.refreshRate ?? 0
		    let bitsPerPixel = 32

		    return Display(
		        id: displayID,
		        isMain: CGMainDisplayID() == displayID,
		        isOnline: CGDisplayIsOnline(displayID) != 0,
		        isBuiltin: CGDisplayIsBuiltin(displayID) != 0,

		        mode: DisplayMode(
		            width: Int(bounds.width),
		            height: Int(bounds.height),
		            pixelWidth: pixelWidth,
		            pixelHeight: pixelHeight,
		            refreshRate: refreshRate,
		            bitsPerPixel: bitsPerPixel,
		        ),
		    )
		}
	}
}
