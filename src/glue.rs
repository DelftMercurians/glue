#![allow(dead_code, unnecessary_transmutes)]

use zerocopy::AsBytes;
pub const crc_calc: crc::Crc<u8> = crc::Crc::<u8>::new(&crc::CRC_8_SMBUS);

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub enum Radio_Message_Rust {
    Command(Radio_Command),
    GlobalCommand(Radio_GlobalCommand),
    ImuReadings(Radio_ImuReadings),
    MultiConfigMessage(Radio_MultiConfigMessage),
    PrimaryStatusHF(Radio_PrimaryStatusHF),
    PrimaryStatusLF(Radio_PrimaryStatusLF),
    OdometryReading(Radio_OdometryReading),
    OverrideOdometry(Radio_OverrideOdometry),
    SerialMessage(Radio_SerialMessage),
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

    pub fn set_default() -> Self {
        Radio_MultiConfigMessage{
            vars: [HG_Variable::NONE; 5],
            operation: HG_ConfigOperation::SET_DEFAULT,
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
            Self::GlobalCommand(gc) => Radio_Message {
                mt: Radio_MessageType::GlobalCommand,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    gc,
                },
            },
            Self::ImuReadings(ir) => Radio_Message {
                mt: Radio_MessageType::ImuReadings,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    __bindgen_anon_1: Radio_Message__bindgen_ty_1__bindgen_ty_1 {
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
                    ps_lf,
                },
            },
            Self::SerialMessage(serial) => Radio_Message {
                mt: Radio_MessageType::SerialMessage,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    serial,
                },
            },
            Self::None => Radio_Message {
                mt: Radio_MessageType::None,
                _pad: [0; 3],
                msg: Radio_Message__bindgen_ty_1 {
                    __bindgen_anon_1: Radio_Message__bindgen_ty_1__bindgen_ty_1 {
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

    // Convert into the Rust representation.
    //
    // `mt`, `robot_command` and the `HG_Status` fields are newtype enums (see
    // build.rs), so every byte pattern is a representable value and reading them
    // out of the union can never be undefined behaviour — no discriminant
    // checks are needed and unknown values pass straight through. The one
    // remaining validity-constrained field is the trio of bools in
    // OverrideOdometry (a bool is UB for any value other than 0/1), which is
    // checked before the union field that constructs them is read.
    pub fn unwrap(msg : Radio_Message) -> Radio_Message_Rust {
        // Accessing fields of the union requires unsafe.
        unsafe {
            match msg.mt {
                Radio_MessageType::Command => {
                    return Radio_Message_Rust::Command(msg.msg.c);
                },
                Radio_MessageType::GlobalCommand => {
                    return Radio_Message_Rust::GlobalCommand(msg.msg.gc);
                },
                Radio_MessageType::ImuReadings =>  return Radio_Message_Rust::ImuReadings(msg.msg.__bindgen_anon_1.ir),
                Radio_MessageType::PrimaryStatusHF => {
                    return Radio_Message_Rust::PrimaryStatusHF(msg.msg.ps_hf)
                },
                Radio_MessageType::OdometryReading => {
                    return Radio_Message_Rust::OdometryReading(msg.msg.odo)
                },
                Radio_MessageType::MultiConfigMessage => {
                    // Temporarily off due to unsafe/segfault issues
                    return Radio_Message_Rust::None
                },
                Radio_MessageType::OverrideOdometry => {
                    // The three bools are the only bytes that would be UB to
                    // construct from an out-of-range value, so validate them
                    // against the raw bytes before reading the union field.
                    let bytes = msg.as_bytes();
                    let is_bool = |o: usize| matches!(bytes[o], 0 | 1);
                    if is_bool(std::mem::offset_of!(Radio_Message, msg.over_odo.set_pos_x))
                        && is_bool(std::mem::offset_of!(Radio_Message, msg.over_odo.set_pos_y))
                        && is_bool(std::mem::offset_of!(Radio_Message, msg.over_odo.set_ang_z))
                    {
                        return Radio_Message_Rust::OverrideOdometry(msg.msg.over_odo);
                    }
                    return Radio_Message_Rust::None;
                },
                Radio_MessageType::SerialMessage => {
                    return Radio_Message_Rust::SerialMessage(msg.msg.serial)
                }
                Radio_MessageType::PrimaryStatusLF => {
                    return Radio_Message_Rust::PrimaryStatusLF(msg.msg.ps_lf);
                },
                _ => return Radio_Message_Rust::None,
            }
        }
    }
}


pub fn to_packet<T: zerocopy::AsBytes>(data : T) -> Vec<u8> {
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
    // Convert raw bytes into Radio_MessageWrapper.
    //
    // Radio_MessageWrapper is made of integers, floats, newtype enums (every
    // byte pattern is a valid value) and a union (which has no per-variant
    // validity requirement — its bytes are untyped until a field is read). None
    // of those constrain the bit pattern, so transmuting any correctly-sized
    // buffer is sound. The only validity-constrained fields in the whole message
    // are the three bools in OverrideOdometry, and those live inside the union
    // and are only materialised by `unwrap`, which validates them there.
    pub fn from_bytes(data : Vec<u8>) -> Option<Self> {
        const SIZE: usize = std::mem::size_of::<Radio_MessageWrapper>();
        let raw = <[u8; SIZE]>::try_from(data).ok()?; // also rejects wrong length
        // SAFETY: see the comment above — no field of Radio_MessageWrapper
        // constrains its bit pattern, so this transmute is defined for any
        // correctly-sized input.
        let res: Self = unsafe { std::mem::transmute(raw) };
        Some(res)
    }
}

#[cfg(test)]
mod validation_tests {
    use super::*;

    fn wrapper_bytes(msg: Radio_Message_Rust) -> Vec<u8> {
        let mw = Radio_MessageWrapper { id: 0, _pad: [0; 3], msg: msg.wrap() };
        mw.as_bytes().to_vec()
    }

    fn sample_gen_command() -> Radio_GenericCommand {
        Radio_GenericCommand {
            dribbler_speed_i: 0,
            kick_time_i: 0,
            time_to_kick: 0,
            smart_kick_couter: 0,
            robot_command: Radio_RobotCommand::NONE,
        }
    }

    // Full receive pipeline: bytes -> typed wrapper -> Rust message.
    fn parse(bytes: Vec<u8>) -> Radio_Message_Rust {
        match Radio_MessageWrapper::from_bytes(bytes) {
            Some(w) => Radio_Message_Rust::unwrap(w.msg),
            None => Radio_Message_Rust::None,
        }
    }

    #[test]
    fn wrong_length_is_rejected() {
        assert!(Radio_MessageWrapper::from_bytes(vec![0u8; 4]).is_none());
        assert!(Radio_MessageWrapper::from_bytes(vec![]).is_none());
    }

    #[test]
    fn unknown_message_type_is_dropped() {
        let mut bytes = wrapper_bytes(Radio_Message_Rust::Command(Radio_Command {
            speed: HG_Pose { x: 0.0, y: 0.0, z: 0.0 },
            gen_command: sample_gen_command(),
            _pad: [0; 8],
        }));
        // 0x02 is not a known Radio_MessageType. The frame is still a valid
        // newtype value, so from_bytes accepts it, but unwrap drops it.
        bytes[std::mem::offset_of!(Radio_MessageWrapper, msg.mt)] = 0x02;
        assert!(matches!(parse(bytes), Radio_Message_Rust::None));
    }

    #[test]
    fn out_of_range_enum_values_pass_through_without_trap() {
        // Newtype enums have no invalid bit pattern, so a garbled robot_command
        // is not UB and is preserved verbatim rather than dropping the frame.
        // (This is a deliberate change from the old rustified-enum behaviour,
        // and is what makes the parser forward-compatible with firmware that
        // introduces new enum values.)
        let mut bytes = wrapper_bytes(Radio_Message_Rust::Command(Radio_Command {
            speed: HG_Pose { x: 0.0, y: 0.0, z: 0.0 },
            gen_command: sample_gen_command(),
            _pad: [0; 8],
        }));
        let o = std::mem::offset_of!(Radio_MessageWrapper, msg.msg.c.gen_command.robot_command);
        bytes[o] = 0xAB;
        match parse(bytes) {
            Radio_Message_Rust::Command(c) => {
                assert_eq!(c.gen_command.robot_command, Radio_RobotCommand(0xAB));
            }
            _ => panic!("command with unknown robot_command should still parse"),
        }
    }

    #[test]
    fn primary_status_lf_unknown_status_passes_through() {
        let s = Radio_PrimaryStatusLF {
            main_board_current: 0,
            pack_voltages: [0; 2],
            motor_driver_temps: [0; 5],
            cap_voltage: 0,
            _pad0: 0,
            primary_status: HG_Status::OK,
            kicker_status: HG_Status::OK,
            imu_status: HG_Status::OK,
            tof_status: HG_Status::OK,
            motor_status: [HG_Status::OK; 5],
            avg_loop_time: 0,
            max_loop_time: 0,
            avg_command_time: 0,
            _pad: [0; 2],
        };
        let mut bytes = wrapper_bytes(Radio_Message_Rust::PrimaryStatusLF(s));
        // 0xFF is well past the last known HG_Status (COOLDOWN = 0x0C).
        bytes[std::mem::offset_of!(Radio_MessageWrapper, msg.msg.ps_lf.primary_status)] = 0xFF;
        match parse(bytes) {
            Radio_Message_Rust::PrimaryStatusLF(got) => {
                assert_eq!(got.primary_status, HG_Status(0xFF));
            }
            _ => panic!("status frame with unknown status should still parse"),
        }
    }

    #[test]
    fn override_odometry_invalid_bool_is_dropped() {
        let over = Radio_OverrideOdometry {
            pos_x: 0.0,
            pos_y: 0.0,
            ang_z: 0.0,
            set_pos_x: false,
            set_pos_y: false,
            set_ang_z: true,
            _pad0: 0,
            _pad: [0; 12],
        };
        let good = wrapper_bytes(Radio_Message_Rust::OverrideOdometry(over));
        // The valid frame round-trips.
        assert!(matches!(parse(good.clone()), Radio_Message_Rust::OverrideOdometry(_)));
        // A bool is UB for any value other than 0/1, so each must be guarded:
        // corrupting any one to 2 must drop the frame (never construct the bool).
        for field in [
            std::mem::offset_of!(Radio_MessageWrapper, msg.msg.over_odo.set_pos_x),
            std::mem::offset_of!(Radio_MessageWrapper, msg.msg.over_odo.set_pos_y),
            std::mem::offset_of!(Radio_MessageWrapper, msg.msg.over_odo.set_ang_z),
        ] {
            let mut bad = good.clone();
            bad[field] = 2;
            assert!(
                matches!(parse(bad), Radio_Message_Rust::None),
                "invalid bool at offset {field} must drop the frame"
            );
        }
    }

    #[test]
    fn every_message_type_survives_hostile_payload() {
        // For each known message type, hand the pipeline a frame whose payload is
        // all-0xFF (every enum/bool byte maximally out of range). Under Miri this
        // is the real proof that no invalid enum/bool is ever constructed; under
        // a normal run it proves nothing panics and hostile frames are handled.
        const SIZE: usize = std::mem::size_of::<Radio_MessageWrapper>();
        let types = [
            Radio_MessageType::None,
            Radio_MessageType::Command,
            Radio_MessageType::ConfigMessage,
            Radio_MessageType::PrimaryStatusHF,
            Radio_MessageType::PrimaryStatusLF,
            Radio_MessageType::ImuReadings,
            Radio_MessageType::OdometryReading,
            Radio_MessageType::OverrideOdometry,
            Radio_MessageType::GlobalCommand,
            Radio_MessageType::SerialMessage,
            Radio_MessageType::MultiConfigMessage,
            Radio_MessageType::NoOp,
        ];
        for mt in types {
            let mut bytes = vec![0xFFu8; SIZE];
            bytes[std::mem::offset_of!(Radio_MessageWrapper, msg.mt)] = mt.0;
            let _ = parse(bytes);
        }
    }
}
