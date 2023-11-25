// Copyright (c) 2023 Julia DeMille.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#![deny(trivial_casts)]
#![warn(clippy::all, clippy::pedantic, clippy::cargo, missing_docs)]
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
use crate::avionics::AvionicsApi;
use crate::camera::CameraApi;
use crate::command::CommandApi;
use crate::data::DataApi;
use crate::feature::FeatureApi;
use crate::flight_loop::{FlightLoop, FlightLoopCallback, FlightLoopPhase};
use crate::menu::MenuApi;
use crate::navigation::{Fms, NavApi};
use crate::paths::PathApi;
use crate::player::PlayerApi;
use crate::plugin::management::PluginApi;
use crate::scenery::SceneryApi;
#[cfg(all(feature = "XPLM400", feature = "fmod"))]
use crate::sound::SoundApi;
#[cfg(feature = "XPLM400")]
use crate::weather::WeatherApi;
use core::ffi::c_void;
use std::{
    ffi::{CStr, CString, NulError},
    marker::PhantomData,
    ptr,
};
use xplane_sys::{
    XPLMDebugString, XPLMGetLanguage, XPLMGetVersions, XPLMGetVirtualKeyDescription,
    XPLMHostApplicationID, XPLMLanguageCode, XPLMReloadScenery, XPLMSpeakString,
};

/// FFI utilities
mod ffi;
/// Plugin macro
mod plugin_macro;

/// Utilities that the `xplane_plugin` macro-generated code uses
mod internal;

/// Avionics
#[cfg(feature = "XPLM400")]
pub mod avionics;
/// Camera access.
pub mod camera;
/// Commands
pub mod command;
/// Datarefs
pub mod data;
/// Error detection
pub mod error;
/// SDK feature management
pub mod feature;

/// Flight loop callbacks
pub mod flight_loop;
/// 2D user interface geometry
pub mod geometry;
/// User interface menus
pub mod menu;
/// Plugin messages
pub mod message;
/// Navigation APIs
pub mod navigation;
#[cfg(feature = "XPLM303")]
/// [`XPLMInstance`] API wrappers.
/// Locked behind XPLM303 due to bugs in earlier versions of X-Plane.
pub mod obj_instance;
/// Path conversion
pub mod paths;
/// Utility functions relating to the player.
pub mod player;
/// Plugin creation and management
pub mod plugin;
/// APIs to interact with X-Plane's scenery system.
pub mod scenery;
/// APIs to interact with Fmod in X-Plane.
#[cfg(all(feature = "XPLM400", feature = "fmod"))]
pub mod sound;
/// Weather system
#[cfg(feature = "XPLM400")]
pub mod weather;
/// Relatively low-level windows
pub mod window;

type NoSendSync = PhantomData<*mut ()>;

/// Access struct for all APIs in this crate. Intentionally neither [`Send`] nor [`Sync`]. Almost nothing in this crate is.
#[allow(missing_docs)]
pub struct XPAPI {
    // Name not decided on.
    #[cfg(feature = "XPLM400")]
    pub avionics: AvionicsApi,
    pub camera: CameraApi,
    pub command: CommandApi,
    pub data: DataApi,
    pub features: FeatureApi,
    pub menu: MenuApi,
    pub nav: NavApi,
    pub paths: PathApi,
    pub player: PlayerApi,
    pub plugins: PluginApi,
    pub scenery: SceneryApi,
    #[cfg(all(feature = "XPLM400", feature = "fmod"))]
    pub sound: SoundApi,
    #[cfg(feature = "XPLM400")]
    pub weather: WeatherApi,
    _phantom: NoSendSync, // Make this !Send + !Sync.
}

impl XPAPI {
    /// Write a string to the X-Plane log. You probably want [`debug!`] or [`debugln!`] instead.
    /// Keep output to the X-Plane log to a minimum. This file can get rather cluttered.
    /// # Errors
    /// This function will error if the passed [`String`] has a NUL ('\0') character in it.
    pub fn debug_string<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<(), NulError> {
        let s = CString::new(s)?;
        unsafe {
            XPLMDebugString(s.as_ptr());
        }
        Ok(())
    }

