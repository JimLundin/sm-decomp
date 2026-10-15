//! Streaming authentication of the pinned gzip/USTAR/PAX source archive.
use anyhow::{Context, Result, ensure};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::OpenOptions,
    io::Read,
    os::unix::fs::OpenOptionsExt,
    path::{Component, Path},
};
use tool_execution::bounded::{MAX_FILE_BYTES, account, check, count, limited, read_limit, safe, same_file_version};
const MAX_ARCHIVE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_ARCHIVE_EXPANDED_BYTES: u64 = 128 * 1024 * 1024;
const MAX_ARCHIVE_MEMBERS: usize = 4096;

/// Validate a gzip/USTAR/PAX archive against an existing pinned source tree.
/// It writes no extracted filesystem paths and retains only one 64 KiB block
/// plus bounded archive/header metadata while comparing each regular member.
pub(crate) fn authenticate_tar_source(
    archive: &Path,
    source: &Path,
    expected_sha256: &str,
) -> Result<BTreeMap<String, String>> {
    limited(|| {
        safe(source)?;
        let packed = read_limit(archive, MAX_ARCHIVE_BYTES)?;
        ensure!(format!("{:x}", Sha256::digest(&packed)) == expected_sha256, "pinned archive identity changed");
        let packed_bytes = packed.len() as u64;
        let mut decoder = flate2::bufread::GzDecoder::new(std::io::Cursor::new(packed));
        let mut expanded = 0u64;
        let mut files = BTreeMap::new();
        let mut names = BTreeSet::new();
        let mut root: Option<String> = None;
        let mut local_pax = BTreeMap::new();
        let mut zeros = 0;
        let mut members = 0usize;
        loop {
            check()?;
            let mut header = [0u8; 512];
            let observed = archive_read(&mut decoder, &mut header, &mut expanded)?;
            ensure!(observed == 512, "pinned tar header is truncated");
            if header.iter().all(|byte| *byte == 0) {
                zeros += 1;
                if zeros == 2 {
                    break;
                }
                continue;
            }
            ensure!(zeros == 0, "pinned tar archive contains entries after its terminator");
            members += 1;
            count(1)?;
            ensure!(members <= MAX_ARCHIVE_MEMBERS, "pinned archive member count exceeded");
            let checksum = tar_number(&header[148..156])?;
            let unsigned = header
                .iter()
                .enumerate()
                .map(|(index, byte)| if (148..156).contains(&index) { 32u64 } else { u64::from(*byte) })
                .sum::<u64>();
            ensure!(checksum == unsigned, "pinned tar checksum changed");
            ensure!(
                [&b"ustar\0"[..], &b"ustar "[..], &b"\0\0\0\0\0\0"[..]].contains(&&header[257..263]),
                "pinned tar header format unsupported"
            );
            let mut name = tar_text(&header[..100])?;
            let prefix = tar_text(&header[345..500])?;
            if !prefix.is_empty() {
                name = format!("{prefix}/{name}");
            }
            let mut bytes = tar_number(&header[124..136])?;
            let kind = header[156];
            if kind == b'x' || kind == b'g' {
                ensure!(bytes <= 65536, "pinned PAX metadata extent exceeded");
                let mut data = vec![0u8; usize::try_from(bytes)?];
                ensure!(
                    archive_read(&mut decoder, &mut data, &mut expanded)? == data.len(),
                    "pinned PAX record truncated"
                );
                let fields = pax_fields(&data)?;
                if kind == b'g' {
                    ensure!(
                        !fields.keys().any(|key| ["path", "size", "linkpath"].contains(&key.as_str())),
                        "pinned global PAX metadata changes member ownership"
                    );
                } else {
                    ensure!(local_pax.is_empty(), "pinned duplicate local PAX metadata");
                    local_pax = fields;
                }
                archive_padding(&mut decoder, bytes, &mut expanded)?;
                continue;
            }
            if let Some(path) = local_pax.remove("path") {
                name = path;
            }
            if let Some(size) = local_pax.remove("size") {
                bytes = size.parse()?;
            }
            ensure!(!local_pax.contains_key("linkpath"), "pinned source archive link metadata refused");
            local_pax.clear();
            ensure!(name.len() <= 4096 && !name.is_empty(), "pinned archive source path extent exceeded");
            let normalized = name.trim_end_matches('/');
            let path = Path::new(normalized);
            ensure!(
                !path.is_absolute()
                    && normalized.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
                    && path.components().all(|part| matches!(part, Component::Normal(_))),
                "unsafe source archive member"
            );
            let mut parts = path.components();
            let top = parts
                .next()
                .context("pinned archive root missing")?
                .as_os_str()
                .to_str()
                .context("archive root UTF8")?;
            if let Some(root) = &root {
                ensure!(root == top, "pinned archive has multiple source roots");
            } else {
                root = Some(top.into());
            }
            let relative = parts.as_path();
            ensure!(names.insert(normalized.to_owned()), "pinned archive repeats a source path");
            match kind {
                b'5' => ensure!(bytes == 0, "pinned archive directory has payload"),
                0 | b'0' => {
                    ensure!(
                        !relative.as_os_str().is_empty() && bytes <= MAX_FILE_BYTES,
                        "pinned source member extent exceeded"
                    );
                    let target = source.join(relative);
                    safe(&target)?;
                    let mut material = OpenOptions::new()
                        .read(true)
                        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
                        .open(&target)?;
                    let before = material.metadata()?;
                    ensure!(
                        before.is_file() && before.len() == bytes,
                        "pinned source extent changed: {}",
                        relative.display()
                    );
                    account(bytes)?;
                    let mut hash = Sha256::new();
                    let mut remaining = bytes;
                    let mut expected = [0u8; 64 * 1024];
                    let mut current = [0u8; 64 * 1024];
                    while remaining > 0 {
                        check()?;
                        let chunk = usize::try_from(remaining.min(expected.len() as u64))?;
                        ensure!(
                            archive_read(&mut decoder, &mut expected[..chunk], &mut expanded)? == chunk,
                            "pinned archive member is truncated"
                        );
                        material.read_exact(&mut current[..chunk])?;
                        ensure!(expected[..chunk] == current[..chunk], "pinned source changed: {}", relative.display());
                        hash.update(&current[..chunk]);
                        remaining -= chunk as u64;
                    }
                    ensure!(
                        same_file_version(&before, &material.metadata()?),
                        "pinned source changed during authentication"
                    );
                    files.insert(
                        relative.to_str().context("pinned member path UTF8")?.to_owned(),
                        format!("{:x}", hash.finalize()),
                    );
                }
                _ => anyhow::bail!("pinned source archive contains unsupported links or special entries"),
            }
            archive_padding(&mut decoder, bytes, &mut expanded)?;
        }
        ensure!(local_pax.is_empty() && !files.is_empty(), "empty or incomplete pinned source archive");
        let mut trailing = [0u8; 64 * 1024];
        loop {
            let n = archive_read(&mut decoder, &mut trailing, &mut expanded)?;
            if n == 0 {
                break;
            }
            ensure!(trailing[..n].iter().all(|byte| *byte == 0), "pinned archive trailing data changed");
        }
        ensure!(decoder.get_ref().position() == packed_bytes, "pinned gzip has trailing or concatenated data");
        Ok(files)
    })
}

