use std::{ffi::CStr, os::raw::c_char};

use xplane_sys::XPLMSetErrorCallback;

/// The current handler
static mut HANDLER: Option<fn(&str)> = None;

/// Sets the error handler
///
/// Once an error handler is set, it cannot be removed.
pub fn set_error_handler(handler: fn(&str)) {
    unsafe {
        HANDLER = Some(handler);
        XPLMSetErrorCallback(Some(error_handler));
    }
}

/// C error handler callback
unsafe extern "C" fn error_handler(message: *const c_char) {
    let message_cs = CStr::from_ptr(message);
    match message_cs.to_str() {
        Ok(message_str) => {
            if let Some(handler) = HANDLER {
                handler(message_str)
            }
        }
        Err(_) => super::debugln!("[xplm] Error handler called with an invalid message"),
    }
}
