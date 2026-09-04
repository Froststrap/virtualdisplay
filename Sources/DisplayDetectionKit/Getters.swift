import CoreGraphics

extension NSDisplayManager {
	public func getPrimaryDisplay() -> CGDirectDisplayID {
	    var display: CGDirectDisplayID? = nil

	    for dsp in internals.getDisplays() {
	        if dsp.isBuiltin {
	            display = dsp.id
	        }
	    }

	    return display.unsafelyUnwrapped
	}

	public func getMode(_ display: CGDirectDisplayID) -> DisplayMode {
	    internals.getDisplayInfo(display).unsafelyUnwrapped.mode
	}
}
