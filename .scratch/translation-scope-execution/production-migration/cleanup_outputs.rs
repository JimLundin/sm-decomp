//! Dispose only this migration turn's reviewed intermediates.
//! Build with rustc; no Cargo packages or external commands are required.
//! Run without arguments (or --plan) first; --apply is the only deleting mode.
use std::{
    collections::BTreeMap,
    ffi::{CString, c_char, c_int, c_ulong},
    fs,
    io::{self, Read},
    os::unix::{ffi::OsStrExt, fs::MetadataExt},
    path::{Component, Path, PathBuf},
};

#[cfg(not(all(target_os = "linux", target_pointer_width = "64", target_env = "gnu")))]
compile_error!("The measured-space ABI is reviewed for 64-bit Linux/glibc only.");

const ROOT: &str = "/home/jim/sm-decomp-translation-execution";
const MANIFEST: &str = ".scratch/translation-scope-execution/production-migration/verification/compile-inputs.sha256";
const DISPOSABLES: [&str; 2] =
    ["/tmp/sm-production-migration-target-20261014", "/tmp/sm-production-capture-recovery-20261014"];
const C_INPUTS: [&str; 6] = [
    "deps/snesrev-sm/src/sm_a0.c",
    "deps/snesrev-sm/src/sm_a6.c",
    "deps/snesrev-sm/src/sm_86.c",
    "deps/snesrev-sm/src/sm_80.c",
    "deps/snesrev-sm/src/sm_90.c",
    "deps/snesrev-sm/src/snes/snes.c",
];

fn reject(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}

// This is glibc's 64-bit Linux statvfs layout; the cfg above fences other ABIs.
#[repr(C)]
#[derive(Default)]
struct Statvfs {
    block_size: c_ulong,
    fragment_size: c_ulong,
    blocks: u64,
    free_blocks: u64,
    available_blocks: u64,
    files: u64,
    free_files: u64,
    available_files: u64,
    filesystem_id: c_ulong,
    flags: c_ulong,
    name_max: c_ulong,
    spare: [c_int; 6],
}
unsafe extern "C" {
    fn statvfs(path: *const c_char, output: *mut Statvfs) -> c_int;
}

fn available_bytes(path: &Path) -> io::Result<u64> {
    let path = CString::new(path.as_os_str().as_bytes()).map_err(|_| reject("space path contains NUL"))?;
    let mut output = Statvfs::default();
    // Both pointers remain valid for this synchronous libc call.
    if unsafe { statvfs(path.as_ptr(), &mut output) } != 0 {
        return Err(io::Error::last_os_error());
    }
    output.available_blocks.checked_mul(output.fragment_size).ok_or_else(|| reject("space extent overflow"))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Entry {
    path: PathBuf,
    directory: bool,
    device: u64,
    inode: u64,
    size: u64,
    allocated: u64,
    links: u64,
    mode: u32,
    modified: (i64, i64),
    changed: (i64, i64),
}

fn observe(path: &Path) -> io::Result<Entry> {
    let metadata = path.symlink_metadata()?;
    if metadata.file_type().is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
        return Err(reject(format!("nonregular or aliased disposal input: {}", path.display())));
    }
    Ok(Entry {
        path: path.into(),
        directory: metadata.is_dir(),
        device: metadata.dev(),
        inode: metadata.ino(),
        size: metadata.len(),
        allocated: metadata.blocks().checked_mul(512).ok_or_else(|| reject("allocated extent overflow"))?,
        links: metadata.nlink(),
        mode: metadata.mode(),
        modified: (metadata.mtime(), metadata.mtime_nsec()),
        changed: (metadata.ctime(), metadata.ctime_nsec()),
    })
}

fn safe_parents(path: &Path) -> io::Result<()> {
    if !path.is_absolute() || path.components().collect::<PathBuf>().as_os_str() != path.as_os_str() {
        return Err(reject("disposal path must be absolute and canonical"));
    }
    let mut prefix = PathBuf::new();
    for component in path.parent().ok_or_else(|| reject("disposal path has no parent"))?.components() {
        if !matches!(component, Component::RootDir | Component::Normal(_)) {
            return Err(reject("disposal path alias"));
        }
        prefix.push(component);
        let metadata = prefix.symlink_metadata()?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(reject(format!("disposal ancestor is not a real directory: {}", prefix.display())));
        }
    }
    Ok(())
}

fn collect(path: &Path, entries: &mut Vec<Entry>, depth: usize) -> io::Result<()> {
    if depth > 64 || entries.len() >= 2_000_000 {
        return Err(reject("disposal inventory exceeds reviewed bounds"));
    }
    let entry = observe(path)?;
    let directory = entry.directory;
    entries.push(entry);
    if directory {
        let mut children =
            fs::read_dir(path)?.map(|entry| entry.map(|entry| entry.path())).collect::<io::Result<Vec<_>>>()?;
        children.sort();
        for child in children {
            collect(&child, entries, depth + 1)?;
        }
    }
    Ok(())
}

