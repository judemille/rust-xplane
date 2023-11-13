
use crate::debugln;


pub fn aircraft_folder() -> String{
    let acf_path = crate::paths::AircraftPath::from_id(0);
    // crate::debugln!("rust-xplm: player_aircraft_folder: {:?}", acf_path);

    acf_path.folder()
}



/// Reload the aircraft currently in use.
pub fn reload_aircraft(){
    // debugln!("xplm_sys: player.rs: reload_aircraft()");

    //get the acf in use..
    let acf_info = crate::paths::AircraftPath::from_id(0);
    let relative_filename = acf_info.relative_filename();

    set_aircraft(&relative_filename);

}


/// Set the aircraft being used. Pass a relative filename.
/// eg: "Aircraft/Laminar Research/Cessna 172SP/Cessna_172SP.acf"
/// X-Plane SDK docs state differntly but they're wrong. :-(
pub fn set_aircraft( filename: &str ){
    // debugln!("xplm_sys: player.rs: reload_aircraft()");
    let filename_c = std::ffi::CString::new(filename).unwrap();
    unsafe{ xplm_sys::XPLMSetUsersAircraft( filename_c.as_ptr() ) };
}


/// Change location using airport ID code eg: KBOS
pub fn place_at_airport( airport_code: &str ){
    let airport_code_c = std::ffi::CString::new(airport_code).unwrap();
    unsafe{ xplm_sys::XPLMPlaceUserAtAirport( airport_code_c.as_ptr() ) };
}



// Suggested wrapper for placing player aircraft at arbitrary locations...

#[allow(dead_code)]
struct PlayerLocation{
    latitude_degrees: f64,
    longitude_degrees: f64,
    elevation_meters_msl: f32,
    heading_degrees_true: f32,
    speed_meters_per_second: f32,
}

#[allow(dead_code)]
impl PlayerLocation{

    // get the current location as a set of sensible defaults.

    // pub fn new() -> Self{
    //     PlayerLocation{
    //         latitude_degrees: 0.0,
    //         longitude_degrees: 0.0,
    //         elevation_meters_msl: 0.0,
    //         heading_degrees_true: 0.0,
    //         speed_meters_per_second: 0.0,
    //     }
    // }


    // move incrementally with direction/vector wrappers...

    // setters
    // getters

    // not sure on this fn name..
    pub fn place_aircraft_at(&self){
        debugln!("xplm::player::place_aircraft_at() - UNTESTED");
        unsafe{
            xplm_sys::XPLMPlaceUserAtLocation(
                self.latitude_degrees,
                self.longitude_degrees,
                self.elevation_meters_msl,
                self.heading_degrees_true,
                self.speed_meters_per_second,
            );
        }

    }

}

