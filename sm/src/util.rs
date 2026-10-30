//! File and lexical helpers from the pinned native utility dependency.
pub use crate::types::{FILE, size_t};
use core::ffi::{c_char, c_void};

pub const NULL: *mut c_void = core::ptr::null_mut::<c_void>();

// Preserve the native lexical policy: directory bytes plus the full new name.
// Absolute names, dots and slash styles are copied without normalization.
#[cfg(all(not(sm_verify_dll), feature = "frontend"))]
pub fn include_path(base: &core::ffi::CStr, name: &core::ffi::CStr) -> std::ffi::CString {
    let base = base.to_bytes();
    let name = name.to_bytes_with_nul();
    let prefix = base
        .iter()
        .rposition(|&byte| byte == b'/' || byte == b'\\')
        .map_or(0, |index| index + 1);
    let mut path = Vec::with_capacity(prefix + name.len());
    path.extend_from_slice(&base[..prefix]);
    path.extend_from_slice(name);
    std::ffi::CString::from_vec_with_nul(path).expect("include path contains a nul byte")
}

/// Why a whole file could not be read.
pub enum ReadFailure {
    Open,
    Read,
}

/// The path a host C string names. Names that are not UTF-8 cannot be opened.
///
/// # Safety
/// `name` must point to a NUL-terminated string.
pub unsafe fn host_path(name: *const c_char) -> Option<std::path::PathBuf> {
    unsafe { core::ffi::CStr::from_ptr(name) }
        .to_str()
        .ok()
        .map(std::path::PathBuf::from)
}

/// Read a whole file, followed by one NUL byte for the C parsers that scan it.
///
/// # Safety
/// `name` must point to a NUL-terminated string.
pub unsafe fn read_whole_file(name: *const c_char) -> Result<Vec<u8>, ReadFailure> {
    use std::io::Read as _;
    let path = unsafe { host_path(name) }.ok_or(ReadFailure::Open)?;
    let mut file = std::fs::File::open(path).map_err(|_| ReadFailure::Open)?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|_| ReadFailure::Read)?;
    bytes.push(0);
    Ok(bytes)
}

/// Fill `buffer` from `source` as `fread` did, stopping early only at the end
/// of the file or on an error. Returns how many bytes arrived.
pub fn read_up_to(source: &mut impl std::io::Read, buffer: &mut [u8]) -> usize {
    let mut filled = 0;
    while filled < buffer.len() {
        match source.read(&mut buffer[filled..]) {
            Ok(0) => break,
            Ok(read) => filled += read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => {}
            Err(_) => break,
        }
    }
    filled
}

#[link(name = "sm_config", kind = "static", modifiers = "-bundle")]
unsafe extern "C-unwind" {
    fn sm_util_matches_source(index: u32, bytes: *const u8, length: usize) -> bool;
}

// The installed host callback may return or unwind. Do not declare it noreturn.
#[unsafe(no_mangle)]
unsafe extern "C-unwind" fn sm_util_die(message: *const c_char) {
    unsafe {
        crate::host::Die(message);
    }
}

#[doc(hidden)]
pub fn native_source_matches() -> bool {
    [
        include_bytes!("util.rs").as_slice(),
        include_bytes!("native_util.c").as_slice(),
        include_bytes!("native_util.h").as_slice(),
        include_bytes!("types.rs").as_slice(),
        include_bytes!("../../tools/install_native_config.py").as_slice(),
    ]
    .into_iter()
    .enumerate()
    .all(|(index, source)| unsafe {
        sm_util_matches_source(index as u32, source.as_ptr(), source.len())
    })
}
