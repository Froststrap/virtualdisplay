// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

use crate::{
    cgvirtual::{Config, VirtualDisplay},
    mirror::Mirror,
};
use dispatch2::{DispatchQueue, DispatchTime};
use objc2_core_foundation::CGSize;
use objc2_core_graphics::*;
use std::{
    cell::{Cell, RefCell},
    ffi::c_void,
    ptr,
    sync::RwLock,
};

/// how long to wait between attempts at settling the display
const POLL_TICK_NS: i64 = 500_000_000;
/// how many setup attempts before we stop touching the display config
const MAX_ATTEMPTS: u32 = 30;
/// how many times we may break/re-mirror to force the main role over
const MAX_FORCES: u32 = 3;

/// publically editable meta
/// by the user on `init()`
#[derive(Clone)]
pub struct DisplayMeta {
    pub name: Option<String>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub rates: Option<Vec<f64>>,
    pub serial: u32,
}

struct Display {
    vd: VirtualDisplay,
    mirror: Mirror,
    mirrored: bool,
    done: bool,
    attempts: u32,
    forces: u32,
    waits: u32,
    token: u64,
}

static META_LOCK: RwLock<Option<DisplayMeta>> = RwLock::new(None);

thread_local! {
    static DISPLAY: RefCell<Option<Display>> = const { RefCell::new(None) };
    static POLL_GEN: Cell<u64> = const { Cell::new(0) };
}

pub fn init(s: DisplayMeta) -> i32 {
    *META_LOCK.write().expect("aquire lock") = Some(s);
    0
}

pub enum StartError {
    /// meta produced an unusable config
    Invalid,
    /// WindowServer rejected display identity
    Rejected,
}

pub fn start() -> i32 {
    DISPLAY.with_borrow_mut(|slot| {
        if slot.is_some() {
            return 0;
        }
        match Display::start() {
            Ok(mut d) => {
                let token = d.token;
                d.ensure();
                *slot = Some(d);
                schedule_poll(token);
                0
            }
            Err(StartError::Invalid) => -1,
            Err(StartError::Rejected) => -2,
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

/// keep poking `ensure` until the display settles, mirroring the old
/// Swift `pollForMirroring` timer the rust port dropped
fn schedule_poll(token: u64) {
    let _ = DispatchQueue::main().after(DispatchTime::NOW.time(POLL_TICK_NS), move || {
        if POLL_GEN.with(|g| g.get()) != token {
            return;
        }
        DISPLAY.with_borrow_mut(|slot| {
            let Some(d) = slot.as_mut() else { return };
            d.ensure();
            schedule_poll(token);
        });
    });
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
        serial: meta.serial,
    })
}

impl Display {
    fn start() -> Result<Self, StartError> {
        let meta = META_LOCK.read().expect("aquire lock").as_ref().cloned().expect("call init() to set data details to use");
        let cfg = build_config(&meta).ok_or(StartError::Invalid)?;

        // GUARD:
        // The physical main display has to be read BEFORE the virtual one
        // exists, otherwise we may capture the virtual display as our own
        // physical target and end up mirroring it onto itself.
        let physical = CGMainDisplayID();

        // Register first so the "display added / became active" event
        // cannot slip past us while the virtual display spins up.
        unsafe {
            CGDisplayRegisterReconfigurationCallback(Some(on_reconfig), ptr::null_mut());
        }

        let vd = VirtualDisplay::create(&cfg).map_err(|_| StartError::Rejected).inspect_err(|_| unsafe {
            CGDisplayRemoveReconfigurationCallback(Some(on_reconfig), ptr::null_mut());
        })?;

        let token = POLL_GEN.with(|g| {
            let next = g.get().wrapping_add(1);
            g.set(next);
            next
        });

        Ok(Self {
            vd,
            mirror: Mirror::new(physical),
            mirrored: false,
            done: false,
            attempts: 0,
            forces: 0,
            waits: 0,
            token,
        })
    }

    /// Get the virtual display to a state where apps vsync against it:
    /// mirrored onto the physical display AND holding the main role.
    fn ensure(&mut self) {
        let id = self.vd.id();

        if !CGDisplayIsActive(id) {
            self.waits += 1;
            if self.waits <= 5 || self.waits.is_multiple_of(20) {
                eprintln!(
                    "[virtualdisplay] waiting for virtual display {id} to activate ({})",
                    self.waits
                );
            }
            return;
        }

        // Re-read reality, the window server can drop either at any time.
        self.mirrored = self.mirror.engaged(id);
        let main = CGDisplayIsMain(id);

        if self.mirrored && main {
            if !self.done {
                eprintln!("[virtualdisplay] virtual display {id} is main + mirrored");
            }
            self.done = true;
            self.attempts = 0;
            self.forces = 0;
            return;
        }
        self.done = false;

        if self.attempts >= MAX_ATTEMPTS {
            eprintln!(
                "[virtualdisplay] giving up: mirrored={} main={main}",
                self.mirrored
            );
            return;
        }
        self.attempts += 1;

        // 1) Hand the main role over while both displays are still separate,
        //    origins are meaningless inside a mirror set.
        if !self.mirrored && !main {
            if CGDisplayIsInMirrorSet(self.mirror.physical()) {
                self.mirror.break_mirror();
            }
            self.mirror.make_main(id);
        }

        // 2) Mirror the physical display onto the virtual one (virtual master).
        if !self.mirrored {
            self.mirrored = self.mirror.enable(id);
            if !self.mirrored && self.mirror.swapped() {
                // Never leave the menu bar stranded on the invisible display.
                eprintln!("[virtualdisplay] mirror failed, rolling back main swap");
                self.mirror.rollback_swap();
            }
        }

        // 3) Mirrored, but macOS still handed main to the physical display.
        //    Break the set, force main over, re-mirror.
        if self.mirrored && !CGDisplayIsMain(id) {
            if self.forces >= MAX_FORCES {
                eprintln!(
                    "[virtualdisplay] mirror is up but virtual display {id} is not main (gave up forcing)"
                );
                return;
            }
            self.forces += 1;
            eprintln!(
                "[virtualdisplay] forcing main onto virtual display (try {}/{MAX_FORCES})",
                self.forces
            );

            self.mirror.break_mirror();
            self.mirrored = false;
            self.mirror.make_main(id);
            self.mirrored = self.mirror.enable(id);
            if !self.mirrored {
                eprintln!("[virtualdisplay] re-mirror failed, rolling back main swap");
                self.mirror.rollback_swap();
                self.mirrored = self.mirror.enable(id);
            }
        }

        eprintln!(
            "[virtualdisplay] ensure: mirrored={} main={}",
            self.mirrored,
            CGDisplayIsMain(id)
        );
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
                d.ensure();
            }
        });
    });
}