fn sha256(bytes: &[u8]) -> String {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98,
        0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
        0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8,
        0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819,
        0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
        0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut words = [0x6a09e667u32, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19];
    let mut padded = bytes.to_vec();
    padded.push(0x80);
    while padded.len() % 64 != 56 {
        padded.push(0);
    }
    padded.extend_from_slice(&((bytes.len() as u64) * 8).to_be_bytes());
    for chunk in padded.chunks_exact(64) {
        let mut schedule = [0u32; 64];
        for (word, bytes) in schedule[..16].iter_mut().zip(chunk.chunks_exact(4)) {
            *word = u32::from_be_bytes(bytes.try_into().unwrap());
        }
        for i in 16..64 {
            let a = schedule[i - 15];
            let b = schedule[i - 2];
            let s0 = a.rotate_right(7) ^ a.rotate_right(18) ^ (a >> 3);
            let s1 = b.rotate_right(17) ^ b.rotate_right(19) ^ (b >> 10);
            schedule[i] = schedule[i - 16].wrapping_add(s0).wrapping_add(schedule[i - 7]).wrapping_add(s1);
        }
        let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut h] = words;
        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let choice = (e & f) ^ (!e & g);
            let t1 = h.wrapping_add(s1).wrapping_add(choice).wrapping_add(K[i]).wrapping_add(schedule[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let majority = (a & b) ^ (a & c) ^ (b & c);
            let t2 = s0.wrapping_add(majority);
            h = g;
            g = f;
            f = e;
            e = d.wrapping_add(t1);
            d = c;
            c = b;
            b = a;
            a = t1.wrapping_add(t2);
        }
        for (value, addition) in words.iter_mut().zip([a, b, c, d, e, f, g, h]) {
            *value = value.wrapping_add(addition);
        }
    }
    words.iter().map(|word| format!("{word:08x}")).collect()
}

