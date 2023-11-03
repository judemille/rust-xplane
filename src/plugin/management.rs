// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use core::ffi::c_int;
use std::{
    ffi::{CStr, CString},
    marker::PhantomData,
};

use xplane_sys::{self, XPLMGetPluginInfo};

use crate::{ffi::StringBuffer, NoSendSync};

/// Returns an iterator over all loaded plugins
#[must_use]
pub fn all_plugins() -> Plugins {
    Plugins {
        next: 0,
        // Subtract 1 because X-Plane is considered a plugin
        count: unsafe { xplane_sys::XPLMCountPlugins() - 1 },
        _phantom: PhantomData,
    }
}

/// An iterator over all loaded plugins
pub struct Plugins {
    /// The index of the next plugin to return
    ///
    /// If this is equal to count, no more plugins are available
    next: c_int,
    /// The total number of plugins available
    count: c_int,
    _phantom: NoSendSync,
}

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
impl Iterator for Plugins {
    type Item = Plugin;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next < self.count {
            let id = unsafe { xplane_sys::XPLMGetNthPlugin(self.next) };
            self.next += 1;
            // Skip past X-Plane
            if id == xplane_sys::XPLM_PLUGIN_XPLANE as xplane_sys::XPLMPluginID {
                self.next()
            } else {
                Some(unsafe { get_plugin(id) })
            }
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.count - self.next) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for Plugins {}

/// Another plugin running in X-Plane (or this plugin)
pub struct Plugin {
    id: xplane_sys::XPLMPluginID,
    name: CString,
    file_path: CString,
    signature: CString,
    description: CString,
    _phantom: NoSendSync,
}

impl Plugin {
    /// Looks for a plugin with the provided signature and returns it if it exists
    #[must_use]
    pub fn from_signature(signature: &str) -> Option<Self> {
        let signature = CString::new(signature).ok()?;
        let plugin_id = unsafe { xplane_sys::XPLMFindPluginBySignature(signature.as_ptr()) };
        if plugin_id == xplane_sys::XPLM_NO_PLUGIN_ID {
            None
        } else {
            Some(unsafe { get_plugin(plugin_id) })
        }
    }

    /// Returns the plugin that is currently running
    /// # Panics
    /// Panics if you've somehow managed to call this without being in a plugin (or X-Plane has no ID for your plugin, somehow). Congratulations.
    #[must_use]
    pub fn this_plugin() -> Plugin {
        let plugin_id = unsafe { xplane_sys::XPLMGetMyID() };
        assert_ne!(
            plugin_id,
            xplane_sys::XPLM_NO_PLUGIN_ID,
            "XPLMGetMyId() returned no plugin ID. Please get in touch -- this should be impossible."
        );
        unsafe { get_plugin(plugin_id) }
    }

    /// Returns the name of this plugin
    #[must_use]
    pub fn name_c(&self) -> &CStr {
        &self.name
    }
    /// Returns the signature of this plugin
    #[must_use]
    pub fn signature_c(&self) -> &CStr {
        &self.signature
    }
    /// Returns the description of this plugin
    #[must_use]
    pub fn description_c(&self) -> &CStr {
        &self.description
    }
    /// Returns the absolute path to this plugin
    #[must_use]
    pub fn path_c(&self) -> &CStr {
        &self.file_path
    }

    /// Returns true if this plugin is enabled
    #[must_use]
    pub fn enabled(&mut self) -> bool {
        unsafe { xplane_sys::XPLMIsPluginEnabled(self.id) == 1 }
    }

    /// Enables or disables the plugin
    pub fn set_enabled(&mut self, enabled: bool) {
        if enabled {
            unsafe {
                xplane_sys::XPLMEnablePlugin(self.id);
            }
        } else {
            unsafe {
                xplane_sys::XPLMDisablePlugin(self.id);
            }
        }
    }
}

/// Retrieve data about a given plugin based on its ID from X-Plane.
unsafe fn get_plugin(id: xplane_sys::XPLMPluginID) -> Plugin {
    let mut name = StringBuffer::new(257);
    let mut fp = StringBuffer::new(257);
    let mut sig = StringBuffer::new(257);
    let mut desc = StringBuffer::new(257);
    unsafe {
        XPLMGetPluginInfo(
            id,
            name.as_mut_ptr(),
            fp.as_mut_ptr(),
            sig.as_mut_ptr(),
            desc.as_mut_ptr(),
        );
    }
    Plugin {
        id,
        name: name.into(),
        file_path: fp.into(),
        signature: sig.into(),
        description: desc.into(),
        _phantom: PhantomData,
    }
}
