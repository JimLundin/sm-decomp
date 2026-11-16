//! The application entry: power-on admission requires its clean production policy.

#[derive(Debug)]
enum StartFault {
    UnsupportedPowerOn,
}

const fn main() -> Result<(), StartFault> {
    Err(StartFault::UnsupportedPowerOn)
}
