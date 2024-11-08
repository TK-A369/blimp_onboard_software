use crate::obsw_interface::*;

#[derive(Clone)]
struct Controls {
    throttle: i32,
    pitch: i32,
    roll: i32,
}

enum BlimpAction {
    SetServo { servo: u8, location: i16 },
    SetMotor { motor: u8, speed: i32 },
}

enum BlimpEvent {
    Control(Controls),
    BaroData { press: f64 },
    GPSLocation { latitude: f64, longitude: f64 },
}

enum FlightMode {
    Manual,            // Throttle -> motors speed; Pitch -> motors pitch; Roll -> motors yaw
    StabilizeAttiAlti, // Maintain altitude and attitude/azimuth
}

struct BlimpMainAlgo {
    action_callback: Option<Box<dyn Fn(BlimpAction) -> ()>>,
    curr_flight_mode: FlightMode,
    controls: Controls,
    altitude: Option<f64>,
    gps_locaiton: Option<(f64, f64)>,
}

impl BlimpAlgorithm<BlimpEvent, BlimpAction> for BlimpMainAlgo {
    fn handle_event(&mut self, ev: &BlimpEvent) -> impl std::future::Future<Output = ()> {
        async {}
    }

    fn set_action_callback(&mut self, callback: Box<dyn Fn(BlimpAction) -> ()>) {
        self.action_callback = Some(callback);
    }
}

impl BlimpMainAlgo {
    async fn step(&mut self) {
        match self.curr_flight_mode {
            FlightMode::Manual => {}
            FlightMode::StabilizeAttiAlti => {}
        }
    }
}
