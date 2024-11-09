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
    gps_location: Option<(f64, f64)>,
}

impl BlimpAlgorithm<BlimpEvent, BlimpAction> for BlimpMainAlgo {
    fn handle_event(&mut self, ev: &BlimpEvent) -> impl std::future::Future<Output = ()> {
        async move {
            match ev {
                BlimpEvent::Control(ctrl) => {
                    self.controls = ctrl.clone();
                }
                BlimpEvent::BaroData { press } => {
                    // Compute altitude
                    // See: https://en.wikipedia.org/wiki/Barometric_formula
                    // p = p_b * exp(-g * M * h / R / T)
                    // ln (p / p_b) = -g * M * h / R / T
                    // h = (ln p - ln p_b) * (-R) * T / g / M
                    // h = (ln p_b - ln p) * R * T / g / M
                    // TODO: Stablize and smoothen
                    // TODO: Allow changing base (sea level) pressure and temperature
                    let base_pressure: f64 = 101325.0;
                    let temperature: f64 = 288.15;
                    let const_coef: f64 = 0.0292718; // R / g / M
                    self.altitude =
                        Some((base_pressure.ln() - press.ln()) * const_coef * temperature);
                }
                BlimpEvent::GPSLocation {
                    latitude,
                    longitude,
                } => {
                    self.gps_location = Some((*latitude, *longitude));
                }
            }
        }
    }

    fn set_action_callback(&mut self, callback: Box<dyn Fn(BlimpAction) -> ()>) {
        self.action_callback = Some(callback);
    }
}

impl BlimpMainAlgo {
    async fn step(&mut self) {
        match self.curr_flight_mode {
            FlightMode::Manual => {
                self.action_callback.as_ref().map(|x| {
                    for i in 0..4 {
                        x(BlimpAction::SetMotor {
                            motor: i,
                            speed: self.controls.throttle,
                        });
                    }
                });
            }
            FlightMode::StabilizeAttiAlti => {}
        }
    }
}
