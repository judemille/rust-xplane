// Copyright (c) 2023 Julia DeMille
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.


#![allow(non_upper_case_globals)] // Some weirdness from the proc-macro.

use num_enum::FromPrimitive;
use xplane_sys::*;

#[repr(i32)]
#[derive(Debug, Eq, PartialEq, FromPrimitive)]
/// Inter-plugin message.
pub enum Message {
    PlaneCrashed = XPLM_MSG_PLANE_CRASHED as i32,
    PlaneLoaded = XPLM_MSG_PLANE_LOADED as i32,
    AirportLoaded = XPLM_MSG_AIRPORT_LOADED as i32,
    SceneryLoaded = XPLM_MSG_SCENERY_LOADED as i32,
    AirplaneCountChanged = XPLM_MSG_AIRPLANE_COUNT_CHANGED as i32,
    #[cfg(feature = "XPLM200")]
    PlaneUnloaded = XPLM_MSG_PLANE_UNLOADED as i32,
    #[cfg(feature = "XPLM210")]
    WillWritePrefs = XPLM_MSG_WILL_WRITE_PREFS as i32,
    #[cfg(feature = "XPLM210")]
    LiveryLoaded = XPLM_MSG_LIVERY_LOADED as i32,
    #[cfg(feature = "XPLM301")]
    EnteredVR = XPLM_MSG_ENTERED_VR as i32,
    #[cfg(feature = "XPLM301")]
    ExitingVR = XPLM_MSG_EXITING_VR as i32,
    #[cfg(feature = "XPLM303")]
    ReleasePlanes = XPLM_MSG_RELEASE_PLANES as i32,
    #[cfg(feature = "XPLM400")]
    FmodBankLoaded = XPLM_MSG_FMOD_BANK_LOADED as i32,
    #[cfg(feature = "XPLM400")]
    FmodBankUnloading = XPLM_MSG_FMOD_BANK_UNLOADING as i32,
    #[cfg(feature = "XPLM400")]
    DatarefsAdded = XPLM_MSG_DATAREFS_ADDED as i32,
    #[num_enum(catch_all)]
    UnknownMessage(i32),
}
