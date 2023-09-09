// Copyright (c) 2023 Julia DeMille
// 
// Licensed under the EUPL, Version 1.2
// 
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

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
            handler(message_str);
        }
    } else {
        let mut x = make_x();
        super::debugln!(x, "[xplm] Error handler called with an invalid message").unwrap();
    }
}
