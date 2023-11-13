// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

#![deny(trivial_casts)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo)]
// Making some lints from clippy::pedantic allow instead of warn.
#![allow(clippy::module_name_repetitions)]

//! Bindings to the X-Plane plugin SDK.
//! These should be mostly safe, although care must be taken in some aspects.
//! Any functions or modules that could behave in unexpected ways will document that.
//! This crate handles panics in the `XPluginStart`, `XPluginEnable`, `XPluginDisable`,
//! and `XPluginStop` callbacks. In those cases, your plugin should be disabled by X-Plane.
//! This may cause a memory leak, however. Unwinds are not caught in any other callback;
//! the philosophy being that if something has gone critically wrong while the plugin is running,
//! it probably affects the integrity of the simulator, and should prevent it running.

#[cfg(feature = "XPLM400")]
use avionics::AvionicsAPI;
use command::CommandAPI;
use core::ffi::c_void;
use data::DataAPI;
use flight_loop::{FlightLoop, FlightLoopCallback, FlightLoopPhase};
use menu::MenuAPI;
use paths::PathApi;
use std::{
    ffi::{CString, NulError},
    marker::PhantomData,
    ptr,
};
use xplane_sys::XPLMDebugString;

/// FFI utilities
mod ffi;
/// Plugin macro
mod plugin_macro;

/// Utilities that the `xplane_plugin` macro-generated code uses
mod internal;

/// Avionics
#[cfg(feature = "XPLM400")]
pub mod avionics;
/// Commands
pub mod command;
/// Datarefs
pub mod data;
/// Drawing
pub mod draw;
/// Error detection
pub mod error;
/// SDK feature management
pub mod feature;
use feature::FeatureAPI;

/// Flight loop callbacks
pub mod flight_loop;
/// 2D user interface geometry
pub mod geometry;
/// User interface menus
pub mod menu;
/// Plugin messages
pub mod message;
/// Path conversion
pub mod paths;
/// Plugin creation and management
pub mod plugin;
/// Weather system
#[cfg(feature = "XPLM400")]
pub mod weather;
/// Relatively low-level windows
pub mod window;

type NoSendSync = PhantomData<*mut ()>;

/// Access struct for all APIs in this crate. Intentionally neither [`Send`] nor [`Sync`]. Nothing in this crate is.
pub struct XPAPI {
    // Name not decided on.
    #[cfg(feature = "XPLM400")]
    pub avionics: AvionicsAPI,
    pub command: CommandAPI,
    pub data: DataAPI,
    pub features: FeatureAPI,
    pub menu: MenuAPI,
    pub paths: PathApi,
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

    /// Creates a new flight loop. The provided callback will not be
    /// called until the loop is scheduled.
    pub fn new_flight_loop<T: 'static>(
        &mut self,
        phase: FlightLoopPhase,
        callback: impl FlightLoopCallback<T>,
        base_state: T,
    ) -> FlightLoop<T> {
        FlightLoop::new(phase, callback, base_state)
    }
}

#[inline]
fn make_x() -> XPAPI {
    XPAPI {
        #[cfg(feature = "XPLM400")]
        avionics: AvionicsAPI {
            _phantom: PhantomData,
        },
        command: CommandAPI {
            _phantom: PhantomData,
        },
        data: DataAPI {
            _phantom: PhantomData,
        },
        features: FeatureAPI {
            _phantom: PhantomData,
        },
        menu: MenuAPI {
            _phantom: PhantomData,
        },
        paths: PathApi {
            _phantom: PhantomData
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
pub fn find_symbol<S: Into<String>>(name: S) -> *mut c_void {
    match std::ffi::CString::new(name.into()) {
        Ok(name_c) => unsafe { xplane_sys::XPLMFindSymbol(name_c.as_ptr()) },
        Err(_) => ptr::null_mut(),
    }
}
