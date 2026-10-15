//! Exact synthetic pinned archives for source-authentication tests.
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::{fs, io::Write, path::Path};
pub fn tar_header(name: &str, kind: u8, size: u64) -> [u8; 512] {
    assert!(name.len() <= 100);
    let mut header = [0u8; 512];
    header[..name.len()].copy_from_slice(name.as_bytes());
    for (offset, extent, value) in [(100, 8, 0o644), (108, 8, 0), (116, 8, 0), (124, 12, size), (136, 12, 0)] {
        let text = format!("{:0width$o}\0", value, width = extent - 1);
        header[offset..offset + extent].copy_from_slice(text.as_bytes());
    }
    header[148..156].fill(b' ');
    header[156] = kind;
    header[257..263].copy_from_slice(b"ustar\0");
    header[263..265].copy_from_slice(b"00");
    let checksum: u64 = header.iter().map(|byte| u64::from(*byte)).sum();
    header[148..156].copy_from_slice(format!("{checksum:06o}\0 ").as_bytes());
    header
}

pub fn append_tar(bytes: &mut Vec<u8>, name: &str, kind: u8, data: &[u8]) {
    bytes.extend_from_slice(&tar_header(name, kind, data.len() as u64));
    bytes.extend_from_slice(data);
    bytes.resize(bytes.len().div_ceil(512) * 512, 0);
}

pub fn gzip_archive(path: &Path, tar: &[u8]) -> Result<String> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(tar)?;
    let packed = encoder.finish()?;
    fs::write(path, &packed)?;
    Ok(format!("{:x}", Sha256::digest(packed)))
}
