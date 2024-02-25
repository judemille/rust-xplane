<!--
SPDX-FileCopyrightText: 2024 Julia DeMille <me@jdemille.com>

SPDX-License-Identifier: MPL-2.0
-->

# Changelog

## Unreleased

* Changed from the `quick-error` crate to the more maintained derive macro `thiserror` crate
* Implemented the `debug!` and `debugln!` macros (same usage as `print!`/`println!`)
* Marked the `debug()` function as deprecated and changed all usages over to the new macros
* Renamed the examples and adjusted their code
* Shortened the minimal example (easier to understand for newcomers)
* Updated the README again, mainly the example and status

  
* README badges for easy access to the docs etc.
* Flatten the module structure: e.g. `plugin/mod.rs` into `plugin.rs`  
* Some refactoring for readability
* Project formatting  
* Updated deprecated code
* Removed editor specific config files

## 0.3.1 - 2020-05-31

* Updated dependency xplm-sys to 0.4.0
* Updated dependency quick-error to 1.2.3
