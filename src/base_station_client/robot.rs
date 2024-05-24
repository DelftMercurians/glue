use super::utils::Stamped;
#[derive(Debug, Clone)]
pub struct Robot {
    pub status_hf: Stamped<crate::glue::Radio_PrimaryStatusHF>,
    pub status_lf: Stamped<crate::glue::Radio_PrimaryStatusLF>,
    pub imu_reading: Stamped<crate::glue::Radio_ImuReadings>,
}

impl Default for Robot {
    fn default() -> Robot {
        Robot {
            status_hf: Stamped::NothingYet,
            status_lf: Stamped::NothingYet,
            imu_reading: Stamped::NothingYet,
        }
    }
}

impl Robot {
    pub fn update_status_hf(&mut self, status_hf : crate::glue::Radio_PrimaryStatusHF) {
        self.status_hf.update(status_hf);
    }
    pub fn update_status_lf(&mut self, status_lf : crate::glue::Radio_PrimaryStatusLF) {
        self.status_lf.update(status_lf);
    }
    pub fn update_imu_reading(&mut self, imu_reading : crate::glue::Radio_ImuReadings) {
        self.imu_reading.update(imu_reading);
    }

    // pub fn time_since_update(&self) -> Duration {
    //     // self.time.elapsed()
    // }

    pub fn check_for_timeout(&mut self) -> bool {
        // if self.status == None { return false }
        // if self.time_since_update() > Duration::from_millis(100) {
        //     self.status = None;
        //     for motor in &mut self.motors {
        //         motor.status = Status::NOREPLY;
        //     }
        //     self.kicker.status = Status::NOREPLY;
        //     self.fan_status = None;
        //     return true
        // }
        false
    }

}
