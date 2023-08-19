// Copyright (c) 2023 Julia DeMille
// 
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! # Flight loop callbacks
//!
//! X-Plane can call plugin code at timed intervals or when it runs its flight model.
//!
//! A FlightLoop object must persist for callbacks to occur. When the FlightLoop is dropped,
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
    ops::DerefMut,
    os::raw::{c_float, c_int, c_void},
    time::Duration,
};

use xplane_sys;

use crate::{XPAPI, make_x, NoSendSync};

/// Tracks a flight loop callback, which can be called by X-Plane periodically for calculations
///
#[derive(Debug)]
pub struct FlightLoop<'a, C> where C: FlightLoopCallback {
    /// The loop data, allocated in a Box
    data: Box<LoopData<'a, C>>,
}

impl<'a, C> FlightLoop<'a, C>
where C: FlightLoopCallback {
    pub(crate) fn new(callback: C) -> Self {
        let mut data = Box::new(LoopData::new(callback));
        let data_ptr: *mut LoopData<C> = data.deref_mut();
        // Create a flight loop
        let mut config = xplane_sys::XPLMCreateFlightLoop_t {
            structSize: mem::size_of::<xplane_sys::XPLMCreateFlightLoop_t>() as c_int,
            phase: xplane_sys::xplm_FlightLoop_Phase_AfterFlightModel as i32,
            callbackFunc: Some(flight_loop_callback::<C>),
            refcon: data_ptr as *mut c_void,
        };
        data.loop_id = unsafe { Some(xplane_sys::XPLMCreateFlightLoop(&mut config)) };
        FlightLoop { data }
    }

    /// Schedules the flight loop callback to be executed in the next flight loop
    ///
    /// After the flight loop callback is first called, it will continue to be called
    /// every flight loop unless it cancels itself or changes its schedule.
    pub fn schedule_immediate(&mut self) {
        self.data.set_interval(LoopResult::Loops(1))
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
    pub fn schedule_after(&mut self, time: Duration) {
        let seconds_f = (time.as_secs() as f32) + (1e-9_f32 * time.subsec_nanos() as f32);
        self.data.set_interval(LoopResult::Seconds(seconds_f));
    }

    /// Deactivates the flight loop
    pub fn deactivate(&mut self) {
        self.data.set_interval(LoopResult::Deactivate);
    }
}

/// Data stored as part of a FlightLoop and used as a refcon
struct LoopData<'a, C> where C: FlightLoopCallback {
    /// The loop result, or None if the loop has not been scheduled
    loop_result: Option<LoopResult>,
    /// The loop ID
    loop_id: Option<xplane_sys::XPLMFlightLoopID>,
    /// The callback (stored here but not used)
    callback: Box<C>,
    _phantom: NoSendSync<'a>,
}

impl<'a, C> fmt::Debug for LoopData<'a, C>
where C: FlightLoopCallback {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("LoopData")
            .field("loop_result", &self.loop_result)
            .field("loop_id", &self.loop_id)
            .field("callback", &String::from("[callback]"))
            .finish()
    }
}

impl<'a, C> LoopData<'a, C>
where C: FlightLoopCallback {
    /// Creates a new LoopData with a callback
    pub(crate) fn new(callback: C) -> Self {
        LoopData {
            loop_result: None,
            loop_id: None,
            callback: Box::new(callback),
            _phantom: PhantomData,
        }
    }

    fn set_interval(&mut self, loop_result: LoopResult) {
        let loop_id = self.loop_id.expect("Loop ID not set");
        unsafe { xplane_sys::XPLMScheduleFlightLoop(loop_id, loop_result.clone().into(), 1) };
        self.loop_result = Some(loop_result);
    }
}

impl<'a, C> Drop for LoopData<'a, C>
where C: FlightLoopCallback {
    fn drop(&mut self) {
        if let Some(loop_id) = self.loop_id {
            unsafe { xplane_sys::XPLMDestroyFlightLoop(loop_id) }
        }
    }
}

/// Trait for objects that can receive flight loop callbacks
pub trait FlightLoopCallback: 'static {
    /// Called periodically by X-Plane according to the provided scheduling
    ///
    /// In this callback, processing can be done. Drawing cannot be done.
    ///
    /// The provided LoopState can be used to get information and change the scheduling of
    /// callbacks. If the scheduling is not changed, this callback will continue to be called
    /// with the same schedule.
    fn flight_loop(&mut self, x: &mut XPAPI, state: &mut LoopState);
}

/// Closures can be used as FlightLoopCallbacks
impl<F> FlightLoopCallback for F
where
    F: 'static + FnMut(&mut XPAPI, &mut LoopState),
{
    fn flight_loop(&mut self, x: &mut XPAPI, state: &mut LoopState) {
        self(x, state)
    }
}

/// Information available during a flight loop callback
///
/// By default, a flight loop callback will continue to be called on its initial schedule.
/// The scheduling functions only need to be called if the callback scheduling should change.
#[derive(Debug)]
pub struct LoopState<'a> {
    /// Time since last callback call
    since_call: Duration,
    /// Time since last flight loop
    since_loop: Duration,
    /// Callback counter
    counter: i32,
    /// The loop result
    result: &'a mut LoopResult,
}

impl<'a> LoopState<'a> {
    /// Returns the duration since the last time this callback was called
    pub fn since_last_call(&self) -> Duration {
        self.since_call
    }
    /// Returns the duration since the last flight loop
    ///
    /// If this callback is not called every flight loop, this may be different from the
    /// value returned from `time_since_last_call`.
    pub fn since_last_loop(&self) -> Duration {
        self.since_loop
    }
    /// Returns the value of a counter that increments every time the callback is called
    pub fn counter(&self) -> i32 {
        self.counter
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

/// Converts a LoopResult into an f32 suitable for returning from a flight loop callback
impl From<LoopResult> for f32 {
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
unsafe extern "C" fn flight_loop_callback<C: FlightLoopCallback>(
    since_last_call: c_float,
    since_loop: c_float,
    counter: c_int,
    refcon: *mut c_void,
) -> c_float {
    // Get the loop data
    let loop_data = refcon as *mut LoopData<C>;
    // Create a state
    let mut state = LoopState {
        since_call: Duration::from_secs_f32(since_last_call),
        since_loop: Duration::from_secs_f32(since_loop),
        counter,
        result: (*loop_data).loop_result.as_mut().unwrap(),
    };
    let mut x = make_x();
    (*loop_data).callback.flight_loop(&mut x, &mut state);

    // Return the next loop time
    f32::from(state.result.clone())
}