fn archive_read(reader: &mut impl Read, buffer: &mut [u8], expanded: &mut u64) -> Result<usize> {
    let mut count = 0usize;
    while count < buffer.len() {
        check()?;
        let n = reader.read(&mut buffer[count..])?;
        if n == 0 {
            break;
        }
        *expanded = expanded.checked_add(n as u64).context("pinned archive expanded extent overflow")?;
        ensure!(*expanded <= MAX_ARCHIVE_EXPANDED_BYTES, "pinned archive expanded byte budget exceeded");
        account(n as u64)?;
        count += n;
    }
    Ok(count)
}
fn archive_padding(reader: &mut impl Read, bytes: u64, expanded: &mut u64) -> Result<()> {
    let count = usize::try_from((512 - bytes % 512) % 512)?;
    let mut padding = [0u8; 512];
    ensure!(archive_read(reader, &mut padding[..count], expanded)? == count, "pinned archive padding truncated");
    ensure!(padding[..count].iter().all(|byte| *byte == 0), "pinned archive padding changed");
    Ok(())
}
fn tar_text(bytes: &[u8]) -> Result<String> {
    let end = bytes.iter().position(|byte| *byte == 0).unwrap_or(bytes.len());
    ensure!(bytes[end..].iter().all(|byte| *byte == 0), "pinned archive path has trailing bytes");
    Ok(std::str::from_utf8(&bytes[..end])?.into())
}
fn tar_number(bytes: &[u8]) -> Result<u64> {
    ensure!(bytes.first().is_none_or(|byte| byte & 0x80 == 0), "pinned archive binary numeric fields unsupported");
    let text = std::str::from_utf8(bytes)?.trim_matches(|c| c == '\0' || c == ' ');
    if text.is_empty() {
        return Ok(0);
    }
    Ok(u64::from_str_radix(text, 8)?)
}
fn pax_fields(bytes: &[u8]) -> Result<BTreeMap<String, String>> {
    let mut fields = BTreeMap::new();
    let mut offset = 0usize;
    while offset < bytes.len() {
        check()?;
        let space =
            bytes[offset..].iter().position(|byte| *byte == b' ').context("pinned PAX length missing")? + offset;
        let length = std::str::from_utf8(&bytes[offset..space])?.parse::<usize>()?;
        let end = offset.checked_add(length).context("pinned PAX length overflow")?;
        ensure!(end <= bytes.len() && end > space + 2 && bytes[end - 1] == b'\n', "pinned PAX extent changed");
        let (key, value) =
            std::str::from_utf8(&bytes[space + 1..end - 1])?.split_once('=').context("pinned PAX value missing")?;
        ensure!(
            !key.is_empty() && fields.insert(key.to_owned(), value.to_owned()).is_none() && fields.len() <= 128,
            "pinned PAX fields repeated or excessive"
        );
        ensure!(
            [
                "path",
                "size",
                "linkpath",
                "mtime",
                "atime",
                "ctime",
                "uid",
                "gid",
                "uname",
                "gname",
                "comment",
                "charset",
                "LIBARCHIVE.creationtime"
            ]
            .contains(&key),
            "pinned PAX field changes unsupported archive semantics"
        );
        offset = end;
    }
    Ok(fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{append_tar, gzip_archive, tar_header};
    use std::{fs, path::PathBuf};
    use tool_execution::{
        bounded::{now_unix_ms, until},
        testing::Directory,
    };
    fn regular_archive(directory: &Path, data: &[u8]) -> Result<(PathBuf, PathBuf, String, Vec<u8>)> {
        let source = directory.join("source");
        fs::create_dir(&source)?;
        fs::write(source.join("file"), data)?;
        let archive = directory.join("source.tar.gz");
        let mut tar = Vec::new();
        append_tar(&mut tar, "root/", b'5', b"");
        append_tar(&mut tar, "root/file", b'0', data);
        tar.extend_from_slice(&[0u8; 1024]);
        let hash = gzip_archive(&archive, &tar)?;
        Ok((archive, source, hash, tar))
    }

    #[test]
    fn tar_compares_exact_bytes_and_rejects_changed_source_archive_and_gzip_tail() -> Result<()> {
        let directory = Directory::new()?;
        let (archive, source, hash, _) = regular_archive(&directory.0, b"original")?;
        let files = authenticate_tar_source(&archive, &source, &hash)?;
        assert_eq!(files, BTreeMap::from([("file".into(), format!("{:x}", Sha256::digest(b"original")))]));
        fs::write(source.join("file"), b"tampered")?;
        assert!(authenticate_tar_source(&archive, &source, &hash).is_err());
        fs::write(source.join("file"), b"original")?;
        assert!(authenticate_tar_source(&archive, &source, &"f".repeat(64)).is_err());
        let mut packed = fs::read(&archive)?;
        packed.extend_from_slice(b"trailing gzip bytes");
        fs::write(&archive, &packed)?;
        assert!(authenticate_tar_source(&archive, &source, &format!("{:x}", Sha256::digest(packed))).is_err());
        Ok(())
    }

    #[test]
    fn tar_rejects_links_duplicate_unsafe_and_truncated_members() -> Result<()> {
        let directory = Directory::new()?;
        let source = directory.0.join("source");
        fs::create_dir(&source)?;
        fs::write(source.join("file"), b"exact")?;
        let archive = directory.0.join("archive.gz");
        for (name, kind, duplicate) in [
            ("root/file", b'1', false),
            ("root/file", b'2', false),
            ("root/file", b'6', false),
            ("root/file", b'0', true),
            ("root/../file", b'0', false),
            ("root/./file", b'0', false),
            ("root//file", b'0', false),
            ("/root/file", b'0', false),
        ] {
            let mut tar = Vec::new();
            append_tar(&mut tar, name, kind, b"exact");
            if duplicate {
                append_tar(&mut tar, name, kind, b"exact");
            }
            tar.extend_from_slice(&[0u8; 1024]);
            let hash = gzip_archive(&archive, &tar)?;
            assert!(authenticate_tar_source(&archive, &source, &hash).is_err(), "accepted {name}/{kind}");
        }
        let mut tar = tar_header("root/file", b'0', 5).to_vec();
        tar.extend_from_slice(b"exa");
        let hash = gzip_archive(&archive, &tar)?;
        assert!(authenticate_tar_source(&archive, &source, &hash).is_err());
        let hash = gzip_archive(&archive, &[0u8; 511])?;
        assert!(authenticate_tar_source(&archive, &source, &hash).is_err());
        Ok(())
    }

    #[test]
    fn tar_expansion_and_member_count_share_outer_limits() -> Result<()> {
        let directory = Directory::new()?;
        let (archive, source, hash, _) = regular_archive(&directory.0, &[0u8; 4096])?;
        let cap = archive.metadata()?.len() + 1024;
        let error =
            until(now_unix_ms()? + 1000, cap, || authenticate_tar_source(&archive, &source, &hash)).unwrap_err();
        assert!(error.to_string().contains("budget"));
        let mut tar = Vec::new();
        for number in 0..=MAX_ARCHIVE_MEMBERS {
            append_tar(&mut tar, &format!("root/directory-{number}/"), b'5', b"");
        }
        tar.extend_from_slice(&[0u8; 1024]);
        let hash = gzip_archive(&archive, &tar)?;
        let error = authenticate_tar_source(&archive, &source, &hash).unwrap_err();
        assert!(error.to_string().contains("member count"));
        Ok(())
    }

    #[test]
    fn pax_path_size_and_metadata_parse_without_extraction() -> Result<()> {
        fn field(key: &str, value: &str) -> String {
            let payload = format!(" {key}={value}\n");
            let mut length = payload.len() + 1;
            loop {
                let next = payload.len() + length.to_string().len();
                if next == length {
                    return format!("{length}{payload}");
                }
                length = next;
            }
        }
        let directory = Directory::new()?;
        let (archive, source, _, _) = regular_archive(&directory.0, b"exact")?;
        let fields = format!("{}{}{}", field("path", "root/file"), field("size", "5"), field("mtime", "12.5"));
        let mut tar = Vec::new();
        append_tar(&mut tar, "pax", b'x', fields.as_bytes());
        append_tar(&mut tar, "ignored", b'0', b"exact");
        tar.extend_from_slice(&[0u8; 1024]);
        let hash = gzip_archive(&archive, &tar)?;
        assert!(authenticate_tar_source(&archive, &source, &hash)?.contains_key("file"));
        let duplicate = format!("{}{}", field("path", "root/file"), field("path", "other/file"));
        assert!(pax_fields(duplicate.as_bytes()).is_err());
        assert!(pax_fields(field("GNU.sparse.map", "0,1").as_bytes()).is_err());
        assert!(pax_fields(b"999 path=outside\n").is_err());
        Ok(())
    }
}
