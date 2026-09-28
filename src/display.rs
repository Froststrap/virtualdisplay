// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

use crate::{ffi, mirror::Mirror};
use dispatch2::DispatchQueue;
use objc2_core_graphics::*;
use std::{
    cell::RefCell,
    ffi::c_void,
    ptr::{self, NonNull},
};

struct Display {
    handle: NonNull<c_void>,
    virtual_id: CGDirectDisplayID,
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

pub fn stop() {
    let d = DISPLAY.with_borrow_mut(|slot| slot.take());
    drop(d);
}

impl Display {
    fn start() -> Option<Self> {
        unsafe {
            let handle = NonNull::new(ffi::sw_delegate_new())?;
            ffi::sw_delegate_start(handle.as_ptr());
            let virtual_id = ffi::sw_delegate_virtual_id(handle.as_ptr());
            if virtual_id == 0 {
                ffi::sw_delegate_stop(handle.as_ptr());
                ffi::sw_delegate_free(handle.as_ptr());
                return None;
            }
            CGDisplayRegisterReconfigurationCallback(Some(on_reconfig), ptr::null_mut());
            Some(Self {
                handle,
                virtual_id,
                mirror: Mirror::new(CGMainDisplayID()),
                mirrored: false,
            })
        }
    }

    fn try_mirror(&mut self) {
        if !self.mirrored && CGDisplayIsActive(self.virtual_id) {
            self.mirror.enable(self.virtual_id);
            self.mirrored = true;
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            CGDisplayRemoveReconfigurationCallback(Some(on_reconfig), ptr::null_mut());
            self.mirror.disable();
            ffi::sw_delegate_stop(self.handle.as_ptr());
            ffi::sw_delegate_free(self.handle.as_ptr());
        }
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
