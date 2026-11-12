#[cfg(test)]
mod t { #[test] fn leak() { let _ = std::fs::read("C:/secret"); } }
