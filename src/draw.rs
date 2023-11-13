use snafu::prelude::*;
use std::os::raw::{c_int, c_void};
use xplane_sys::{self, XPLMFontID};
//use crate::debugln;

use std::ffi::{CString, NulError};

use crate::geometry::Rect;

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
    callback: *mut dyn DrawCallback,
    /// The draw phase (used when unregistering)
    phase: Phase,
    /// The C callback (used when unregistering)
    c_callback: xplane_sys::XPLMDrawCallback_f,
}

impl Draw {
    /// Creates a new drawing callback
    pub fn new<C: DrawCallback>(phase: Phase, callback: C) -> Result<Self, Error> {
        let xplm_phase = phase.to_xplm();
        let callback = Box::into_raw(Box::new(callback));
        let status = unsafe {
            xplane_sys::XPLMRegisterDrawCallback(
                Some(draw_callback::<C>),
                xplm_phase,
                0,
                callback.cast(),
            )
        };
        if status == 1 {
            Ok(Draw {
                callback,
                phase,
                c_callback: Some(draw_callback::<C>),
            })
        } else {
            Err(Error::UnsupportedPhase { phase })
        }
    }
}

impl Drop for Draw {
    /// Unregisters this draw callback
    fn drop(&mut self) {
        let phase = self.phase.to_xplm();
        unsafe {
            xplane_sys::XPLMUnregisterDrawCallback(self.c_callback, phase, 0, self.callback.cast());
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

impl Phase {
    /// Converts this phase into an [`XPLMDrawingPhase`] and a 0 for after or 1 for before
    fn to_xplm(self) -> xplane_sys::XPLMDrawingPhase {
        match self {
            Phase::AfterPanel => xplane_sys::XPLMDrawingPhase::Panel,
            Phase::AfterGauges => xplane_sys::XPLMDrawingPhase::Gauges,
            Phase::AfterWindows => xplane_sys::XPLMDrawingPhase::Window,
            Phase::AfterLocalMap2D => xplane_sys::XPLMDrawingPhase::LocalMap2D,
            Phase::AfterLocalMap3D => xplane_sys::XPLMDrawingPhase::LocalMap3D,
            Phase::AfterLocalMapProfile => xplane_sys::XPLMDrawingPhase::LocalMapProfile,
        }
    }
}

/// Errors that can occur when creating a draw callback
#[derive(Snafu, Debug)]
pub enum Error {
    /// X-Plane does not support the provided phase
    #[snafu(display("Unsupported draw phase: {phase:?}"))]
    UnsupportedPhase { phase: Phase },
}

/// Stores various flags that can be enabled or disabled
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
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
/// This function should be used instead of `glBindTexture`
pub fn bind_texture(texture_number: i32, texture_unit: i32) {
    unsafe {
        xplane_sys::XPLMBindTexture2d(texture_number, texture_unit);
    }
}

/// Generates texture numbers in a range not reserved for X-Plane.
///
/// This function should be used instead of `glGenTextures`.
///
/// Texture IDs are placed in the provided slice. If the slice contains more than [`i32::MAX`]
/// elements, no more than [`i32::MAX`] texture IDs will be generated.
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
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
/// See [`generate_texture_numbers`] for more detail.
#[must_use]
pub fn generate_texture_number() -> i32 {
    let number = 0;
    generate_texture_numbers(&mut [number]);
    number
}

pub fn draw_translucent_dark_box<R: Into<Rect<i32>>>(bounds: R) {
    let bounds = bounds.into();
    unsafe {
        xplane_sys::XPLMDrawTranslucentDarkBox(
            bounds.left(),
            bounds.top(),
            bounds.right(),
            bounds.bottom(),
        );
    }
}

// FIXME: Prototype wrapper. Probably needs refactor to be more idiomatic.

pub struct Color {
    red: f32,
    green: f32,
    blue: f32,
    alpha: f32,
}

#[allow(dead_code)]
impl Color {
    #[must_use]
    pub fn from_rgb(red: f32, green: f32, blue: f32) -> Color {
        Color {
            red,
            green,
            blue,
            alpha: 1.0,
        }
    }

    #[must_use]
    pub fn from_rgba(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
        Color {
            red,
            green,
            blue,
            alpha,
        }
    }

    pub fn set_red(&mut self, new: f32) {
        self.red = new;
    }
    pub fn set_green(&mut self, new: f32) {
        self.red = new;
    }
    pub fn set_blue(&mut self, new: f32) {
        self.red = new;
    }
    pub fn set_alpha(&mut self, new: f32) {
        self.red = new;
    }

    fn as_array_rgb(&self) -> [f32; 3] {
        [self.red, self.green, self.blue]
    }
    fn as_array_rgba(&self) -> [f32; 4] {
        [self.red, self.green, self.blue, self.alpha]
    }
}

/// Font ID is forced to XPSDK proportional font.
pub fn draw_string(color: &Color, left: i32, bottom: i32, value: &str) -> Result<(), NulError> {
    let value_c = CString::new(value)?;

    // Word-wrap support has been omitted because it's trash.
    // 1 - does not report actual wrapped width
    // 2 - does not report how many times wrap was applied
    // 3 - does not report height of new lines
    // 4 - support requires ugly if-tree
    // 5 - taking a 0 for no wrapping and returning a 0 for no wrapping applied is not rustic.

    unsafe {
        xplane_sys::XPLMDrawString(
            color.as_array_rgb().as_ptr() as *mut f32,
            left,
            bottom,
            value_c.as_bytes_with_nul().as_ptr() as *mut i8,
            0 as *mut i32, //word wrap arg is forced to a safe value
            XPLMFontID::Proportional,
        );
    }

    Ok(())
}

pub fn measure_string(
    value: &str,
    //num_chars: i32
) -> Result<f32, NulError> {
    let value_c = CString::new(value)?;

    let pixels: f32 = unsafe {
        xplane_sys::XPLMMeasureString(
            XPLMFontID::Proportional,
            value_c.as_bytes_with_nul().as_ptr() as *mut i8,
            value.len() as i32,
        )
    };

    Ok(pixels)
}
