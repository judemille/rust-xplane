// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

//!
//! This plugin creates a submenu under the Plugins menu. The submenu has one checkable item
//! and one action item.
//!

use xplane::{
    debugln,
    menu::{ActionItem, CheckHandler, CheckItem, ClickHandler, Menu},
    message::MessageId,
    plugin::{Plugin, PluginInfo},
    xplane_plugin,
};

struct MenuPlugin {
    _plugins_submenu: Menu,
}

impl Plugin for MenuPlugin {
    type Error = std::convert::Infallible;

    fn start() -> Result<Self, Self::Error> {
        let plugins_submenu = Menu::new("Menu Test Plugin").unwrap();
        plugins_submenu.add_child(CheckItem::new("Checkable 1", false, CheckHandler1).unwrap());
        plugins_submenu.add_child(ActionItem::new("Action 1", ActionHandler1).unwrap());
        plugins_submenu.add_to_plugins_menu();

        // The menu needs to be part of the plugin struct, or it will immediately get dropped and
        // will not appear
        Ok(MenuPlugin {
            _plugins_submenu: plugins_submenu,
        })
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("Rust Menu Plugin"),
            signature: String::from("org.samcrow.xplm.examples.menu"),
            description: String::from("A plugin written in Rust that creates menus and menu items"),
        }
    }
    fn receive_message(&mut self, _from: i32, _message: MessageId, _param: *mut core::ffi::c_void) {
    }
}

xplane_plugin!(MenuPlugin);

struct CheckHandler1;

impl CheckHandler for CheckHandler1 {
    fn item_checked(&mut self, _item: &CheckItem, checked: bool) {
        debugln!("Checkable 1 checked = {}", checked);
    }
}

struct ActionHandler1;

impl ClickHandler for ActionHandler1 {
    fn item_clicked(&mut self, _item: &ActionItem) {
        debugln!("Action 1 selected");
    }
}
