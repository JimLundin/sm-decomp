fn kept() {}
#[cfg(test)]
use std::{fmt, io};
#[cfg(test)]
#[allow(dead_code)]
fn helper() -> [u8; 2] {
    [1, 2]
}
#[cfg(test)]
mod tests {
    // a comment inside the tests
    #[test]
    fn works() {}
}
const AFTER: u8 = 1;
