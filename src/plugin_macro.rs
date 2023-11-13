// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

/// Creates an X-Plane plugin
///
/// Provide the name of your plugin struct. The callbacks that X-Plane uses will be created.
///
/// Creating a plugin involves three steps:
///
/// 1. Create a struct for your plugin
/// 2. Implement Plugin for your plugin struct
/// 3. Place `xplane_plugin!(YourPluginStruct)` in a file, not in any function
///
#[macro_export]
macro_rules! xplane_plugin {
    ($plugin_type: ty) => {
        // The plugin
        static mut PLUGIN: $crate::plugin::internal::PluginData<$plugin_type> =
            $crate::plugin::internal::PluginData {
                plugin: 0 as *mut _,
                panicked: false,
            };

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn XPluginStart(
            name: *mut ::std::os::raw::c_char,
            signature: *mut ::std::os::raw::c_char,
            description: *mut ::std::os::raw::c_char,
        ) -> ::std::os::raw::c_int {
            $crate::plugin::internal::xplugin_start(&mut PLUGIN, name, signature, description)
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn XPluginStop() {
            $crate::plugin::internal::xplugin_stop(&mut PLUGIN)
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn XPluginEnable() -> ::std::os::raw::c_int {
            $crate::plugin::internal::xplugin_enable(&mut PLUGIN)
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn XPluginDisable() {
            $crate::plugin::internal::xplugin_disable(&mut PLUGIN)
        }

        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        #[no_mangle]
        pub unsafe extern "C" fn XPluginReceiveMessage(
            from: ::std::os::raw::c_int,
            message: ::std::os::raw::c_int,
            param: *mut ::std::os::raw::c_void,
        ) {
            $crate::plugin::internal::xplugin_receive_message(&mut PLUGIN, from, message, param)
        }
    };
}
