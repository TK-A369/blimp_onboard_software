use crate::obsw_interface::*;

use postcard;
use serde;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Controls {
    pub throttle: i32,
    pub elevation: i32,
    pub yaw: i32,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum BlimpAction {
    SetServo { servo: u8, location: i16 },
    SetMotor { motor: u8, speed: i32 },
    SendMsg(Vec<u8>),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum SensorType {
    Barometer,
    GPSLatitude,
    GPSLongitude,
    GPSAltitude,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum BlimpEvent {
    Control(Controls),
    GetMsg(Vec<u8>),
    SensorDataF64(SensorType, f64),
}

#[derive(Debug)]
pub enum FlightMode {
    Manual,            // Throttle -> motors speed; Pitch -> motors pitch; Roll -> motors yaw
    StabilizeAttiAlti, // Maintain altitude and attitude/azimuth
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum MessageG2B {
    Ping(u32),
    Pong(u32),
    Control(Controls),
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub enum MessageB2G {
    Ping(u32),
    Pong(u32),
    ForwardAction(BlimpAction),
    ForwardEvent(BlimpEvent),
}

pub struct BlimpMainAlgo {
    action_callback: Option<Box<dyn Fn(BlimpAction) -> () + Send>>,
    curr_flight_mode: FlightMode,
    controls: Controls,
    altitude: Option<f64>,
    gps_location: Option<(f64, f64)>,
}

impl BlimpAlgorithm<BlimpEvent, BlimpAction> for BlimpMainAlgo {
    fn handle_event(
        &mut self,
        ev: &BlimpEvent,
    ) -> std::pin::Pin<Box<impl std::future::Future<Output = ()>>> {
        Box::pin(async move {
            match ev {
                BlimpEvent::Control(ctrl) => {
                    self.controls = ctrl.clone();
                }
                BlimpEvent::SensorDataF64(SensorType::Barometer, press) => {
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
                BlimpEvent::SensorDataF64(SensorType::GPSLatitude, latitude) => {
                    self.gps_location =
                        Some((*latitude, self.gps_location.unwrap_or((0.0, 0.0)).1));
                }
                BlimpEvent::SensorDataF64(SensorType::GPSLongitude, longitude) => {
                    self.gps_location =
                        Some((self.gps_location.unwrap_or((0.0, 0.0)).0, *longitude));
                }
                BlimpEvent::GetMsg(msg) => {
                    if let Ok(msg_deserialized) = postcard::from_bytes::<MessageG2B>(msg) {
                        match msg_deserialized {
                            MessageG2B::Ping(id) => {
                                self.action_callback.as_ref().map(|x| {
                                    x(BlimpAction::SendMsg(
                                        postcard::to_stdvec::<MessageB2G>(&MessageB2G::Pong(id))
                                            .unwrap(),
                                    ));
                                });
                            }
                            MessageG2B::Pong(id) => {}
                            MessageG2B::Control(ctrl) => {
                                self.handle_event(&BlimpEvent::Control(ctrl)).await;
                            }
                        }
                    } else {
                        eprintln!("Error occurred while deseerializing message");
                    }
                }
                _ => {}
            }
            if matches!(ev, BlimpEvent::SensorDataF64(..)) {
                self.action_callback.as_ref().map(|x| {
                    x(BlimpAction::SendMsg(
                        postcard::to_stdvec::<MessageB2G>(&MessageB2G::ForwardEvent(ev.clone()))
                            .unwrap(),
                    ));
                });
            }
        })
    }

    fn set_action_callback(&mut self, callback: Box<dyn Fn(BlimpAction) -> () + Send>) {
        self.action_callback = Some(callback);
    }
}

impl BlimpMainAlgo {
    pub fn new() -> Self {
        Self {
            action_callback: None,
            curr_flight_mode: FlightMode::Manual,
            controls: Controls {
                throttle: 0,
                elevation: 0,
                yaw: 0,
            },
            altitude: None,
            gps_location: None,
        }
    }

    pub async fn step(&mut self) {
        match self.curr_flight_mode {
            FlightMode::Manual => {
                self.action_callback.as_ref().map(|x| {
                    for i in 0..4 {
                        let speed: i32 = self.controls.throttle
                            + (if i % 2 == 0 { 1 } else { -1 }) * self.controls.yaw
                            + self.controls.elevation;
                        //Motor
                        self.perform_action(x, BlimpAction::SetMotor { motor: i, speed });
                        // Up-down servo
                        self.perform_action(
                            x,
                            BlimpAction::SetServo {
                                servo: 2 * i,
                                location: self.controls.elevation as i16,
                            },
                        );
                        //Sideways servo
                        self.perform_action(
                            x,
                            BlimpAction::SetServo {
                                servo: 2 * i + 1,
                                location: self.controls.yaw as i16,
                            },
                        );
                    }
                });
            }
            FlightMode::StabilizeAttiAlti => {}
        }
    }

    fn perform_action(
        &self,
        action_callback: &(dyn Fn(BlimpAction) -> () + Send),
        action: BlimpAction,
    ) {
        action_callback(action.clone());
        if matches!(
            action,
            BlimpAction::SetMotor { .. } | BlimpAction::SetServo { .. }
        ) {
            action_callback(BlimpAction::SendMsg(
                postcard::to_stdvec::<MessageB2G>(&MessageB2G::ForwardAction(action)).unwrap(),
            ));
        }
    }
}
