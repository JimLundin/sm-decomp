macro_rules! documented {
    ($text:literal) => {
        #[doc = $text]
        pub fn next(n: u8) -> u8 {
            n.wrapping_add(1)
        }
    };
}
macro_rules! forward {
    ($text:literal) => {
        documented!($text);
    };
}
forward!("    assert!(true);");
