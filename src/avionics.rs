// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use core::ffi::{c_int, c_void};
use std::{fmt, marker::PhantomData, mem};

use snafu::prelude::*;

use xplane_sys::{
    XPLMAvionicsID, XPLMCustomizeAvionics_t, XPLMDeviceID, XPLMRegisterAvionicsCallbacksEx,
    XPLMUnregisterAvionicsCallbacks,
};

use crate::{make_x, NoSendSync, XPAPI};

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

#[derive(Debug, Snafu)]
#[snafu(display("No match for XPLMDeviceID: {id:?}"))]
pub struct DeviceUnmatchedError {
    id: XPLMDeviceID,
}

impl TryFrom<XPLMDeviceID> for DeviceID {
    type Error = DeviceUnmatchedError;
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
            _ => return Err(DeviceUnmatchedError { id: value }),
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

pub enum AvionicsCallbackResult {
    AllowDraw,
    SuppressDraw,
    Irrelevant,
}

impl From<AvionicsCallbackResult> for c_int {
    fn from(val: AvionicsCallbackResult) -> Self {
        match val {
            AvionicsCallbackResult::AllowDraw | AvionicsCallbackResult::Irrelevant => 0,
            AvionicsCallbackResult::SuppressDraw => 1,
        }
    }
}

pub trait AvionicsDrawCallback<T: 'static>: 'static {
    fn do_draw(
        &mut self,
        x: &mut XPAPI,
        device_id: XPLMDeviceID,
        is_before: bool,
        state_data: &mut T,
    ) -> AvionicsCallbackResult;
}

impl<F, T> AvionicsDrawCallback<T> for F
where
    F: 'static + FnMut(&mut XPAPI, XPLMDeviceID, bool, &mut T) -> AvionicsCallbackResult,
    T: 'static,
{
    fn do_draw(
        &mut self,
        x: &mut XPAPI,
        device_id: XPLMDeviceID,
        is_before: bool,
        state_data: &mut T,
    ) -> AvionicsCallbackResult {
        self(x, device_id, is_before, state_data)
    }
}

#[derive(Debug, Snafu)]
#[snafu(display("X-Plane didn't give a handle for the customization. I have no way to know why."))]
pub struct AvionicsCustomizationError;

#[derive(Debug)]
pub struct AvionicsCustomization<T: 'static> {
    data: *mut AvionicsCustomizationData<T>,
    _phantom: NoSendSync,
}

impl<T: 'static> AvionicsCustomization<T> {
    fn try_new(
        device_id: DeviceID,
        cb_before: Option<impl AvionicsDrawCallback<T>>,
        cb_after: Option<impl AvionicsDrawCallback<T>>,
        initial_state: T,
    ) -> Result<Self, AvionicsCustomizationError> {
        let cb_before =
            cb_before.map(|cb| -> *mut dyn AvionicsDrawCallback<T> { Box::into_raw(Box::new(cb)) });
        let cb_after =
            cb_after.map(|cb| -> *mut dyn AvionicsDrawCallback<T> { Box::into_raw(Box::new(cb)) });
        let state_data = Box::into_raw(Box::new(initial_state));
        let data = Box::into_raw(Box::new(AvionicsCustomizationData {
            handle: None,
            cb_before,
            cb_after,
            state_data,
            _phantom: PhantomData,
        }));
        #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
        let mut customize_avionics_struct = XPLMCustomizeAvionics_t {
            structSize: mem::size_of::<XPLMCustomizeAvionics_t>() as i32,
            deviceId: device_id.into(),
            drawCallbackBefore: cb_before.and(Some(avionics_draw_callback::<T>)),
            drawCallbackAfter: cb_after.and(Some(avionics_draw_callback::<T>)),
            refcon: data.cast::<c_void>(),
        };
        let handle = unsafe { XPLMRegisterAvionicsCallbacksEx(&mut customize_avionics_struct) };
        if handle.is_null() {
            let _ = unsafe { Box::from_raw(data) };
            Err(AvionicsCustomizationError)
        } else {
            unsafe {
                (*data).handle = Some(handle);
            }
            Ok(Self {
                data,
                _phantom: PhantomData,
            })
        }
    }
}

impl<T: 'static> Drop for AvionicsCustomization<T> {
    fn drop(&mut self) {
        let _ = unsafe { Box::from_raw(self.data) };
    }
}

struct AvionicsCustomizationData<T: 'static> {
    handle: Option<XPLMAvionicsID>,
    cb_before: Option<*mut dyn AvionicsDrawCallback<T>>,
    cb_after: Option<*mut dyn AvionicsDrawCallback<T>>,
    state_data: *mut T,
    _phantom: NoSendSync,
}

