use super::utils::Stamped;
use super::robot::*;
use super::serial::*;
use crate::glue::*;

pub const MAX_NUM_ROBOTS : usize = 16;

const DEBUG_SCROLLBACK_LIMIT : usize = 500;


pub struct Debug {
    pub incoming_lines : std::collections::vec_deque::VecDeque<(chrono::DateTime<chrono::Local>, String, String)>,
    pub imu_values : [std::collections::vec_deque::VecDeque<(chrono::DateTime<chrono::Local>, Radio_ImuReadings)>; MAX_NUM_ROBOTS],
    pub update : bool,
}

#[derive(Debug)]
pub struct BaseStation {

    pub robots: [Robot; MAX_NUM_ROBOTS],
    pub base_info: Stamped<Base_Information>,

    serial: Serial,
    start_time: std::time::Instant,
}

impl BaseStation {
    pub fn new(port_name : &str) -> Result<BaseStation, serialport::Error> {
        let serial = Serial::new(port_name)?;
        let start_time = std::time::Instant::now();
        Ok(BaseStation {
            robots: Default::default(),
            base_info: Stamped::NothingYet,
            serial,
            start_time,
        })
    }

    pub fn connection_time(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn read_and_parse(&mut self, debug : Option<&mut Debug>) -> Result<bool, ()> {
        // Parse contents of serial buffer
        let mut b = false;
        loop{
            if let Some(data) = self.serial.read_packet() {
                const LEN_BASE_INFORMATION : usize = std::mem::size_of::<crate::glue::Base_Information>();
                const LEN_MESSAGE_WRAPER : usize = std::mem::size_of::<crate::glue::Radio_MessageWrapper>();
                match data.len() {
                    LEN_BASE_INFORMATION => {
                        if let Some(base_info) = Base_Information::from_bytes(data) {
                            self.base_info.update(base_info);
                            if let Some(&mut ref mut dbg) = debug {
                                (*dbg).incoming_lines.push_front((chrono::Local::now(), format!("B"), format!("{:?}", base_info)));
                                (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                (*dbg).update = true;
                            }
                        }
                    },
                    LEN_MESSAGE_WRAPER => {
                        if let Some(msg) = Radio_MessageWrapper::from_bytes(data) {
                            if msg.id as usize >= MAX_NUM_ROBOTS { continue; } // Invalid robot id, continue to next frame
                            b = true;
                            match Radio_Message_Rust::unwrap(msg.msg) {
                                Radio_Message_Rust::PrimaryStatusHF(status_hf) => {
                                    self.robots[msg.id as usize].update_status_hf(status_hf);
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((chrono::Local::now(), format!("{}", msg.id), format!("{:?}", status_hf)));
                                        (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).update = true;
                                    }
                                }
                                Radio_Message_Rust::PrimaryStatusLF(status_lf) => {
                                    self.robots[msg.id as usize].update_status_lf(status_lf);
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((chrono::Local::now(), format!("{}", msg.id), format!("{:?}", status_lf)));
                                        (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).update = true;
                                    }
                                }
                                Radio_Message_Rust::ImuReadings(imu_reading) => {
                                    self.robots[msg.id as usize].update_imu_reading(imu_reading);
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((chrono::Local::now(), format!("{}", msg.id), format!("{:?}", imu_reading)));
                                        (*dbg).imu_values[msg.id as usize].push_front((chrono::Local::now(), imu_reading));
                                        (*dbg).imu_values[msg.id as usize].truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).update = true;
                                    }
                                }
                                _ => println!("Unhandled message type"),
                            }
                        }
                    },
                    _ => ()
                }
            } else {
                break;
            }
        }
        Ok(b)
    }

    
} // impl Monitor




pub struct Monitor {
    base_station_mux: std::sync::Arc<std::sync::Mutex<Option<BaseStation>>>,
    debug_mux : std::sync::Arc<std::sync::Mutex<Debug>>,
}

impl Monitor {

