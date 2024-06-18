#![allow(dead_code)]

use num_traits::FromPrimitive;
pub const crc_calc: crc::Crc<u8> = crc::Crc::<u8>::new(&crc::CRC_8_SMBUS);
use zerocopy::AsBytes;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub enum Radio_Message_Rust {
    Command(Radio_Command),
    ImuReadings(Radio_ImuReadings),
    MultiConfigMessage(Radio_MultiConfigMessage),
    PrimaryStatusHF(Radio_PrimaryStatusHF),
    PrimaryStatusLF(Radio_PrimaryStatusLF),
    OdometryReading(Radio_OdometryReading),
    OverrideOdometry(Radio_OverrideOdometry),
    // Reply(Radio_Reply),
    None,
}

impl Radio_MultiConfigMessage {
    pub fn write() -> Self {
        Radio_MultiConfigMessage{
            vars: [HG_Variable::NONE; 5],
            operation: HG_ConfigOperation::WRITE,
            type_: HG_VariableType::VOID,
            _pad: 0,
            values: [0; 5],
        }
    }

    pub fn add(&mut self, var : HG_Variable, value : u32) -> Self {
        for i in 0..5 {
            if self.vars[i] != HG_Variable::NONE { continue }
            self.vars[i] = var;
            self.values[i] = value;
            break
        }
        *self
    }

    pub fn read() -> Self {
        Radio_MultiConfigMessage{
            vars: [HG_Variable::NONE; 5],
            operation: HG_ConfigOperation::READ,
            type_: HG_VariableType::VOID,
            _pad: 0,
            values: [0; 5],
        }
    }
}


impl HG_Version {
    pub fn version_to_string(&self) -> String {
        format!("v{}.{}.{}", self.major, self.minor, self.patch)
    }
    pub fn protocol_version_to_string(&self) -> String {
        format!("#{}.{}", self.protocols_major, self.protocols_minor)
    }
    pub fn protcol_version_matches(&self) -> bool {
        (self.protocols_major  == crate::glue::CONST_PROTOCOL_VERSION_MAJOR) &&
        (self.protocols_minor == crate::glue::CONST_PROTOCOL_VERSION_MINOR)
    }
}

impl Radio_Message_Rust {