#[allow(clippy::missing_fields_in_debug)]
impl<T: 'static> fmt::Debug for AvionicsCustomizationData<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AvionicsCustomizationData")
            .field("handle", &self.handle)
            .field("cb_before", &"[before-draw callback]")
            .field("cb_after", &"[after-draw callback]")
            .field("state_data", &"[callback state]")
            .finish()
    }
}

impl<T: 'static> AvionicsCustomizationData<T> {}

impl<T: 'static> Drop for AvionicsCustomizationData<T> {
    fn drop(&mut self) {
        unsafe {
            if let Some(handle) = self.handle {
                XPLMUnregisterAvionicsCallbacks(handle);
            }
            if let Some(cb) = self.cb_before {
                let _ = Box::from_raw(cb);
            }
            if let Some(cb) = self.cb_after {
                let _ = Box::from_raw(cb);
            }
            let _ = Box::from_raw(self.state_data);
        }
    }
}

unsafe extern "C" fn avionics_draw_callback<T: 'static>(
    device_id: XPLMDeviceID,
    is_before: c_int,
    refcon: *mut c_void,
) -> c_int {
    let cb_data = refcon.cast::<AvionicsCustomizationData<T>>();
    let mut x = make_x();
    if is_before == 1 {
        if let Some(cb) = (*cb_data).cb_before {
            (*cb)
                .do_draw(
                    &mut x,
                    device_id,
                    true,
                    (*cb_data).state_data.as_mut().unwrap(),
                )
                .into()
        } else {
            AvionicsCallbackResult::AllowDraw.into()
        }
    } else {
        #[allow(clippy::collapsible_else_if)] // Clarity.
        if let Some(cb) = (*cb_data).cb_after {
            (*cb)
                .do_draw(
                    &mut x,
                    device_id,
                    false,
                    (*cb_data).state_data.as_mut().unwrap(),
                )
                .into()
        } else {
            AvionicsCallbackResult::Irrelevant.into()
        }
    }
}

pub struct AvionicsAPI {
    pub(crate) _phantom: NoSendSync,
}

impl AvionicsAPI {
    /// Try to make a new [`AvionicsCustomization`].
    /// # Errors
    /// Returns an error if X-Plane doesn't give a handle upon creation.
    /// There is no way for this crate to know *why* that happened, only that it did.
    pub fn try_new_customization<T: 'static>(
        &mut self,
        device_id: DeviceID,
        cb_before: Option<impl AvionicsDrawCallback<T>>,
        cb_after: Option<impl AvionicsDrawCallback<T>>,
        initial_state: T,
    ) -> Result<AvionicsCustomization<T>, AvionicsCustomizationError> {
        AvionicsCustomization::try_new(device_id, cb_before, cb_after, initial_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::*;
    #[test]
    fn test_avionics_customization() {
        let mut x = make_x();
        let mut seq = Sequence::new();
        let customize_avionics_ctx = xplane_sys::XPLMRegisterAvionicsCallbacksEx_context();
        customize_avionics_ctx
            .expect()
            .withf(|s| {
                let s = unsafe { **s };
                s.deviceId == XPLMDeviceID::Primus_MFD_Center // All I can really check here.
            })
            .once()
            .in_sequence(&mut seq)
            .return_once_st(|s| {
                let thing: *mut i32 = &mut 1;
                unsafe {
                    let s = *s;
                    assert_eq!(avionics_draw_callback::<i32>(s.deviceId, 1, s.refcon), 0);
                    assert_eq!(avionics_draw_callback::<i32>(s.deviceId, 0, s.refcon), 0);
                }
                thing.cast::<c_void>()
            }); // Pointer meaningless.
        let unregister_customize_avionics_ctx =
            xplane_sys::XPLMUnregisterAvionicsCallbacks_context();
        unregister_customize_avionics_ctx
            .expect()
            .once()
            .in_sequence(&mut seq)
            .return_once_st(|_| ());
        x.avionics
            .try_new_customization(
                DeviceID::PrimusMfd(ThreeSideDevice::Center),
                Some(|_: &mut _, _, before: bool, state: &mut _| {
                    *state = 10;
                    assert!(before);
                    AvionicsCallbackResult::AllowDraw
                }),
                Some(|_: &mut _, _, before: bool, state: &mut _| {
                    *state = 5;
                    assert!(!before);
                    AvionicsCallbackResult::Irrelevant
                }),
                1,
            )
            .expect("Could not customize avionics!");
    }
}
