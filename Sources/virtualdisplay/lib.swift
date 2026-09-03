import AppKit
import Foundation

private var appDelegate: AppDelegate?

@_cdecl("start_display")
public func startDisplay(
    _ customWidth: Int32,
    _ customHeight: Int32,
    _ showMenu: Bool
) -> Int32 {

    let delegate = AppDelegate(
        showMenu: showMenu,
        width: Int(customWidth),
        height: Int(customHeight)
    )

    appDelegate = delegate

    delegate.start()

    return 0
}

@_cdecl("end_display")
public func endDisplay() -> Int32 {
    if Thread.isMainThread {
        appDelegate?.stop()
        appDelegate = nil
    } else {
        let semaphore = DispatchSemaphore(value: 0)
        DispatchQueue.main.async {
            appDelegate?.stop()
            appDelegate = nil
            semaphore.signal()
        }
        semaphore.wait()
    }
    return 0
}
