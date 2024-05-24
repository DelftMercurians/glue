// use std::time::{self, Instant, Duration};

// #[derive(Debug)]
// pub struct Breakbeam {
//     pub ball_detected: Option<bool>,
// }

// #[derive(Debug)]
// pub struct Robot {
//     pub status: Option<Status>,
//     pub motors: [Motor; 5],
//     pub pack_voltage: Option<[f32; 2]>,
//     pub kicker: Kicker,
//     pub fan_status: Option<Status>,
//     pub breakbeam: Breakbeam,
//     pub imu: IMU,
//     pub time: time::Instant,
//     pub version: Option<String>,
//     pub protocol_version: Option<String>,
// }

// impl Default for Robot {
//     fn default() -> Robot {
//         Robot {
//             status: None,
//             motors: Default::default(),
//             pack_voltage: None,
//             kicker: Default::default(),
//             fan_status: None,
//             breakbeam: Breakbeam {
//                 ball_detected: None,
//             },
//             imu: Default::default(),
//             time: Instant::now(),
//             version: None,
//             protocol_version: None,
//         }
//     }
// }

// impl Robot {
//     pub fn update(&mut self) -> &mut Robot {
//         self.time = Instant::now();
//         if self.status == None {
//             self.status = Some(Status::NOREPLY);
//         }
//         self
//     }

//     pub fn time_since_update(&self) -> Duration {
//         self.time.elapsed()
//     }

//     pub fn check_for_timeout(&mut self) -> bool {
//         if self.status == None { return false }
//         if self.time_since_update() > Duration::from_millis(100) {
//             self.status = None;
//             for motor in &mut self.motors {
//                 motor.status = Status::NOREPLY;
//             }
//             self.kicker.status = Status::NOREPLY;
//             self.fan_status = None;
//             return true
//         }
//         false
//     }

// }
