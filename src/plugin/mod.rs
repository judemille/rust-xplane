// Copyright (c) 2023 Julia DeMille.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use core::ffi::c_void;

use crate::{message::MessageId, XPAPI};

/// Accessing and communicating with other plugins
pub mod management;

/// Items used by the xplane_plugin! macro, which must be public
#[doc(hidden)]
pub mod internal;

/// Information about a plugin
pub struct PluginInfo {
    /// The plugin name
    pub name: String,
    /// The plugin's signature, in reverse DNS format
    pub signature: String,
    /// A description of the plugin
    pub description: String,
}

/// The trait that all plugins should implement
pub trait Plugin: Sized {
    /// The error type that a plugin may encounter when starting up or enabling
    type Error: std::error::Error;

    /// Called when X-Plane loads this plugin
    ///
    /// On success, returns a plugin object
    /// # Errors
    /// This function should error if something occurs that must prevent the plugin's use.
    fn start(xpapi: &mut XPAPI) -> Result<Self, Self::Error>;
    /// Called when the plugin is enabled
    ///
    /// If this function returns an Err, the plugin will remain disabled.
    /// The default implementation returns Ok(()).
    /// # Errors
    /// This function should error if something occurs that must prevent the plugin's use.
    fn enable(&mut self, xpapi: &mut XPAPI) -> Result<(), Self::Error>;
    /// Called when the plugin is disabled
    ///
    /// The default implementation does nothing.;
    fn disable(&mut self, xpapi: &mut XPAPI);

    /// Returns information on this plugin
    fn info(&self) -> PluginInfo;

    /// Called when a message is received.
    fn receive_message(
        &mut self,
        xpapi: &mut XPAPI,
        from: i32,
        message: MessageId,
        param: *mut c_void,
    );
}
