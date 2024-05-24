

use num_traits::{FromPrimitive, ToBytes, ToPrimitive};
pub const crc_calc: crc::Crc<u8> = crc::Crc::<u8>::new(&crc::CRC_8_SMBUS);
use zerocopy::AsBytes;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));


// #[derive(Debug)]
pub enum PacketContents {
    Radio_MessageWrapper(Radio_MessageWrapper),
    Base_Information(Base_Information),
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

pub fn wrap_command(command : Radio_Command) -> Radio_Message {
    Radio_Message {
        mt: Radio_MessageType::Command,
        _pad: [0, 0, 0],
        msg: Radio_Message__bindgen_ty_1 {
            c: command,
        },
    }
}

pub fn messagewrapper_to_bytes(msg : Radio_MessageWrapper) -> [u8; std::mem::size_of::<Radio_MessageWrapper>()] {
    unsafe{
        std::mem::transmute(msg)
    }
}

pub fn wrap_message_to_packet(msg : Radio_Message, id : Radio_SSL_ID) -> [u8; std::mem::size_of::<Radio_MessageWrapper>()+3] {
    let rmw = Radio_MessageWrapper {
        msg,
        _pad: [0, 0, 0],
        id,
    };
    let mut a = [0; std::mem::size_of::<Radio_MessageWrapper>() + 3];
    a[0] = 0b10100101;
    a[1] = std::mem::size_of::<Radio_MessageWrapper>() as u8;
    let data = messagewrapper_to_bytes(rmw);
    a[2..std::mem::size_of::<Radio_MessageWrapper>()+2].copy_from_slice(&data);
    a[std::mem::size_of::<Radio_MessageWrapper>()+2] = crc_calc.checksum(&data);
    a
}

pub fn message_to_bytes(msg : Radio_Message) -> [u8; std::mem::size_of::<Radio_Message>()] {
    unsafe{
        std::mem::transmute(msg)
    }
}

pub fn bytes_to_packet(data : &[u8; std::mem::size_of::<Radio_Message>()]) -> [u8; std::mem::size_of::<Radio_Message>()+3] {
    let mut a: [u8; 35] = [0; std::mem::size_of::<Radio_Message>() + 3];
    a[0] = 0b10100101;
    a[1] = std::mem::size_of::<Radio_Message>() as u8;
    a[2..std::mem::size_of::<Radio_Message>()+2].copy_from_slice(data);
    a[std::mem::size_of::<Radio_Message>()+2] = crc_calc.checksum(data);
    a
}

pub fn to_radio_message(raw: [u8; std::mem::size_of::<Radio_Message>()], crc : Option<u8>) -> Option<Radio_Message> {
    // Check CRC8 checksum
    if let Some(crc_value) = crc {
        if crc_calc.checksum(&raw) != crc_value { return None };
    }

    unsafe {
        // Convert to struct
        let mut rm: Radio_Message = std::mem::transmute(raw);

        // Catch bad values
        rm.mt = FromPrimitive::from_u8(std::mem::transmute(rm.mt))?;

        return Some(rm);
    }
}

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