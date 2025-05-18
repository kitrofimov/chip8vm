#[macro_export]
macro_rules! split_u16 {
    ($val:expr) => {{
        let val = $val;
        vec![(val >> 8) as u8, val as u8]
    }};
}
