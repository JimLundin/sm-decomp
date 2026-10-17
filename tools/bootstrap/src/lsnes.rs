//! Pinned headless lsnes build, with compatibility-only source edits.
use crate::{digest, document, run};
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
#[derive(Deserialize)]
struct Manifest {
    schema_version: u32,
    archive: Download,
    packages: Vec<Download>,
}
#[derive(Deserialize)]
struct Download {
    filename: String,
    url: String,
    sha256: String,
    source_tar_sha256: Option<String>,
}
use std::{
    ffi::{CStr, CString, c_char, c_int, c_void},
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

#[link(name = "dl")]
unsafe extern "C" {
    fn dlopen(name: *const c_char, flags: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
}
struct Archive {
    handle: *mut c_void,
    new: unsafe extern "C" fn() -> *mut c_void,
    format: unsafe extern "C" fn(*mut c_void) -> c_int,
    filter: unsafe extern "C" fn(*mut c_void) -> c_int,
    open: unsafe extern "C" fn(*mut c_void, *const c_char, usize) -> c_int,
    next: unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> c_int,
    name: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    data: unsafe extern "C" fn(*mut c_void, *mut c_void, usize) -> isize,
    error: unsafe extern "C" fn(*mut c_void) -> *const c_char,
    free: unsafe extern "C" fn(*mut c_void) -> c_int,
}
impl Archive {
    fn load() -> Result<Self> {
        let handle = unsafe { dlopen(c"libarchive.so.13".as_ptr(), 2) };
        ensure!(!handle.is_null(), "libarchive runtime is required to read the official 7z");
        macro_rules! symbol {
            ($name:literal, $ty:ty) => {{
                let name = CString::new($name)?;
                let address = unsafe { dlsym(handle, name.as_ptr()) };
                if address.is_null() {
                    unsafe {
                        dlclose(handle);
                    }
                    anyhow::bail!("missing libarchive symbol: {}", $name);
                }
                // Each declared type is the corresponding public libarchive ABI.
                unsafe { std::mem::transmute::<*mut c_void, $ty>(address) }
            }};
        }
        Ok(Self {
            handle,
            new: symbol!("archive_read_new", unsafe extern "C" fn() -> *mut c_void),
            format: symbol!("archive_read_support_format_7zip", unsafe extern "C" fn(*mut c_void) -> c_int),
            filter: symbol!("archive_read_support_filter_all", unsafe extern "C" fn(*mut c_void) -> c_int),
            open: symbol!(
                "archive_read_open_filename",
                unsafe extern "C" fn(*mut c_void, *const c_char, usize) -> c_int
            ),
            next: symbol!("archive_read_next_header", unsafe extern "C" fn(*mut c_void, *mut *mut c_void) -> c_int),
            name: symbol!("archive_entry_pathname", unsafe extern "C" fn(*mut c_void) -> *const c_char),
            data: symbol!("archive_read_data", unsafe extern "C" fn(*mut c_void, *mut c_void, usize) -> isize),
            error: symbol!("archive_error_string", unsafe extern "C" fn(*mut c_void) -> *const c_char),
            free: symbol!("archive_read_free", unsafe extern "C" fn(*mut c_void) -> c_int),
        })
    }
}
impl Drop for Archive {
    fn drop(&mut self) {
        unsafe {
            dlclose(self.handle);
        }
    }
}
struct Reader<'a>(*mut c_void, &'a Archive);
impl Drop for Reader<'_> {
    fn drop(&mut self) {
        unsafe {
            (self.1.free)(self.0);
        }
    }
}
impl Reader<'_> {
    fn checked(&self, status: isize) -> Result<isize> {
        if status < 0 {
            let error = unsafe { (self.1.error)(self.0) };
            anyhow::bail!(
                "libarchive: {}",
                if error.is_null() {
                    "read failed".to_owned()
                } else {
                    unsafe { CStr::from_ptr(error) }.to_string_lossy().into_owned()
                }
            );
        }
        Ok(status)
    }
}
fn extract(archive: &Path, destination: &Path) -> Result<()> {
    use std::os::unix::ffi::OsStrExt;
    let name = CString::new(archive.as_os_str().as_bytes())?;
    let api = Archive::load()?;
    let reader = Reader(unsafe { (api.new)() }, &api);
    ensure!(!reader.0.is_null(), "could not allocate libarchive reader");
    unsafe {
        reader.checked((api.filter)(reader.0) as isize)?;
        reader.checked((api.format)(reader.0) as isize)?;
        reader.checked((api.open)(reader.0, name.as_ptr(), 65536) as isize)?;
        let mut entry = std::ptr::null_mut();
        while reader.checked((api.next)(reader.0, &mut entry) as isize)? != 1 {
            let name = (api.name)(entry);
            ensure!(!name.is_null(), "archive entry lacks pathname");
            if Path::new(CStr::from_ptr(name).to_str()?).file_name().is_none_or(|n| n != "sources.tar") {
                continue;
            }
            let mut output = File::create(destination)?;
            let mut buffer = [0u8; 65536];
            loop {
                let count = reader.checked((api.data)(reader.0, buffer.as_mut_ptr().cast(), buffer.len()))? as usize;
                if count == 0 {
                    return Ok(());
                }
                output.write_all(&buffer[..count])?;
            }
        }
    }
    anyhow::bail!("official archive contains no sources.tar")
}
fn fetch(root: &Path, record: &Download, offline: bool) -> Result<PathBuf> {
    let archive = root.join("deps/downloads").join(&record.filename);
    fs::create_dir_all(archive.parent().unwrap())?;
    if !archive.exists() {
        ensure!(!offline, "missing cached download: {}", archive.display());
        let partial = archive.with_extension("part");
        run(
            root,
            Command::new("curl")
                .args([
                    "-L",
                    "--fail",
                    "--silent",
                    "--show-error",
                    "--retry",
                    "3",
                    "--connect-timeout",
                    "30",
                    &record.url,
                    "-o",
                ])
                .arg(&partial),
        )?;
        ensure!(digest(&partial)? == record.sha256.as_str(), "download checksum mismatch");
        fs::rename(partial, &archive)?;
    }
    ensure!(digest(&archive)? == record.sha256.as_str(), "archive checksum mismatch: {}", archive.display());
    Ok(archive)
}
fn write(path: &Path, text: &str) -> Result<()> {
    if fs::read_to_string(path).ok().as_deref() != Some(text) {
        fs::write(path, text)?;
    }
    Ok(())
}
pub fn configure(source: &Path, original: &Path, libraries: &Path) -> Result<()> {
    let libraries = fs::canonicalize(libraries)?;
    for (relative, header) in [
        ("include/library/httpreq.hpp", "functional"),
        ("include/library/loadlib.hpp", "functional"),
        ("bsnes/snes/snes.hpp", "vector"),
    ] {
        let path = source.join(relative);
        let text = fs::read_to_string(&path)?;
        let include = format!("#include <{header}>\n");
        if !text.contains(&include) {
            write(&path, &format!("{include}{text}"))?;
        }
    }
    let path = source.join("bsnes/Makefile");
    write(&path, &fs::read_to_string(&path)?.replace("-std=gnu++0x", "-std=gnu++14"))?;
    let path = source.join("Makefile");
    let text = fs::read_to_string(&path)?;
    if !text.contains("\nheadless:") {
        write(
            &path,
            &format!(
                "{text}\n.PHONY: headless\nheadless: src/core/version.cpp buildaux/mkdeps$(DOT_EXECUTABLE_SUFFIX) buildaux/txt2cstr$(DOT_EXECUTABLE_SUFFIX)\n\t$(MAKE) -C src precheck\n\t$(MAKE) -C src util/lsnes-dumpavi.util\n"
            ),
        )?;
    }
    let include = libraries.join("usr/include");
    let lib = libraries.join("usr/lib/x86_64-linux-gnu");
    let linker = format!("-L{} -Wl,-rpath,{}", lib.display(), lib.display());
    let mut settings = std::collections::BTreeMap::from([
        ("CC", "g++".to_owned()),
        ("LD", "ld".into()),
        ("RANLIB", "ranlib".into()),
        ("CROSS_PREFIX", "".into()),
        ("HOSTCC", "$(CC) $(USER_HOSTCCFLAGS) -DNATIVE_THREADS".into()),
        ("USER_HOSTCCFLAGS", format!("-I{} {linker}", include.display())),
        (
            "USER_CFLAGS",
            format!(
                "-std=gnu++14 -O2 -g -Wno-deprecated -I{} -I{}/lua5.2 -I{}/x86_64-linux-gnu",
                include.display(),
                include.display(),
                include.display()
            ),
        ),
        ("USER_LDFLAGS", linker),
        ("LUA", "lua5.2".into()),
        ("REGEX", "NATIVE".into()),
        ("THREADS", "NATIVE".into()),
        ("GRAPHICS", "DUMMY".into()),
        ("SOUND", "DUMMY".into()),
        ("JOYSTICK", "DUMMY".into()),
        ("BUILD_BSNES", "yes".into()),
        ("BSNES_IS_COMPAT", "yes".into()),
    ]);
    for key in [
        "BUILD_GAMBATTE",
        "SECRET_RABBIT_CODE",
        "OPUS_CODEC",
        "OPUS_CODEC_SUPPORTS_SURROUND",
        "USE_LIBGCRYPT",
        "USE_LIBLZMA",
        "BOOST_NEEDS_MT",
        "HOST_BOOST_NEEDS_MT",
    ] {
        settings.insert(key, String::new());
    }
    let output = Command::new("tar").arg("-xOf").arg(original).arg("options.build").output()?;
    ensure!(output.status.success(), "cannot read official build options");
    let mut options = String::from_utf8(output.stdout)?;
    for (key, value) in settings {
        let prefix = format!("{key}=");
        ensure!(
            options.lines().filter(|line| line.starts_with(&prefix)).count() == 1,
            "unexpected original build setting: {key}"
        );
        options = options
            .split_inclusive('\n')
            .map(|line| {
                if line.starts_with(&prefix) {
                    format!("{prefix}{value}{}", if line.ends_with('\n') { "\n" } else { "" })
                } else {
                    line.to_owned()
                }
            })
            .collect();
    }
    write(&source.join("options.build"), &options)
}
pub fn setup(root: &Path, offline: bool, jobs: usize, configure_only: bool) -> Result<()> {
    ensure!(
        cfg!(all(target_os = "linux", target_arch = "x86_64")),
        "lsnes private development packages require Linux x86_64"
    );
    ensure!(jobs > 0, "jobs must be positive");
    ensure!(
        !root.to_string_lossy().chars().any(|c| c.is_whitespace() || c == '$' || c == '#'),
        "legacy build requires a repository path without whitespace, $ or #"
    );
    let local = root.join("deps/lsnes");
    let original = local.join("sources.tar");
    let source = local.join("source");
    let libraries = root.join("deps/lsnes-build-deps");
    let manifest: Manifest = document(&root.join("docs/lsnes-dependencies.toml"))?;
    ensure!(manifest.schema_version == 1, "unknown lsnes manifest version");
    if configure_only {
        ensure!(
            digest(&original)? == manifest.archive.source_tar_sha256.as_deref().context("missing source tar digest")?,
            "original source tar checksum mismatch"
        );
        return configure(&source, &original, &libraries);
    }
    let curl = Path::new("/usr/lib/x86_64-linux-gnu/libcurl.so.4");
    ensure!(
        Path::new("/usr/include/zlib.h").exists() && curl.exists(),
        "host zlib headers and libcurl OpenSSL runtime are required"
    );
    let archive = fetch(root, &manifest.archive, offline)?;
    fs::create_dir_all(&local)?;
    if !original.exists() {
        extract(&archive, &original)?;
    }
    ensure!(
        digest(&original)? == manifest.archive.source_tar_sha256.as_deref().context("missing source tar digest")?,
        "original source tar checksum mismatch"
    );
    if !source.join("VERSION").exists() {
        fs::create_dir_all(&source)?;
        run(root, Command::new("tar").arg("-xf").arg(&original).arg("-C").arg(&source))?;
    }
    fs::create_dir_all(&libraries)?;
    for record in &manifest.packages {
        let archive = fetch(root, record, offline)?;
        let stamp = libraries.join(format!(".setup-{}", record.filename));
        if fs::read_to_string(&stamp).ok().is_none_or(|v| v.trim() != record.sha256) {
            run(root, Command::new("dpkg-deb").arg("-x").arg(&archive).arg(&libraries))?;
            fs::write(stamp, format!("{}\n", record.sha256.as_str()))?;
        }
    }
    for (alias, target) in [
        (libraries.join("usr/bin/lua"), PathBuf::from("lua5.2")),
        (libraries.join("usr/bin/luac"), PathBuf::from("luac5.2")),
        (libraries.join("usr/lib/x86_64-linux-gnu/libcurl.so"), curl.to_path_buf()),
    ] {
        if alias.is_symlink() {
            if fs::read_link(&alias)? == target {
                continue;
            }
            fs::remove_file(&alias)?;
        }
        ensure!(!alias.exists(), "unexpected file at private symlink: {}", alias.display());
        std::os::unix::fs::symlink(target, alias)?;
    }
    configure(&source, &original, &libraries)?;
    let log = local.join("build.log");
    println!("Building lsnes rr2-beta23 / bsnes v085 Compatibility; log: {}", log.display());
    let output = File::create(log)?;
    let status = Command::new("make")
        .arg(format!("-j{jobs}"))
        .arg("headless")
        .current_dir(source)
        .env("PATH", format!("{}:{}", libraries.join("usr/bin").display(), std::env::var("PATH").unwrap_or_default()))
        .env("PKG_CONFIG_PATH", libraries.join("usr/lib/x86_64-linux-gnu/pkgconfig"))
        .stdout(Stdio::from(output.try_clone()?))
        .stderr(Stdio::from(output))
        .status()?;
    ensure!(status.success(), "lsnes build failed: {status}");
    Ok(())
}