    /// Display a string on the screen, and speak it with TTS, if enabled.
    /// # Errors
    /// Returns a [`NulError`] if the passed string contains a NUL byte.
    pub fn speak_string<S: Into<Vec<u8>>>(&mut self, s: S) -> Result<(), NulError> {
        let s = CString::new(s)?;
        unsafe {
            XPLMSpeakString(s.as_ptr());
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

    /// Attempts to locate a symbol. If it exists, returns a pointer to it.
    /// Otherwise, a null pointer is returned.
    pub fn find_symbol<S: Into<String>>(&mut self, name: S) -> *mut c_void {
        match std::ffi::CString::new(name.into()) {
            Ok(name_c) => unsafe { xplane_sys::XPLMFindSymbol(name_c.as_ptr()) },
            Err(_) => ptr::null_mut(),
        }
    }

    /// Get the [`Version`]s of X-Plane and XPLM, respectively.
    /// There are no guarantees about the form of the version numbers, except
    /// that subsequent versions will have larger numbers.
    pub fn get_versions(&mut self) -> (i32, i32) {
        let mut xp = 0i32;
        let mut xplm = 0i32;
        let mut host_id = XPLMHostApplicationID::XPlane;
        unsafe {
            XPLMGetVersions(&mut xp, &mut xplm, &mut host_id);
        }
        (xp, xplm)
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    /// Get the description of a virtual key.
    /// # Panics
    /// Panics if X-Plane gives invalid UTF-8. If this happens, panicking here
    /// is the least of your problems.
    pub fn get_vkey_desc(&mut self, key: window::Key) -> &str {
        let desc = unsafe { XPLMGetVirtualKeyDescription(u32::from(key) as i8) };
        unsafe {
            CStr::from_ptr(desc).to_str().unwrap() // UNWRAP: X-Plane promises to give good UTF-8.
        }
    }

    /// Reload the current set of scenery.
    ///
    /// This will only cause X-Plane to re-read already loaded scenery, not load new scenery.
    /// Equivalent to pressing "reload scenery" in the developer menu.
    pub fn reload_scenery(&mut self) {
        unsafe {
            XPLMReloadScenery();
        }
    }

    /// Get the language X-Plane is running in.
    /// If recognized, returns [`Some`], with the ISO 639-1 code.
    /// Returns [`None`] if it is not recognized.
    pub fn get_language(&mut self) -> Option<&'static str> {
        let lang = unsafe { XPLMGetLanguage() };
        match lang {
            XPLMLanguageCode::English => Some("en"),
            XPLMLanguageCode::French => Some("fr"),
            XPLMLanguageCode::German => Some("de"),
            XPLMLanguageCode::Italian => Some("it"),
            XPLMLanguageCode::Spanish => Some("es"),
            XPLMLanguageCode::Korean => Some("ko"),
            XPLMLanguageCode::Russian => Some("ru"),
            XPLMLanguageCode::Greek => Some("el"),
            XPLMLanguageCode::Japanese => Some("ja"),
            #[cfg(feature = "XPLM300")]
            XPLMLanguageCode::Chinese => Some("zh"),
            _ => None,
        }
    }
}

#[inline]
fn make_x() -> XPAPI {
    XPAPI {
        #[cfg(feature = "XPLM400")]
        avionics: AvionicsApi {
            _phantom: PhantomData,
        },
        camera: CameraApi {
            _phantom: PhantomData,
        },
        command: CommandApi {
            _phantom: PhantomData,
        },
        data: DataApi {
            _phantom: PhantomData,
        },
        features: FeatureApi {
            _phantom: PhantomData,
        },
        menu: MenuApi {
            _phantom: PhantomData,
        },
        nav: NavApi {
            fms: Fms {
                _phantom: PhantomData,
            },
            _phantom: PhantomData,
        },
        paths: PathApi {
            _phantom: PhantomData,
        },
        player: PlayerApi {
            _phantom: PhantomData,
        },
        plugins: PluginApi {
            _phantom: PhantomData,
        },
        scenery: SceneryApi {
            _phantom: PhantomData,
        },
        #[cfg(all(feature = "XPLM400", feature = "fmod"))]
        sound: SoundApi {
            _phantom: PhantomData,
        },
        #[cfg(feature = "XPLM400")]
        weather: WeatherApi {
            _phantom: PhantomData,
        },
        _phantom: PhantomData,
    }
}

/// Writes a message to the developer console and Log.txt file.
/// Keep output to the X-Plane log to a minimum. This file can get rather cluttered.
/// # Errors
/// This macro will return a `Result<(), NulError>`. An [`Err`] may be returned if
/// the formatting you specify produces a NUL byte within the string.
#[macro_export]
macro_rules! debug {
    ($x:ident, $($arg:tt)*) => ({
        let formatted_string: String = std::fmt::format(std::format_args!($($arg)*));
        $x.debug_string(formatted_string)
    });
}

/// Writes a message to the developer console and Log.txt file, with a newline.
/// Keep output to the X-Plane log to a minimum. This file can get rather cluttered.
/// # Errors
/// This macro will return a `Result<(), NulError>`. An [`Err`] may be returned if
/// the formatting you specify produces a NUL byte within the string.
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
