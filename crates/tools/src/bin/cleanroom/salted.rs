//! The lexicon's hash. The salt is public, so a reader with the salt can test a guess; what the
//! files don't give is a list to read.

/// FNV-1a over the salt, a separator and the item.
pub fn hash(salt: &str, item: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in salt.bytes().chain([0]).chain(item.bytes()) {
        h ^= u64::from(byte);
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}
