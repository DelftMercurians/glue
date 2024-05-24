use super::utils::{Stamped};
// use super::robot::*;
use super::serial::*;
use crate::glue::*;

pub const MAX_NUM_ROBOTS : usize = 16;

#[derive(Debug)]
pub struct Monitor {

    // pub robots: [Robot; MAX_NUM_ROBOTS],
    pub base_info: Stamped<Base_Information>,

    serial: Serial,
    start_time: std::time::Instant,
}

impl Monitor {
    pub fn new(port_name : &str) -> Result<Monitor, serialport::Error> {
        let serial = Serial::new(port_name)?;
        let start_time = std::time::Instant::now();
        Ok(Monitor {
            // robots: Default::default(),
            base_info: Stamped::NothingYet,
            serial,
            start_time,
        })
    }

    
} // impl Monitor



