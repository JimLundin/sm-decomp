macro_rules! documented {
    ($text:expr) => {
        #[doc = $text]
        pub fn next(n: u8) -> u8 {
            n.wrapping_add(1)
        }
    };
}
documented!(concat!("``", "`\nassert!(true);\n``", "`"));
