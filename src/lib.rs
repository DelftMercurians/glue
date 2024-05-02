#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

pub fn to_status(line : &[u8]) -> Option<HG_Status> {
    if let Ok(status) = String::from_utf8_lossy(line).to_string().parse::<i32>() {
        return FromPrimitive::from_i32(status);
    }
    None
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

        // let rm : Radio_Message = {Radio_MessageType_Command; Radio_Command}
    }
}
