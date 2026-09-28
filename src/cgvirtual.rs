// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

use block2::RcBlock;
use dispatch2::DispatchQueue;
use objc2::{
    msg_send,
    rc::{Allocated, Retained},
    runtime::{AnyClass, AnyObject, Bool},
};
use objc2_core_foundation::CGSize;
use objc2_foundation::{NSArray, NSString};

fn class(name: &std::ffi::CStr) -> &'static AnyClass {
    AnyClass::get(name).unwrap_or_else(|| panic!("private class {name:?} not found"))
}

pub struct VirtualDisplay(Retained<AnyObject>);

pub struct Config {
    pub name: &'static str,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub size_mm: CGSize,
    pub hidpi: bool,
    pub width: usize,
    pub height: usize,
    pub refresh_rates: &'static [f64],
}

impl VirtualDisplay {
    pub fn create(cfg: &Config) -> Option<Self> {
        unsafe {
            let desc: Retained<AnyObject> = msg_send![class(c"CGVirtualDisplayDescriptor"), new];
            let queue = DispatchQueue::main();
            let queue_ptr = &*queue as *const DispatchQueue as *mut AnyObject;
            let _: () = msg_send![&*desc, setQueue: queue_ptr];
            let _: () = msg_send![&*desc, setName: &*NSString::from_str(cfg.name)];
            let _: () = msg_send![&*desc, setMaxPixelsWide: cfg.pixel_width];
            let _: () = msg_send![&*desc, setMaxPixelsHigh: cfg.pixel_height];
            let _: () = msg_send![&*desc, setSizeInMillimeters: cfg.size_mm];
            let _: () = msg_send![&*desc, setProductID: 0x1234_u32];
            let _: () = msg_send![&*desc, setVendorID: 0x3456_u32];
            let _: () = msg_send![&*desc, setSerialNum: 0x0002_u32];

            let handler = RcBlock::new(|_: *mut AnyObject, _: *mut AnyObject| {});
            let _: () = msg_send![&*desc, setTerminationHandler: &*handler];

            let alloc: Allocated<AnyObject> = msg_send![class(c"CGVirtualDisplay"), alloc];
            let vd: Option<Retained<AnyObject>> = msg_send![alloc, initWithDescriptor: &*desc];
            let vd = vd?;

            let modes: Vec<Retained<AnyObject>> = cfg
                .refresh_rates
                .iter()
                .map(|&hz| {
                    let a: Allocated<AnyObject> = msg_send![class(c"CGVirtualDisplayMode"), alloc];
                    msg_send![a, initWithWidth: cfg.width, height: cfg.height, refreshRate: hz]
                })
                .collect();

            let settings: Retained<AnyObject> = msg_send![class(c"CGVirtualDisplaySettings"), new];
            let _: () = msg_send![&*settings, setHiDPI: u32::from(cfg.hidpi)];
            let _: () = msg_send![&*settings, setModes: &*NSArray::from_retained_slice(&modes)];

            let ok: Bool = msg_send![&*vd, applySettings: &*settings];
            ok.as_bool().then_some(Self(vd))
        }
    }

    pub fn id(&self) -> u32 {
        unsafe { msg_send![&*self.0, displayID] }
    }
}
