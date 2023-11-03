// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

#![allow(non_upper_case_globals)] // Some weirdness from the proc-macro.
#![allow(clippy::cast_possible_wrap)]

use num_enum::{FromPrimitive, IntoPrimitive};
use xplane_sys::{
    XPLM_MSG_AIRPLANE_COUNT_CHANGED, XPLM_MSG_AIRPORT_LOADED, XPLM_MSG_LIVERY_LOADED,
    XPLM_MSG_PLANE_CRASHED, XPLM_MSG_PLANE_LOADED, XPLM_MSG_PLANE_UNLOADED,
    XPLM_MSG_SCENERY_LOADED, XPLM_MSG_WILL_WRITE_PREFS,
};

#[cfg(feature = "XPLM301")]
use xplane_sys::{XPLM_MSG_ENTERED_VR, XPLM_MSG_EXITING_VR};

#[cfg(feature = "XPLM303")]
use xplane_sys::XPLM_MSG_RELEASE_PLANES;

#[cfg(feature = "XPLM400")]
use xplane_sys::{
    XPLM_MSG_DATAREFS_ADDED, XPLM_MSG_FMOD_BANK_LOADED, XPLM_MSG_FMOD_BANK_UNLOADING,
};

#[repr(i32)]
#[derive(Debug, Eq, PartialEq, FromPrimitive, IntoPrimitive, Copy, Clone)]
/// Inter-plugin message.
pub enum MessageId {
    PlaneCrashed = XPLM_MSG_PLANE_CRASHED,
    PlaneLoaded = XPLM_MSG_PLANE_LOADED,
    AirportLoaded = XPLM_MSG_AIRPORT_LOADED,
    SceneryLoaded = XPLM_MSG_SCENERY_LOADED,
    AirplaneCountChanged = XPLM_MSG_AIRPLANE_COUNT_CHANGED,
    PlaneUnloaded = XPLM_MSG_PLANE_UNLOADED,
    WillWritePrefs = XPLM_MSG_WILL_WRITE_PREFS,
    LiveryLoaded = XPLM_MSG_LIVERY_LOADED,
    #[cfg(feature = "XPLM301")]
    EnteredVR = XPLM_MSG_ENTERED_VR,
    #[cfg(feature = "XPLM301")]
    ExitingVR = XPLM_MSG_EXITING_VR,
    #[cfg(feature = "XPLM303")]
    ReleasePlanes = XPLM_MSG_RELEASE_PLANES,
    #[cfg(feature = "XPLM400")]
    FmodBankLoaded = XPLM_MSG_FMOD_BANK_LOADED,
    #[cfg(feature = "XPLM400")]
    FmodBankUnloading = XPLM_MSG_FMOD_BANK_UNLOADING,
    #[cfg(feature = "XPLM400")]
    DatarefsAdded = XPLM_MSG_DATAREFS_ADDED,
    #[num_enum(catch_all)]
    UnknownMessage(i32),
}

impl MessageId {
    #[must_use]
    pub fn is_xp_reserved(&self) -> bool {
        i32::from(*self) < 0x00FF_FFFF_i32
    }
}

pub struct Message {
    pub id: MessageId,
}
