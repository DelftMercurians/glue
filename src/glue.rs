

use num_traits::{FromPrimitive, ToPrimitive};
pub const crc_calc: crc::Crc<u8> = crc::Crc::<u8>::new(&crc::CRC_8_SMBUS);
use zerocopy::AsBytes;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub enum Radio_Message_Rust {
    Radio_Command(Radio_Command),
    Radio_ImuReadings(Radio_ImuReadings),
    // Radio_ConfigMessage(Radio_ConfigMessage),
    Radio_PrimaryStatusHF(Radio_PrimaryStatusHF),
    Radio_PrimaryStatusLF(Radio_PrimaryStatusLF),
    // Radio_Reply(Radio_Reply),
}

impl Radio_Message_Rust {
    pub fn wrap(&self) -> Radio_Message {
        return match *self {
            Self::Radio_Command(c) => Radio_Message {
                mt: Radio_MessageType::Command,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    c,
                },
            },
            Self::Radio_ImuReadings(ir) => Radio_Message {
                mt: Radio_MessageType::ImuReadings,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    __bindgen_anon_2: Radio_Message__bindgen_ty_1__bindgen_ty_2 {
                        ir,
                        _pad1: [0; 4],
                    }
                },
            },
            Self::Radio_PrimaryStatusHF(ps_hf) => Radio_Message {
                mt: Radio_MessageType::PrimaryStatusHF,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    ps_hf,
                },
            },
            Self::Radio_PrimaryStatusLF(ps_lf) => Radio_Message {
                mt: Radio_MessageType::PrimaryStatusLF,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    __bindgen_anon_1: Radio_Message__bindgen_ty_1__bindgen_ty_1 {
                        ps_lf,
                        _pad0: [0; 10],
                    }
                },
            }
        }
    }
}


pub fn to_packet<T: AsBytes>(data : T) -> Vec<u8> {
    let raw_data = data.as_bytes();

    let mut a = vec![0; raw_data.len() + 3];
    a[0] = 0b10100101;
    a[1] = raw_data.len() as u8;
    a[2..raw_data.len()+2].copy_from_slice(&raw_data);
    a[raw_data.len()+2] = crc_calc.checksum(&raw_data);
    a
}


// pub fn to_status(line : &[u8]) -> Option<HG_Status> {
//     if let Ok(status) = String::from_utf8_lossy(line).to_string().parse::<i32>() {
//         return FromPrimitive::from_i32(status);
//     }
//     None
// }

// pub fn to_int(status : HG_Status) -> i32 {
//     return ToPrimitive::to_i32(&status).unwrap_or(ToPrimitive::to_i32(&HG_Status::EMERGENCY).unwrap_or(0));
// }

// pub fn to_radio_status_hf(raw: [u8; std::mem::size_of::<Radio_PrimaryStatusHF>()], crc : Option<u8>) -> Option<Radio_PrimaryStatusHF> {
//     // Check CRC8 checksum
//     if let Some(crc_value) = crc {
//         if crc_calc.checksum(&raw) != crc_value { return None };
//     }
    
//     // Check padding
//     if raw[2] != 0x00 { return None };
//     if raw[3] != 0x00 { return None };
//     if raw[26] != 0x00 { return None };
//     if raw[27] != 0x00 { return None };

//     unsafe {
//         return Some(std::mem::transmute(raw));
//     }
// }

// pub fn to_radio_status_lf(raw: [u8; std::mem::size_of::<Radio_PrimaryStatusLF>()], crc : Option<u8>) -> Option<Radio_PrimaryStatusLF> {
//     // Check CRC8 checksum
//     if let Some(crc_value) = crc {
//         if crc_calc.checksum(&raw) != crc_value { return None };
//     }

//     unsafe {
//         // Convert to struct
//         let mut s: Radio_PrimaryStatusLF = std::mem::transmute(raw);

//         // Catch bad values
//         s.primary_status = FromPrimitive::from_u8(std::mem::transmute(s.primary_status))?;
//         s.kicker_status = FromPrimitive::from_u8(std::mem::transmute(s.kicker_status))?;
//         s.fan_status = FromPrimitive::from_u8(std::mem::transmute(s.fan_status))?;
//         s.imu_status = FromPrimitive::from_u8(std::mem::transmute(s.imu_status))?;
//         for ms in &mut s.motor_status {
//             *ms = FromPrimitive::from_u8(std::mem::transmute(*ms))?;
//         }
//         return Some(s);
//     }
// }

pub fn extract_radio_status_lf(message : Radio_Message) -> Option<Radio_PrimaryStatusLF> {
    // Check message type
    if message.mt != Radio_MessageType::PrimaryStatusLF { return None };

    unsafe {
        // Convert to struct
        let mut s: Radio_PrimaryStatusLF = message.msg.__bindgen_anon_1.ps_lf;

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

pub fn extract_radio_status_hf(message : Radio_Message) -> Option<Radio_PrimaryStatusHF> {
    // Check message type
    if message.mt != Radio_MessageType::PrimaryStatusHF { return None };

    // Check padding
    unsafe {
        let raw : [u8; 28] = std::mem::transmute(message.msg);
        if raw[2] != 0x00 { return None };
        if raw[3] != 0x00 { return None };
        if raw[26] != 0x00 { return None };
        if raw[27] != 0x00 { return None };
  
        return Some(message.msg.ps_hf);
    }
}

pub fn extract_imu(message : Radio_Message) -> Option<Radio_ImuReadings> {
    // Check message type
    if message.mt != Radio_MessageType::ImuReadings { return None };

    // Check padding
    unsafe {
        return Some(message.msg.__bindgen_anon_2.ir);
    }
}

// pub fn wrap_command(command : Radio_Command) -> Radio_Message {
//     Radio_Message {
//         mt: Radio_MessageType::Command,
//         _pad: [0, 0, 0],
//         msg: Radio_Message__bindgen_ty_1 {
//             c: command,
//         },
//     }
// }

pub fn to_base_info(raw: [u8; std::mem::size_of::<Base_Information>()], crc : Option<u8>) -> Option<Base_Information> {
    // Check CRC8 checksum
    if let Some(crc_value) = crc {
        if crc_calc.checksum(&raw) != crc_value { return None };
    }

    unsafe {
        // Convert to struct
        let bi: Base_Information = std::mem::transmute(raw);

        return Some(bi);
    }
}

pub fn to_radio_message_wrapper(raw: [u8; std::mem::size_of::<Radio_MessageWrapper>()], crc : Option<u8>) -> Option<(u8, Radio_Message)> {
    // Check CRC8 checksum
    if let Some(crc_value) = crc {
        if crc_calc.checksum(&raw) != crc_value { return None };
    }

    unsafe {
        // Convert to struct
        let mut rmw: Radio_MessageWrapper = std::mem::transmute(raw);

        // Catch bad values
        rmw.msg.mt = FromPrimitive::from_u8(std::mem::transmute(rmw.msg.mt))?;

        return Some((rmw.id, rmw.msg));
    }
}