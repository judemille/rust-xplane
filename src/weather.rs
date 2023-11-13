// Copyright (c) 2023 Julia DeMille
//
// Licensed under the EUPL, Version 1.2
//
// You may not use this work except in compliance with the Licence.
// You should have received a copy of the Licence along with this work. If not, see:
// <https://joinup.ec.europa.eu/collection/eupl/eupl-text-eupl-12>.
// See the Licence for the specific language governing permissions and limitations under the Licence.

use std::ffi::{CString, NulError};
use std::mem::{self, MaybeUninit};

use snafu::prelude::*;

use xplane_sys::{
    XPLMFixedString150_t, XPLMGetMETARForAirport, XPLMGetWeatherAtLocation, XPLMWeatherInfo_t,
};

use crate::NoSendSync;

#[derive(Snafu, Debug)]
pub enum WeatherError {
    #[snafu(context(false))]
    #[snafu(display("The passed in string contained a NUL byte."))]
    Null { source: NulError },
    #[snafu(display(
        "Could not get detailed weather at the location: lat: {lat}, lon: {lon}, alt: {alt_m} m."
    ))]
    NoDetailedWeather { lat: f64, lon: f64, alt_m: f64 },
}

pub struct WeatherApi {
    _phantom: NoSendSync,
}

impl WeatherApi {
    /// Get the METAR for the given aerodrome.
    /// # Errors
    /// Returns an error if the aerodrome ID contains a null byte, or if the METAR returned
    /// by X-Plane is not valid UTF-8. X-Plane should be giving UTF-8, per the developers.
    /// # Panics
    /// Panics if X-Plane provides invalid UTF-8. This should be impossible.
    #[allow(clippy::cast_sign_loss)]
    pub fn get_aerodrome_metar<S: Into<Vec<u8>>>(ad: S) -> Result<String, WeatherError> {
        let ad = CString::new(ad)?;
        let mut get_metar_out = XPLMFixedString150_t { buffer: [0i8; 150] };
        unsafe {
            XPLMGetMETARForAirport(ad.as_ptr(), &mut get_metar_out);
        };
        let buffer =
            Vec::from(unsafe { mem::transmute::<[i8; 150], [u8; 150]>(get_metar_out.buffer) });
        Ok(String::from_utf8(buffer).unwrap())
    }

    /// Get the weather at the given location.
    /// The location must be near the user.
    /// # Errors
    /// This function will return an error if detailed weather was not found.
    pub fn get_weather_at_location(
        lat: f64,
        lon: f64,
        alt_m: f64,
    ) -> Result<XPLMWeatherInfo_t, WeatherError> {
        let mut weather: MaybeUninit<XPLMWeatherInfo_t> = MaybeUninit::zeroed();
        if unsafe { XPLMGetWeatherAtLocation(lat, lon, alt_m, weather.as_mut_ptr()) } == 1 {
            unsafe { Ok(weather.assume_init()) }
        } else {
            Err(WeatherError::NoDetailedWeather { lat, lon, alt_m })
        }
    }
}
