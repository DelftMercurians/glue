use serialport::SerialPort;
use std::time::Duration;
use std::io;
// use crate::glue::*;

const SERIAL_BUF_LEN: usize = 100000;

#[derive(Debug)]
pub struct Serial {
    port : Box<dyn SerialPort>,
    mirror : Option<Box<dyn SerialPort>>,
    serial_buf: Vec<u8>,
    glob_index: usize,
}

impl Serial {
    pub fn new(port_name : &str) -> Result<Serial, serialport::Error> {
        Ok(Serial {
            port: serialport::new(port_name, 115200)
                            .timeout(Duration::from_millis(10))
                            .open()?,
            mirror: None,
            serial_buf: vec![0; SERIAL_BUF_LEN],
            glob_index: 0,
        }.set_dtr())
    }

    fn set_dtr(mut self) -> Self {
        let _ = self.port.write_data_terminal_ready(true);
        self
    }

    pub fn open_mirror(&mut self, port_name : &str) -> Result<(), serialport::Error> {
        self.mirror = Some(serialport::new(port_name, 115200)
                            .timeout(Duration::from_millis(10))
                            .open()?);
        Ok(())
    }

    pub fn close_mirror(&mut self) {
        self.mirror = None;
    }

    pub fn is_mirror_connected(&self) -> bool {
        self.mirror.is_some()
    }

    fn is_basestation(port : &serialport::SerialPortInfo) -> bool {
        match &port.port_type {
            serialport::SerialPortType::UsbPort(info) => {
                (info.vid == 0x0483) && (info.pid == 0x5740)
            },
            _ => false
        }
    }

    fn is_virtual(port : &serialport::SerialPortInfo) -> bool {
        match &port.port_type {
            serialport::SerialPortType::Unknown => {
                true
            },
            _ => false
        }
    }

    #[cfg(target_os = "windows")]
    pub fn list_ports(filter : bool) -> Vec<String> {
        let ports = serialport::available_ports().expect("No ports found!");
        ports.into_iter().filter(|p| !filter || Self::is_basestation(&p)).map(|p| p.port_name).collect()
    }

    #[cfg(target_os = "linux")]
    pub fn list_ports(filter : bool) -> Vec<String> {
        let ports = serialport::available_ports().expect("No ports found!");
        let mut a : Vec<String>= ports.into_iter().filter(|p| !filter || Self::is_basestation(&p)).map(|p| p.port_name).collect();
        if !filter {
            let mut b : Vec<String> = std::fs::read_dir("/dev/pts").unwrap().into_iter().map(|path| path.unwrap().path().display().to_string()).collect();
            a.append(&mut b);
        }
        a.sort();
        a
    }

    #[cfg(target_os = "linux")]
    pub fn list_unknown_ports() -> Vec<String> {
        let ports = serialport::available_ports().expect("No ports found!");
        let mut a : Vec<String>= ports.into_iter().filter(|p| Self::is_virtual(&p)).map(|p| p.port_name).collect();
        let mut b : Vec<String> = std::fs::read_dir("/dev/pts").unwrap().into_iter().map(|path| path.unwrap().path().display().to_string()).collect();
        a.append(&mut b);
        a.sort();
        a
    }

    #[cfg(target_os = "windows")]
    pub fn list_unknown_ports() -> Vec<String> {
        let ports = serialport::available_ports().expect("No ports found!");
        ports.into_iter().filter(|p| Self::is_virtual(&p)).map(|p| p.port_name).collect()
    }

    pub fn send(&mut self, line : &str) {
        match self.port.write_all(line.as_bytes()) {
            Ok(()) => (),
            Err(e) => eprintln!("{:?}", e),
        };
    }

    pub fn send_command(&mut self, id : crate::glue::Radio_SSL_ID , command : crate::glue::Radio_Command) -> Result<(), std::io::Error> {
        let msg = crate::glue::Radio_Message_Rust::Command(command).wrap();
        let mw = crate::glue::Radio_MessageWrapper {
            id,
            _pad: [0, 0, 0],
            msg,
        };
        let bytes = crate::glue::to_packet(mw);

        self.port.write_all(&bytes)
    }

