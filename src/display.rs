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
use std::{sync::RwLock, cell::RefCell, ffi::c_void, ptr};

/// publically editable meta
/// by the user on `init()`
#[derive(Clone)]
pub struct DisplayMeta {
    pub name: Option<String>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub rates: Option<Vec<f64>>,
}

struct Display {
    vd: VirtualDisplay,
    mirror: Mirror,
    mirrored: bool,
}

static META_LOCK: RwLock<Option<DisplayMeta>> = RwLock::new(None);

thread_local! {
    static DISPLAY: RefCell<Option<Display>> = const { RefCell::new(None) };
}

pub fn init(s: DisplayMeta) -> i32 {
    *META_LOCK.write().expect("aquire lock") = Some(s);
    0
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

pub fn running() -> i32 {
    DISPLAY.with_borrow_mut(|slot| {
        match slot.is_some() {
            false => 0,
            true => 1,
        }
    })
}

pub fn stop() -> i32 {
    let d = DISPLAY.with_borrow_mut(|slot| slot.take());
    drop(d);
    0
}

fn build_config(meta: &DisplayMeta) -> Option<Config<'_>> {
    let main = CGMainDisplayID();
    let mode = CGDisplayCopyDisplayMode(main)?;
    let mode_pw = CGDisplayMode::pixel_width(Some(&mode));

    let mode_w = CGDisplayMode::width(Some(&mode));
    let mode_h = CGDisplayMode::height(Some(&mode));

    let is_native = meta.width.is_none() && meta.height.is_none();
    eprintln!("is_native={is_native}");

    // GUARD:
    // Width OR Height manual management
    // means you handle scale manually also via resolution
    // so scale is going to be 1.
    let scale = is_native.then(|| (mode_pw / mode_w.max(1)).max(1)).unwrap_or(1);

    let width = meta.width.unwrap_or(mode_w);
    let height = meta.height.unwrap_or(mode_h);
    eprintln!("width={width}");
    eprintln!("height={height}");

    // GUARD:
    // CG Will probably segfault getting this
    // as a device.
    if width == 0 || height == 0 {
        return None;
    };
    
    let pixel_width = width.checked_mul(scale)?;
    let pixel_height = height.checked_mul(scale)?;
    eprintln!("pixel_width={pixel_width}");
    eprintln!("pixel_height={pixel_height}");

    let mm = CGDisplayScreenSize(main);
    let size_mm = if mm.width > 0.0 && mm.height > 0.0 {
        mm
    } else {
        CGSize::new(600.0, 340.0)
    };

    let refresh_rates = 
        meta.rates.as_deref().unwrap_or(&[60.0, 120.0, 240.0]);

    // GUARD:
    // No really retarded numbers like big precsion
    // or dumb ones.
    for rate in refresh_rates {
        let scaled = rate * 100.0;
        // GUARD:
        // You get the gist.
        if !rate.is_finite()
            || *rate <= 0.0
            || (scaled - scaled.round()).abs() > 1e-9
        {
            return None;
        }
    }

    // GUARD:
    // Fail if no refresh rates were found in the Vec
    if refresh_rates.len() < 1 {
        return None;
    }

    Some(Config {
        name: meta.name.as_deref().unwrap_or("Froststrap Virtual Display"),
        pixel_width: u32::try_from(pixel_width).ok()?,
        pixel_height: u32::try_from(pixel_height).ok()?,
        size_mm,
        hidpi: is_native && scale > 1,
        width,
        height,
        refresh_rates,
    })
}

impl Display {
    fn start() -> Option<Self> {
        let meta = META_LOCK.read().expect("aquire lock").as_ref().cloned().expect("call init() to set data details to use");
        let cfg = build_config(&meta)?;
        
        let vd = VirtualDisplay::create(&cfg)?;
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

extern "C-unwind" fn on_reconfig(
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
