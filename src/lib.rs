#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use num_traits::ToPrimitive;

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
}
