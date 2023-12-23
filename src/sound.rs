// Copyright (c) 2023 Julia DeMille.
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use libfmod::{ChannelGroup, Studio};
use xplane_sys::{XPLMAudioBus, XPLMGetFMODChannelGroup, XPLMGetFMODStudio};

use crate::NoSendSync;

/// Access struct for X-Plane's sound APIs.
pub struct SoundApi {
    pub(crate) _phantom: NoSendSync,
}

impl SoundApi {
    #[must_use]
    /// Get the Fmod Studio system used by X-Plane.
    pub fn fmod_studio(&mut self) -> Studio {
        let sys = unsafe { XPLMGetFMODStudio() };
        Studio::from(sys.cast())
    }

    #[must_use]
    /// Get the Fmod channel group corresponding to `bus`.
    /// If `bus` is not valid, [`None`] will be returned.
    pub fn fmod_channel_group(&mut self, bus: XPLMAudioBus) -> Option<ChannelGroup> {
        if audio_bus_valid(bus) {
            let cg = unsafe { XPLMGetFMODChannelGroup(bus) };
            Some(ChannelGroup::from(cg.cast()))
        } else {
            None
        }
    }
}

#[inline]
fn audio_bus_valid(bus: XPLMAudioBus) -> bool {
    matches!(
        bus,
        XPLMAudioBus::RadioCom1
            | XPLMAudioBus::RadioCom2
            | XPLMAudioBus::RadioPilot
            | XPLMAudioBus::RadioCopilot
            | XPLMAudioBus::ExteriorAircraft
            | XPLMAudioBus::ExteriorEnvironment
            | XPLMAudioBus::ExteriorUnprocessed
            | XPLMAudioBus::Interior
            | XPLMAudioBus::UI
            | XPLMAudioBus::Ground
            | XPLMAudioBus::Master
    )
}
