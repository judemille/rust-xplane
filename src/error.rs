// Copyright (c) 2023 Julia DeMille
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{ffi::CStr, os::raw::c_char};

use xplane_sys::XPLMSetErrorCallback;

use crate::make_x;

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
    if let Ok(message_str) = message_cs.to_str() {
        if let Some(handler) = HANDLER {
            handler(message_str)
        }
    } else {
        let mut x = make_x();
        super::debugln!(x, "[xplm] Error handler called with an invalid message").unwrap()
    }
}
