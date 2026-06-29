use crate::glue::{self, Radio_ImuReadings};

use super::utils::Stamped;
#[derive(Debug, Clone, Copy)]
pub struct Robot {
    status_hf: Stamped<crate::glue::Radio_PrimaryStatusHF>,
    status_lf: Stamped<crate::glue::Radio_PrimaryStatusLF>,
    imu_reading: Stamped<crate::glue::Radio_ImuReadings>,
    command: Stamped<crate::glue::Radio_Command>,
    global_command: Stamped<crate::glue::Radio_GlobalCommand>,
}

impl Default for Robot {
    fn default() -> Robot {
        Robot {
            status_hf: Stamped::NothingYet,
            status_lf: Stamped::NothingYet,
            imu_reading: Stamped::NothingYet,
            command: Stamped::NothingYet,
            global_command: Stamped::NothingYet,
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
    pub fn update_command(&mut self, command : crate::glue::Radio_Command) {
        self.command.update(command);
    }
    pub fn update_global_command(&mut self, global_command : crate::glue::Radio_GlobalCommand) {
        self.global_command.update(global_command);
    }

    pub fn time_since_update(&self) -> Option<std::time::Duration> {
        const infinite_time : std::time::Duration = std::time::Duration::from_secs(300);
        let mut dur = infinite_time;
        if let Some(d) = self.status_lf.time_since() {
            if d < dur {
                dur = d;
            }
        }
        if let Some(d) = self.status_hf.time_since() {
            if d < dur {
                dur = d;
            }
        }
        if let Some(d) = self.imu_reading.time_since() {
            if d < dur {
                dur = d;
            }
        }
        if dur < infinite_time {
            Some(dur)
        } else {
            None
        }
    }

    pub fn time_since_status_hf_update(&self) -> Option<std::time::Duration> {
        self.status_hf.time_since()
    }

    pub fn time_since_status_lf_update(&self) -> Option<std::time::Duration> {
        self.status_lf.time_since()
    }

    pub fn time_since_imu_reading_update(&self) -> Option<std::time::Duration> {
        self.imu_reading.time_since()
    }

    pub fn time_since_command_update(&self) -> Option<std::time::Duration> {
        self.command.time_since()
    }

    pub fn time_since_global_command_update(&self) -> Option<std::time::Duration> {
        self.global_command.time_since()
    }

    pub fn is_online(&self) -> bool {
        self.time_since_update().map_or(false, |time| time < std::time::Duration::from_millis(400)) 
    }

    //* Accessors for various internal bits *//

    // Returns an Option containing the command to the robot
    pub fn command(&self) -> Option<glue::Radio_Command> {
        self.command.have(|a| a)
    }

    // Returns an Option containing the command to the robot
    pub fn global_command(&self) -> Option<glue::Radio_GlobalCommand> {
        self.global_command.have(|a| a)
    }

    // Returns an Option containing the main microcontroller status
    pub fn primary_status(&self) -> Option<glue::HG_Status> {
        self.status_lf.have(|status_lf| {status_lf.primary_status})
    }

    // Returns an Option containing the kicker status
    pub fn kicker_status(&self) -> Option<glue::HG_Status> {
        self.status_lf.have(|status_lf| {status_lf.kicker_status})
    }

    // Returns an Option containing the imu status
    pub fn imu_status(&self) -> Option<glue::HG_Status> {
        self.status_lf.have(|status_lf| {status_lf.imu_status})
    }

    // Returns an Option containing the tof status
    pub fn tof_status(&self) -> Option<glue::HG_Status> {
        self.status_lf.have(|status_lf| {status_lf.tof_status})
    }

    // Returns an Option containing the kicker capacitor voltage in Volts
    pub fn kicker_cap_voltage(&self) -> Option<f32> {
        self.status_lf.have(|status_lf| {status_lf.cap_voltage as f32 * glue::Scale_KICKER_VCAP})
    }

    pub fn smart_kick_counter(&self) -> Option<u8> {
        self.status_hf.have(|status_hf| {status_hf.smart_kick_counter_return})
    }

    pub fn kick_ok_flag(&self) -> Option<bool> {
        self.status_hf.have(|status_hf| {status_hf.__bindgen_anon_1.last_kick_ok()})
    }

    pub fn reflex_kick_counter(&self) -> Option<u8> {
        self.status_hf.have(|status_hf| {status_hf.kick_counter})
    }

    pub fn reflex_kick_state(&self) -> Option<glue::HG_ReflexState> {
        self.status_hf.have(|status_hf| {status_hf.__bindgen_anon_1.reflex_state()})
    }

    // Returns an Option containing the kicker board temperature in degrees Celsius
    pub fn kicker_temperature(&self) -> Option<f32> {
        None // Has been deprecated by new hardware design
    }

    // Returns an Option of an array of all 5 motor statuses
    pub fn motor_statuses(&self) -> Option<[glue::HG_Status; 5]> {
        self.status_lf.have(|status_lf| {status_lf.motor_status})
    }

    // Returns an Option of an individual motor status (note, use motor_statuses() when multiple motor statuses are required)
    pub fn motor_status(&self, index : u8) -> Option<glue::HG_Status> {
        if index >= 5 { return None; }
        self.status_lf.have(|status_lf| {status_lf.motor_status[index as usize]})
    }

    // Returns an Option of an array of all 5 motor speeds in rad/s
    pub fn motor_speeds(&self) -> Option<[f32; 5]> {
        self.status_hf.have(|status_hf| {
            status_hf.motor_speeds_i.map(|x| x as f32 * glue::Scale_WHEEL_SPEED)
        })
    }

    // Returns an Option of an individual motor speed in rad/s (note, use motor_speeds() when multiple motor speeds are required)
    pub fn motor_speed(&self, index : u8) -> Option<f32> {
        if index >= 5 { return None; }
        self.status_hf.have(|status_hf| {status_hf.motor_speeds_i[index as usize] as f32 * glue::Scale_WHEEL_SPEED})
    }

    // Returns an Option of an array of all 5 motor currents in Amperes
    pub fn motor_currents(&self) -> Option<[f32; 5]> {
        self.status_hf.have(|status_hf| {
            status_hf.motor_currents_i.map(|x| x as f32 * glue::Scale_CURRENT)
        })
    }

    // Returns an Option of an individual motor current in Amperes (note, use motor_currents() when multiple motor speeds are required)
    pub fn motor_current(&self, index : u8) -> Option<f32> {
        if index >= 5 { return None; }
        self.status_hf.have(|status_hf| {status_hf.motor_currents_i[index as usize] as f32 * glue::Scale_CURRENT})
    }

    // Returns an Option of an array of all 5 motor temperatures in degrees Celsius
    pub fn motor_temperatures(&self) -> Option<[f32; 5]> {
        self.status_lf.have(|status_lf| {
            status_lf.motor_driver_temps.map(|temp| {
                temp as f32 * glue::Scale_MD_TEMP
            })
        })
    }

    // Returns an Option of an individual motor temperature in degrees Celsius (note, use motor_temperatures() when multiple motor temps are required)
    pub fn motor_temperature(&self, index : u8) -> Option<f32> {
        if index >= 5 { return None; }
        self.motor_temperatures().map_or(None, |arr| { Some(arr[index as usize]) })
    }

    //  Returns an Option containing the breakbeam ball detection status (true = ball present)
    pub fn breakbeam_ball_detected(&self) -> Option<bool> {
        self.status_hf.have(|status_hf| {status_hf.__bindgen_anon_1.breakbeam_ball_detected() && status_hf.__bindgen_anon_1.breakbeam_sensor_ok()})
    }

    //  Returns an Option containing the breakbeam sensor ok status (false = sensor not functional)
    pub fn breakbeam_sensor_ok(&self) -> Option<bool> {
        self.status_hf.have(|status_hf| {status_hf.__bindgen_anon_1.breakbeam_sensor_ok()})
    }

    //  Returns an Option containing the time of flight ball detection status (true = ball present)
    pub fn tof_ball_detected(&self) -> Option<bool> {
        self.status_hf.have(|status_hf| {status_hf.__bindgen_anon_1.tof_ball_detected() && status_hf.__bindgen_anon_1.tof_sensor_ok()})
    }

    //  Returns an Option containing the time of flight sensor ok status (false = sensor not functional)
    pub fn tof_sensor_ok(&self) -> Option<bool> {
        self.status_hf.have(|status_hf| {status_hf.__bindgen_anon_1.tof_sensor_ok()})
    }

    //  Returns an Option containing a pair of xy coordinates from the tof sensor. Note y is +forward/-back, x is -left/+right.
    //  Returns None if the tof sensor is not detecting a ball or inoperational
    pub fn tof_xy(&self) -> Option<(i8, u8)> {
        self.tof_ball_detected().filter(|&b| b).and_then(|_| {
            self.status_hf.have(|status_hf| {(status_hf.tof_ball_x, status_hf.tof_ball_y)})
        })
    }

    //  Returns an Option containing the main board current in Amperes
    pub fn main_board_current(&self) -> Option<f32> {
        self.status_lf.have(|status_lf| {
            status_lf.main_board_current as f32 * glue::Scale_CURRENT
        })
    }

    //  Returns an Option containing the average loop time in microseconds
    pub fn avg_loop_time(&self) -> Option<u32> {
        self.status_lf.have(|status_lf| {
            status_lf.avg_loop_time as u32 * 10
        })
    }

    //  Returns an Option containing the average loop time in microseconds
    pub fn max_loop_time(&self) -> Option<u32> {
        self.status_lf.have(|status_lf| {
            status_lf.max_loop_time as u32 * 10
        })
    }

    //  Returns an Option containing the average command time in milliseconds
    pub fn avg_command_time(&self) -> Option<u32> {
        self.status_lf.have(|status_lf| {
            status_lf.avg_command_time as u32
        })
    }

    //  Returns an Option containing the imu reading struct
    pub fn imu_reading(&self) -> Option<Radio_ImuReadings> {
        self.imu_reading.have(|imu| {imu})
    }

    pub fn breakbeam_raw(&self) -> Option<u16> {
        self.status_hf.have(|status_hf| {status_hf.breakbeam_raw})
    }

    // Returns an Option containing the left (0) and right (1) pack voltages in Volts
    pub fn pack_voltages(&self) -> Option<[f32; 2]> {
        self.status_lf.have(|status_lf| {
            status_lf.pack_voltages.map(|voltage| {
                voltage as f32 * glue::Scale_MD_BATV
            })
        })
    }

    

}
