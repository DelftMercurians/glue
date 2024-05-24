use super::utils::Stamped;
use super::robot::*;
use super::serial::*;
use crate::glue::*;

pub const MAX_NUM_ROBOTS : usize = 16;

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

    pub fn read_and_parse(&mut self) -> Result<bool, ()> {
        // Parse contents of serial buffer
        let mut b = false;
        loop{
            if let Some(data) = self.serial.read_packet() {
                const LEN_BASE_INFORMATION : usize = std::mem::size_of::<crate::glue::Base_Information>();
                const LEN_MESSAGE_WRAPER : usize = std::mem::size_of::<crate::glue::Radio_MessageWrapper>();
                match data.len() {
                    LEN_BASE_INFORMATION => {
                        if let Some(base_info) = crate::glue::to_base_info(data.try_into().expect("ERR"), None) {
                            self.base_info.update(base_info);
                        }
                    },
                    LEN_MESSAGE_WRAPER => {
                        if let Some((id, message)) = crate::glue::to_radio_message_wrapper(data.try_into().expect("ERR"), None) {
                            b = true;
                            if let Some(status_hf) = crate::glue::extract_radio_status_hf(message) {
                                self.robots[id as usize].update_status_hf(status_hf);
                            }
                            if let Some(status_lf) = crate::glue::extract_radio_status_lf(message) {  
                                self.robots[id as usize].update_status_lf(status_lf);
                            }
                            if let Some(imu_reading) = crate::glue::extract_imu(message) {
                                self.robots[id as usize].update_imu_reading(imu_reading);
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
    pub base_station_mux: std::sync::Arc<std::sync::Mutex<Option<BaseStation>>>,
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

    // Start the background monitoring thread
    pub fn start() -> Self {
        let base_station_mux: std::sync::Arc<std::sync::Mutex<Option<BaseStation>>> = std::sync::Arc::new(std::sync::Mutex::new(None));

        let mux_clone = std::sync::Arc::clone(&base_station_mux);
        let _thread_join_handle = std::thread::spawn(move || {
            loop {
                { // monitor mutex
                    let start_time = std::time::Instant::now();
                    let mut disconnect : bool = false;
                    let mut monitor_mut = mux_clone.lock().unwrap();
                    if let Some(base_station) = &mut *monitor_mut {
                        match base_station.read_and_parse() {
                            Ok(_) => (),
                            Err(_) => disconnect = true,
                        };
                    }
                    if disconnect {
                        *monitor_mut = None;   
                    }
                    println!("time = {:?}", start_time.elapsed());
                } // monitor mutex
                std::thread::sleep(std::time::Duration::from_secs(1) / 60); // Give other threads an opportunity to access the mutex
            }
        });
        Monitor {
            base_station_mux,
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
            if let Some(bs) = &mut **base_station {
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
        match monitor.connect_to("COM50") {
        // match monitor.connect_to_first() {
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
                kicker_command: b'T',
                _pad: [0, 0, 0],
                kick_time: 1.0,
                fan_speed: 2.0,
            });
            // }

            let _ = monitor.send(commands);
            // }
            
            

            // }
            if let Stamped::Have(timestamp, info) = monitor.get_base_info() {
                println!("{:?} => {:?}", timestamp.elapsed(), info);
            }
            if let Some(robots) = monitor.get_robots() {
                if let Stamped::Have(timestamp, status) = robots[4].status_lf {
                    println!("{:?} => {:?}", timestamp.elapsed(), status);
                }
                if let Stamped::Have(timestamp, status) = robots[4].status_hf {
                    println!("{:?} => {:?}", timestamp.elapsed(), status);
                }
                if let Stamped::Have(timestamp, imu_reading) = robots[4].imu_reading {
                    println!("{:?} => {:?}", timestamp.elapsed(), imu_reading);
                }
            }
            println!();
        }

        let _ = monitor.disconnect();
    }

}