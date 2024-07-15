use super::robot::*;
use super::serial::*;
use super::utils::Stamped;
use crate::glue::*;

use std::num::NonZeroUsize;

pub const MAX_NUM_ROBOTS: usize = 16;

const DEBUG_SCROLLBACK_LIMIT: usize = 500;

pub struct Debug {
    pub incoming_lines:
        std::collections::vec_deque::VecDeque<(chrono::DateTime<chrono::Local>, String, String)>,
    pub imu_values: [std::collections::vec_deque::VecDeque<(
        chrono::DateTime<chrono::Local>,
        Radio_ImuReadings,
    )>; MAX_NUM_ROBOTS],
    pub odo_values: [std::collections::vec_deque::VecDeque<(
        chrono::DateTime<chrono::Local>,
        Radio_OdometryReading,
    )>; MAX_NUM_ROBOTS],
    pub config_variable_returns: [[Stamped<u32>; 256]; MAX_NUM_ROBOTS],
    pub update: bool,
}

#[derive(Debug)]
pub struct BaseStation {
    pub robots: [Robot; MAX_NUM_ROBOTS],
    pub base_info: Stamped<Base_Information>,

    serial: Serial,
    start_time: std::time::Instant,
}

impl BaseStation {
    pub fn new(port_name: &str) -> Result<BaseStation, serialport::Error> {
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

    pub fn read_and_parse(&mut self, debug: Option<&mut Debug>) -> Result<(bool, bool), ()> {
        // Parse contents of serial buffer
        let mut update_robots = false;
        let mut update_base_info = false;
        loop {
            if let Some(data) = self.serial.read_packet() {
                const LEN_BASE_INFORMATION: usize =
                    std::mem::size_of::<crate::glue::Base_Information>();
                const LEN_MESSAGE_WRAPER: usize =
                    std::mem::size_of::<crate::glue::Radio_MessageWrapper>();
                match data.len() {
                    LEN_BASE_INFORMATION => {
                        if let Some(base_info) = Base_Information::from_bytes(data) {
                            self.base_info.update(base_info);
                            update_base_info = true;
                            if let Some(&mut ref mut dbg) = debug {
                                (*dbg).incoming_lines.push_front((
                                    chrono::Local::now(),
                                    format!("B"),
                                    format!("{:?}", base_info),
                                ));
                                (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                (*dbg).update = true;
                            }
                        }
                    }
                    LEN_MESSAGE_WRAPER => {
                        if let Some(msg) = Radio_MessageWrapper::from_bytes(data) {
                            if msg.id as usize >= MAX_NUM_ROBOTS {
                                continue;
                            } // Invalid robot id, continue to next frame
                            match Radio_Message_Rust::unwrap(msg.msg) {
                                Radio_Message_Rust::PrimaryStatusHF(status_hf) => {
                                    self.robots[msg.id as usize].update_status_hf(status_hf);
                                    update_robots = true;
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((
                                            chrono::Local::now(),
                                            format!("{}", msg.id),
                                            format!("{:?}", status_hf),
                                        ));
                                        (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).update = true;
                                    }
                                }
                                Radio_Message_Rust::PrimaryStatusLF(status_lf) => {
                                    self.robots[msg.id as usize].update_status_lf(status_lf);
                                    update_robots = true;
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((
                                            chrono::Local::now(),
                                            format!("{}", msg.id),
                                            format!("{:?}", status_lf),
                                        ));
                                        (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).update = true;
                                    }
                                }
                                Radio_Message_Rust::ImuReadings(imu_reading) => {
                                    self.robots[msg.id as usize].update_imu_reading(imu_reading);
                                    update_robots = true;
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((
                                            chrono::Local::now(),
                                            format!("{}", msg.id),
                                            format!("{:?}", imu_reading),
                                        ));
                                        (*dbg).imu_values[msg.id as usize]
                                            .push_front((chrono::Local::now(), imu_reading));
                                        (*dbg).imu_values[msg.id as usize]
                                            .truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).update = true;
                                    }
                                }
                                Radio_Message_Rust::OdometryReading(odo_reading) => {
                                    // self.robots[msg.id as usize].update_odo_reading(odo_reading);
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((
                                            chrono::Local::now(),
                                            format!("{}", msg.id),
                                            format!("{:?}", odo_reading),
                                        ));
                                        (*dbg).odo_values[msg.id as usize]
                                            .push_front((chrono::Local::now(), odo_reading));
                                        (*dbg).odo_values[msg.id as usize]
                                            .truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).update = true;
                                    }
                                }
                                Radio_Message_Rust::OverrideOdometry(over_odo) => {
                                    // self.robots[msg.id as usize].update_odo_reading(odo_reading);
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((
                                            chrono::Local::now(),
                                            format!("{}", msg.id),
                                            format!("{:?}", over_odo),
                                        ));
                                        (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).update = true;
                                    }
                                }
                                Radio_Message_Rust::MultiConfigMessage(mcm) => {
                                    // self.robots[msg.id as usize].update_odo_reading(odo_reading);
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((
                                            chrono::Local::now(),
                                            format!("{}", msg.id),
                                            format!("{:?}", mcm),
                                        ));
                                        (*dbg).incoming_lines.truncate(DEBUG_SCROLLBACK_LIMIT);
                                        (*dbg).update = true;

                                        match mcm.operation {
                                            HG_ConfigOperation::READ_RETURN
                                            | HG_ConfigOperation::WRITE_RETURN
                                            | HG_ConfigOperation::SET_DEFAULT_RETURN => {
                                                for i in 0..5 {
                                                    if mcm.vars[i] == HG_Variable::NONE {
                                                        continue;
                                                    }
                                                    (*dbg).config_variable_returns
                                                        [msg.id as usize]
                                                        [mcm.vars[i] as usize] =
                                                        Stamped::make_now(mcm.values[i]);
                                                }
                                            }
                                            _ => (),
                                        }
                                    }
                                }
                                _ => {
                                    if let Some(&mut ref mut dbg) = debug {
                                        (*dbg).incoming_lines.push_front((
                                            chrono::Local::now(),
                                            format!("{}", msg.id),
                                            format!("Unknown Message Type"),
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        if let Some(&mut ref mut dbg) = debug {
                            (*dbg).incoming_lines.push_front((
                                chrono::Local::now(),
                                format!("?"),
                                format!("Unknown Data: {:?}", String::from_utf8(data)),
                            ));
                        }
                    }
                }
            } else {
                break;
            }
        }
        Ok((update_robots, update_base_info))
    }
} // impl Monitor