    // Get a mutex on the base station
    fn get_base_station_mux(&self) -> Option<std::sync::MutexGuard<'_, Option<BaseStation>>> {
        let time_start = std::time::Instant::now();
        loop {
            match self.base_station_mux.try_lock() {
                Ok(monitor_mut) => {
                    return Some(monitor_mut);
                }
                Err(_) => {
                    if time_start.elapsed() > std::time::Duration::from_millis(40) {
                        return None
                    }
                }
            }
        }
    }

    // Get a mutex on the debug strct
    pub fn get_debug_mux(&self) -> Option<std::sync::MutexGuard<'_, Debug>> {
        let time_start = std::time::Instant::now();
        loop {
            match self.debug_mux.try_lock() {
                Ok(monitor_mut) => {
                    return Some(monitor_mut);
                }
                Err(_) => {
                    if time_start.elapsed() > std::time::Duration::from_millis(40) {
                        return None
                    }
                }
            }
        }
    }

    // Start the background monitoring thread
    pub fn start() -> Self {
        let base_station_mux: std::sync::Arc<std::sync::Mutex<Option<BaseStation>>> = std::sync::Arc::new(std::sync::Mutex::new(None));
        let debug_mux: std::sync::Arc<std::sync::Mutex<Debug>> = std::sync::Arc::new(std::sync::Mutex::new(Debug{
            incoming_lines : Default::default(),
            imu_values : Default::default(),
            update : false,
        }));

        let mux_clone = std::sync::Arc::clone(&base_station_mux);
        let debug_mux_clone = std::sync::Arc::clone(&debug_mux);
        let _thread_join_handle = std::thread::spawn(move || {
            loop {
                { // monitor mutex
                    // let start_time = std::time::Instant::now();
                    let mut disconnect : bool = false;
                    let mut monitor_mut = mux_clone.lock().unwrap();
                    let mut debug_mut = debug_mux_clone.lock().unwrap();
                    if let Some(base_station) = &mut *monitor_mut {
                        let debug = &mut *debug_mut;
                        match base_station.read_and_parse(Some(debug)) {
                            Ok(_) => (),
                            Err(_) => disconnect = true,
                        };
                    }
                    if disconnect {
                        *monitor_mut = None;   
                    }
                    // println!("time = {:?}", start_time.elapsed());
                } // monitor mutex
                std::thread::sleep(std::time::Duration::from_secs(1) / 60); // Give other threads an opportunity to access the mutex
            }
        });
        Monitor {
            base_station_mux,
            debug_mux,
        }
    }

    

    // Get base station info
    pub fn get_base_info(&self) -> Stamped<Base_Information> {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            if let Some(bs) = & (**base_station) {
                return bs.base_info.clone();
            }
        }
        Stamped::NothingYet
    }

    // Get base station connection duration
    pub fn base_connection_duration(&self) -> Option<std::time::Duration> {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            if let Some(bs) = & (**base_station) {
                return Some(bs.connection_time())
            }
        }
        None
    }

    // Get robots, read only
    pub fn get_robots(&self) -> Option<[Robot; MAX_NUM_ROBOTS]> {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            if let Some(bs) = & (**base_station) {
                return Some(bs.robots.clone());
            }
        }
        None
    }

    // Send command to robot
    pub fn send(&self, commands : [Option<crate::glue::Radio_Command>; MAX_NUM_ROBOTS]) -> Result<(), ()> {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            if let Some(bs) = &mut **base_station {
                for i in 0..MAX_NUM_ROBOTS {
                    if let Some(command) = commands[i] {
                        match bs.serial.send_command(i as u8, command) {
                            Ok(_) => (),
                            Err(_) => return Err(()),
                        }
                    }
                }
            }
        }
        Err(())
    }


    // Connect to a base station over a serial COM port
    pub fn connect_to(&self, port: &str) -> Result<(),()> {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            **base_station = BaseStation::new(port).ok();
            if let Some(_) = & **base_station {
                return Ok(())
            }
        }
        Err(())
    }

    // Connect to the first found basestation
    pub fn connect_to_first(&self) -> Result<(),()> {
        let ports = Serial::list_ports(true);
        if ports.len() >= 1 {
            return self.connect_to(&ports[0]);
        }
        Err(())
    }

    pub fn is_connected(&self) -> bool {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            if let Some(_) = & **base_station {
                return true
            }
        }
        false
    }

    // Disconnect from connected base station (does nothing if not connected already)
    pub fn disconnect(&self) -> Result<(),()> {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            **base_station = None;
            return Ok(());
        }
        Err(())
    }
} // impl Monitor


#[cfg(test)]
mod basestation_tests {
    use super::*;

    // #[test]
    // #[allow(unused_assignments)]
    // fn connect() {
    //     println!();
    //     if let Ok(mut bs) = BaseStation::new(&Serial::list_ports(true)[0]) {
    //         std::thread::sleep(std::time::Duration::from_secs(2));
    //         for _ in 0..1 {
    //             std::thread::sleep(std::time::Duration::from_millis(100));
    //             let _ = bs.read_and_parse();
    //             println!("Succesfully connected!");
    //             if let Stamped::Have(timestamp, info) = bs.base_info {
    //                 println!("{:?} => {:?}", std::time::Instant::now() - timestamp, info);
    //             }
    //             if let Stamped::Have(timestamp, status) = bs.robots[4].status_lf {
    //                 println!("{:?} => {:?}", std::time::Instant::now() - timestamp, status);
    //             }
    //             if let Stamped::Have(timestamp, status) = bs.robots[4].status_hf {
    //                 println!("{:?} => {:?}", std::time::Instant::now() - timestamp, status);
    //             }
    //         }
    //     }else{
    //         println!("Failed to connect!");
    //     }
    //     println!();
    // }

    #[test]
    fn thread_it() {
        println!("Thread its");
        println!("Trying to connect");
        std::thread::sleep(std::time::Duration::from_millis(100));

        let monitor = Monitor::start();
        // match monitor.connect_to("COM50") {
        match monitor.connect_to_first() {
            Ok(_) => println!("Connected successfully"),
            Err(_) => {println!("Conection failed"); return},
        }
        
        for i in 0..100 {
            std::thread::sleep(std::time::Duration::from_millis(20)); // Allow thread to get stuff done
            println!("i = {i}");
            // if i % 10 == 0 {
                
            let mut commands = [None; MAX_NUM_ROBOTS];
            
            // for j in 0..3 {
            commands[i%10] = Some(crate::glue::Radio_Command {
                speed: crate::glue::HG_Pose {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                dribbler_speed: 2.0,
                kicker_command: Radio_KickerCommand::NONE,
                _pad: [0, 0, 0],
                kick_time: 1.0,
                fan_speed: 2.0,
            });
            // }

            // let _ = monitor.send(commands);
            // }
            
            

            // }
            if let Stamped::Have(timestamp, info) = monitor.get_base_info() {
                println!("{:?} => {:?}", timestamp.elapsed(), info);
            }
            if let Some(robots) = monitor.get_robots() {
                if let Some(timestamp,) = robots[4].time_since_status_lf_update() {
                    println!("{:?} => status_lf", timestamp);
                }
                if let Some(timestamp) = robots[4].time_since_status_hf_update() {
                    println!("{:?} => status_hf", timestamp);
                }
                if let Some(timestamp) = robots[4].time_since_imu_reading_update() {
                    println!("{:?} => imu_reading", timestamp);
                }
            }
            println!();
        }

        let _ = monitor.disconnect();
    }

}