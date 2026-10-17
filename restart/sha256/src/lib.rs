//! Compatibility boundary for SHA-256 evidence identities. The maintained
//! RustCrypto implementation owns hashing; this module only preserves the
//! project's existing streaming API, fixed output bytes and lowercase hex.
use sha2_core::Digest as CoreDigest;

#[cfg(any(test, feature = "private-checks"))]
mod controls;

#[cfg(feature = "private-checks")]
pub fn private_assertions() -> Vec<(&'static str, fn())> {
    vec![("independent_sha256_raw_streaming_clone_vectors", controls::independent_adapter_vectors as fn())]
}

#[derive(Clone)]
pub struct Sha256(sha2_core::Sha256);

pub trait Digest: Sized {
    fn new() -> Self;
    fn update(&mut self, bytes: impl AsRef<[u8]>);
    fn finalize(self) -> Output;
    fn digest(bytes: impl AsRef<[u8]>) -> Output {
        let mut hash = Self::new();
        hash.update(bytes);
        hash.finalize()
    }
}

impl Digest for Sha256 {
    fn new() -> Self {
        Self(sha2_core::Sha256::new())
    }
    fn update(&mut self, bytes: impl AsRef<[u8]>) {
        self.0.update(bytes);
    }
    fn finalize(self) -> Output {
        Output(self.0.finalize().into())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Output([u8; 32]);

impl From<Output> for [u8; 32] {
    fn from(output: Output) -> Self {
        output.0
    }
}
impl AsRef<[u8]> for Output {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
impl std::ops::Deref for Output {
    type Target = [u8; 32];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl std::fmt::LowerHex for Output {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Digest, Sha256};
    #[test]
    fn independent_adapter_vectors() {
        super::controls::independent_adapter_vectors();
    }
    #[test]
    fn literal_vectors_and_streaming() {
        assert_eq!(
            format!("{:x}", Sha256::digest(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        let mut hash = Sha256::new();
        hash.update(b"a");
        hash.update(b"bc");
        assert_eq!(<[u8; 32]>::from(hash.finalize()), <[u8; 32]>::from(Sha256::digest(b"abc")));
    }
}
