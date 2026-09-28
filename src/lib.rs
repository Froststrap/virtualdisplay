// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

mod cgvirtual;
mod display;
mod mirror;

use dispatch2::run_on_main;

#[unsafe(no_mangle)]
pub extern "C" fn start_display() -> i32 {
    run_on_main(|_| display::start())
}

#[unsafe(no_mangle)]
pub extern "C" fn end_display() -> i32 {
    run_on_main(|_| display::stop())
}
