// Copyright (c) 2023 Julia DeMille
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

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
        static mut PLUGIN: ::xplane::plugin::internal::PluginData<$plugin_type> =
            ::xplane::plugin::internal::PluginData {
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
            ::xplane::plugin::internal::xplugin_start(&mut PLUGIN, name, signature, description)
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn XPluginStop() {
            ::xplane::plugin::internal::xplugin_stop(&mut PLUGIN)
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn XPluginEnable() -> ::std::os::raw::c_int {
            ::xplane::plugin::internal::xplugin_enable(&mut PLUGIN)
        }

        #[allow(non_snake_case)]
        #[no_mangle]
        pub unsafe extern "C" fn XPluginDisable() {
            ::xplane::plugin::internal::xplugin_disable(&mut PLUGIN)
        }

        #[allow(non_snake_case)]
        #[allow(unused_variables)]
        #[no_mangle]
        pub unsafe extern "C" fn XPluginReceiveMessage(
            from: ::std::os::raw::c_int,
            message: ::std::os::raw::c_int,
            param: *mut ::std::os::raw::c_void,
        ) {
            ::xplane::plugin::internal::xplugin_receive_message(&mut PLUGIN, from, message, param)
        }
    };
}
