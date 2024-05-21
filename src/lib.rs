#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use num_traits::ToPrimitive;

const crc_calc: crc::Crc<u8> = crc::Crc::<u8>::new(&crc::CRC_8_SMBUS);

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn to_status(line : &[u8]) -> Option<HG_Status> {
    if let Ok(status) = String::from_utf8_lossy(line).to_string().parse::<i32>() {
        return FromPrimitive::from_i32(status);
    }
    None
}

pub fn to_int(status : HG_Status) -> i32 {
    return ToPrimitive::to_i32(&status).unwrap_or(ToPrimitive::to_i32(&HG_Status::EMERGENCY).unwrap_or(0));
}

pub fn to_radio_status_hf(raw: [u8; std::mem::size_of::<Radio_PrimaryStatusHF>()], crc : Option<u8>) -> Option<Radio_PrimaryStatusHF> {
    // Check CRC8 checksum
    if let Some(crc_value) = crc {
        if crc_calc.checksum(&raw) != crc_value { return None };
    }
    
    // Check padding
    if raw[2] != 0x00 { return None };
    if raw[3] != 0x00 { return None };
    if raw[26] != 0x00 { return None };
    if raw[27] != 0x00 { return None };

    unsafe {
        return Some(std::mem::transmute(raw));
    }
}

pub fn to_radio_status_lf(raw: [u8; std::mem::size_of::<Radio_PrimaryStatusLF>()], crc : Option<u8>) -> Option<Radio_PrimaryStatusLF> {
    // Check CRC8 checksum
    if let Some(crc_value) = crc {
        if crc_calc.checksum(&raw) != crc_value { return None };
    }

    unsafe {
        // Convert to struct
        let mut s: Radio_PrimaryStatusLF = std::mem::transmute(raw);

        // Catch bad values
        s.primary_status = FromPrimitive::from_u8(std::mem::transmute(s.primary_status))?;
        s.kicker_status = FromPrimitive::from_u8(std::mem::transmute(s.kicker_status))?;
        s.fan_status = FromPrimitive::from_u8(std::mem::transmute(s.fan_status))?;
        s.imu_status = FromPrimitive::from_u8(std::mem::transmute(s.imu_status))?;
        for ms in &mut s.motor_status {
            *ms = FromPrimitive::from_u8(std::mem::transmute(*ms))?;
        }
        return Some(s);
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hg_status_conversions() {
        let status: HG_Status = HG_Status::OK;
        assert_eq!(Some(status), to_status(b"1"));
        assert_ne!(Some(status), to_status(b"2"));
        assert_eq!(None, to_status(b"f"));

        assert_eq!(1, to_int(status));
        assert_ne!(2, to_int(status));

        // let rm : Radio_Message = {Radio_MessageType_Command; Radio_Command}
    }

    #[test]
    fn radio_status_hf() {
        let raw: [u8; 28] = [0xD2, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x3F, 0x00, 0x00, 0x00, 0x40, 0x00, 0x00, 0x40, 0x40, 0x00, 0x00, 0x80, 0x40, 0x01, 0x00, 0x00, 0x00];
        
        assert!(to_radio_status_hf(raw, Some(0x48)).is_none()); // Wrong CRC

        if let Some(status) = to_radio_status_hf(raw, Some(0x47)) { // Correct expansion
            assert_eq!(status.pressure, 1234);
            assert_eq!(status.motor_speeds[0], 0.0);
            assert_eq!(status.motor_speeds[1], 1.0);
            assert_eq!(status.motor_speeds[2], 2.0);
            assert_eq!(status.motor_speeds[3], 3.0);
            assert_eq!(status.motor_speeds[4], 4.0);
            assert_eq!(status.breakbeam_state[0], true);
            assert_eq!(status.breakbeam_state[1], false);
        } else {
            assert!(false);
        }

        let raw2: [u8; 28] = [0xFF, 0xFF, 0x00, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01, 0x01, 0x00, 0x00];
        assert!(to_radio_status_hf(raw2, None).is_none()); // Bad padding
        
    }

    #[test]
    fn radio_status_lf() {
        let raw: [u8; 18] = [0x0C, 0x22, 0x00, 0x01, 0x02, 0x03, 0x04, 0x07, 0x09, 0x01, 0x06, 0x09, 0x00, 0x01, 0x08, 0x04, 0x03, 0x05];
        
        assert!(to_radio_status_lf(raw, Some(0x92)).is_none()); // Wrong CRC

        if let Some(status) = to_radio_status_lf(raw, Some(0x93)) { // Correct expansion
            assert_eq!(status.pack_voltages[0], 12);
            assert_eq!(status.pack_voltages[1], 34);
            assert_eq!(status.motor_driver_temps[0], 0);
            assert_eq!(status.motor_driver_temps[1], 1);
            assert_eq!(status.motor_driver_temps[2], 2);
            assert_eq!(status.motor_driver_temps[3], 3);
            assert_eq!(status.motor_driver_temps[4], 4);
            assert_eq!(status.cap_voltage, 7);
            assert_eq!(status.kicker_temp, 9);
            assert_eq!(status.primary_status, HG_Status::OK);
            assert_eq!(status.kicker_status, HG_Status::ARMED);
            assert_eq!(status.fan_status, HG_Status::NOT_INSTALLED);
            assert_eq!(status.imu_status, HG_Status::EMERGENCY);
            assert_eq!(status.motor_status[0], HG_Status::OK);
            assert_eq!(status.motor_status[1], HG_Status::SAFE);
            assert_eq!(status.motor_status[2], HG_Status::OVERTEMP);
            assert_eq!(status.motor_status[3], HG_Status::STARTING);
            assert_eq!(status.motor_status[4], HG_Status::NO_REPLY);
        } else {
            assert!(false);
        }

        let raw3: [u8; 18] = [0x0C, 0x22, 0x00, 0x01, 0x02, 0x03, 0x04, 0x07, 0x09, 0x01, 0x06, 0x09, 0x00, 0x01, 0x08, 0x04, 0x03, 0x0F];
        assert!(to_radio_status_lf(raw3, None).is_none()); // Bad enums
    }
}
