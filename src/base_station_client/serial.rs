use serialport::SerialPort;
use std::io;
use std::time::Duration;
// use crate::glue::*;

const SERIAL_BUF_LEN: usize = 100000;
/// Start-of-frame marker byte.
const START_BYTE: u8 = 0b10100101;
/// Largest payload length accepted in a single frame.
const MAX_PACKET_DATA_LEN: usize = 50;

#[derive(Debug)]
pub struct Serial {
    port : Box<dyn SerialPort>,
    mirror : Option<Box<dyn SerialPort>>,
    serial_buf: Vec<u8>,
    glob_index: usize,
}

impl Serial {
    pub fn new(port_name: &str) -> Result<Serial, serialport::Error> {
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

    pub fn open_mirror(&mut self, _port_name : &str) -> Result<(), serialport::Error> {
        // self.mirror = Some(serialport::new(port_name, 115200)
        //                     .timeout(Duration::from_millis(10))
        //                     .open()?);
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
                ((info.vid == 0x0483) && (info.pid == 0x5740))
                    || ((info.vid == 0xC0DE) && (info.pid == 0xCAFE))
            }
            _ => false,
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

    #[cfg(target_os = "macos")]
    pub fn list_ports(filter : bool) -> Vec<String> {
        let ports = serialport::available_ports().expect("No ports found!");
        ports.into_iter().filter(|p| !filter || Self::is_basestation(&p)).map(|p| p.port_name).collect()
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

    #[cfg(target_os = "macos")]
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

    pub fn send_command(
        &mut self,
        id: crate::glue::Radio_SSL_ID,
        command: crate::glue::Radio_Command,
    ) -> Result<(), std::io::Error> {
        let msg = crate::glue::Radio_Message_Rust::Command(command).wrap();
        let mw = crate::glue::Radio_MessageWrapper {
            id,
            _pad: [0, 0, 0],
            msg,
        };
        let bytes = crate::glue::to_packet(mw);

        self.port.write_all(&bytes)
    }

    pub fn send_global_command(
        &mut self,
        id: crate::glue::Radio_SSL_ID,
        command: crate::glue::Radio_GlobalCommand,
    ) -> Result<(), std::io::Error> {
        let msg = crate::glue::Radio_Message_Rust::GlobalCommand(command).wrap();
        let mw = crate::glue::Radio_MessageWrapper {
            id,
            _pad: [0, 0, 0],
            msg,
        };
        let bytes = crate::glue::to_packet(mw);

        self.port.write_all(&bytes)
    }

    pub fn send_message(&mut self, id : crate::glue::Radio_SSL_ID , msg : crate::glue::Radio_Message_Rust) -> Result<(), std::io::Error> {
        let msg = msg.wrap();
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
        // if let Some(mirror) = &mut self.mirror {
        //     match mirror.bytes_to_read() {
        //         Ok(0) => (),
        //         Ok(_) => {
        //             let mut drop_buff = [0; 256];
        //             let _ = mirror.read(&mut drop_buff);
        //         },
        //         Err(_) => (),
        //     }
        // }
        match self.port.bytes_to_read() {
            Ok(0) => return Ok(()),
            Ok(_) => (),
            Err(_) => return Err(()),
        }
        if self.glob_index >= SERIAL_BUF_LEN {
            // The buffer is full and still holds no complete packet, so the
            // incoming stream is garbage or we are badly desynced. Drop it and
            // resync rather than crashing the monitor thread.
            eprintln!("SERIAL: buffer full without a valid packet, discarding to resync");
            self.glob_index = 0;
        }
        match self.port.read(&mut self.serial_buf[self.glob_index..]) {
            Ok(length) => {
                // Transmit everything on the mirror port
                // if self.check_carrier_detect() {
                //     if let Some(mirror) = &mut self.mirror {
                //         let mut buffer_copy: Vec<u8> = vec![0; length];
                //         buffer_copy.copy_from_slice(&self.serial_buf[self.glob_index..self.glob_index+length]);
                //         let _ = mirror.write(&buffer_copy);
                //     }
                // }
                self.glob_index += length;
                return Ok(());
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => eprintln!("{:?}", e),
            Err(e) => eprintln!("{:?}", e),
        }
        return Err(());
    }

    /// Drop the first `n` bytes from the front of the buffer, keeping the
    /// remaining live bytes contiguous at the start. Only moves the live
    /// region, unlike `rotate_left` which touches the whole backing store.
    fn discard(buf: &mut Vec<u8>, glob_index: &mut usize, n: usize) {
        let n = n.min(*glob_index);
        buf.copy_within(n..*glob_index, 0);
        *glob_index -= n;
    }

    /// Scan `buf[..*glob_index]` for one complete, CRC-valid frame, discarding
    /// leading/garbage bytes in place. Returns the payload of the first valid
    /// frame found, or `None` if more data is needed or only garbage remained.
    /// Every loop iteration that does not return discards at least one byte, so
    /// this always terminates and keeps consumption in step with even a fully
    /// garbled stream. Pure with respect to I/O, so it is unit-testable without
    /// a serial port.
    fn extract_packet(buf: &mut Vec<u8>, glob_index: &mut usize) -> Option<Vec<u8>> {
        loop {
            // Find the next start-of-frame marker.
            let start = match buf[..*glob_index].iter().position(|&x| x == START_BYTE) {
                Some(i) => i,
                None => {
                    // No marker anywhere: everything received so far is garbage.
                    *glob_index = 0;
                    return None;
                }
            };

            // Drop any leading garbage so the marker sits at index 0.
            if start > 0 {
                Self::discard(buf, glob_index, start);
            }

            // Need at least the marker and the length byte.
            if *glob_index < 2 {
                return None;
            }

            // Length of data
            let len: usize = buf[1] as usize;

            if len > MAX_PACKET_DATA_LEN {
                // Not a plausible frame: skip this marker and keep scanning.
                eprintln!("SERIAL: Bad packet: size too large");
                Self::discard(buf, glob_index, 1);
                continue;
            }

            // Need marker + length + `len` data bytes + CRC.
            if *glob_index < len + 3 {
                // Wait for the rest of the frame to arrive.
                return None;
            }

            let data = buf[2..(2 + len)].to_vec();
            let crc = buf[2 + len];

            if crc != crate::glue::crc_calc.checksum(&data) {
                // Bad packet, skip this start of packet indicator and resync.
                eprintln!("SERIAL: Bad packet: CRC failed");
                Self::discard(buf, glob_index, 1);
                continue;
            }

            Self::discard(buf, glob_index, len + 3);
            return Some(data);
        }
    }

    #[allow(dead_code)]
    #[cfg(target_os = "linux")]
    fn check_carrier_detect(&mut self) -> bool {
        true
    }

    #[allow(dead_code)]
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

    pub fn read_packet(&mut self) -> Result<Option<Vec<u8>>, ()> {
        self.read()?;
        Ok(Self::extract_packet(&mut self.serial_buf, &mut self.glob_index))
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

    // Build a well-formed frame: marker, length, payload, CRC-8.
    fn frame(payload: &[u8]) -> Vec<u8> {
        let mut f = vec![START_BYTE, payload.len() as u8];
        f.extend_from_slice(payload);
        f.push(crate::glue::crc_calc.checksum(payload));
        f
    }

    // Drive the real buffer parser the way read_packet does.
    fn parse_all(bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut buf = bytes.to_vec();
        buf.resize(SERIAL_BUF_LEN, 0); // mimic the fixed backing store
        let mut idx = bytes.len();
        let mut out = Vec::new();
        while let Some(p) = Serial::extract_packet(&mut buf, &mut idx) {
            out.push(p);
        }
        out
    }

    #[test]
    fn accepts_valid_frame() {
        let payload = vec![1, 2, 3, 4, 5];
        let got = parse_all(&frame(&payload));
        assert_eq!(got, vec![payload]);
    }

    #[test]
    fn rejects_bad_crc() {
        let mut f = frame(&[10, 20, 30]);
        let crc_pos = f.len() - 1;
        f[crc_pos] ^= 0xFF; // corrupt the CRC
        assert!(parse_all(&f).is_empty(), "frame with wrong CRC must be dropped");
    }

    #[test]
    fn rejects_corrupt_payload_via_crc() {
        let mut f = frame(&[10, 20, 30]);
        f[3] ^= 0xFF; // corrupt a payload byte; stored CRC no longer matches
        assert!(parse_all(&f).is_empty(), "payload corruption must fail the CRC");
    }

    #[test]
    fn resyncs_past_leading_garbage_and_false_markers() {
        let payload = vec![7, 7, 7];
        let good = frame(&payload);
        let mut stream = vec![0x00, 0xFF, START_BYTE, 0x02]; // garbage + a false marker w/ short len
        stream.extend_from_slice(&good);
        // The false marker at index 2 fails its CRC, we skip it and recover the real frame.
        assert_eq!(parse_all(&stream), vec![payload]);
    }

    #[test]
    fn oversized_length_is_skipped() {
        // A marker whose length byte exceeds the cap must not stall the parser;
        // a valid frame right after it is still recovered.
        let payload = vec![9, 9];
        let mut stream = vec![START_BYTE, (MAX_PACKET_DATA_LEN + 1) as u8, 0, 0];
        stream.extend_from_slice(&frame(&payload));
        assert_eq!(parse_all(&stream), vec![payload]);
    }

    #[test]
    fn all_marker_flood_does_not_grow_buffer() {
        // The historic panic case: a solid run of marker bytes. Every byte is a
        // "length too large" false start, so the parser must drain them all and
        // leave the buffer empty rather than accumulating toward overflow.
        let mut buf = vec![START_BYTE; 1000];
        buf.resize(SERIAL_BUF_LEN, 0);
        let mut idx = 1000;
        assert!(Serial::extract_packet(&mut buf, &mut idx).is_none());
        // At most a single dangling marker is retained (it may begin a frame
        // whose length byte has not arrived yet); the other 999 are drained, so
        // the buffer shrinks toward empty instead of accumulating.
        assert!(idx <= 1, "marker flood must not accumulate, got {idx}");
    }

    #[test]
    fn incomplete_frame_waits_without_consuming_marker() {
        // Only marker + length arrive; the rest of the frame is still in flight.
        let mut buf = vec![START_BYTE, 4, 1, 2];
        let n = buf.len();
        buf.resize(SERIAL_BUF_LEN, 0);
        let mut idx = n;
        assert!(Serial::extract_packet(&mut buf, &mut idx).is_none());
        assert_eq!(idx, n, "a partial frame must be retained until it completes");
    }
}