pub struct Monitor {
    base_station_mux: std::sync::Arc<std::sync::Mutex<Option<BaseStation>>>,
    debug_mux: std::sync::Arc<std::sync::Mutex<Debug>>,
    stop_channel: std::sync::mpsc::Sender<()>,
    send_command_channel: std::sync::mpsc::Sender<(Radio_SSL_ID, Radio_Command)>,
    send_message_channel: std::sync::mpsc::Sender<(Radio_SSL_ID, Radio_Message_Rust)>,

    robot_status_channel: ring_channel::RingReceiver<[Robot; MAX_NUM_ROBOTS]>,
    most_recent_robot_status: [Robot; MAX_NUM_ROBOTS],

    base_station_info_channel: ring_channel::RingReceiver<Stamped<Base_Information>>,
    most_recent_base_station_info: Stamped<Base_Information>,

    // con_rq: ring_channel::RingSender<String>,
    // con_rq_ack: ring_channel::RingReceiver<String>,

    bs_connected: ring_channel::RingReceiver<bool>,
    most_recent_bs_connected: bool,
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
                        return None;
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
                        return None;
                    }
                }
            }
        }
    }

    // Start the background monitoring thread
    pub fn start() -> Self {
        let base_station_mux: std::sync::Arc<std::sync::Mutex<Option<BaseStation>>> =
            std::sync::Arc::new(std::sync::Mutex::new(None));
        let debug_mux: std::sync::Arc<std::sync::Mutex<Debug>> =
            std::sync::Arc::new(std::sync::Mutex::new(Debug {
                incoming_lines: Default::default(),
                imu_values: Default::default(),
                odo_values: Default::default(),
                config_variable_returns: [[Stamped::NothingYet; 256]; MAX_NUM_ROBOTS],
                update: false,
            }));

        let mux_clone = std::sync::Arc::clone(&base_station_mux);
        let debug_mux_clone = std::sync::Arc::clone(&debug_mux);
        let (stop_channel, stop_receiver) = std::sync::mpsc::channel();
        let (send_command_channel, command_receiver) = std::sync::mpsc::channel();
        let (send_message_channel, message_receiver) = std::sync::mpsc::channel();
        
        let (robot_status_sender, robot_status_channel) = ring_channel::ring_channel(NonZeroUsize::new(1).unwrap());
        let (base_station_info_sender, base_station_info_channel) = ring_channel::ring_channel(NonZeroUsize::new(1).unwrap());
            
        let (bs_connected_sender, bs_connected) = ring_channel::ring_channel(NonZeroUsize::new(1).unwrap());
        
        // let (con_rq, con_rq_rec) = ring_channel::ring_channel(NonZeroUsize::new(1).unwrap());
        // let (con_rq_ack_send, con_rq_ack) = ring_channel::ring_channel(NonZeroUsize::new(1).unwrap());
        
        let _thread_join_handle = std::thread::spawn(move || {
            loop {
                {
                    if stop_receiver.try_recv().is_ok() {
                        break;
                    }
                    // monitor mutex
                    // let start_time = std::time::Instant::now();
                    let mut disconnect: bool = false;
                    let mut monitor_mut = mux_clone.lock().unwrap();
                    let mut debug_mut = debug_mux_clone.lock().unwrap();
                    if let Some(base_station) = &mut *monitor_mut {
                        let _ = bs_connected_sender.send(true);
                        let debug = &mut *debug_mut;
                        match base_station.read_and_parse(Some(debug)) {
                            Ok((update_robots, update_base_info)) => {
                                if update_robots {
                                    let _ = robot_status_sender.send(base_station.robots);
                                }
                                if update_base_info {
                                    let _ = base_station_info_sender.send(base_station.base_info);
                                }
                            },
                            Err(_) => disconnect = true,
                        };
                        for _ in 0..2 { // Limit how often this can run
                            match command_receiver.try_recv() {
                                Ok((id, command)) => {
                                    match base_station.serial.send_command(id,command) {
                                        Ok(_) => (),
                                        Err(_) => println!("Error transmitting command"),
                                    }
                                }
                                Err(std::sync::mpsc::TryRecvError::Disconnected) => { break; }
                                Err(std::sync::mpsc::TryRecvError::Empty) => { break; }
                            }
                        }
                        match message_receiver.try_recv() {
                            Ok((id, msg)) => {
                                match base_station.serial.send_message(id,msg) {
                                    Ok(_) => (),
                                    Err(_) => println!("Error transmitting message"),
                                }
                            }
                            Err(std::sync::mpsc::TryRecvError::Disconnected) => (),
                            Err(std::sync::mpsc::TryRecvError::Empty) => (),
                        }
                        let time_start = std::time::Instant::now();
                        while time_start.elapsed() < std::time::Duration::from_millis(15) {  }  // Blocking sleep alternative
                        // std::thread::sleep(std::time::Duration::from_micros(1)); // Give other threads an opportunity to access the mutex
                    } else {
                        let _ = bs_connected_sender.send(false);
                    }
                    if disconnect {
                        *monitor_mut = None;
                    }
                    // println!("time = {:?}", start_time.elapsed());
                    
                } // monitor mutex
            }
        });
        Monitor {
            base_station_mux,
            debug_mux,
            stop_channel,
            send_command_channel,
            send_message_channel,
            robot_status_channel,
            most_recent_robot_status: [Robot::default(); MAX_NUM_ROBOTS],
            base_station_info_channel,
            most_recent_base_station_info: Stamped::NothingYet,
            bs_connected,
            most_recent_bs_connected: false,
            // con_rq,
            // con_rq_ack,
        }
    }

    /// Stop the monitor thread
    pub fn stop(self) {
        let _ = self.stop_channel.send(());
    }

    // Get base station info
    pub fn get_base_info(&mut self) -> Stamped<Base_Information> {
        if let Ok(fresh_base_info) = self.base_station_info_channel.try_recv() {
            self.most_recent_base_station_info = fresh_base_info;
        }
        self.most_recent_base_station_info
    }

    // Get base station connection duration
    pub fn base_connection_duration(&self) -> Option<std::time::Duration> {
        // if let Some(base_station) = &mut self.get_base_station_mux() {
        //     if let Some(bs) = &(**base_station) {
        //         return Some(bs.connection_time());
        //     }
        // }
        None
    }

    // Get robots, read only
    pub fn get_robots(&mut self) -> Option<[Robot; MAX_NUM_ROBOTS]> {
        if let Ok(robots) = self.robot_status_channel.try_recv() {
            self.most_recent_robot_status = robots;
        }
        Some(self.most_recent_robot_status)
    }

    // Send command to robot
    pub fn send(
        &self,
        commands: [Option<crate::glue::Radio_Command>; MAX_NUM_ROBOTS],
    ) -> Result<(), ()> {
        for i in 0..MAX_NUM_ROBOTS {
            if let Some(command) = commands[i] {
                self.send_single(i as u8, command)?;
            }
        }
        Ok(())
    }

    // Send command to single robot
    pub fn send_single(
        &self,
        id: crate::glue::Radio_SSL_ID,
        command: crate::glue::Radio_Command,
    ) -> Result<(), ()> {
        self.send_command_channel.send((id, command)).map_err(|_| ())?;
        Ok(())
    }

    // Send command to all robots
    pub fn send_broadcast(
        &self,
        command : crate::glue::Radio_Command,
    ) -> Result<(), ()> {
        self.send_command_channel.send((Radio_Broadcast_ID, command)).map_err(|_| ())?;
        Ok(())
    }

    pub fn send_mcm(
        &self,
        id: crate::glue::Radio_SSL_ID,
        mcm: crate::glue::Radio_MultiConfigMessage,
    ) -> Result<(), ()> {
        self.send_message_channel.send((id, Radio_Message_Rust::MultiConfigMessage(mcm))).map_err(|_| ())?;
        Ok(())
    }

    pub fn set_channel(
        &self,
        chan : u8,
    ) -> Result<(), ()> {
        let mcm = crate::glue::Radio_MultiConfigMessage{
            operation: crate::glue::HG_ConfigOperation::WRITE,
            vars: [HG_Variable::RADIO_CHANNEL, HG_Variable::NONE, HG_Variable::NONE, HG_Variable::NONE, HG_Variable::NONE],
            type_: HG_VariableType::VOID,
            _pad: 0,
            values: [chan as u32, 0, 0, 0, 0],
        };
        self.send_mcm(crate::glue::Radio_BaseStation_ID, mcm)
    }

    // Send odometry override
    pub fn send_over_odo(
        &self,
        id: u8,
        over_odo: crate::glue::Radio_OverrideOdometry,
    ) -> Result<(), ()> {
        self.send_message_channel.send((id, Radio_Message_Rust::OverrideOdometry(over_odo))).map_err(|_| ())?;
        Ok(())
    }

    // Connect to a base station over a serial COM port
    pub fn connect_to(&self, port: &str) -> Result<(), ()> {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            **base_station = BaseStation::new(port)
                .map_err(|e| eprintln!("Error connecting to {}: {:?}", port, e))
                .ok();
            if let Some(_) = &**base_station {
                return Ok(());
            }
        }
        Err(())
    }

    pub fn connect_to_mirror(&self, port: &str) -> Result<(), ()> {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            if let Some(bs) = &mut **base_station {
                bs.serial.open_mirror(port).map_err(|e| eprintln!("Error connecting to mirror {}: {:?}", port, e))?;
                return Ok(());
            }
        }
        Err(())
    }

    pub fn disconnect_mirror(&self) {
        if let Some(base_station) = &mut self.get_base_station_mux() {
            if let Some(bs) = &mut **base_station {
                bs.serial.close_mirror();
            }
        }
    }

    // Connect to the first found basestation
    pub fn connect_to_first(&self) -> Result<(), ()> {
        let ports = Serial::list_ports(true);
        if ports.len() >= 1 {
            return self.connect_to(&ports[0]);
        }
        Err(())
    }

    pub fn is_connected(&mut self) -> bool {
        match self.bs_connected.try_recv() {
            Ok(val) => {
                self.most_recent_bs_connected = val;
            }
            Err(_) => ()
        }
        self.most_recent_bs_connected
    }

    pub fn is_connected_to_mirror(&self) -> bool {
        // if let Some(base_station) = &self.get_base_station_mux() {
        //     if let Some(bs) = &**base_station {
        //         return bs.serial.is_mirror_connected();
        //     }
        // }
        false
    }

    // Disconnect from connected base station (does nothing if not connected already)
    pub fn disconnect(&self) -> Result<(), ()> {
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

        let mut monitor = Monitor::start();
        // match monitor.connect_to("COM50") {
        match monitor.connect_to_first() {
            Ok(_) => println!("Connected successfully"),
            Err(_) => {
                println!("Conection failed");
                return;
            }
        }

        for i in 0..100 {
            std::thread::sleep(std::time::Duration::from_millis(20)); // Allow thread to get stuff done
            println!("i = {i}");
            // if i % 10 == 0 {

            let mut commands = [None; MAX_NUM_ROBOTS];

            // for j in 0..3 {
            commands[i % 10] = Some(crate::glue::Radio_Command {
                speed: crate::glue::HG_Pose {
                    x: 0.0,
                    y: 0.0,
                    z: 0.0,
                },
                dribbler_speed: 2.0,
                robot_command: Radio_RobotCommand::NONE,
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
                if let Some(timestamp) = robots[4].time_since_status_lf_update() {
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
