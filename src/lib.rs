// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

mod ffi;
use dispatch2::DispatchQueue;
use std::{cell::RefCell, ffi::c_void, ptr::NonNull};

struct Display(NonNull<c_void>);

impl Display {
    fn start() -> Option<Self> {
        unsafe {
            let p = NonNull::new(ffi::sw_delegate_new())?;
            ffi::sw_delegate_start(p.as_ptr());
            Some(Self(p))
        }
    }
}

impl Drop for Display {
    fn drop(&mut self) {
        unsafe {
            ffi::sw_delegate_stop(self.0.as_ptr());
            ffi::sw_delegate_free(self.0.as_ptr());
        }
    }
}

thread_local! {
    static DISPLAY: RefCell<Option<Display>> = const { RefCell::new(None) };
}

fn on_main<R: Send>(f: impl FnOnce() -> R + Send) -> R {
    if unsafe { libc::pthread_main_np() } != 0 {
        f()
    } else {
        let mut out = None;
        DispatchQueue::main().exec_sync(|| out = Some(f()));
        out.unwrap()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn start_display() -> i32 {
    on_main(|| {
        DISPLAY.with_borrow_mut(|slot| {
            if slot.is_some() {
                return 0;
            }
            match Display::start() {
                Some(d) => {
                    *slot = Some(d);
                    0
                }
                None => -1,
            }
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn end_display() -> i32 {
    on_main(|| DISPLAY.with_borrow_mut(|slot| *slot = None));
    0
}
