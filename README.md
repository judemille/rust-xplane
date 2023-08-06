# X-Plane plugin APIs for Rust

[![Crates.io Version](https://img.shields.io/crates/v/xplm.svg)](https://crates.io/crates/xplane)
[![Documentation](https://docs.rs/xplane/badge.svg)](https://docs.rs/xplane)
[![License](https://img.shields.io/crates/l/xplane.svg)](https://github.com/judemille/rust-xplane#license)

## Purpose

**Rust X-Plane** provides a convenient interface for X-Plane plugin development in the Rust programming language for all
platforms. These interfaces should *mostly* be safe.

This project is designed to support any 64-bit version of X-Plane, so long as the right feature gates are used.

## Status

The library is still in an incomplete state. As a result some parts of the SDK may only be sparsely covered or missing
completely.

- [x] Compiles and is callable from X-Plane
- [x] Debug logging to the console / log file
- [x] DataRef reading and writing
- [x] Commands
- [ ] GUI - Needs further work
- [ ] Drawing - Needs further work

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
The author of this project is a trans lesbian who unequivocally supports Ukraine, and opposes any and all human rights violations.  
Do not use this project if you:
 * Do not unequivocally support the LGBTQ+ population, including transgender individuals.
 * Do not support Ukraine
 * Support any far-right parties or politicians (including Vladimir Putin, the GOP, AfD, FdI, and similar)

I cannot stop you, but if anyone found to meet the above listed criteria interacts with the project, they will be blocked from posting issues or pull requests.

## License

Licensed under the Mozilla Public License, version 2.0.  
Any code from commit `ba89d4234c5b4d7088a40b2bb8f537f72e1e2df3` and before is dual-licensed under the Apache License and the MIT License, at your choice.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall
be licensed as above, without any additional terms or conditions.
