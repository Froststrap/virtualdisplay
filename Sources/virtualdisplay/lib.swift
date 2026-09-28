import AppKit
import Foundation

@_cdecl("sw_delegate_new")
public func delegateNew() -> UnsafeMutableRawPointer {
    Unmanaged.passRetained(AppDelegate()).toOpaque()
}

@_cdecl("sw_delegate_start")
public func delegateStart(_ p: UnsafeMutableRawPointer) {
    Unmanaged<AppDelegate>.fromOpaque(p).takeUnretainedValue().start()
}

@_cdecl("sw_delegate_stop")
public func delegateStop(_ p: UnsafeMutableRawPointer) {
    Unmanaged<AppDelegate>.fromOpaque(p).takeUnretainedValue().stop()
}

@_cdecl("sw_delegate_free")
public func delegateFree(_ p: UnsafeMutableRawPointer) {
    Unmanaged<AppDelegate>.fromOpaque(p).release()
}
