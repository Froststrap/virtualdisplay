// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

use crate::{
    cgvirtual::{Config, VirtualDisplay},
    mirror::Mirror,
};
use dispatch2::DispatchQueue;
use objc2_core_foundation::CGSize;
use objc2_core_graphics::*;
use std::{cell::RefCell, ffi::c_void, ptr};

struct Display {
    vd: VirtualDisplay,
    mirror: Mirror,
    mirrored: bool,
}

thread_local! {
    static DISPLAY: RefCell<Option<Display>> = const { RefCell::new(None) };
}

pub fn start() -> i32 {
    DISPLAY.with_borrow_mut(|slot| {
        if slot.is_some() {
            return 0;
        }
        match Display::start() {
            Some(mut d) => {
                d.try_mirror();
                *slot = Some(d);
                0
            }
            None => -1,
        }
    })
}

pub fn stop() -> i32 {
    let d = DISPLAY.with_borrow_mut(|slot| slot.take());
    drop(d);
    0
}

fn build_config() -> Option<Config> {
    let main = CGMainDisplayID();
    let mode = CGDisplayCopyDisplayMode(main)?;

    let pw = CGDisplayMode::pixel_width(Some(&mode));
    let ph = CGDisplayMode::pixel_height(Some(&mode));
    let w = CGDisplayMode::width(Some(&mode));

    let scale = (pw / w.max(1)).max(1);

    let mm = CGDisplayScreenSize(main);
    let size_mm = if mm.width > 0.0 && mm.height > 0.0 {
        mm
    } else {
        CGSize::new(600.0, 340.0)
    };

    Some(Config {
        name: "Virtual 240Hz",
        pixel_width: pw as u32,
        pixel_height: ph as u32,
        size_mm,
        hidpi: scale > 1,
        width: pw / scale,
        height: ph / scale,
        refresh_rates: &[240.0, 60.0],
    })
}

impl Display {
    fn start() -> Option<Self> {
        let vd = VirtualDisplay::create(&build_config()?)?;
        unsafe {
            CGDisplayRegisterReconfigurationCallback(Some(on_reconfig), ptr::null_mut());
        }
        Some(Self {
            vd,
            mirror: Mirror::new(CGMainDisplayID()),
            mirrored: false,
        })
    }

    fn try_mirror(&mut self) {
        let id = self.vd.id();
        if !self.mirrored && CGDisplayIsActive(id) {
            self.mirror.enable(id);
            self.mirrored = true;
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            CGDisplayRemoveReconfigurationCallback(Some(on_reconfig), ptr::null_mut());
        }
        self.mirror.disable();
    }
}

unsafe extern "C-unwind" fn on_reconfig(
    _id: CGDirectDisplayID,
    _flags: CGDisplayChangeSummaryFlags,
    _ctx: *mut c_void,
) {
    DispatchQueue::main().exec_async(|| {
        DISPLAY.with_borrow_mut(|d| {
            if let Some(d) = d {
                d.try_mirror();
            }
        });
    });
}
