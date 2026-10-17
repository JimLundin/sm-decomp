use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{self, Read};
use std::path::Path;

// Carried forward from movies/manifest.toml and tools/dependencies.toml.
// These are input pins, not claims about any extracted or executable source.
pub const ROM: InputPin =
    InputPin { bytes: Some(3_145_728), sha256: "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72" };
pub const REFERENCE_ARCHIVE: InputPin =
    InputPin { bytes: None, sha256: "69c78831351112887575f852dadf0281dd5c23cef890b07a268ae6cf60aa9193" };
pub const REFERENCE_REVISION: &str = "578f90b3cc49557bb70060ad033bb90b8cf8ac50";

pub struct InputPin {
    bytes: Option<u64>,
    sha256: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Identity {
    pub bytes: u64,
    pub sha256: String,
}

pub fn check_file(path: &Path, pin: &InputPin) -> Result<Identity, String> {
    let file = File::open(path).map_err(|error| format!("cannot open input: {error}"))?;
    if !file.metadata().map_err(|error| format!("cannot inspect input: {error}"))?.is_file() {
        return Err("input must be a regular file".into());
    }
    check_reader(file, pin)
}

fn check_reader(mut reader: impl Read, pin: &InputPin) -> Result<Identity, String> {
    let mut hash = Sha256::new();
    let mut bytes = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => count,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(format!("cannot read input: {error}")),
        };
        hash.update(&buffer[..count]);
        bytes += count as u64;
    }
    if let Some(expected) = pin.bytes
        && bytes != expected
    {
        return Err(format!("size mismatch: expected {expected} bytes, observed {bytes}"));
    }
    let sha256 = format!("{:x}", hash.finalize());
    if sha256 != pin.sha256 {
        return Err(format!("SHA-256 mismatch: expected {}, observed {sha256}", pin.sha256));
    }
    Ok(Identity { bytes, sha256 })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ABC_PIN: InputPin =
        InputPin { bytes: Some(3), sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad" };

    #[test]
    fn accepts_a_known_sha256_vector_with_and_without_size_pin() {
        let expected = Identity { bytes: 3, sha256: ABC_PIN.sha256.into() };
        assert_eq!(check_reader(&b"abc"[..], &ABC_PIN).unwrap(), expected);
        let archive_pin = InputPin { bytes: None, sha256: ABC_PIN.sha256 };
        assert_eq!(check_reader(&b"abc"[..], &archive_pin).unwrap(), expected);
    }

    #[test]
    fn rejects_same_length_corruption_and_wrong_length() {
        assert!(check_reader(&b"abd"[..], &ABC_PIN).unwrap_err().contains("SHA-256 mismatch"));
        assert!(check_reader(&b"ab"[..], &ABC_PIN).unwrap_err().contains("size mismatch"));
    }

    #[test]
    fn propagates_a_read_failure_after_valid_prefix() {
        struct BrokenReader;
        impl Read for BrokenReader {
            fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
                Err(io::Error::other("synthetic read failure"))
            }
        }
        let reader = (&b"abc"[..]).chain(BrokenReader);
        assert!(check_reader(reader, &ABC_PIN).unwrap_err().contains("synthetic read failure"));
    }
}
