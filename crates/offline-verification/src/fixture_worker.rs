//! Single-threaded, preloaded fixture image. Compiled into generated subjects.
//! The parent never invokes gameplay; each request gets a fresh fork child.
use std::{
    ffi::{CString, c_char, c_int, c_void},
    io::{self, Read, Write},
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

const LIMIT: usize = 65536;
const SIGTERM: c_int = 15;
const SIGKILL: c_int = 9;
const PR_SET_PDEATHSIG: c_int = 1;
const PR_SET_CHILD_SUBREAPER: c_int = 36;
const WNOHANG: c_int = 1;
const WEXITED: c_int = 4;
const WNOWAIT: c_int = 0x1000000;
const ECHILD: c_int = 10;
static STOP: AtomicBool = AtomicBool::new(false);
extern "C" fn stop(_: c_int) {
    STOP.store(true, Ordering::Relaxed);
}
#[repr(C)]
struct Timeval {
    seconds: i64,
    micros: i64,
}
#[repr(C)]
struct Usage {
    user: Timeval,
    system: Timeval,
    rest: [i64; 14],
}
// Linux x86_64 rusage: two timeval values followed by fourteen longs.
// waitid's 128-byte siginfo stores si_pid at byte 16 on this ABI.
unsafe extern "C" {
    fn fork() -> c_int;
    fn getpid() -> c_int;
    fn getppid() -> c_int;
    fn setpgid(pid: c_int, group: c_int) -> c_int;
    fn prctl(option: c_int, ...) -> c_int;
    fn signal(signal: c_int, handler: usize) -> usize;
    fn kill(pid: c_int, signal: c_int) -> c_int;
    fn waitid(kind: c_int, pid: u32, info: *mut c_void, flags: c_int) -> c_int;
    fn wait4(pid: c_int, status: *mut c_int, flags: c_int, usage: *mut Usage) -> c_int;
    fn waitpid(pid: c_int, status: *mut c_int, flags: c_int) -> c_int;
    fn dup2(old: c_int, new: c_int) -> c_int;
    fn fflush(stream: *mut c_void) -> c_int;
    fn _exit(code: c_int) -> !;
}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn exact(input: &mut impl Read, bytes: &mut [u8]) -> io::Result<()> {
    let mut offset = 0;
    while offset < bytes.len() {
        match input.read(&mut bytes[offset..]) {
            Ok(0) => return Err(io::ErrorKind::UnexpectedEof.into()),
            Ok(n) => offset += n,
            Err(e) if e.kind() == io::ErrorKind::Interrupted && !STOP.load(Ordering::Relaxed) => {}
            Err(e) => return Err(e),
        }
    }
    Ok(())
}
fn take<'a>(bytes: &mut &'a [u8], n: usize) -> io::Result<&'a [u8]> {
    if n > bytes.len() {
        return Err(invalid("truncated fixture request"));
    }
    let (value, rest) = bytes.split_at(n);
    *bytes = rest;
    Ok(value)
}
fn word(bytes: &mut &[u8]) -> io::Result<u64> {
    Ok(u64::from_le_bytes(take(bytes, 8)?.try_into().unwrap()))
}
fn string(bytes: &mut &[u8]) -> io::Result<String> {
    let n = word(bytes)? as usize;
    String::from_utf8(take(bytes, n)?.to_vec()).map_err(|_| invalid("invalid fixture string"))
}
fn read(input: &mut impl Read) -> io::Result<Option<(u64, u64, String, String, Vec<String>)>> {
    let mut first = [0; 1];
    match input.read(&mut first) {
        Ok(0) => return Ok(None),
        Ok(_) => {}
        Err(e) => return Err(e),
    }
    let mut header = [0; 4];
    header[0] = first[0];
    exact(input, &mut header[1..])?;
    let n = u32::from_le_bytes(header) as usize;
    if n > LIMIT {
        return Err(invalid("oversized fixture request"));
    }
    let mut raw = vec![0; n];
    exact(input, &mut raw)?;
    let mut bytes = raw.as_slice();
    let id = word(&mut bytes)?;
    let timeout = word(&mut bytes)?;
    let directory = string(&mut bytes)?;
    let label = string(&mut bytes)?;
    let count = word(&mut bytes)?;
    if !(1..=32).contains(&count) || !(1..=240000).contains(&timeout) || !["rust", "c"].contains(&label.as_str()) {
        return Err(invalid("invalid fixture invocation"));
    }
    let args = (0..count).map(|_| string(&mut bytes)).collect::<io::Result<Vec<_>>>()?;
    if !bytes.is_empty() {
        return Err(invalid("trailing fixture request"));
    }
    Ok(Some((id, timeout, directory, label, args)))
}
fn flush() {
    let _ = io::stdout().flush();
    let _ = io::stderr().flush();
    unsafe {
        fflush(std::ptr::null_mut());
    }
}
struct ChildGuard(c_int);
impl Drop for ChildGuard {
    fn drop(&mut self) {
        if self.0 == 0 {
            return;
        }
        unsafe {
            kill(-self.0, SIGKILL);
            kill(self.0, SIGKILL);
        }
        let mut status = 0;
        loop {
            if unsafe { waitpid(self.0, &mut status, 0) } >= 0 {
                break;
            }
            if io::Error::last_os_error().kind() != io::ErrorKind::Interrupted {
                break;
            }
        }
    }
}

