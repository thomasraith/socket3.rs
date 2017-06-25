//!SOCKET3 - RAW Socket Crate
pub mod raw;

pub fn htons(i : u16) -> u16 {
    i.to_be()
}