fn bounded_bytes(path: &Path, maximum: u64) -> io::Result<Vec<u8>> {
    let entry = observe(path)?;
    if entry.directory || entry.size > maximum {
        return Err(reject(format!("hash input is not a bounded regular file: {}", path.display())));
    }
    let mut bytes = Vec::new();
    fs::File::open(path)?.take(maximum + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 != entry.size || observe(path)? != entry {
        return Err(reject("hash input changed while reading"));
    }
    Ok(bytes)
}

fn manifest() -> io::Result<(Vec<u8>, BTreeMap<String, String>)> {
    let path = Path::new(ROOT).join(MANIFEST);
    safe_parents(&path)?;
    let bytes = bounded_bytes(&path, 64 * 1024)?;
    let mut hashes = BTreeMap::new();
    for line in std::str::from_utf8(&bytes).map_err(|_| reject("manifest is not UTF-8"))?.lines() {
        let (hash, name) = line.split_once("  ").ok_or_else(|| reject("invalid SHA-256 manifest line"))?;
        if hash.len() != 64
            || !hash.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !C_INPUTS.contains(&name)
            || hashes.insert(name.to_owned(), hash.to_ascii_lowercase()).is_some()
        {
            return Err(reject("hash manifest has an unexpected, duplicated or invalid compile input"));
        }
    }
    if hashes.len() != C_INPUTS.len() {
        return Err(reject("hash manifest must contain exactly the six reviewed C copies"));
    }
    Ok((bytes, hashes))
}

fn check_copies(hashes: &BTreeMap<String, String>) -> io::Result<()> {
    for (name, expected) in hashes {
        let path = Path::new(ROOT).join(name);
        safe_parents(&path)?;
        match path.symlink_metadata() {
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(error),
            Ok(_) => {}
        }
        if sha256(&bounded_bytes(&path, 16 * 1024 * 1024)?) != *expected {
            return Err(reject(format!("compile-input hash differs; preserving {}", path.display())));
        }
    }
    Ok(())
}

fn totals(entries: &[Entry]) -> io::Result<(u64, u64, u64)> {
    let mut logical = 0u64;
    let mut inodes = BTreeMap::new();
    for entry in entries {
        logical = logical.checked_add(entry.size).ok_or_else(|| reject("logical extent overflow"))?;
        let inode = inodes.entry((entry.device, entry.inode)).or_insert((0u64, entry));
        inode.0 += 1;
    }
    let mut allocated = 0u64;
    let mut reclaimable = 0u64;
    for (count, entry) in inodes.values() {
        allocated = allocated.checked_add(entry.allocated).ok_or_else(|| reject("allocated extent overflow"))?;
        if entry.directory || *count == entry.links {
            reclaimable =
                reclaimable.checked_add(entry.allocated).ok_or_else(|| reject("reclaimable extent overflow"))?;
        }
    }
    Ok((logical, allocated, reclaimable))
}

fn main() -> io::Result<()> {
    let arguments = std::env::args().skip(1).collect::<Vec<_>>();
    let apply = match arguments.as_slice() {
        [] => false,
        [mode] if mode == "--plan" => false,
        [mode] if mode == "--apply" => true,
        _ => return Err(reject("usage: cleanup_outputs [--plan|--apply]")),
    };
    if sha256(b"") != "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        || sha256(b"abc") != "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    {
        return Err(reject("internal SHA-256 known-answer validation failed"));
    }
    let (manifest_bytes, hashes) = manifest()?;
    check_copies(&hashes)?;
    let before_root = available_bytes(Path::new(ROOT))?;
    let before_tmp = available_bytes(Path::new("/tmp"))?;
    let action = if apply { "apply" } else { "plan" };
    let targets = DISPOSABLES
        .iter()
        .map(PathBuf::from)
        .chain(C_INPUTS.iter().map(|path| Path::new(ROOT).join(path)))
        .collect::<Vec<_>>();
    let mut all = Vec::new();
    let mut present = 0;
    for path in &targets {
        safe_parents(path)?;
        match path.symlink_metadata() {
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                println!(
                    "{{\"schema\":\"translation-production-disposal-item-v1\",\"action\":\"{action}\",\"path\":{path:?},\"present\":false}}"
                );
                continue;
            }
            Err(error) => return Err(error),
            Ok(_) => {}
        }
        present += 1;
        let mut entries = Vec::new();
        collect(path, &mut entries, 0)?;
        let (logical, allocated, reclaimable) = totals(&entries)?;
        println!(
            "{{\"schema\":\"translation-production-disposal-item-v1\",\"action\":\"{action}\",\"path\":{path:?},\"present\":true,\"entries\":{},\"logical_bytes\":{logical},\"allocated_bytes\":{allocated},\"reclaimable_allocated_bytes\":{reclaimable}}}",
            entries.len()
        );
        all.extend(entries);
    }
    let (logical, allocated, reclaimable) = totals(&all)?;
    if apply {
        if manifest()?.0 != manifest_bytes {
            return Err(reject("compile-input manifest changed after measurement"));
        }
        check_copies(&hashes)?;
        for expected in &all {
            if observe(&expected.path)? != *expected {
                return Err(reject(format!("disposal subject changed after measurement: {}", expected.path.display())));
            }
        }
        // Remove exactly measured files; unexpected newly created children make
        // remove_dir fail instead of broadening deletion into unreviewed output.
        let mut removed_links = BTreeMap::<(u64, u64), u64>::new();
        for expected in all.iter().filter(|entry| !entry.directory) {
            safe_parents(&expected.path)?;
            let observed = observe(&expected.path)?;
            let removed = removed_links.entry((expected.device, expected.inode)).or_default();
            let mut remaining = expected.clone();
            remaining.links =
                remaining.links.checked_sub(*removed).ok_or_else(|| reject("disposal hardlink count changed"))?;
            if *removed > 0 {
                // Removing an earlier reviewed alias changes this inode's
                // ctime and link count; its source bytes/mtime stay fixed.
                remaining.changed = observed.changed;
            }
            if observed != remaining {
                return Err(reject("disposal file changed immediately before removal"));
            }
            fs::remove_file(&expected.path)?;
            *removed += 1;
        }
        for expected in all.iter().rev().filter(|entry| entry.directory) {
            safe_parents(&expected.path)?;
            let observed = observe(&expected.path)?;
            if !observed.directory || observed.device != expected.device || observed.inode != expected.inode {
                return Err(reject("disposal directory was replaced before removal"));
            }
            fs::remove_dir(&expected.path)?;
        }
    }
    let after_root = available_bytes(Path::new(ROOT))?;
    let after_tmp = available_bytes(Path::new("/tmp"))?;
    println!(
        "{{\"schema\":\"translation-production-disposal-summary-v1\",\"action\":\"{action}\",\"reviewed_targets\":{},\"present_targets\":{present},\"entries\":{},\"logical_bytes\":{logical},\"allocated_bytes\":{allocated},\"reclaimable_allocated_bytes\":{reclaimable},\"removed_logical_bytes\":{},\"removed_reclaimable_allocated_bytes\":{},\"workspace_available_before\":{before_root},\"workspace_available_after\":{after_root},\"tmp_available_before\":{before_tmp},\"tmp_available_after\":{after_tmp},\"compile_manifest_sha256\":\"{}\"}}",
        targets.len(),
        all.len(),
        if apply { logical } else { 0 },
        if apply { reclaimable } else { 0 },
        sha256(&manifest_bytes)
    );
    Ok(())
}