pub fn serve(mut invoke: impl FnMut(&[String]) -> i32) -> io::Result<()> {
    // This ABI is deliberately Linux x86_64; never guess another platform layout.
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) {
        return Err(invalid("fixture workers require Linux x86_64"));
    }
    unsafe {
        if prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) != 0 {
            return Err(io::Error::last_os_error());
        }
        if signal(SIGTERM, stop as *const () as usize) == usize::MAX {
            return Err(io::Error::last_os_error());
        }
    }
    let mut output = io::stdout().lock();
    output.write_all(b"SMFX0001")?;
    output.flush()?;
    drop(output);
    let mut input = io::stdin().lock();
    while !STOP.load(Ordering::Relaxed) {
        let request = match read(&mut input) {
            Ok(Some(r)) => r,
            Ok(None) => break,
            Err(_) if STOP.load(Ordering::Relaxed) => break,
            Err(e) => return Err(e),
        };
        let (id, timeout, directory, label, args) = request;
        flush();
        let parent = unsafe { getpid() };
        let started = Instant::now();
        let pid = unsafe { fork() };
        if pid < 0 {
            return Err(io::Error::last_os_error());
        }
        if pid == 0 {
            unsafe {
                signal(SIGTERM, 0);
                if setpgid(0, 0) != 0 || prctl(PR_SET_PDEATHSIG, SIGKILL, 0, 0, 0) != 0 {
                    _exit(98);
                }
                if getppid() != parent {
                    _exit(98);
                }
            }
            let result = (|| -> io::Result<i32> {
                use std::os::fd::AsRawFd;
                let directory = std::path::Path::new(&directory);
                let stdout = std::fs::File::create(directory.join(format!("{label}.stdout")))?;
                let stderr = std::fs::File::create(directory.join(format!("{label}.stderr")))?;
                if unsafe { dup2(stdout.as_raw_fd(), 1) } < 0 || unsafe { dup2(stderr.as_raw_fd(), 2) } < 0 {
                    return Err(io::Error::last_os_error());
                }
                std::env::set_current_dir(directory)?;
                Ok(invoke(&args))
            })();
            let code = match result {
                Ok(code) => code,
                Err(e) => {
                    eprintln!("fixture child: {e}");
                    4
                }
            };
            flush();
            unsafe {
                _exit(code);
            }
        }
        unsafe {
            setpgid(pid, pid);
        }
        let mut child_guard = ChildGuard(pid);
        let mut timed_out = false;
        // Observe without reaping: the leader still anchors its process group.
        loop {
            let mut info = [0u64; 16];
            let result = unsafe { waitid(1, pid as u32, info.as_mut_ptr().cast(), WNOHANG | WEXITED | WNOWAIT) };
            if result < 0 {
                let e = io::Error::last_os_error();
                if e.kind() != io::ErrorKind::Interrupted {
                    return Err(e);
                }
            } else if info[2] as u32 != 0 {
                break;
            }
            if STOP.load(Ordering::Relaxed) || started.elapsed() >= Duration::from_millis(timeout) {
                timed_out = true;
                unsafe {
                    kill(-pid, SIGKILL);
                    kill(pid, SIGKILL);
                }
            }
            std::thread::sleep(Duration::from_micros(200));
        }
        unsafe {
            kill(-pid, SIGKILL);
        }
        let mut usage: Usage = unsafe { std::mem::zeroed() };
        let mut status = 0;
        loop {
            if unsafe { wait4(pid, &mut status, 0, &mut usage) } == pid {
                break;
            }
            let e = io::Error::last_os_error();
            if e.kind() != io::ErrorKind::Interrupted {
                return Err(e);
            }
        }
        child_guard.0 = 0;
        let mut descendants = 0;
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let mut ignored = 0;
            let result = unsafe { waitpid(-1, &mut ignored, WNOHANG) };
            if result > 0 {
                descendants += 1;
                continue;
            }
            if result < 0 {
                let e = io::Error::last_os_error();
                if e.raw_os_error() == Some(ECHILD) {
                    break;
                }
                if e.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                return Err(e);
            }
            if Instant::now() >= deadline {
                return Err(invalid("fixture descendants did not exit"));
            }
            std::thread::sleep(Duration::from_micros(200));
        }
        if STOP.load(Ordering::Relaxed) {
            break;
        }
        let cpu = |v: Timeval| v.seconds as u64 * 1_000_000_000 + v.micros as u64 * 1000;
        let values = [
            id,
            pid as u64,
            status as u32 as u64,
            started.elapsed().as_nanos() as u64,
            cpu(usage.user),
            cpu(usage.system),
            usage.rest[0] as u64 * 1024,
            timed_out as u64,
            descendants,
        ];
        let mut output = io::stdout().lock();
        for value in values {
            output.write_all(&value.to_le_bytes())?;
        }
        output.flush()?;
    }
    Ok(())
}

/// C subject entry: C preloads ROM and supplies its ordinary invocation function.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sm_fixture_worker(invoke: unsafe extern "C" fn(c_int, *mut *mut c_char) -> c_int) -> c_int {
    match serve(|args| {
        let strings = match args.iter().map(|s| CString::new(s.as_bytes())).collect::<Result<Vec<_>, _>>() {
            Ok(v) => v,
            Err(_) => return 4,
        };
        let mut pointers = strings.iter().map(|s| s.as_ptr() as *mut c_char).collect::<Vec<_>>();
        pointers.push(std::ptr::null_mut());
        unsafe { invoke(args.len() as c_int, pointers.as_mut_ptr()) }
    }) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("fixture worker: {e}");
            4
        }
    }
}
