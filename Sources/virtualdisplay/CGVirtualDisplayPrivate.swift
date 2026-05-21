import CoreGraphics
import Foundation

struct CGVirtualDisplayMode {
    internal let handle: AnyObject
    
    let refreshRate: CGFloat
    let width: UInt
    let height: UInt
    
    init(width: UInt, height: UInt, refreshRate: CGFloat) {
        guard let realClass: AnyObject = NSClassFromString("CGVirtualDisplayMode") else {
            fatalError("CoreGraphics Private Class 'CGVirtualDisplayMode' not found.")
        }
        
        let allocSelector = NSSelectorFromString("alloc")
        guard let allocated = realClass.perform(allocSelector)?.takeUnretainedValue() else {
            fatalError("Failed to allocate real CGVirtualDisplayMode")
        }
        
        let initSelector = NSSelectorFromString("initWithWidth:height:refreshRate:")
        typealias ObjCInit = @convention(c) (AnyObject, Selector, UInt, UInt, CGFloat) -> AnyObject
        let impl = unsafeBitCast(allocated.method(for: initSelector), to: ObjCInit.self)
        
        self.handle = impl(allocated, initSelector, width, height, refreshRate)
        self.width = width
        self.height = height
        self.refreshRate = refreshRate
    }
}

struct CGVirtualDisplaySettings {
    internal let handle: AnyObject
    
    var modes: [CGVirtualDisplayMode] = [] {
        didSet {
            let objCArray = modes.map { $0.handle }
            handle.setValue(objCArray, forKey: "modes")
        }
    }
    
    var hiDPI: UInt32 = 0 {
        didSet {
            handle.setValue(hiDPI, forKey: "hiDPI")
        }
    }
    
    init() {
        guard let realClass = NSClassFromString("CGVirtualDisplaySettings") as? NSObject.Type else {
            fatalError("CoreGraphics Private Class 'CGVirtualDisplaySettings' not found.")
        }
        self.handle = realClass.init()
    }
}

struct CGVirtualDisplayDescriptor {
    internal let handle: AnyObject
    
    var queue: DispatchQueue? {
        didSet { handle.setValue(queue, forKey: "queue") }
    }
    var name: String? {
        didSet { handle.setValue(name, forKey: "name") }
    }
    var colorSpace: CGColorSpace? {
        didSet { handle.setValue(colorSpace, forKey: "colorSpace") }
    }
    
    var maxPixelsHigh: UInt32 = 0 {
        didSet { handle.setValue(maxPixelsHigh, forKey: "maxPixelsHigh") }
    }
    var maxPixelsWide: UInt32 = 0 {
        didSet { handle.setValue(maxPixelsWide, forKey: "maxPixelsWide") }
    }
    var sizeInMillimeters: CGSize = .zero {
        didSet { handle.setValue(NSValue(size: sizeInMillimeters), forKey: "sizeInMillimeters") }
    }
    var serialNum: UInt32 = 0 {
        didSet { handle.setValue(serialNum, forKey: "serialNum") }
    }
    var productID: UInt32 = 0 {
        didSet { handle.setValue(productID, forKey: "productID") }
    }
    var vendorID: UInt32 = 0 {
        didSet { handle.setValue(vendorID, forKey: "vendorID") }
    }
    var terminationHandler: ((AnyObject, AnyObject) -> Void)? {
        didSet { handle.setValue(terminationHandler, forKey: "terminationHandler") }
    }
    
    init() {
        guard let realClass = NSClassFromString("CGVirtualDisplayDescriptor") as? NSObject.Type else {
            fatalError("CoreGraphics Private Class 'CGVirtualDisplayDescriptor' not found.")
        }
        self.handle = realClass.init()
    }
    
    func setDispatchQueue(_ q: DispatchQueue) {
        handle.setValue(q, forKey: "queue")
    }
}

struct CGVirtualDisplay {
    internal let handle: AnyObject
    
    var displayID: CGDirectDisplayID {
        return handle.value(forKey: "displayID") as? CGDirectDisplayID ?? 0
    }
    
    init(descriptor: CGVirtualDisplayDescriptor) {
        guard let realClass: AnyObject = NSClassFromString("CGVirtualDisplay") else {
            fatalError("CoreGraphics Private Class 'CGVirtualDisplay' not found.")
        }
        
        let allocSelector = NSSelectorFromString("alloc")
        guard let allocated = realClass.perform(allocSelector)?.takeUnretainedValue() else {
            fatalError("Failed to allocate real CGVirtualDisplay")
        }
        
        let initSelector = NSSelectorFromString("initWithDescriptor:")
        typealias ObjCInit = @convention(c) (AnyObject, Selector, AnyObject) -> AnyObject
        let impl = unsafeBitCast(allocated.method(for: initSelector), to: ObjCInit.self)
        
        self.handle = impl(allocated, initSelector, descriptor.handle)
    }
    
    func applySettings(_ settings: CGVirtualDisplaySettings) -> Bool {
        let selector = NSSelectorFromString("applySettings:")
        typealias ObjCMethod = @convention(c) (AnyObject, Selector, AnyObject) -> Bool
        let impl = unsafeBitCast(handle.method(for: selector), to: ObjCMethod.self)
        return impl(handle, selector, settings.handle)
    }
}
