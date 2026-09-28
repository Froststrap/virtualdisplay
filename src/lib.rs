// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

mod virtual_display {
    #[link(name = "swvirtualdisplay")]
    unsafe extern "C" {
        #[link_name = "sw_start_display"]
        pub unsafe fn start_display() -> libc::c_int;
        #[link_name = "sw_end_display"]
        pub unsafe fn end_display() -> libc::c_int;
    }
}

#[unsafe(no_mangle)]
pub fn start_display() -> i32 {
    eprintln!("rshim: vdsp start");
    unsafe { virtual_display::start_display() }
}

#[unsafe(no_mangle)]
pub fn end_display() -> i32 {
    eprintln!("rshim: vdsp end");
    unsafe { virtual_display::end_display() }
}
