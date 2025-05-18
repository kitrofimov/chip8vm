//! A utility macro for code generation

/// Split a 16-bit value into a vector of two bytes
#[macro_export]
macro_rules! split_u16 {
    ($val:expr) => {{
        let val = $val;
        vec![(val >> 8) as u8, val as u8]
    }};
}
