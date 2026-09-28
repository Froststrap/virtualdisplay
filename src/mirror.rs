// SPDX-FileCopyrightText: 2026 Froststrap
//
// SPDX-License-Identifier: MPL-2.0

use objc2_core_foundation::CFRetained;
use objc2_core_graphics::*;
use std::ptr;

pub struct Mirror {
    physical: CGDirectDisplayID,
    native_mode: Option<CFRetained<CGDisplayMode>>,
}

impl Mirror {
    pub fn new(physical: CGDirectDisplayID) -> Self {
        Self {
            physical,
            native_mode: None,
        }
    }

    pub fn enable(&mut self, virtual_id: CGDirectDisplayID) {
        self.native_mode = CGDisplayCopyDisplayMode(self.physical);

        if CGDisplayIsInMirrorSet(self.physical) {
            Self::configure(|cfg| unsafe {
                CGConfigureDisplayMirrorOfDisplay(cfg, self.physical, kCGNullDirectDisplay);
            });
        }
        Self::configure(|cfg| unsafe {
            CGConfigureDisplayMirrorOfDisplay(cfg, self.physical, virtual_id);
        });
    }

    pub fn disable(&mut self) {
        if !CGDisplayIsInMirrorSet(self.physical) {
            return;
        }
        let mode = self.native_mode.take();
        Self::configure(|cfg| unsafe {
            CGConfigureDisplayMirrorOfDisplay(cfg, self.physical, kCGNullDirectDisplay);
            if let Some(m) = &mode {
                CGConfigureDisplayWithDisplayMode(cfg, self.physical, Some(m), None);
            }
        });
    }

    fn configure(f: impl FnOnce(CGDisplayConfigRef)) {
        let mut cfg: CGDisplayConfigRef = ptr::null_mut();
        unsafe {
            if CGBeginDisplayConfiguration(&mut cfg) != CGError::Success {
                return;
            }
            f(cfg);
            CGCompleteDisplayConfiguration(cfg, CGConfigureOption::empty());
        }
    }
}