    // Convert into the C representation
    pub fn wrap(&self) -> Radio_Message {
        return match *self {
            Self::Command(c) => Radio_Message {
                mt: Radio_MessageType::Command,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    c,
                },
            },
            Self::ImuReadings(ir) => Radio_Message {
                mt: Radio_MessageType::ImuReadings,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    __bindgen_anon_2: Radio_Message__bindgen_ty_1__bindgen_ty_2 {
                        ir,
                        _pad1: [0; 4],
                    }
                },
            },
            Self::MultiConfigMessage(mcm) => Radio_Message {
                mt: Radio_MessageType::MultiConfigMessage,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    mcm,
                },
            },
            Self::OdometryReading(odo) => Radio_Message {
                mt: Radio_MessageType::OdometryReading,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    odo,
                },
            },
            Self::OverrideOdometry(over_odo) => Radio_Message {
                mt: Radio_MessageType::OverrideOdometry,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    over_odo,
                },
            },
            Self::PrimaryStatusHF(ps_hf) => Radio_Message {
                mt: Radio_MessageType::PrimaryStatusHF,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    ps_hf,
                },
            },
            Self::PrimaryStatusLF(ps_lf) => Radio_Message {
                mt: Radio_MessageType::PrimaryStatusLF,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    __bindgen_anon_1: Radio_Message__bindgen_ty_1__bindgen_ty_1 {
                        ps_lf,
                        _pad0: [0; 10],
                    }
                },
            },
            Self::None => Radio_Message {
                mt: Radio_MessageType::None,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    __bindgen_anon_2: Radio_Message__bindgen_ty_1__bindgen_ty_2 {
                        ir: Radio_ImuReadings {
                            ang_wx: 0.0,
                            ang_wy: 0.0,
                            ang_wz: 0.0,
                            ang_x: 0.0,
                            ang_y: 0.0,
                            ang_z: 0.0,
                        },
                        _pad1: [0; 4],
                    }
                }, // Just pack this with zeros
            },
        }
    }

    // Convert into the Rust representation
    pub fn unwrap(msg : Radio_Message) -> Radio_Message_Rust {
        unsafe {
            // This transmute requires unsafe block
            if let None = crate::glue::Radio_MessageType::from_u8(std::mem::transmute(msg.mt)) { // Check that message type isn't giberish
                return Radio_Message_Rust::None;
            }
            
            // Accessing stuff from a union requires unsafe
            match msg.mt {
                Radio_MessageType::Command =>  return Radio_Message_Rust::Command(msg.msg.c),
                Radio_MessageType::ImuReadings =>  return Radio_Message_Rust::ImuReadings(msg.msg.__bindgen_anon_2.ir),
                Radio_MessageType::PrimaryStatusHF => {
                    return Radio_Message_Rust::PrimaryStatusHF(msg.msg.ps_hf)
                },
                Radio_MessageType::OdometryReading => {
                    return Radio_Message_Rust::OdometryReading(msg.msg.odo)
                },
                Radio_MessageType::MultiConfigMessage => {
                    return Radio_Message_Rust::MultiConfigMessage(msg.msg.mcm)
                },
                Radio_MessageType::OverrideOdometry => {
                    return Radio_Message_Rust::OverrideOdometry(msg.msg.over_odo)
                },
                Radio_MessageType::PrimaryStatusLF => {
                    // Convert to struct
                    let mut s: Radio_PrimaryStatusLF = msg.msg.__bindgen_anon_1.ps_lf;

                    // Catch bad values
                    if let None = crate::glue::HG_Status::from_u8(std::mem::transmute(s.primary_status)) { return Radio_Message_Rust::None; };
                    if let None = crate::glue::HG_Status::from_u8(std::mem::transmute(s.kicker_status)) { return Radio_Message_Rust::None; };
                    if let None = crate::glue::HG_Status::from_u8(std::mem::transmute(s.fan_status)) { return Radio_Message_Rust::None; };
                    if let None = crate::glue::HG_Status::from_u8(std::mem::transmute(s.imu_status)) { return Radio_Message_Rust::None; };
                    for ms in &mut s.motor_status {
                        if let None = crate::glue::HG_Status::from_u8(std::mem::transmute(*ms)) { return Radio_Message_Rust::None; };
                    }
                    return Radio_Message_Rust::PrimaryStatusLF(msg.msg.__bindgen_anon_1.ps_lf);
                },
                _ => return Radio_Message_Rust::None,
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



impl Base_Information {
    // Convert raw bytes into Base_Information, with some checks
    pub fn from_bytes(data : Vec<u8>) -> Option<Self> {
        if std::mem::size_of::<Self>() != data.len() { return None; }
        if let Ok(raw) = <[u8;std::mem::size_of::<Self>()]>::try_from(data) {
            unsafe {
                let res: Self = std::mem::transmute(raw);
                return Some(res);
            }
        }
        None
    }
}

impl Radio_MessageWrapper {
    // Convert raw bytes into Radio_MessageWrapper, with some checks
    pub fn from_bytes(data : Vec<u8>) -> Option<Self> {
        if std::mem::size_of::<Self>() != data.len() { return None; }
        if let Ok(raw) = <[u8;std::mem::size_of::<Self>()]>::try_from(data) {
            unsafe {
                let res: Self = std::mem::transmute(raw);
                if let None = crate::glue::Radio_MessageType::from_u8(std::mem::transmute(res.msg.mt)) { // Check that message type isn't giberish
                    return None;
                }
                return Some(res);
            }
        }
        None
    }
}
