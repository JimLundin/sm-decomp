//! Linux producer confinement. No writable host directory enters the sandbox.
//!
//! The trusted helper mounts one size-limited tmpfs, captures all material output
//! there, drops privileges before exec, and returns only declared bounded bytes.
use crate::budget::ByteBudget;
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::{Component, Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReadOnlyBind {
    pub source: PathBuf,
    pub destination: PathBuf,
}

/// Constructed by installed execution adapters, never by rewrite responses.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Program {
    pub executable: PathBuf,
    pub arguments: Vec<String>,
    pub inputs: Vec<ReadOnlyBind>,
    pub runtime: Vec<ReadOnlyBind>,
    pub helper: PathBuf,
    /// Regular files relative to /work. Other persistent outputs are rejected.
    pub outputs: Vec<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RunLimits {
    pub timeout: Duration,
    pub writable_bytes: u64,
    pub stdout_bytes: usize,
    pub stderr_bytes: usize,
    pub stdin_bytes: usize,
}

#[derive(Debug)]
pub struct RunOutput {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: ExitStatus,
    pub outputs: BTreeMap<PathBuf, Vec<u8>>,
}

#[derive(Serialize, Deserialize)]
struct HelperLimits {
    writable_bytes: u64,
    stdout_bytes: usize,
    stderr_bytes: usize,
    outputs: Vec<PathBuf>,
}

const MAGIC: &[u8; 8] = b"SMTX0001";

fn bounded_text(value: &str, maximum: usize) -> &str {
    let mut end = value.len().min(maximum);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    &value[..end]
}

fn relative(path: &Path) -> bool {
    !path.as_os_str().is_empty() && path.components().all(|c| matches!(c, Component::Normal(_)))
}

fn real_path(path: &Path) -> Result<()> {
    ensure!(path.is_absolute(), "sandbox source must be absolute");
    let mut part = PathBuf::new();
    for component in path.components() {
        ensure!(!matches!(component, Component::ParentDir | Component::CurDir), "sandbox source alias");
        part.push(component);
        ensure!(!part.symlink_metadata()?.file_type().is_symlink(), "sandbox source symlink: {}", part.display());
    }
    Ok(())
}

fn self_executable_marker(bind: &ReadOnlyBind) -> bool {
    bind.destination.as_os_str() == std::ffi::OsStr::new("/proc/self/exe")
}

fn validate(program: &Program, limits: &RunLimits, input: &[u8]) -> Result<()> {
    ensure!(cfg!(all(target_os = "linux", target_arch = "x86_64")), "strict confinement requires Linux x86_64");
    ensure!(!limits.timeout.is_zero(), "sandbox timeout must be positive");
    ensure!(limits.writable_bytes >= 4096, "sandbox writable quota is below one page");
    ensure!(input.len() <= limits.stdin_bytes, "sandbox stdin budget exceeded");
    real_path(&program.executable)?;
    real_path(&program.helper)?;
    ensure!(program.executable.is_file() && program.helper.is_file(), "sandbox executable/helper is not a file");
    let mut destinations = BTreeSet::new();
    for bind in program.inputs.iter().chain(&program.runtime) {
        real_path(&bind.source)?;
        ensure!(
            bind.destination.is_absolute() && relative(bind.destination.strip_prefix("/")?),
            "unsafe sandbox bind destination"
        );
        let self_executable = self_executable_marker(bind);
        if self_executable {
            ensure!(
                bind.source.as_os_str() == program.executable.as_os_str()
                    && bind.source.as_os_str() == fs::canonicalize(&program.executable)?.as_os_str(),
                "synthetic self executable source differs or is aliased"
            );
        }
        ensure!(
            self_executable
                || !["/work", "/tmp", "/proc", "/dev", "/program", "/trusted-helper", "/quota-proc"]
                    .iter()
                    .any(|p| bind.destination.starts_with(p)),
            "reserved sandbox destination"
        );
        ensure!(
            bind.destination != Path::new("/home") && bind.destination != Path::new("/root"),
            "broad sandbox runtime/input mount refused"
        );
        if program.inputs.iter().any(|input| std::ptr::eq(input, bind)) {
            ensure!(
                !["/usr", "/lib", "/lib64"].iter().any(|path| bind.destination.starts_with(path)),
                "input overlaps executable runtime"
            );
        }
        ensure!(destinations.insert(bind.destination.clone()), "duplicate sandbox destination");
    }
    ensure!(program.outputs.len() <= 1024, "too many sandbox outputs");
    let mut outputs = BTreeSet::new();
    for path in &program.outputs {
        ensure!(relative(path) && !path.starts_with("tmp"), "unsafe sandbox output declaration");
        ensure!(outputs.insert(path.clone()), "duplicate sandbox output declaration");
    }
    ensure!(
        program.arguments.iter().map(String::len).sum::<usize>() <= 1024 * 1024,
        "sandbox argument extent exceeded"
    );
    Ok(())
}

/// Minimal dynamic runtime for the installed Rust helper on this Linux host.
/// Bind canonical files individually, so this does not expose host programs.
pub fn helper_runtime() -> Result<Vec<ReadOnlyBind>> {
    [
        ("/lib/x86_64-linux-gnu/libc.so.6", "/lib/x86_64-linux-gnu/libc.so.6"),
        ("/lib/x86_64-linux-gnu/libgcc_s.so.1", "/lib/x86_64-linux-gnu/libgcc_s.so.1"),
        ("/lib/x86_64-linux-gnu/libpthread.so.0", "/lib/x86_64-linux-gnu/libpthread.so.0"),
        ("/lib/x86_64-linux-gnu/libm.so.6", "/lib/x86_64-linux-gnu/libm.so.6"),
        ("/lib/x86_64-linux-gnu/libdl.so.2", "/lib/x86_64-linux-gnu/libdl.so.2"),
        ("/lib64/ld-linux-x86-64.so.2", "/lib64/ld-linux-x86-64.so.2"),
    ]
    .into_iter()
    .map(|(source, destination)| {
        Ok(ReadOnlyBind { source: fs::canonicalize(source)?, destination: destination.into() })
    })
    .collect()
}

/// Installed compiler adapter runtime. The toolchain mount contains no HOME,
/// package cache, credentials, checkout or controller state.
pub fn compiler_runtime(rustc: &Path) -> Result<Vec<ReadOnlyBind>> {
    let rustc = fs::canonicalize(rustc)?;
    real_path(&rustc)?;
    let toolchain = rustc.parent().and_then(Path::parent).context("rustc toolchain root missing")?;
    ensure!(
        toolchain.join("bin/rustc") == rustc && toolchain.join("lib/rustlib").is_dir(),
        "rustc must be an installed compiler, not a rustup shim"
    );
    let mut binds = vec![ReadOnlyBind { source: fs::canonicalize("/usr")?, destination: "/usr".into() }];
    for path in ["/lib", "/lib64"] {
        binds.push(ReadOnlyBind { source: fs::canonicalize(path)?, destination: path.into() });
    }
    binds.push(ReadOnlyBind { source: toolchain.to_path_buf(), destination: "/toolchain".into() });
    Ok(binds)
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn run(program: &Program, limits: &RunLimits, input: &[u8]) -> Result<RunOutput> {
    run_observed(program, limits, input, |_| Ok(()))
}

/// The durable host runner records the exact namespace owner before waiting
/// for its result. Candidate code never receives this callback or host state.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub(crate) fn run_observed(
    program: &Program,
    limits: &RunLimits,
    input: &[u8],
    observed: impl FnOnce(u32) -> Result<()>,
) -> Result<RunOutput> {
    use std::os::unix::process::CommandExt;
    validate(program, limits, input)?;
    let quota = limits.writable_bytes / 4096 * 4096;
    let config = HelperLimits {
        writable_bytes: quota,
        stdout_bytes: limits.stdout_bytes,
        stderr_bytes: limits.stderr_bytes,
        outputs: program.outputs.clone(),
    };
    let mut command = Command::new("/usr/bin/bwrap");
    command
        .env_clear()
        .args([
            "--unshare-all",
            "--uid",
            "0",
            "--gid",
            "0",
            "--cap-add",
            "CAP_SYS_ADMIN",
            "--cap-add",
            "CAP_SETPCAP",
            "--die-with-parent",
        ])
        .args(["--dir", "/work", "--dir", "/dev", "--dir", "/quota-proc"])
        // A normal bind is nodev in bubblewrap. Only these two reviewed device
        // nodes may retain device semantics; no host /dev directory is exposed.
        .args(["--dev-bind", "/dev/null", "/dev/null", "--dev-bind", "/dev/urandom", "/dev/urandom"])
        .arg("--ro-bind")
        .arg(&program.helper)
        .arg("/trusted-helper")
        .arg("--ro-bind")
        .arg(&program.executable)
        .arg("/program");
    for bind in program.runtime.iter().chain(&program.inputs) {
        if self_executable_marker(bind) {
            // This explicit marker names the already staged executable. It
            // never binds procfs, a host proc path, or a second executable.
            command.args(["--dir", "/proc", "--dir", "/proc/self", "--symlink", "/program", "/proc/self/exe"]);
        } else {
            command.arg("--ro-bind").arg(&bind.source).arg(&bind.destination);
        }
    }
    command
        .args([
            "--symlink",
            "/work/tmp",
            "/tmp",
            "--remount-ro",
            "/",
            "--setenv",
            "PATH",
            "/nonexistent",
            "--setenv",
            "CARGO_HOME",
            "/work/tmp/cargo-home",
            "--setenv",
            "TMPDIR",
            "/work/tmp",
            "--setenv",
            "HOME",
            "/work/tmp/home",
            "--chdir",
            "/work",
            "--",
            "/trusted-helper",
        ])
        .arg(serde_json::to_string(&config)?)
        .arg("--")
        .args(&program.arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    // A separate host process group belongs only to this bounded invocation.
    unsafe {
        command.pre_exec(|| {
            if libc::setpgid(0, 0) < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let mut child = command.spawn().context("cannot establish sandbox namespaces")?;
    let pid = child.id();
    if let Err(error) = observed(pid) {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
        let _ = child.kill();
        let _ = child.wait();
        return Err(error);
    }
    let mut input_pipe = child.stdin.take().context("sandbox stdin missing")?;
    let output_pipe = child.stdout.take().context("sandbox stdout missing")?;
    let error_pipe = child.stderr.take().context("sandbox stderr missing")?;
    let exceeded = Arc::new(AtomicBool::new(false));
    let wire_limit = usize::try_from(quota)?
        .checked_add(64 + program.outputs.len() * 4096)
        .context("sandbox wire budget overflow")?;
    let started = Instant::now();
    let result = std::thread::scope(|scope| -> Result<RunOutput> {
        let send = scope.spawn(move || {
            // No candidate can begin before the durable namespace identity
            // callback succeeds. The helper consumes this trusted launch byte;
            // candidate stdin receives only the original bounded request.
            input_pipe.write_all(b"S")?;
            input_pipe.write_all(input)
        });
        let read = |mut pipe: Box<dyn Read + Send>, limit: usize, exceeded: Arc<AtomicBool>| {
            let mut out = Vec::new();
            let mut buffer = [0u8; 8192];
            loop {
                let count = pipe.read(&mut buffer)?;
                if count == 0 {
                    break;
                }
                if out.len().checked_add(count).is_none_or(|n| n > limit) {
                    exceeded.store(true, Ordering::SeqCst);
                    bail!("sandbox response budget exceeded");
                }
                out.extend_from_slice(&buffer[..count]);
            }
            Ok::<_, anyhow::Error>(out)
        };
        let output = scope.spawn({
            let exceeded = exceeded.clone();
            move || read(Box::new(output_pipe), wire_limit, exceeded)
        });
        let error = scope.spawn({
            let exceeded = exceeded.clone();
            move || read(Box::new(error_pipe), limits.stderr_bytes.min(65536), exceeded)
        });
        let mut interrupted = None;
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if exceeded.load(Ordering::SeqCst) || started.elapsed() >= limits.timeout {
                interrupted = Some(if exceeded.load(Ordering::SeqCst) {
                    "sandbox response budget exceeded"
                } else {
                    "sandbox timeout"
                });
                unsafe {
                    libc::kill(-(pid as i32), libc::SIGKILL);
                }
                let _ = child.kill();
                break child.wait()?;
            }
            std::thread::sleep(Duration::from_millis(5));
        };
        // Namespace pid 1 exits with the helper. Explicitly reap the host group
        // before joining pipes, including failed/bootstrap paths.
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
        let send_result = send.join().map_err(|_| anyhow::anyhow!("sandbox stdin thread panicked"))?;
        let output = output.join().map_err(|_| anyhow::anyhow!("sandbox stdout thread panicked"))??;
        let error = error.join().map_err(|_| anyhow::anyhow!("sandbox stderr thread panicked"))??;
        if let Some(reason) = interrupted {
            bail!("{reason}");
        }
        ensure!(status.success(), "sandbox setup/finalization failed: {}", String::from_utf8_lossy(&error));
        send_result.context("sandbox stdin transfer failed")?;
        decode(&output, &config)
    });
    if result.is_err() {
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
        let _ = child.kill();
        let _ = child.wait();
    }
    result
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
pub fn run(_: &Program, _: &RunLimits, _: &[u8]) -> Result<RunOutput> {
    bail!("strict confinement requires Linux x86_64")
}

fn put_bytes(out: &mut dyn Write, bytes: &[u8]) -> Result<()> {
    out.write_all(&(bytes.len() as u64).to_le_bytes())?;
    out.write_all(bytes)?;
    Ok(())
}
fn get_bytes(input: &mut &[u8], maximum: usize) -> Result<Vec<u8>> {
    ensure!(input.len() >= 8, "truncated sandbox result");
    let count = usize::try_from(u64::from_le_bytes(input[..8].try_into()?))?;
    *input = &input[8..];
    ensure!(count <= maximum && count <= input.len(), "sandbox member extent exceeded");
    let bytes = input[..count].to_vec();
    *input = &input[count..];
    Ok(bytes)
}
fn decode(mut input: &[u8], config: &HelperLimits) -> Result<RunOutput> {
    use std::os::unix::process::ExitStatusExt;
    ensure!(input.len() >= 12 && &input[..8] == MAGIC, "sandbox result lacks trusted completion");
    let status = ExitStatus::from_raw(i32::from_le_bytes(input[8..12].try_into()?));
    input = &input[12..];
    let stdout = get_bytes(&mut input, config.stdout_bytes)?;
    let stderr = get_bytes(&mut input, config.stderr_bytes)?;
    let mut budget = ByteBudget::new(config.writable_bytes);
    budget.reserve((stdout.len() + stderr.len()) as u64)?;
    let mut outputs = BTreeMap::new();
    for path in &config.outputs {
        let bytes = get_bytes(&mut input, usize::try_from(budget.remaining())?)?;
        budget.reserve(bytes.len() as u64)?;
        outputs.insert(path.clone(), bytes);
    }
    ensure!(input.is_empty(), "sandbox result has trailing bytes");
    Ok(RunOutput { stdout, stderr, status, outputs })
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn errno(result: i32, operation: &str) -> Result<()> {
    ensure!(result >= 0, "{operation}: {}", std::io::Error::last_os_error());
    Ok(())
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn restrict_child(writable_bytes: u64) -> Result<()> {
    // The helper already has its executable mapped. Remove its host controller
    // image from the namespace before candidate exec and capability dropping.
    errno(unsafe { libc::umount2(c"/trusted-helper".as_ptr(), libc::MNT_DETACH) }, "hide trusted controller helper")?;
    // The trusted observer retains its own detached proc directory descriptor.
    // The candidate namespace cannot see proc, including observer descriptors.
    errno(unsafe { libc::umount2(c"/quota-proc".as_ptr(), libc::MNT_DETACH) }, "hide trusted quota observation")?;
    // The trusted helper observes quota errors, including errors a producer
    // deliberately ignores before deleting temporary files and returning zero.
    errno(unsafe { libc::ptrace(libc::PTRACE_TRACEME, 0, 0, 0) as i32 }, "establish quota-error observation")?;
    let file_limit = libc::rlimit { rlim_cur: writable_bytes, rlim_max: writable_bytes };
    errno(unsafe { libc::setrlimit(libc::RLIMIT_FSIZE, &file_limit) }, "bound sparse file extents")?;
    let core_limit = libc::rlimit { rlim_cur: 0, rlim_max: 0 };
    errno(unsafe { libc::setrlimit(libc::RLIMIT_CORE, &core_limit) }, "disable core output")?;
    // Preserve locked capability semantics through future execs even though uid
    // is namespace root; no exec can recover the parent's mount capability.
    const SECBITS: libc::c_ulong = 1 | 2 | 4 | 8;
    errno(unsafe { libc::prctl(libc::PR_SET_SECUREBITS, SECBITS, 0, 0, 0) }, "lock securebits")?;
    for cap in 0..=40 {
        errno(unsafe { libc::prctl(libc::PR_CAPBSET_DROP, cap, 0, 0, 0) }, "drop capability bounding set")?;
    }
    #[repr(C)]
    struct Header {
        version: u32,
        pid: i32,
    }
    #[repr(C)]
    struct Caps {
        effective: u32,
        permitted: u32,
        inheritable: u32,
    }
    let header = Header { version: 0x20080522, pid: 0 };
    let caps =
        [Caps { effective: 0, permitted: 0, inheritable: 0 }, Caps { effective: 0, permitted: 0, inheritable: 0 }];
    errno(unsafe { libc::syscall(libc::SYS_capset, &header, caps.as_ptr()) as i32 }, "drop effective capabilities")?;
    errno(unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) }, "set no-new-privileges")?;
    // Architecture check also rejects compat/x32 syscalls that could bypass the
    // numeric deny list. Namespace setup belongs solely to the trusted helper.
    let statement = |code, k| libc::sock_filter { code, jt: 0, jf: 0, k };
    let jump = |k, jt, jf| libc::sock_filter { code: 0x15, jt, jf, k };
    let mut filter = vec![
        statement(0x20, 4),
        jump(0xc000003e, 1, 0),
        statement(0x06, 0x80000000),
        statement(0x20, 0),
        libc::sock_filter { code: 0x35, jt: 0, jf: 1, k: 0x40000000 },
        statement(0x06, 0x80000000),
    ];
    // clone3 is deliberately unavailable; libc falls back to clone. Permit
    // ordinary forks/threads but refuse every namespace flag in clone.
    filter.push(jump(libc::SYS_clone3 as u32, 0, 1));
    filter.push(statement(0x06, 0x00050000 | libc::ENOSYS as u32));
    filter.push(jump(libc::SYS_clone as u32, 0, 4));
    filter.push(statement(0x20, 16));
    filter.push(libc::sock_filter {
        code: 0x45,
        jt: 0,
        jf: 1,
        k: (libc::CLONE_NEWNS
            | libc::CLONE_NEWTIME
            | libc::CLONE_NEWUTS
            | libc::CLONE_NEWIPC
            | libc::CLONE_NEWUSER
            | libc::CLONE_NEWPID
            | libc::CLONE_NEWNET
            | libc::CLONE_NEWCGROUP) as u32,
    });
    filter.push(statement(0x06, 0x00050000 | libc::EPERM as u32));
    filter.push(statement(0x20, 0));
    // Rust's pre_exec error handshake uses an already connected local Unix
    // pair. Permit only that exact close-on-exec form; socket() and every other
    // socketpair domain/type/protocol remain unavailable.
    let socket_pair_arguments = [
        (16, libc::AF_UNIX as u32),
        (20, 0),
        (24, (libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC) as u32),
        (28, 0),
        (32, 0),
        (36, 0),
    ];
    filter.push(jump(libc::SYS_socketpair as u32, 0, u8::try_from(socket_pair_arguments.len() * 3 + 1)?));
    for (offset, expected) in socket_pair_arguments {
        filter.push(statement(0x20, offset));
        filter.push(jump(expected, 1, 0));
        filter.push(statement(0x06, 0x00050000 | libc::EPERM as u32));
    }
    filter.push(statement(0x06, 0x7fff0000));
    filter.push(statement(0x20, 0));
    let forbidden = [
        libc::SYS_mount,
        libc::SYS_umount2,
        libc::SYS_pivot_root,
        libc::SYS_chroot,
        libc::SYS_unshare,
        libc::SYS_setns,
        libc::SYS_socket,
        libc::SYS_ptrace,
        libc::SYS_process_vm_readv,
        libc::SYS_process_vm_writev,
        libc::SYS_open_by_handle_at,
        libc::SYS_bpf,
        libc::SYS_userfaultfd,
        libc::SYS_perf_event_open,
        libc::SYS_keyctl,
        libc::SYS_add_key,
        libc::SYS_request_key,
        libc::SYS_fsopen,
        libc::SYS_fsconfig,
        libc::SYS_fsmount,
        libc::SYS_move_mount,
        libc::SYS_open_tree,
        libc::SYS_mount_setattr,
        // Asynchronous IO hides write/quota results from syscall-exit tracing.
        // Installed compilers use ordinary traced IO; candidates cannot create
        // an alternative, unobserved output producer.
        libc::SYS_io_uring_setup,
        libc::SYS_io_uring_enter,
        libc::SYS_io_uring_register,
        libc::SYS_io_setup,
        libc::SYS_io_submit,
        // Anonymous file-backed output would use a different tmpfs superblock
        // and escape both this mount's quota and its inode extent observer.
        libc::SYS_memfd_create,
        libc::SYS_shmget,
    ];
    for syscall in forbidden {
        filter.push(jump(syscall as u32, 0, 1));
        filter.push(statement(0x06, 0x00050000 | libc::EPERM as u32));
    }
    filter.push(statement(0x06, 0x7fff0000));
    let program = libc::sock_fprog { len: filter.len().try_into()?, filter: filter.as_mut_ptr() };
    errno(unsafe { libc::prctl(libc::PR_SET_SECCOMP, 2, &program, 0, 0) }, "install sandbox syscall restrictions")?;
    errno(unsafe { libc::syscall(libc::SYS_close_range, 3u32, u32::MAX, 0u32) as i32 }, "close inherited descriptors")?;
    Ok(())
}

/// Peek only at a PID still owned by this tracer and never reaped by another
/// caller. WNOWAIT preserves that identity and the main loop's exit result;
/// stopped, running, or inaccessible live tasks are never treated as exited.
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn terminated_owned_task(pid: i32) -> Result<bool> {
    let mut info = unsafe { std::mem::zeroed::<libc::siginfo_t>() };
    loop {
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                pid as u32,
                &mut info,
                libc::WEXITED | libc::WNOHANG | libc::WNOWAIT | libc::__WALL,
            )
        };
        if result == 0 {
            return Ok(info.si_signo == libc::SIGCHLD
                && unsafe { info.si_pid() } == pid
                && [libc::CLD_EXITED, libc::CLD_KILLED, libc::CLD_DUMPED].contains(&info.si_code));
        }
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() != Some(libc::EINTR) {
            return Err(error).context("authenticate owned task termination");
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
fn wait_traced(pid: u32, writable_bytes: u64, proc: &File, logs: &[File]) -> Result<ExitStatus> {
    use std::os::unix::process::ExitStatusExt;
    use std::os::unix::{
        fs::MetadataExt,
        io::{AsRawFd, FromRawFd},
    };
    // Directories and regular file extents are observed before any mutating
    // syscall resumes. Inodes deduplicate hard links and unlinked open files;
    // the trusted log files count even though they have no directory entry.
    let work_device = fs::metadata("/work")?.dev();
    let mut observed = BTreeSet::from([pid as i32]);
    fn extent(path: &Path, device: u64, files: &mut BTreeMap<(u64, u64), u64>, count: &mut usize) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            *count += 1;
            ensure!(*count <= 65536, "sandbox output inode budget exceeded");
            let metadata = entry.path().symlink_metadata()?;
            if metadata.is_file() && metadata.dev() == device {
                files.insert((metadata.dev(), metadata.ino()), metadata.len());
            } else if metadata.is_dir() {
                extent(&entry.path(), device, files, count)?;
            }
        }
        Ok(())
    }
    let inspect = |observed: &BTreeSet<i32>| -> Result<()> {
        let mut files = BTreeMap::new();
        extent(Path::new("/work"), work_device, &mut files, &mut 0)?;
        for log in logs {
            let metadata = log.metadata()?;
            files.insert((metadata.dev(), metadata.ino()), metadata.len());
        }
        for pid in observed {
            let directory = std::ffi::CString::new(format!("{pid}/fd"))?;
            let descriptor = unsafe {
                libc::openat(proc.as_raw_fd(), directory.as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC)
            };
            if descriptor < 0 {
                let error = std::io::Error::last_os_error();
                if error.raw_os_error() == Some(libc::ENOENT) {
                    continue;
                }
                // Linux can deny access after exit has destroyed the task's
                // descriptor table but before its ptrace wait result is reaped.
                // The kernel must prove termination of this exact owned PID.
                if error.raw_os_error() == Some(libc::EACCES) && terminated_owned_task(*pid)? {
                    continue;
                }
                return Err(error).with_context(|| format!("observe candidate descriptor directory for pid {pid}"));
            }
            let owned = unsafe { File::from_raw_fd(descriptor) };
            let stream = unsafe { libc::fdopendir(libc::dup(owned.as_raw_fd())) };
            ensure!(!stream.is_null(), "cannot inspect owned candidate file descriptors");
            loop {
                let entry = unsafe { libc::readdir(stream) };
                if entry.is_null() {
                    break;
                }
                let name = unsafe { std::ffi::CStr::from_ptr((*entry).d_name.as_ptr()) };
                if name.to_bytes() == b"." || name.to_bytes() == b".." {
                    continue;
                }
                let mut metadata = std::mem::MaybeUninit::<libc::stat>::uninit();
                if unsafe { libc::fstatat(owned.as_raw_fd(), name.as_ptr(), metadata.as_mut_ptr(), 0) } == 0 {
                    let metadata = unsafe { metadata.assume_init() };
                    if metadata.st_dev == work_device && metadata.st_mode & libc::S_IFMT == libc::S_IFREG {
                        files.insert((metadata.st_dev, metadata.st_ino), u64::try_from(metadata.st_size)?);
                    }
                } else {
                    let error = std::io::Error::last_os_error();
                    // A disappearing descriptor is a normal close/exit race.
                    // Denied metadata cannot establish its output extent.
                    if !matches!(error.raw_os_error(), Some(libc::ENOENT | libc::ESRCH)) {
                        unsafe {
                            libc::closedir(stream);
                        }
                        return Err(error)
                            .with_context(|| format!("observe candidate descriptor extent for pid {pid}"));
                    }
                }
            }
            unsafe {
                libc::closedir(stream);
            }
        }
        let bytes = files
            .values()
            .try_fold(0u64, |sum, value| sum.checked_add(*value).context("sandbox logical extent overflow"))?;
        ensure!(bytes <= writable_bytes, "aggregate sandbox logical output extent budget exhausted");
        Ok(())
    };
    // Only syscalls that can grow output trigger the namespace inode walk.
    // The kernel tmpfs quota independently enforces all allocated data pages.
    let mut mutations = BTreeMap::<i32, bool>::new();
    let mut status = 0;
    loop {
        let result = unsafe { libc::waitpid(pid as i32, &mut status, libc::__WALL) };
        if result < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        ensure!(result == pid as i32 && libc::WIFSTOPPED(status), "candidate quota observer failed before exec");
        break;
    }
    let options = libc::PTRACE_O_TRACESYSGOOD
        | libc::PTRACE_O_TRACEFORK
        | libc::PTRACE_O_TRACEVFORK
        | libc::PTRACE_O_TRACECLONE
        | libc::PTRACE_O_TRACEEXEC
        | libc::PTRACE_O_EXITKILL;
    errno(unsafe { libc::ptrace(libc::PTRACE_SETOPTIONS, pid, 0, options) as i32 }, "set inherited quota observer")?;
    errno(unsafe { libc::ptrace(libc::PTRACE_SYSCALL, pid, 0, 0) as i32 }, "start quota observer")?;
    loop {
        let stopped = unsafe { libc::waitpid(-1, &mut status, libc::__WALL) };
        if stopped < 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            bail!("candidate quota observer lost process ownership");
        }
        if libc::WIFEXITED(status) || libc::WIFSIGNALED(status) {
            observed.remove(&stopped);
            mutations.remove(&stopped);
            if stopped == pid as i32 {
                return Ok(ExitStatus::from_raw(status));
            }
            continue;
        }
        if !libc::WIFSTOPPED(status) {
            continue;
        }
        let signal = libc::WSTOPSIG(status);
        let mut deliver = 0;
        if signal == (libc::SIGTRAP | 0x80) {
            // ptrace_syscall_info: common header24 + union exit {i64 rval,
            // u8 is_error}. Kernel handles entry/exit tracking across exec and
            // signal interruptions; no fragile toggled syscall bookkeeping.
            let mut info = [0u8; 128];
            let size = unsafe { libc::ptrace(libc::PTRACE_GET_SYSCALL_INFO, stopped, info.len(), info.as_mut_ptr()) };
            ensure!(size >= 24, "kernel lacks required quota-error observation");
            if info[0] == 2 {
                ensure!(size >= 33, "kernel quota-error observation is incomplete");
            }
            if info[0] == 2 && info[32] != 0 {
                let result = i64::from_ne_bytes(info[24..32].try_into()?);
                ensure!(
                    ![-(libc::ENOSPC as i64), -(libc::EDQUOT as i64), -(libc::EFBIG as i64)].contains(&result),
                    "aggregate sandbox quota exhausted during production"
                );
            }
            if info[0] == 1 {
                ensure!(size >= 80, "kernel syscall-entry observation is incomplete");
                let syscall = u64::from_ne_bytes(info[24..32].try_into()?);
                // Linkers reopen many read-only inputs after producing a large
                // object tree. Only explicit nonmutating open/openat flags can
                // skip the full extent walk. openat2 keeps the conservative
                // walk because its flags reside in candidate memory.
                let open_flags = match syscall as libc::c_long {
                    libc::SYS_open => Some(u64::from_ne_bytes(info[40..48].try_into()?)),
                    libc::SYS_openat => Some(u64::from_ne_bytes(info[48..56].try_into()?)),
                    _ => None,
                };
                let readonly_open = open_flags.is_some_and(|flags| {
                    flags & (libc::O_ACCMODE | libc::O_CREAT | libc::O_TRUNC | libc::O_TMPFILE) as u64 == 0
                });
                let mutating = !readonly_open && [
                    libc::SYS_write,
                    libc::SYS_writev,
                    libc::SYS_pwrite64,
                    libc::SYS_pwritev,
                    libc::SYS_pwritev2,
                    libc::SYS_truncate,
                    libc::SYS_ftruncate,
                    libc::SYS_fallocate,
                    libc::SYS_copy_file_range,
                    libc::SYS_splice,
                    libc::SYS_sendfile,
                    libc::SYS_open,
                    libc::SYS_openat,
                    libc::SYS_openat2,
                    libc::SYS_mkdir,
                    libc::SYS_mkdirat,
                ]
                .iter()
                .any(|number| *number as u64 == syscall);
                mutations.insert(stopped, mutating);
                if mutating {
                    inspect(&observed)?;
                }
            } else if info[0] == 2 && mutations.remove(&stopped).unwrap_or(false) {
                inspect(&observed)?;
            }
        } else if signal != libc::SIGTRAP && signal != libc::SIGSTOP {
            ensure!(signal != libc::SIGXFSZ, "sandbox file extent budget exhausted during production");
            deliver = signal;
        }
        let event = status >> 16;
        if [libc::PTRACE_EVENT_FORK, libc::PTRACE_EVENT_VFORK, libc::PTRACE_EVENT_CLONE].contains(&event) {
            let mut child = 0u64;
            errno(
                unsafe { libc::ptrace(libc::PTRACE_GETEVENTMSG, stopped, 0, &mut child) as i32 },
                "observe candidate descendant",
            )?;
            observed.insert(i32::try_from(child)?);
            ensure!(observed.len() <= 512, "sandbox descendant count budget exhausted");
        }
        let resumed = unsafe { libc::ptrace(libc::PTRACE_SYSCALL, stopped, 0, deliver) };
        if resumed < 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
            bail!("candidate quota observer cannot resume owned process");
        }
    }
}

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub fn helper_main() -> Result<()> {
    use std::os::unix::{
        io::FromRawFd,
        process::{CommandExt, ExitStatusExt},
    };
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(args.len() >= 2 && args[1] == "--", "invalid trusted sandbox helper request");
    let config: HelperLimits = serde_json::from_str(&args[0])?;
    let mut launch = [0u8; 1];
    // Use an unbuffered one-byte read: a Stdin buffer could consume capsule
    // bytes that the child must inherit on the original descriptor.
    loop {
        let count = unsafe { libc::read(libc::STDIN_FILENO, launch.as_mut_ptr().cast(), 1) };
        if count < 0 && std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
            continue;
        }
        ensure!(count == 1, "trusted launch identity handshake incomplete");
        break;
    }
    ensure!(launch == *b"S", "invalid trusted launch identity handshake");
    ensure!(
        config.writable_bytes >= 4096 && config.writable_bytes.is_multiple_of(4096),
        "invalid aggregate sandbox quota"
    );
    let mut seen = BTreeSet::new();
    for output in &config.outputs {
        ensure!(
            relative(output) && !output.starts_with("tmp") && seen.insert(output.clone()),
            "invalid declared sandbox output"
        );
    }
    let options = std::ffi::CString::new(format!(
        "size={},nr_inodes={},mode=0700",
        config.writable_bytes,
        (config.writable_bytes / 4096).clamp(64, 65536)
    ))?;
    errno(
        unsafe {
            libc::mount(
                c"tmpfs".as_ptr(),
                c"/work".as_ptr(),
                c"tmpfs".as_ptr(),
                libc::MS_NOSUID | libc::MS_NODEV,
                options.as_ptr().cast(),
            )
        },
        "establish aggregate tmpfs quota",
    )?;
    fs::create_dir("/work/tmp")?;
    fs::create_dir("/work/tmp/home")?;
    fs::create_dir("/work/tmp/cargo-home")?;
    errno(
        unsafe {
            libc::mount(
                c"proc".as_ptr(),
                c"/quota-proc".as_ptr(),
                c"proc".as_ptr(),
                libc::MS_NOSUID | libc::MS_NODEV | libc::MS_NOEXEC,
                std::ptr::null(),
            )
        },
        "establish trusted logical output observation",
    )?;
    let quota_proc = File::open("/quota-proc")?;
    std::env::set_current_dir("/work")?;
    let log = |name: &str| -> Result<File> {
        let file = OpenOptions::new().read(true).write(true).create_new(true).open(name)?;
        fs::remove_file(name)?;
        Ok(file)
    };
    let mut stdout = log(".stdout")?;
    let mut stderr = log(".stderr")?;
    let pipe = || -> Result<(File, File)> {
        let mut fds = [0; 2];
        errno(unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC) }, "establish bounded output pipe")?;
        // SAFETY: pipe2 returned two independent owned descriptors.
        Ok(unsafe { (File::from_raw_fd(fds[0]), File::from_raw_fd(fds[1])) })
    };
    let (stdout_read, stdout_write) = pipe()?;
    let (stderr_read, stderr_write) = pipe()?;
    errno(unsafe { libc::prctl(libc::PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0) }, "establish descendant ownership")?;
    let mut command = Command::new("/program");
    command
        .args(&args[2..])
        .env_clear()
        .env("PATH", "/nonexistent")
        .env("TMPDIR", "/work/tmp")
        .env("HOME", "/work/tmp/home")
        .env("CARGO_HOME", "/work/tmp/cargo-home")
        .env("LD_LIBRARY_PATH", "/toolchain/lib")
        .stdin(Stdio::inherit())
        .stdout(stdout_write)
        .stderr(stderr_write);
    unsafe {
        command.pre_exec(move || restrict_child(config.writable_bytes).map_err(std::io::Error::other));
    }
    let child = command.spawn().context("sandbox restriction setup refused candidate launch")?;
    drop(command);
    let drain = |mut input: File, mut output: File, limit: usize| -> Result<()> {
        let mut budget = ByteBudget::new(limit as u64);
        let mut buffer = [0u8; 8192];
        let result = (|| {
            loop {
                let count = input.read(&mut buffer)?;
                if count == 0 {
                    return Ok(());
                }
                budget.reserve(count as u64)?;
                output.write_all(&buffer[..count])?;
            }
        })();
        if result.is_err() {
            unsafe {
                libc::kill(-1, libc::SIGKILL);
            }
        }
        result
    };
    let stdout_log = stdout.try_clone()?;
    let stderr_log = stderr.try_clone()?;
    let quota_logs = [stdout.try_clone()?, stderr.try_clone()?];
    let (observed, stdout_result, stderr_result) = std::thread::scope(|scope| {
        let stdout = scope.spawn(|| drain(stdout_read, stdout_log, config.stdout_bytes));
        let stderr = scope.spawn(|| drain(stderr_read, stderr_log, config.stderr_bytes));
        let result = wait_traced(child.id(), config.writable_bytes, &quota_proc, &quota_logs);
        // pid1 and self are excluded. Kill before joining output pipes so a
        // detached writer cannot keep a drain or the controller alive.
        unsafe {
            libc::kill(-1, libc::SIGKILL);
        }
        (result, stdout.join(), stderr.join())
    });
    // kill(-1) is confined to this PID namespace; pid 1 and the helper itself
    // are excluded by Linux. Reap all adopted descendants before reading bytes.
    unsafe {
        libc::kill(-1, libc::SIGKILL);
    }
    loop {
        let result = unsafe { libc::waitpid(-1, std::ptr::null_mut(), libc::__WALL) };
        if result < 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::EINTR) {
                continue;
            }
            break;
        }
    }
    stdout_result.map_err(|_| anyhow::anyhow!("sandbox stdout collector panicked"))??;
    stderr_result.map_err(|_| anyhow::anyhow!("sandbox stderr collector panicked"))??;
    let status = observed?;
    let mut budget = ByteBudget::new(config.writable_bytes);
    let mut read = |file: &mut File, limit: u64| -> Result<Vec<u8>> {
        let len = file.metadata()?.len();
        ensure!(len <= limit, "sandbox material output byte budget exceeded");
        budget.reserve(len)?;
        file.seek(SeekFrom::Start(0))?;
        let mut bytes = Vec::new();
        file.take(len).read_to_end(&mut bytes)?;
        ensure!(bytes.len() as u64 == len, "sandbox output changed");
        Ok(bytes)
    };
    let stdout = read(&mut stdout, config.stdout_bytes as u64)?;
    let stderr = read(&mut stderr, config.stderr_bytes as u64)?;
    let allowed = config.outputs.iter().cloned().collect::<BTreeSet<_>>();
    fn validate_tree(path: &Path, allowed: &BTreeSet<PathBuf>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let p = entry.path();
            let m = entry.metadata()?;
            ensure!(!entry.file_type()?.is_symlink(), "sandbox output symlink rejected");
            if m.is_dir() {
                validate_tree(&p, allowed)?;
            } else {
                ensure!(
                    m.is_file() && allowed.contains(p.strip_prefix("/work")?),
                    "undeclared sandbox output: {}",
                    bounded_text(&p.to_string_lossy(), 512)
                );
            }
        }
        Ok(())
    }
    let outputs = (|| -> Result<Vec<Vec<u8>>> {
        for entry in fs::read_dir("/work")? {
            let p = entry?.path();
            if p == Path::new("/work/tmp") {
                continue;
            }
            let m = p.symlink_metadata()?;
            if m.is_dir() {
                validate_tree(&p, &allowed)?;
            } else {
                ensure!(
                    m.is_file() && allowed.contains(p.strip_prefix("/work")?),
                    "undeclared sandbox output: {}",
                    bounded_text(&p.to_string_lossy(), 512)
                );
            }
        }
        let mut outputs = Vec::new();
        for relative in &config.outputs {
            let path = Path::new("/work").join(relative);
            let meta = path.symlink_metadata().with_context(|| {
                format!("cannot inspect declared sandbox output: {}", bounded_text(&path.to_string_lossy(), 512))
            })?;
            ensure!(
                meta.is_file() && !meta.file_type().is_symlink(),
                "declared sandbox output is not regular: {}",
                bounded_text(&path.to_string_lossy(), 512)
            );
            let mut file = File::open(&path)?;
            outputs.push(read(&mut file, config.writable_bytes)?);
        }
        let mut stat = std::mem::MaybeUninit::<libc::statfs>::uninit();
        errno(unsafe { libc::statfs(c"/work".as_ptr(), stat.as_mut_ptr()) }, "inspect sandbox quota")?;
        let stat = unsafe { stat.assume_init() };
        ensure!(stat.f_bavail > 0, "aggregate sandbox quota exhausted");
        Ok(outputs)
    })()
    .map_err(|error| {
        let detail = format!("{error:#}");
        let stderr = String::from_utf8_lossy(&stderr[..stderr.len().min(2048)]);
        anyhow::anyhow!("{}; candidate stderr: {}", bounded_text(&detail, 2048), bounded_text(&stderr, 1536))
    })?;
    let mut out = std::io::stdout().lock();
    out.write_all(MAGIC)?;
    out.write_all(&status.into_raw().to_le_bytes())?;
    put_bytes(&mut out, &stdout)?;
    put_bytes(&mut out, &stderr)?;
    for bytes in outputs {
        put_bytes(&mut out, &bytes)?;
    }
    out.flush()?;
    Ok(())
}

#[cfg(not(all(target_os = "linux", target_arch = "x86_64")))]
pub fn helper_main() -> Result<()> {
    bail!("strict confinement requires Linux x86_64")
}
