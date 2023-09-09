// Copyright (c) 2023 Julia DeMille
// 
// Licensed under the EUPL, Version 1.2
// 
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use std::os::raw::{c_int, c_void};

use xplane_sys;

/// A callback that can be called while X-Plane draws graphics
pub trait DrawCallback: 'static {
    /// Draws
    fn draw(&mut self);
}

impl<F> DrawCallback for F
where
    F: 'static + FnMut(),
{
    fn draw(&mut self) {
        self();
    }
}

/// Sets up a draw callback
pub struct Draw {
    /// The callback to execute
    _callback: Box<dyn DrawCallback>,
    /// The draw phase (used when unregistering)
    phase: Phase,
    /// The callback pointer (used when unregistering)
    callback_ptr: *mut c_void,
    /// The C callback (used when unregistering)
    c_callback: xplane_sys::XPLMDrawCallback_f,
}

impl Draw {
    /// Creates a new drawing callback
    /// # Errors
    /// Errors if X-Plane does not support the provided drawing phase.
    pub fn new<C: DrawCallback>(phase: Phase, callback: C) -> Result<Self, Error> {
        let xplm_phase = phase.to_xplm();
        let callback_box = Box::new(callback);
        let callback_ptr: *const _ = &*callback_box;
        let status = unsafe {
            xplane_sys::XPLMRegisterDrawCallback(
                Some(draw_callback::<C>),
                xplm_phase,
                0,
                callback_ptr as *mut _,
            )
        };
        if status == 1 {
            Ok(Draw {
                _callback: callback_box,
                phase,
                callback_ptr: callback_ptr as *mut _,
                c_callback: Some(draw_callback::<C>),
            })
        } else {
            Err(Error::UnsupportedPhase(phase))
        }
    }
}

impl Drop for Draw {
    /// Unregisters this draw callback
    fn drop(&mut self) {
        let phase = self.phase.to_xplm();
        unsafe {
            xplane_sys::XPLMUnregisterDrawCallback(self.c_callback, phase, 0, self.callback_ptr);
        }
    }
}

/// The draw callback provided to X-Plane
///
/// This is instantiated separately for each callback type.
unsafe extern "C" fn draw_callback<C: DrawCallback>(
    _phase: xplane_sys::XPLMDrawingPhase,
    _before: c_int,
    refcon: *mut c_void,
) -> c_int {
    let callback_ptr = refcon.cast::<C>();
    (*callback_ptr).draw();
    // Always allow X-Plane to draw
    1
}

/// Phases in which drawing can occur
#[derive(Debug, Copy, Clone)]
pub enum Phase {
    // TODO: Some phases have been removed because they were removed from the upstream X-Plane SDK.
    // The replacements should be added back in.
    AfterPanel,
    /// After X-Plane draws panel gauges
    AfterGauges,
    /// After X-Plane draws user interface windows
    AfterWindows,
    /// After X-Plane draws 3D content in the local map window
    AfterLocalMap3D,
    /// After X-Plane draws 2D content in the local map window
    AfterLocalMap2D,
    /// After X-Plane draws 2D content in the local map profile view
    AfterLocalMapProfile,
}

#[allow(clippy::cast_possible_wrap)]
impl Phase {
    /// Converts this phase into an `XPLMDrawingPhase` and a 0 for after or 1 for before
    fn to_xplm(self) -> xplane_sys::XPLMDrawingPhase {
        #[allow(clippy::enum_glob_use)]
        use self::Phase::*;
        let phase = match self {
            AfterPanel => xplane_sys::xplm_Phase_Panel,
            AfterGauges => xplane_sys::xplm_Phase_Gauges,
            AfterWindows => xplane_sys::xplm_Phase_Window,
            AfterLocalMap2D => xplane_sys::xplm_Phase_LocalMap2D,
            AfterLocalMap3D => xplane_sys::xplm_Phase_LocalMap3D,
            AfterLocalMapProfile => xplane_sys::xplm_Phase_LocalMapProfile,
        };
        phase as xplane_sys::XPLMDrawingPhase
    }
}

/// Errors that can occur when creating a draw callback
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// X-Plane does not support the provided phase
    #[error("Unsupported draw phase: {0:?}")]
    UnsupportedPhase(Phase),
}

/// Stores various flags that can be enabled or disabled
#[derive(Debug, Clone)]
pub struct GraphicsState {
    /// Enable status of fog
    ///
    /// During 3-d rendering fog is set up to cause a fade-to-fog effect at the visibility limit.
    pub fog: bool,
    /// Enable status of 3D lighting
    pub lighting: bool,
    /// Enable status of alpha testing
    ///
    /// Alpha testing stops pixels from being rendered to the frame buffer if their alpha is zero.
    pub alpha_testing: bool,
    /// Enable status of alpha blending
    pub alpha_blending: bool,
    /// Enable status of depth testing
    pub depth_testing: bool,
    /// Enable status of depth writing
    pub depth_writing: bool,
    /// The number of textures that are enabled for use
    pub textures: i32,
}

/// Sets the graphics state
pub fn set_state(state: &GraphicsState) {
    unsafe {
        xplane_sys::XPLMSetGraphicsState(
            i32::from(state.fog),
            state.textures,
            i32::from(state.lighting),
            i32::from(state.alpha_testing),
            i32::from(state.alpha_blending),
            i32::from(state.depth_testing),
            i32::from(state.depth_writing),
        );
    }
}

/// Binds a texture ID to a texture number
///
/// This function should be used instead of glBindTexture
pub fn bind_texture(texture_number: i32, texture_id: i32) {
    unsafe {
        xplane_sys::XPLMBindTexture2d(texture_number, texture_id);
    }
}

/// Generates texture numbers in a range not reserved for X-Plane.
///
/// This function should be used instead of glGenTextures.
///
/// Texture IDs are placed in the provided slice. If the slice contains more than `i32::MAX`
/// elements, no more than `i32::MAX` texture IDs will be generated.
#[allow(clippy::cast_possible_wrap, clippy::cast_possible_truncation)]
pub fn generate_texture_numbers(numbers: &mut [i32]) {
    let count = if numbers.len() < (i32::max_value() as usize) {
        numbers.len() as i32
    } else {
        i32::max_value()
    };
    unsafe {
        xplane_sys::XPLMGenerateTextureNumbers(numbers.as_mut_ptr(), count);
    }
}

///
/// Generates a single texture number
///
/// See `generate_texture_numbers` for more detail.
///
#[must_use]
pub fn generate_texture_number() -> i32 {
    let number = 0;
    generate_texture_numbers(&mut [number]);
    number
}
