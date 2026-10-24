// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

use objc2_core_foundation::CFRetained;
use objc2_core_graphics::*;
use std::ptr;

pub struct Mirror {
    physical: CGDirectDisplayID,
    native_mode: Option<CFRetained<CGDisplayMode>>,
    swapped: bool,
}

impl Mirror {
    pub fn new(physical: CGDirectDisplayID) -> Self {
        Self {
            physical,
            native_mode: None,
            swapped: false,
        }
    }

    pub fn physical(&self) -> CGDirectDisplayID {
        self.physical
    }

    /// whether `make_main` moved the arrangement out from under `physical`
    pub fn swapped(&self) -> bool {
        self.swapped
    }

    /// `true` when `physical` is mirroring `master` (so `master` drives it).
    pub fn engaged(&self, master: CGDirectDisplayID) -> bool {
        master != 0
            && self.physical != master
            && CGDisplayMirrorsDisplay(self.physical) == master
    }

    pub fn enable(&mut self, master: CGDirectDisplayID) -> bool {
        // GUARD:
        // A display cannot mirror itself.
        if master == 0 || self.physical == master {
            log::warn!(
                "refusing to mirror {} onto {master}",
                self.physical
            );
            return false;
        }
        if self.engaged(master) {
            return true;
        }

        if self.native_mode.is_none() {
            self.native_mode = CGDisplayCopyDisplayMode(self.physical);
        }

        // Drop whatever mirror set `physical` is part of first,
        // otherwise the new master gets ignored.
        if CGDisplayIsInMirrorSet(self.physical) {
            let physical = self.physical;
            let err = Self::configure(|cfg| unsafe {
                CGConfigureDisplayMirrorOfDisplay(cfg, physical, kCGNullDirectDisplay)
            });
            if err != CGError::Success {
                log::warn!("could not break old mirror set: {err:?}");
                return false;
            }
        }

        let physical = self.physical;
        let err = Self::configure(|cfg| unsafe {
            CGConfigureDisplayMirrorOfDisplay(cfg, physical, master)
        });
        if err != CGError::Success {
            log::warn!("mirror config rejected: {err:?}");
            return false;
        }

        let ok = self.engaged(master);
        log::debug!(
            "mirror physical={} master={master} engaged={ok}",
            self.physical
        );
        ok
    }

    pub fn break_mirror(&mut self) {
        if !CGDisplayIsInMirrorSet(self.physical) {
            return;
        }
        let physical = self.physical;
        Self::configure(|cfg| unsafe {
            CGConfigureDisplayMirrorOfDisplay(cfg, physical, kCGNullDirectDisplay)
        });
    }

    /// Put `target` at origin (0,0), shifting every other display by the same
    /// amount so the arrangement keeps its shape. macOS treats the display at
    /// (0,0) as the main one (menu bar / Dock live there).
    pub fn make_main(&mut self, target: CGDirectDisplayID) -> bool {
        if target == 0 {
            return false;
        }
        if CGDisplayIsMain(target) {
            return true;
        }

        let mut ids = [0 as CGDirectDisplayID; 32];
        let mut count: u32 = 0;
        let err = unsafe { CGGetActiveDisplayList(32, ids.as_mut_ptr(), &mut count) };
        if err != CGError::Success {
            log::error!("CGGetActiveDisplayList failed: {err:?}");
            return false;
        }

        let bounds = CGDisplayBounds(target);
        let dx = bounds.origin.x.round();
        let dy = bounds.origin.y.round();

        // GUARD:
        // Nothing to move, so the arrangement cannot be trusted to flip main.
        if dx == 0.0 && dy == 0.0 {
            let main = CGDisplayIsMain(target);
            log::debug!("target already at origin, isMain={main}");
            return main;
        }

        // GUARD:
        // Only shift displays we can actually see, and never miss the target.
        let ids = &ids[..count.min(ids.len() as u32) as usize];
        if !ids.contains(&target) {
            log::error!("target {target} missing from active display list");
            return false;
        }

        let err = Self::configure_with(CGConfigureOption::ForSession, |cfg| {
            let mut result = CGError::Success;
            for &id in ids {
                let b = CGDisplayBounds(id);
                let x = (b.origin.x.round() - dx) as i32;
                let y = (b.origin.y.round() - dy) as i32;
                let err = unsafe { CGConfigureDisplayOrigin(cfg, id, x, y) };
                if err != CGError::Success {
                    log::warn!("CGConfigureDisplayOrigin({id}) failed: {err:?}");
                    result = err;
                    break;
                }
            }
            result
        });
        if err != CGError::Success {
            return false;
        }

        self.swapped = true;
        let main = CGDisplayIsMain(target);
        log::debug!("make_main target={target} -> isMain={main}");
        main
    }

    /// Put `physical` back at (0,0) and forget the swap.
    pub fn rollback_swap(&mut self) {
        if !self.swapped {
            return;
        }
        self.swapped = false;
        let physical = self.physical;
        Self::configure_with(CGConfigureOption::ForSession, |cfg| unsafe {
            CGConfigureDisplayOrigin(cfg, physical, 0, 0)
        });
    }

    pub fn disable(&mut self) {
        let mirrored = CGDisplayIsInMirrorSet(self.physical);
        if !mirrored && !self.swapped {
            return;
        }

        let mode = self.native_mode.take();
        let swapped = self.swapped;
        self.swapped = false;
        let physical = self.physical;

        let err = Self::configure(|cfg| unsafe {
            let mut result = CGError::Success;
            if mirrored {
                result = CGConfigureDisplayMirrorOfDisplay(cfg, physical, kCGNullDirectDisplay);
            }
            if result == CGError::Success && swapped {
                result = CGConfigureDisplayOrigin(cfg, physical, 0, 0);
            }
            if result == CGError::Success
                && let Some(m) = &mode
            {
                result = CGConfigureDisplayWithDisplayMode(cfg, physical, Some(m), None);
            }
            result
        });
        if err != CGError::Success {
            log::warn!("mirror teardown failed: {err:?}");
        }
    }

    fn configure(f: impl FnOnce(CGDisplayConfigRef) -> CGError) -> CGError {
        Self::configure_with(CGConfigureOption::empty(), f)
    }

    fn configure_with(
        option: CGConfigureOption,
        f: impl FnOnce(CGDisplayConfigRef) -> CGError,
    ) -> CGError {
        let mut cfg: CGDisplayConfigRef = ptr::null_mut();
        unsafe {
            let err = CGBeginDisplayConfiguration(&mut cfg);
            if err != CGError::Success {
                log::error!("CGBeginDisplayConfiguration failed: {err:?}");
                return err;
            }

            let err = f(cfg);
            if err != CGError::Success {
                CGCancelDisplayConfiguration(cfg);
                return err;
            }

            let err = CGCompleteDisplayConfiguration(cfg, option);
            if err != CGError::Success {
                log::error!("CGCompleteDisplayConfiguration failed: {err:?}");
            }
            err
        }
    }
}
