// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

mod cgvirtual;
mod display;
mod mirror;

use dispatch2::DispatchQueue;

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
    on_main(display::start)
}

#[unsafe(no_mangle)]
pub extern "C" fn end_display() -> i32 {
    on_main(display::stop);
    0
}