    pub fn send_mcm(&mut self, id : crate::glue::Radio_SSL_ID , mcm : crate::glue::Radio_MultiConfigMessage) -> Result<(), std::io::Error> {
        let msg = crate::glue::Radio_Message_Rust::MultiConfigMessage(mcm).wrap();
        let mw = crate::glue::Radio_MessageWrapper {
            id,
            _pad: [0, 0, 0],
            msg,
        };
        let bytes = crate::glue::to_packet(mw);

        self.port.write_all(&bytes)
    }

    pub fn send_over_odo(&mut self, id : crate::glue::Radio_SSL_ID , over_odo : crate::glue::Radio_OverrideOdometry) -> Result<(), std::io::Error> {
        let msg = crate::glue::Radio_Message_Rust::OverrideOdometry(over_odo).wrap();
        let mw = crate::glue::Radio_MessageWrapper {
            id,
            _pad: [0, 0, 0],
            msg,
        };
        let bytes = crate::glue::to_packet(mw);

        self.port.write_all(&bytes)
    }

    fn read(&mut self) -> Result<(), ()> {
        // Drop everything coming in on the mirror port
        if let Some(mirror) = &mut self.mirror {
            match mirror.bytes_to_read() {
                Ok(0) => (),
                Ok(_) => {
                    let mut drop_buff = [0; 256];
                    let _ = mirror.read(&mut drop_buff);
                },
                Err(_) => (),
            }
        }
        match self.port.bytes_to_read() {
            Ok(0) => return Ok(()),
            Ok(_) => (),
            Err(_) => return Err(()),
        }
        match self.port.read(&mut self.serial_buf[self.glob_index..]) {
            Ok(length) => {
                // Transmit everything on the mirror port
                if self.check_carrier_detect() {
                    if let Some(mirror) = &mut self.mirror {
                        let mut buffer_copy: Vec<u8> = vec![0; length];
                        buffer_copy.copy_from_slice(&self.serial_buf[self.glob_index..self.glob_index+length]);
                        let _ = mirror.write(&buffer_copy);
                    }
                }
                self.glob_index += length;
                if self.glob_index == SERIAL_BUF_LEN {
                    panic!("Buffer is full");   // TODO figure out what to do here
                }
                return Ok(())
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => eprintln!("{:?}", e),
            Err(e) => eprintln!("{:?}", e),
        }
        return Err(())
    }

    #[cfg(target_os = "linux")]
    fn check_carrier_detect(&mut self) -> bool {
        true
    }

    #[cfg(target_os = "windows")]
    fn check_carrier_detect(&mut self) -> bool {
        if let Some(mirror) = &mut self.mirror {
            if let Ok(b) = mirror.read_carrier_detect() {
                if b {
                    return true;
                }
            }
        }
        false
    }

    pub fn read_packet(&mut self) -> Option<Vec<u8>> {
        self.read().ok()?;
        if let Some(index) = self.serial_buf[..self.glob_index].iter().position(|&x| x == 0b10100101) {
            // Start byte found

            if self.glob_index < index + 2 {
                // println!("Buffer not long enough to get a size");
                return None;
            }
            
            // Length of data
            let len: usize = self.serial_buf[index+1] as usize;

            if self.glob_index < index + len + 3 {
                // println!("Buffer not long enough for all data len = {len}");
                return None;
            }

            if len > 50 {
                eprintln!("SERIAL: Bad packet: size too large");
                self.serial_buf.rotate_left(index + 1); // probably quite expensive, consider a circular buffer
                self.glob_index -= index + 1;
                return None;
            }

            let data = self.serial_buf[(index+2)..(index+2+len)].to_vec();

            let crc = self.serial_buf[index + 2 + len];

            // println!("data = {data:02X?}");
            // println!("crc = {crc:#02X}");

            if crc != crate::glue::crc_calc.checksum(&data) {
                // Bad packet, skip this start of packet indicator
                eprintln!("SERIAL: Bad packet: CRC failed");
                self.serial_buf.rotate_left(index + 1);
                self.glob_index -= index + 1;
                return None;
            }

            self.serial_buf.rotate_left(index + 3 + len);
            self.glob_index -= index + 3 + len;

            return Some(data);
        } else {
            // No start byte, keep rotating buffer
            self.serial_buf.rotate_left(self.glob_index);
            self.glob_index -= self.glob_index;
        }
        None
    }
}

#[cfg(test)]
mod serial_tests {
    use super::*;

    #[test]
    fn serial() {
        println!("{:?}", Serial::list_ports(true));
        println!("{:?}", Serial::list_ports(false));
    }
}
