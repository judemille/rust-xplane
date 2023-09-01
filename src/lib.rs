// Copyright (c) 2023 Julia DeMille
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
#![deny(trivial_casts)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
// Making some lints from clippy::pedantic allow instead of warn.
#![allow(clippy::module_name_repetitions)]

//! Bindings to the X-Plane plugin SDK

use flight_loop::{FlightLoop, FlightLoopCallback};
use std::{
    ffi::{CString, NulError},
    marker::PhantomData,
};
use xplane_sys::XPLMDebugString;

/// FFI utilities
mod ffi;
/// Path conversion
mod paths;
/// Plugin macro
mod plugin_macro;

/// Utilities that the `xplane_plugin` macro-generated code uses
mod internal;

/// Commands
pub mod command;
/// Datarefs
pub mod data;
/// Low-level drawing callbacks
pub mod draw;
/// Error detection
pub mod error;
/// SDK feature management
pub mod feature;
use feature::FeatureAPI;

/// Flight loop callbacks
// TODO: Flight loop implementation that supports SDK 1.0
pub mod flight_loop;
/// 2D user interface geometry
pub mod geometry;
/// Access handles for state data.
mod state;
pub use state::StateData;
/// User interface menus
pub mod menu;
/// Plugin messages
pub mod message;
/// Plugin creation and management
pub mod plugin;
/// Relatively low-level windows
pub mod window;

type NoSendSync = PhantomData<*mut ()>;

/// Access struct for all APIs in this crate. Intentionally neither [`Send`] nor [`Sync`]. Nothing in this crate is.
pub struct XPAPI {
    // Name not decided on.
    pub features: FeatureAPI,
    _phantom: NoSendSync, // Make this !Send + !Sync.
}

impl XPAPI {
    /// Write a string to the X-Plane log. You probably want [`debug!`] or [`debugln!`] instead.
    /// # Errors
    /// This function will error if the passed [`String`] has a NUL ('\0') character in it.
    pub fn debug_string(&mut self, s: String) -> Result<(), NulError> {
        let s = CString::new(s)?;
        unsafe {
            XPLMDebugString(s.as_ptr());
        }
        Ok(())
    }

    /// Get a handle to mutable state data.
    pub fn with_handle<T, U, V>(&mut self, sd: &mut StateData<T>, cb: U) -> V
    where
        U: FnOnce(&mut T) -> V,
    {
        sd.with_handle(cb)
    }

    /// Creates a new flight loop. The provided callback will not be
    /// called until the loop is scheduled.
    pub fn new_flight_loop<T, C>(&mut self, callback: C, base_state: T) -> FlightLoop<T, C>
    where
        C: FlightLoopCallback<T>,
    {
        FlightLoop::new(callback, base_state)
    }
}

#[inline]
fn make_x() -> XPAPI {
    XPAPI {
        features: FeatureAPI {
            _phantom: PhantomData,
        },
        _phantom: PhantomData,
    }
}

/// Writes a message to the developer console and Log.txt file
#[macro_export]
macro_rules! debug {
    ($x:ident, $($arg:tt)*) => ({
        let formatted_string: String = std::fmt::format(std::format_args!($($arg)*));
        $x.debug_string(formatted_string)
    });
}

/// Writes a message to the developer console and Log.txt file, with a newline
#[macro_export]
#[allow(unused_unsafe)]
macro_rules! debugln {
    () => ($crate::debug!("\n"));
    ($x:ident, $($arg:tt)*) => ({
        let mut formatted_string: String = std::fmt::format(std::format_args!($($arg)*));
        formatted_string.push_str("\n");
        $x.debug_string(formatted_string)
    });
}

/// Attempts to locate a symbol. If it exists, returns a pointer to it
pub fn find_symbol<S: Into<String>>(name: S) -> *mut std::os::raw::c_void {
    use std::ptr;
    match std::ffi::CString::new(name.into()) {
        Ok(name_c) => unsafe { xplane_sys::XPLMFindSymbol(name_c.as_ptr()) },
        Err(_) => ptr::null_mut(),
    }
}
