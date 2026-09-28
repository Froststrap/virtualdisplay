use std::ffi::c_void;
#[link(name = "swvirtualdisplay", kind = "static")]
unsafe extern "C" {
    pub fn sw_delegate_new() -> *mut c_void;
    pub fn sw_delegate_start(p: *mut c_void);
    pub fn sw_delegate_stop(p: *mut c_void);
    pub fn sw_delegate_free(p: *mut c_void);
}
