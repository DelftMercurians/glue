use serialport::SerialPort;
use std::time::Duration;
use std::io;
// use crate::glue::*;

const SERIAL_BUF_LEN: usize = 100000;

#[derive(Debug)]
pub struct Serial {
    port : Box<dyn SerialPort>,
    serial_buf: Vec<u8>,
    glob_index: usize,
}

impl Serial {
    pub fn new(port_name : &str) -> Result<Serial, serialport::Error> {
        Ok(Serial {
            port: serialport::new(port_name, 115200)
                            .timeout(Duration::from_millis(10))
                            .open()?,
            serial_buf: vec![0; SERIAL_BUF_LEN],
            glob_index: 0,
        })
    }

    fn is_basestation(port : &serialport::SerialPortInfo) -> bool {
        match &port.port_type {
            serialport::SerialPortType::UsbPort(info) => {
                (info.vid == 0x0483) && (info.pid == 0x5740)
            },
            _ => false
        }
    }

    pub fn list_ports(filter : bool) -> Vec<String> {
        let ports = serialport::available_ports().expect("No ports found!");
        // for p in &ports {
        //     println!("  {}", p.port_name);
        //     match &p.port_type {
        //         SerialPortType::UsbPort(info) => {
        //             println!("    Type: USB");
        //             println!("    VID:{:04x} PID:{:04x}", info.vid, info.pid);
        //             println!("     Serial Number: {}",
        //                 info.serial_number.as_ref().map_or("", String::as_str)
        //             );
        //             println!(
        //                 "      Manufacturer: {}",
        //                 info.manufacturer.as_ref().map_or("", String::as_str)
        //             );
        //             println!(
        //                 "           Product: {}",
        //                 info.product.as_ref().map_or("", String::as_str)
        //             );
        //             #[cfg(feature = "usbportinfo-interface")]
        //             println!(
        //                 "         Interface: {}",
        //                 info.interface
        //                     .as_ref()
        //                     .map_or("".to_string(), |x| format!("{:02x}", *x))
        //             );
        //         }
        //         SerialPortType::BluetoothPort => {
        //             println!("    Type: Bluetooth");
        //         }
        //         SerialPortType::PciPort => {
        //             println!("    Type: PCI");
        //         }
        //         SerialPortType::Unknown => {
        //             println!("    Type: Unknown");
        //         }
        //     }
        // }
        ports.into_iter().filter(|p| !filter || Self::is_basestation(&p)).map(|p| p.port_name).collect()
    }

    pub fn send(&mut self, line : &str) {
        match self.port.write_all(line.as_bytes()) {
            Ok(()) => (),
            Err(e) => eprintln!("{:?}", e),
        };
    }

    pub fn send_command(&mut self, id : crate::glue::Radio_SSL_ID , command : crate::glue::Radio_Command) -> Result<(), std::io::Error> {
        let msg = crate::glue::Radio_Message_Rust::Radio_Command(command).wrap();
        let mw = crate::glue::Radio_MessageWrapper {
            id,
            _pad: [0, 0, 0],
            msg,
        };
        let bytes = crate::glue::to_packet(mw);

        self.port.write_all(&bytes)
    }

    fn read(&mut self) -> Result<(), ()> {
        match self.port.bytes_to_read() {
            Ok(0) => return Ok(()),
            Ok(_) => (),
            Err(_) => return Err(()),
        }
        match self.port.read(&mut self.serial_buf[self.glob_index..]) {
            Ok(length) => {
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

    pub fn read_packet(&mut self) -> Option<Vec<u8>> {
        self.read().ok()?;
        if let Some(index) = self.serial_buf[..self.glob_index].iter().position(|&x| x == 0b10100101) {
            // Start byte found

            if self.glob_index < index + 2 {
                println!("Buffer not long enough to get a size");
                return None;
            }
            
            // Length of data
            let len: usize = self.serial_buf[index+1] as usize;

            if self.glob_index < index + len + 3 {
                println!("Buffer not long enough for all data len = {len}");
                return None;
            }

            if len > 50 {
                println!("Bad packet: size too large");
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
                println!("Bad packet: CRC failed");
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
