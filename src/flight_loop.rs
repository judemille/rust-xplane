// Copyright (c) 2023 Julia DeMille
// 
// Licensed under the EUPL, Version 1.2
// 
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

//! # Flight loop callbacks
//!
//! X-Plane can call plugin code at timed intervals or when it runs its flight model.
//!
//! A `FlightLoop` object must persist for callbacks to occur. When the `FlightLoop` is dropped,
//! its callbacks will stop.
//!
//! # Examples
//!
//! Closure handler:
//!
//! ```no_run
//! use xplane::flight_loop::{FlightLoop, LoopState};
//!
//! let handler = |loop_state: &mut LoopState| {
//!     println!("Flight loop callback running");
//! };
//!
//! let mut flight_loop = FlightLoop::new(handler);
//! flight_loop.schedule_immediate();
//! ```
//!
//! Struct handler:
//!
//! ```no_run
//! use xplane::flight_loop::{FlightLoop, FlightLoopCallback, LoopState};
//!
//! struct LoopHandler;
//!
//! impl FlightLoopCallback for LoopHandler {
//!     fn flight_loop(&mut self, state: &mut LoopState) {
//!         println!("Flight loop callback running");
//!     }
//! }
//!
//! let mut flight_loop = FlightLoop::new(LoopHandler);
//! flight_loop.schedule_immediate();
//! ```
//!

use std::{
    f32, fmt,
    marker::PhantomData,
    mem,
    os::raw::{c_float, c_int, c_void},
    time::Duration, cell::RefCell,
};

use xplane_sys;

use crate::{make_x, NoSendSync, XPAPI};

/// Tracks a flight loop callback, which can be called by X-Plane periodically for calculations
///
#[derive(Debug)]
pub struct FlightLoop<T, C>
where
    C: FlightLoopCallback<T>,
{
    /// The loop data, allocated in a Box
    data: Box<LoopData<T, C>>,
}

impl<T, C> FlightLoop<T, C>
where
    C: FlightLoopCallback<T>,
{
    pub(crate) fn new(callback: C, base_state: T) -> Self {
        let mut data = Box::new(LoopData::new(callback, base_state));
        let data_ptr: *mut LoopData<T, C> = &mut *data;
        // Create a flight loop
        #[allow(clippy::cast_possible_wrap)]
        let mut config = xplane_sys::XPLMCreateFlightLoop_t {
            structSize: mem::size_of::<xplane_sys::XPLMCreateFlightLoop_t>() as c_int,
            phase: xplane_sys::xplm_FlightLoop_Phase_AfterFlightModel as i32,
            callbackFunc: Some(flight_loop_callback::<T, C>),
            refcon: data_ptr.cast::<c_void>(),
        };
        data.loop_id = unsafe { Some(xplane_sys::XPLMCreateFlightLoop(&mut config)) };
        FlightLoop { data }
    }

    /// Schedules the flight loop callback to be executed in the next flight loop
    ///
    /// After the flight loop callback is first called, it will continue to be called
    /// every flight loop unless it cancels itself or changes its schedule.
    pub fn schedule_immediate(&mut self) {
        self.data.set_interval(LoopResult::Loops(1));
    }

    /// Schedules the flight loop callback to be executed after a specified number of flight loops
    ///
    /// After the callback is first called, it will continue to be called with the provided loop
    /// interval.
    pub fn schedule_after_loops(&mut self, loops: u32) {
        self.data.set_interval(LoopResult::Loops(loops));
    }

    /// Schedules the flight loop callback to be executed after the specified delay
    ///
    /// After the callback is first called, it will continue to be called with that interval.
    #[allow(clippy::cast_precision_loss)]
    pub fn schedule_after(&mut self, time: Duration) {
        let seconds_f = (time.as_secs() as f32) + (1e-9_f32 * time.subsec_nanos() as f32);
        self.data.set_interval(LoopResult::Seconds(seconds_f));
    }

    /// Deactivates the flight loop
    pub fn deactivate(&mut self) {
        self.data.set_interval(LoopResult::Deactivate);
    }
}

/// Data stored as part of a `FlightLoop` and used as a refcon
struct LoopData<T, C>
where
    C: FlightLoopCallback<T>,
{
    /// The loop result, or None if the loop has not been scheduled
    loop_result: Option<LoopResult>,
    /// The loop ID
    loop_id: Option<xplane_sys::XPLMFlightLoopID>,
    /// The callback (stored here but not used)
    callback: Box<C>,
    /// The flight loop's stored state
    loop_state: RefCell<T>,
    _phantom: NoSendSync,
}

#[allow(clippy::missing_fields_in_debug)] // Clippy thinks _phantom is missing. There is no reason to include it, and a lack of inclusion does not make it non-exhaustive.
impl<T, C> fmt::Debug for LoopData<T, C>
where
    C: FlightLoopCallback<T>,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LoopData")
            .field("loop_result", &self.loop_result)
            .field("loop_id", &self.loop_id)
            .field("callback", &"[callback]")
            .field("loop_state", &"[callback state]")
            .finish()
    }
}

