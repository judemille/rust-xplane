// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use xplane::{
    debugln,
    message::MessageId,
    plugin::{Plugin, PluginInfo},
    xplane_plugin, XPAPI,
};

struct MinimalPlugin;

impl Plugin for MinimalPlugin {
    type Error = std::convert::Infallible;

    fn start(xpapi: &mut XPAPI) -> Result<Self, Self::Error> {
        // The following message should be visible in the developer console and the Log.txt file
        debugln!(xpapi, "Hello, World! From the Minimal Rust Plugin").unwrap(); // No NUL bytes.
        Ok(MinimalPlugin)
    }

    fn enable(&mut self, _xpapi: &mut XPAPI) -> Result<(), Self::Error> {
        Ok(())
    }

    fn disable(&mut self, _xpapi: &mut XPAPI) {}

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("Minimal Rust Plugin"),
            signature: String::from("com.jdemille.xplane.examples.minimal"),
            description: String::from("A plugin written in Rust"),
        }
    }
    fn receive_message(
        &mut self,
        _xpapi: &mut XPAPI,
        _from: i32,
        _message: MessageId,
        _param: *mut core::ffi::c_void,
    ) {
    }
}

xplane_plugin!(MinimalPlugin);
