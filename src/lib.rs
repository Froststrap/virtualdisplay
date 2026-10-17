// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

mod cgvirtual;
mod display;
mod mirror;
mod util;

use std::ffi::c_char;
use dispatch2::run_on_main;

const MAX_VEC_LEN: usize = 255;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn init(
    name: *const c_char,
    width: usize,
    height: usize,
    item: *const f64,
    capacity: usize,
    serial: u32,
) -> i32 {
    fn some_or_usize(i: usize) -> Option<usize> {
        (i != 0).then_some(i)
    }

    let name = util::IStr::from(name);
    let name = if name.0 == "" {
        None
    } else {
        Some(name.0)
    };
    let width = some_or_usize(width);
    let height = some_or_usize(height);

    // GUARD:
    // Check if rates even has a data length
    if capacity == 0 {
        return -1;
    }
    
    // GUARD:
    // Check if rates has good data
    if item.is_null() || capacity > MAX_VEC_LEN {
        return -1;
    }

    let rates = Some(unsafe {
        std::slice::from_raw_parts(item, capacity)
    }.to_vec()); 
    
    let serial = if serial == 0 {
        use std::collections::hash_map::RandomState;
        use std::hash::{BuildHasher, Hasher};
        let mut hasher = RandomState::new().build_hasher();
        if let Some(name) = &name {
            hasher.write(name.as_bytes());
        }
        (hasher.finish() as u32).max(1)
    } else {
        serial
    };

    let data = display::DisplayMeta {
        name,
        width,
        height,
        rates,
        serial
    };
    display::init(data)
}

#[unsafe(no_mangle)]
pub extern "C" fn start_display() -> i32 {
    run_on_main(|_| display::start())
}

#[unsafe(no_mangle)]
pub extern "C" fn is_running() -> i32 {
    run_on_main(|_| display::running())
}

#[unsafe(no_mangle)]
pub extern "C" fn end_display() -> i32 {
    run_on_main(|_| display::stop())
}