impl<T, C> LoopData<T, C>
where
    C: FlightLoopCallback<T>,
{
    /// Creates a new `LoopData` with a callback
    pub(crate) fn new(callback: C, base_state: T) -> Self {
        LoopData {
            loop_result: None,
            loop_id: None,
            callback: Box::new(callback),
            loop_state: RefCell::new(base_state),
            _phantom: PhantomData,
        }
    }

    fn set_interval(&mut self, loop_result: LoopResult) {
        let loop_id = self.loop_id.expect("Loop ID not set");
        unsafe { xplane_sys::XPLMScheduleFlightLoop(loop_id, loop_result.clone().into(), 1) };
        self.loop_result = Some(loop_result);
    }
}

impl<T, C> Drop for LoopData<T, C>
where
    C: FlightLoopCallback<T>,
{
    fn drop(&mut self) {
        if let Some(loop_id) = self.loop_id {
            unsafe { xplane_sys::XPLMDestroyFlightLoop(loop_id) }
        }
    }
}

/// Trait for objects that can receive flight loop callbacks
pub trait FlightLoopCallback<T>: 'static {
    /// Called periodically by X-Plane according to the provided scheduling
    ///
    /// In this callback, processing can be done. Drawing cannot be done.
    ///
    /// The provided `LoopState` can be used to get information and change the scheduling of
    /// callbacks. If the scheduling is not changed, this callback will continue to be called
    /// with the same schedule.
    fn flight_loop(&mut self, x: &mut XPAPI, state: &mut LoopState<T>);
}

/// Closures can be used as `FlightLoopCallback`s
impl<F, T> FlightLoopCallback<T> for F
where
    F: 'static + FnMut(&mut XPAPI, &mut LoopState<T>),
{
    fn flight_loop(&mut self, x: &mut XPAPI, state: &mut LoopState<T>) {
        self(x, state);
    }
}

/// Information available during a flight loop callback
///
/// By default, a flight loop callback will continue to be called on its initial schedule.
/// The scheduling functions only need to be called if the callback scheduling should change.
#[derive(Debug)]
pub struct LoopState<'a, T> {
    /// Time since last callback call
    since_call: Duration,
    /// Time since last flight loop
    since_loop: Duration,
    /// Callback counter
    counter: i32,
    state_data: &'a mut T,
    /// The loop result
    result: &'a mut LoopResult,
}

impl<'a, T> LoopState<'a, T> {
    /// Returns the duration since the last time this callback was called
    #[must_use]
    pub fn since_last_call(&self) -> Duration {
        self.since_call
    }
    /// Returns the duration since the last flight loop
    ///
    /// If this callback is not called every flight loop, this may be different from the
    /// value returned from `time_since_last_call`.
    #[must_use]
    pub fn since_last_loop(&self) -> Duration {
        self.since_loop
    }
    /// Returns the value of a counter that increments every time the callback is called
    #[must_use]
    pub fn counter(&self) -> i32 {
        self.counter
    }
    #[must_use]
    pub fn state(&mut self) -> &mut T {
        self.state_data
    }
    /// Deactivates this flight loop. It will not be called again until it is scheduled.
    pub fn deactivate(&mut self) {
        *self.result = LoopResult::Deactivate;
    }
    /// Configures this callback to be called on the next flight loop
    pub fn call_next_loop(&mut self) {
        *self.result = LoopResult::Loops(1);
    }
    /// Configures this callback to be called after the specified number of loops
    pub fn call_after_loops(&mut self, loops: u32) {
        *self.result = LoopResult::Loops(loops);
    }
    /// Configures this callback to be called after the provided duration
    #[allow(clippy::cast_precision_loss)]
    pub fn call_after(&mut self, time: Duration) {
        let seconds_f = (time.as_secs() as f32) + (1e-9_f32 * time.subsec_nanos() as f32);
        *self.result = LoopResult::Seconds(seconds_f);
    }
}

/// Loop results, which determine when the callback will be called next
#[derive(Debug, Clone)]
enum LoopResult {
    /// Callback will be called after the provided number of seconds
    Seconds(f32),
    /// Callback will be called after the provided number of loops
    Loops(u32),
    /// Callback will not be called again until it is rescheduled
    Deactivate,
}

/// Converts a `LoopResult` into an f32 suitable for returning from a flight loop callback
impl From<LoopResult> for f32 {
    #[allow(clippy::cast_precision_loss)]
    fn from(lr: LoopResult) -> Self {
        match lr {
            LoopResult::Deactivate => 0f32,
            LoopResult::Seconds(secs) => secs,
            LoopResult::Loops(loops) => -1.0f32 * (loops as f32),
        }
    }
}

/// The flight loop callback that X-Plane calls
///
/// This expands to a separate callback for every type C.
unsafe extern "C" fn flight_loop_callback<T, C: FlightLoopCallback<T>>(
    since_last_call: c_float,
    since_loop: c_float,
    counter: c_int,
    refcon: *mut c_void,
) -> c_float {
    // Get the loop data
    let loop_data = refcon.cast::<LoopData<T, C>>();
    // Create a state
    let mut state = LoopState {
        since_call: Duration::from_secs_f32(since_last_call),
        since_loop: Duration::from_secs_f32(since_loop),
        counter,
        state_data: (*loop_data).loop_state.get_mut(), // If this causes an issue, I would be very surprised, but for the moment I'm leaving the check in.
        result: (*loop_data).loop_result.as_mut().unwrap(), // If we've gotten here, the associated flight loop should be scheduled, and as such
        // have a result that is not None.
    };
    let mut x = make_x();
    (*loop_data).callback.flight_loop(&mut x, &mut state);

    // Return the next loop time
    f32::from(state.result.clone())
}
