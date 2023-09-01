// Copyright (c) 2023 Julia DeMille
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::XPAPI;

/// Enables native paths
pub fn path_init(x: &mut XPAPI) {
    // Feature specified to exist in SDK 2.1
    let native_path_feature = x
        .features
        .find("XPLM_USE_NATIVE_PATHS")
        .unwrap() // Unwrap in this case is for NulError -- should not occur.
        .expect("No native paths feature"); // Expecting Some.
    native_path_feature.set_enabled(true);
}
