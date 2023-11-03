// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use thiserror::Error;
use xplane_sys::XPLMDeviceID;

#[non_exhaustive]
pub enum DeviceID {
    GNS430(TwoSideDevice),
    GNS530(TwoSideDevice),
    CDU739(TwoSideDevice),
    G1000Pfd(TwoSideDevice),
    G1000Mfd,
    CDU815(TwoSideDevice),
    PrimusPfd(TwoSideDevice),
    PrimusMfd(ThreeSideDevice),
    PrimusRmu(TwoSideDevice),
}

pub enum ThreeSideDevice {
    Pilot,
    Copilot,
    Center,
}

pub enum TwoSideDevice {
    Pilot,
    Copilot,
}

#[derive(Debug, Error)]
pub enum DeviceConvertError {
    #[error("No match for XPLMDeviceID: {0:?}")]
    UnexpectedInput(XPLMDeviceID),
}

impl TryFrom<XPLMDeviceID> for DeviceID {
    type Error = DeviceConvertError;
    fn try_from(value: XPLMDeviceID) -> Result<Self, Self::Error> {
        use xplane_sys::XPLMDeviceID as di;
        Ok(match value {
            di::GNS430_Pilot => DeviceID::GNS430(TwoSideDevice::Pilot),
            di::GNS430_Copilot => DeviceID::GNS430(TwoSideDevice::Copilot),
            di::GNS530_Pilot => DeviceID::GNS530(TwoSideDevice::Pilot),
            di::GNS530_Copilot => DeviceID::GNS530(TwoSideDevice::Copilot),
            di::CDU739_Pilot => DeviceID::CDU739(TwoSideDevice::Pilot),
            di::CDU739_Copilot => DeviceID::CDU739(TwoSideDevice::Copilot),
            di::G1000_PFD_Pilot => DeviceID::G1000Pfd(TwoSideDevice::Pilot),
            di::G1000_PFD_Copilot => DeviceID::G1000Pfd(TwoSideDevice::Copilot),
            di::G1000_MFD => DeviceID::G1000Mfd,
            di::CDU815_Pilot => DeviceID::CDU815(TwoSideDevice::Pilot),
            di::CDU815_Copilot => DeviceID::CDU815(TwoSideDevice::Copilot),
            di::Primus_PFD_Pilot => DeviceID::PrimusPfd(TwoSideDevice::Pilot),
            di::Primus_PFD_Copilot => DeviceID::PrimusPfd(TwoSideDevice::Copilot),
            di::Primus_MFD_Pilot => DeviceID::PrimusMfd(ThreeSideDevice::Pilot),
            di::Primus_MFD_Copilot => DeviceID::PrimusMfd(ThreeSideDevice::Copilot),
            di::Primus_MFD_Center => DeviceID::PrimusMfd(ThreeSideDevice::Center),
            di::Primus_RMU_Pilot => DeviceID::PrimusRmu(TwoSideDevice::Pilot),
            di::Primus_RMU_Copilot => DeviceID::PrimusRmu(TwoSideDevice::Copilot),
            _ => return Err(DeviceConvertError::UnexpectedInput(value)),
        })
    }
}

impl From<DeviceID> for XPLMDeviceID {
    fn from(val: DeviceID) -> Self {
        use xplane_sys::XPLMDeviceID as di;
        match val {
            DeviceID::GNS430(side) => match side {
                TwoSideDevice::Pilot => di::GNS430_Pilot,
                TwoSideDevice::Copilot => di::GNS430_Copilot,
            },
            DeviceID::GNS530(side) => match side {
                TwoSideDevice::Pilot => di::GNS530_Pilot,
                TwoSideDevice::Copilot => di::GNS530_Copilot,
            },
            DeviceID::CDU739(side) => match side {
                TwoSideDevice::Pilot => di::CDU739_Pilot,
                TwoSideDevice::Copilot => di::CDU739_Copilot,
            },
            DeviceID::G1000Pfd(side) => match side {
                TwoSideDevice::Pilot => di::G1000_PFD_Pilot,
                TwoSideDevice::Copilot => di::G1000_PFD_Copilot,
            },
            DeviceID::G1000Mfd => XPLMDeviceID::G1000_MFD,
            DeviceID::CDU815(side) => match side {
                TwoSideDevice::Pilot => di::CDU815_Pilot,
                TwoSideDevice::Copilot => di::CDU815_Copilot,
            },
            DeviceID::PrimusPfd(side) => match side {
                TwoSideDevice::Pilot => di::Primus_PFD_Pilot,
                TwoSideDevice::Copilot => di::Primus_PFD_Copilot,
            },
            DeviceID::PrimusMfd(side) => match side {
                ThreeSideDevice::Pilot => di::Primus_MFD_Pilot,
                ThreeSideDevice::Copilot => di::Primus_MFD_Copilot,
                ThreeSideDevice::Center => di::Primus_MFD_Center,
            },
            DeviceID::PrimusRmu(side) => match side {
                TwoSideDevice::Pilot => di::Primus_RMU_Pilot,
                TwoSideDevice::Copilot => di::Primus_RMU_Copilot,
            },
        }
    }
}
