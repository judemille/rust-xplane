// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

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
