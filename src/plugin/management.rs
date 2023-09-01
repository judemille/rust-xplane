use std::{
    ffi::{CStr, CString},
    os::raw::{c_char, c_int},
    path::PathBuf,
    ptr,
};

use xplane_sys;

/// Looks for a plugin with the provided signature and returns it if it exists
#[must_use]
pub fn plugin_with_signature(signature: &str) -> Option<Plugin> {
    match CString::new(signature) {
        Ok(signature) => {
            let plugin_id = unsafe { xplane_sys::XPLMFindPluginBySignature(signature.as_ptr()) };
            if plugin_id == xplane_sys::XPLM_NO_PLUGIN_ID {
                None
            } else {
                Some(Plugin(plugin_id))
            }
        }
        Err(_) => None,
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
        "XPLMGetMyId() returned no plugin ID"
    );
    Plugin(plugin_id)
}

/// Returns an iterator over all loaded plugins
#[must_use]
pub fn all_plugins() -> Plugins {
    Plugins {
        next: 0,
        // Subtract 1 because X-Plane is considered a plugin
        count: unsafe { xplane_sys::XPLMCountPlugins() - 1 },
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
}

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
impl Iterator for Plugins {
    type Item = Plugin;
    fn next(&mut self) -> Option<Self::Item> {
        if self.next < self.count {
            let plugin = Plugin(unsafe { xplane_sys::XPLMGetNthPlugin(self.next) });
            self.next += 1;
            // Skip past X-Plane
            if plugin.0 == xplane_sys::XPLM_PLUGIN_XPLANE as xplane_sys::XPLMPluginID {
                self.next()
            } else {
                Some(plugin)
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
pub struct Plugin(xplane_sys::XPLMPluginID);

impl Plugin {
    /// Returns the name of this plugin
    #[must_use]
    pub fn name(&self) -> String {
        read_to_buffer(|buffer| unsafe {
            xplane_sys::XPLMGetPluginInfo(
                self.0,
                buffer,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
        })
    }
    /// Returns the signature of this plugin
    #[must_use]
    pub fn signature(&self) -> String {
        read_to_buffer(|buffer| unsafe {
            xplane_sys::XPLMGetPluginInfo(
                self.0,
                ptr::null_mut(),
                ptr::null_mut(),
                buffer,
                ptr::null_mut(),
            );
        })
    }
    /// Returns the description of this plugin
    #[must_use]
    pub fn description(&self) -> String {
        read_to_buffer(|buffer| unsafe {
            xplane_sys::XPLMGetPluginInfo(
                self.0,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                buffer,
            );
        })
    }
    /// Returns the absolute path to this plugin
    #[must_use]
    pub fn path(&self) -> PathBuf {
        let os_path = read_to_buffer(|buffer| unsafe {
            xplane_sys::XPLMGetPluginInfo(
                self.0,
                ptr::null_mut(),
                buffer,
                ptr::null_mut(),
                ptr::null_mut(),
            );
        });
        PathBuf::from(os_path)
    }

    /// Returns true if this plugin is enabled
    #[must_use]
    pub fn enabled(&self) -> bool {
        unsafe { xplane_sys::XPLMIsPluginEnabled(self.0) == 1 }
    }

    /// Enables or disables the plugin
    pub fn set_enabled(&self, enabled: bool) {
        if enabled {
            unsafe {
                xplane_sys::XPLMEnablePlugin(self.0);
            }
        } else {
            unsafe {
                xplane_sys::XPLMDisablePlugin(self.0);
            }
        }
    }
}

/// Allocates a buffer of at least 256 bytes and passes it to the provided callback, then tries
/// to convert it into a String and returns the result
fn read_to_buffer<F: Fn(*mut c_char)>(read_callback: F) -> String {
    // Create a buffer of 256 nulls
    let mut buffer: [c_char; 256] = [b'\0' as c_char; 256];
    read_callback(buffer.as_mut_ptr());
    let cstr = unsafe { CStr::from_ptr(buffer.as_ptr()) };
    cstr.to_string_lossy().into_owned()
}
