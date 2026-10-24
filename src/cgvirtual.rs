// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

use std::ffi::CString;

use block2::RcBlock;
use dispatch2::DispatchQueue;
use objc2::{
    class, msg_send,
    rc::{Allocated, Retained},
    runtime::{AnyClass, AnyObject, Bool},
};
use objc2_core_foundation::CGSize;

fn class(name: &std::ffi::CStr) -> &'static AnyClass {
    AnyClass::get(name).unwrap_or_else(|| panic!("private class {name:?} not found"))
}

const VENDOR_ID: u32 = 0x4653;
const PRODUCT_ID: u32 = 0x5A50;

pub struct VirtualDisplay(Retained<AnyObject>);

pub enum CreateError {
    Rejected,
}

pub struct Config<'a> {
    pub name: &'a str,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub size_mm: CGSize,
    pub hidpi: bool,
    pub width: usize,
    pub height: usize,
    pub refresh_rates: &'a [f64],
    pub serial: u32,
}

fn next_serial(serial: u32) -> u32 {
    serial.wrapping_add(1).max(1)
}

impl VirtualDisplay {
    pub fn create(cfg: &Config) -> Result<Self, CreateError> {
        // GUARD:
        // macOS remembers a mode per identity and may hand the display
        // back in a poisoned state, so one salt retry with serial+1
        // before giving up (see ErrWrongMode in go-macos/virtualdisplay).
        for serial in [cfg.serial, next_serial(cfg.serial)] {
            if let Some(vd) = Self::try_create(cfg, serial) {
                log::debug!(
                    "created with identity vendor={VENDOR_ID:#06x} \
                     product={PRODUCT_ID:#06x} serial={serial:#010x}"
                );
                return Ok(Self(vd));
            }
            log::warn!("identity serial={serial:#010x} rejected");
        }
        Err(CreateError::Rejected)
    }

    fn try_create(cfg: &Config, serial: u32) -> Option<Retained<AnyObject>> {
        unsafe {
            let desc: Retained<AnyObject> = msg_send![class(c"CGVirtualDisplayDescriptor"), new];
            let queue = DispatchQueue::main();
            let queue_ptr = &*queue as *const DispatchQueue as *mut AnyObject;
            let _: () = msg_send![&*desc, setQueue: queue_ptr];
            let name = CString::new(cfg.name).unwrap();
            let ns_name: Retained<AnyObject> =
                msg_send![class!(NSString), stringWithUTF8String: name.as_ptr()];

            let _: () = msg_send![&*desc, setName: &*ns_name];
            let _: () = msg_send![&*desc, setMaxPixelsWide: cfg.pixel_width];
            let _: () = msg_send![&*desc, setMaxPixelsHigh: cfg.pixel_height];
            let _: () = msg_send![&*desc, setSizeInMillimeters: cfg.size_mm];
            let _: () = msg_send![&*desc, setProductID: PRODUCT_ID];
            let _: () = msg_send![&*desc, setVendorID: VENDOR_ID];
            let _: () = msg_send![&*desc, setSerialNum: serial];

            let handler = RcBlock::new(|_: *mut AnyObject, _: *mut AnyObject| {});
            let _: () = msg_send![&*desc, setTerminationHandler: &*handler];

            let alloc: Allocated<AnyObject> = msg_send![class(c"CGVirtualDisplay"), alloc];
            let vd: Option<Retained<AnyObject>> = msg_send![alloc, initWithDescriptor: &*desc];
            let vd = vd?;

            let modes: Vec<Retained<AnyObject>> = cfg
                .refresh_rates
                .iter()
                .map(|&hz| {
                    let a: Allocated<AnyObject> =
                        msg_send![class(c"CGVirtualDisplayMode"), alloc];
                    msg_send![
                        a,
                        initWithWidth: cfg.width as u32,
                        height: cfg.height as u32,
                        refreshRate: hz
                    ]
                })
                .collect();

            let settings: Retained<AnyObject> = msg_send![class(c"CGVirtualDisplaySettings"), new];
            let _: () = msg_send![&*settings, setHiDPI: u32::from(cfg.hidpi)];

            let ptrs: Vec<*const AnyObject> = modes
                .iter()
                .map(|m| Retained::as_ptr(m) as *const AnyObject)
                .collect();

            let arr: Retained<AnyObject> =
                msg_send![class!(NSArray), arrayWithObjects: ptrs.as_ptr(), count: ptrs.len()];

            let _: () = msg_send![&*settings, setModes: &*arr];

            let ok: Bool = msg_send![&*vd, applySettings: &*settings];
            ok.as_bool().then_some(vd)
        }
    }

    pub fn id(&self) -> u32 {
        unsafe { msg_send![&*self.0, displayID] }
    }
}
