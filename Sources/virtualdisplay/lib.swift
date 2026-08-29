import AppKit

@_cdecl("start_display")
public func startDisplay(
    _ customWidth: Int32,
    _ customHeight: Int32,
    _ showMenu: Bool,
) -> Int32 { 
    let app = NSApplication.shared
    let delegate = AppDelegate(
        showMenu: showMenu,
        width: Int(customWidth),
        height: Int(customHeight)
    )
    app.delegate = delegate
    app.run()

    return 0
}
