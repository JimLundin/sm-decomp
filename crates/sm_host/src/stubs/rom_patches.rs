//! The engine's images are unpatched: no byte depends on the oracle's patch policy.
pub fn policy_dependent(_offset: usize) -> bool {
    false
}
