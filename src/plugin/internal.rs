// Copyright (c) 2023 Julia DeMille
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use std::{
    os::raw::{c_char, c_int, c_void},
    panic,
    panic::AssertUnwindSafe,
    ptr,
};



use crate::make_x;

use super::{
    super::{debugln, internal::copy_to_c_buffer},
    Plugin,
};

/// Information on a plugin
pub struct PluginData<P> {
    /// A pointer to the plugin, allocated in a Box
    pub plugin: *mut P,
    /// If the plugin has panicked in any XPLM callback
    ///
    /// Plugins that have panicked will not receive any further
    /// XPLM callbacks.
    pub panicked: bool,
}

/// Implements the XPluginStart callback
///
/// This reduces the amount of code in the xplane_plugin! macro.
///
/// data is a reference to a PluginData object where the created plugin will be stored.
/// The other parameters are the same as for XPluginStart.
///
/// This function tries to create and allocate a plugin. On success, it stores a pointer to the
/// plugin in data.plugin and returns 1. If the plugin fails to start, it stores a null pointer
/// in data.plugin and returns 0.
///
/// This function never unwinds. It catches any unwind that may occur.
pub unsafe fn xplugin_start<P>(
    data: &mut PluginData<P>,
    name: *mut c_char,
    signature: *mut c_char,
    description: *mut c_char,
) -> c_int
where
    P: Plugin,
{
    let unwind = panic::catch_unwind(AssertUnwindSafe(|| {
        let mut x = make_x();
        super::super::internal::xplm_init(&mut x);
        match P::start(&mut x) {
            Ok(plugin) => {
                let info = plugin.info();
                copy_to_c_buffer(info.name, name);
                copy_to_c_buffer(info.signature, signature);
                copy_to_c_buffer(info.description, description);

                let plugin_box = Box::new(plugin);
                data.plugin = Box::into_raw(plugin_box);
                1
            }
            Err(e) => {
                debugln!(x, "Plugin failed to start: {}", e).unwrap(); // This string should be valid.
                data.plugin = ptr::null_mut();
                0
            }
        }
    }));
    unwind.unwrap_or_else(|_| {
        eprintln!("Panic in XPluginStart");
        data.panicked = true;
        data.plugin = ptr::null_mut();
        0
    })
}

/// Implements the XPluginStop callback
///
/// This function never unwinds. It catches any unwind that may occur.
pub unsafe fn xplugin_stop<P>(data: &mut PluginData<P>)
where
    P: Plugin,
{
    if !data.panicked {
        let unwind = panic::catch_unwind(AssertUnwindSafe(|| {
            let plugin = Box::from_raw(data.plugin);
            data.plugin = ptr::null_mut();
            drop(plugin);
        }));
        if unwind.is_err() {
            eprintln!("Panic in XPluginStop");
            data.panicked = true;
        }
    } else {
        let mut x = make_x();
        debugln!(x, "Warning: A plugin that panicked cannot be stopped. It may leak resources.").unwrap(); // This string should be valid.
    }
}

/// Implements the XPluginEnable callback
///
/// This function never unwinds. It catches any unwind that may occur.
pub unsafe fn xplugin_enable<P>(data: &mut PluginData<P>) -> c_int
where
    P: Plugin,
{
    if !data.panicked {
        let mut x = make_x();
        let unwind = panic::catch_unwind(AssertUnwindSafe(|| match (*data.plugin).enable(&mut x) {
            Ok(_) => 1,
            Err(e) => {
                debugln!(x, "Plugin failed to enable: {}", e).unwrap(); // This string should be valid.
                0
            }
        }));
        unwind.unwrap_or_else(|_| {
            eprintln!("Panic in XPluginEnable");
            data.panicked = true;
            0
        })
    } else {
        // Can't enable a plugin that has panicked
        0
    }
}

/// Implements the XPluginDisable callback
///
/// This function never unwinds. It catches any unwind that may occur.
pub unsafe fn xplugin_disable<P>(data: &mut PluginData<P>)
where
    P: Plugin,
{
    if !data.panicked {
        let mut x = make_x();
        let unwind = panic::catch_unwind(AssertUnwindSafe(|| {
            (*data.plugin).disable(&mut x);
        }));
        if unwind.is_err() {
            eprintln!("Panic in XPluginDisable");
            data.panicked = true;
        }
    }
}

pub unsafe fn xplugin_receive_message<P>(
    data: &mut PluginData<P>,
    from: c_int,
    message: c_int,
    param: *mut c_void,
) where
    P: Plugin,
{
    let mut x = make_x();
    (*data.plugin).receive_message(&mut x, from, message.into(), param);
}
