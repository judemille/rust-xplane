# X-Plane plugin APIs for Rust

[![Crates.io Version](https://img.shields.io/crates/v/xplane.svg)](https://crates.io/crates/xplane)
[![Documentation](https://docs.rs/xplane/badge.svg)](https://docs.rs/xplane)
[![License](https://img.shields.io/crates/l/xplane.svg)](https://github.com/judemille/rust-xplane#license)

## Purpose

**Rust X-Plane** provides a convenient interface for X-Plane plugin development in the Rust programming language for all
platforms. These interfaces should *mostly* be safe.

This project is designed to support any 64-bit version of X-Plane, so long as the right feature gates are used.
Testing is performed with XPLM400 and X-Plane 12, but an effort will be made to keep base API compatibility at XPLM210.
Please open an issue if there is a compatibility regression.

## Status

This library is going through a a complete rewrite to enforce threading invariants. Do not use it in
its current state. Here is a checklist of components:

- [ ] Compiles and is callable from X-Plane
- [ ] Debug logging to the console / log file
- [ ] DataRef reading and writing
- [ ] Commands
- [ ] GUI
- [ ] Drawing

## Example

Some more examples can be found in the `examples/` directory.

~~This small snippet, however, is the minimal boilerplate needed to make your plugin compile.~~

```rust
extern crate xplm;

use xplane::plugin::{Plugin, PluginInfo};
use xplane::{debugln, xplane_plugin};

struct MinimalPlugin;

impl Plugin for MinimalPlugin {
    type Error = std::convert::Infallible;

    fn start() -> Result<Self, Self::Error> {
        // The following message should be visible in the developer console and the Log.txt file
        debugln!("Hello, World! From the Minimal Rust Plugin");
        Ok(MinimalPlugin)
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from("Minimal Rust Plugin"),
            signature: String::from("org.samcrow.xplm.examples.minimal"),
            description: String::from("A plugin written in Rust"),
        }
    }
}

xplane_plugin!(MinimalPlugin);
```

## Disclaimer
The current maintainer of this project is a trans lesbian who unequivocally supports Ukraine, and opposes any and all human rights violations.
Do not use this project if you:
 * Do not unequivocally support the LGBTQ+ population, including transgender individuals.
 * Think that LGBTQ+ people "shouldn't put it out on display"
 * Refuse to address people with their preferred name, pronouns, and gender labels.
 * Do not support Ukraine
 * Support any far-right parties or politicians (including Vladimir Putin, the GOP, AfD, FdI, and similar)

I cannot stop you, but if anyone observed to meet the above listed criteria interacts with the project,
they will be blocked from posting issues or pull requests.

## License

Licensed under the European Union Public License, version 1.2. As the author currently resides outside the EU,
all license disputes shall be handled in the courts of Belgium.
Any code from commit `ba89d4234c5b4d7088a40b2bb8f537f72e1e2df3` and before is dual-licensed under the Apache License and the MIT License, at your choice.

## Minimum Supported Rust Version
The MSRV of this crate is always the latest stable compiler version at the time of a given commit.
Maybe it'll work on an older version. Maybe it won't. No guarantees.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall
be licensed as above, without any additional terms or conditions.
