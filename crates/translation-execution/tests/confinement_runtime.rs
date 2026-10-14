#![cfg(all(target_os = "linux", target_arch = "x86_64"))]

use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
use translation_execution::confinement::{Program, ReadOnlyBind, RunLimits, helper_runtime, run};

fn probe(name: &str) -> Program {
    Program {
        executable: fs::canonicalize(std::env::current_exe().unwrap()).unwrap(),
        arguments: vec!["--ignored".into(), "--exact".into(), name.into(), "--nocapture".into()],
        inputs: vec![],
        runtime: helper_runtime().unwrap(),
        helper: std::env::var_os("SM_CONFINEMENT_TEST_HELPER")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_sm-translation-sandbox"))),
        outputs: vec![],
    }
}

fn limits() -> RunLimits {
    RunLimits {
        timeout: Duration::from_secs(5),
        writable_bytes: 1024 * 1024,
        stdout_bytes: 65536,
        stderr_bytes: 65536,
        stdin_bytes: 0,
    }
}

#[test]
fn read_only_input_opens_scale_with_existing_outputs() {
    let limits = RunLimits { writable_bytes: 8 * 1024 * 1024, ..limits() };
    let output = run(&probe("probe_read_only_input_opens"), &limits, b"").unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn read_only_directory_opens_scale_with_existing_outputs() {
    let limits = RunLimits { writable_bytes: 8 * 1024 * 1024, ..limits() };
    let output = run(&probe("probe_read_only_directory_opens"), &limits, b"").unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn reviewed_devices_allow_null_child_stdio() {
    let output = run(&probe("probe_null_stdio"), &limits(), b"").unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn rust_pre_exec_handshake_allows_only_its_local_unix_pair() {
    let output = run(&probe("probe_pre_exec_handshake"), &limits(), b"").unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn exited_threads_do_not_refuse_live_writer_observation() {
    for attempt in 0..16 {
        let output = run(&probe("probe_threaded_writes_and_exits"), &limits(), b"")
            .unwrap_or_else(|error| panic!("threaded writer attempt {attempt}: {error:#}"));
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    // Also retain the original high-interleaving reproducer after minimization.
    let output = run(&probe("probe_many_threaded_writes_and_exits"), &limits(), b"").unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn concurrent_execs_do_not_refuse_live_writer_observation() {
    let parameter = |name: &str, default: usize| {
        std::env::var(name).map(|value| value.parse::<usize>().unwrap()).unwrap_or(default)
    };
    let payload = format!(
        "{} {} {}",
        parameter("SM_CONFINEMENT_EXEC_THREADS", 2),
        parameter("SM_CONFINEMENT_EXEC_CHILDREN", 8),
        parameter("SM_CONFINEMENT_EXEC_WRITES", 512),
    );
    let limits = RunLimits { stdin_bytes: 64, ..limits() };
    for attempt in 0..16 {
        let output = run(&probe("probe_parallel_exec_and_writes"), &limits, payload.as_bytes())
            .unwrap_or_else(|error| panic!("concurrent exec attempt {attempt}: {error:#}"));
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn shared_descriptor_swap_and_restore_keeps_aggregate_quota_enforced() {
    let bounds = RunLimits { writable_bytes: 128 * 1024, stdin_bytes: 16, ..limits() };
    let program = probe("probe_shared_descriptor_swap_and_restore");
    let output = run(&program, &bounds, b"bounded").unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    for _ in 0..4 {
        let error = run(&program, &bounds, b"overflow").unwrap_err().to_string();
        assert!(error.contains("logical output extent budget"), "{error}");
    }
}

#[test]
fn clone_files_shares_descriptors_across_process_groups() {
    let bounds = RunLimits { writable_bytes: 128 * 1024, stdin_bytes: 16, ..limits() };
    let program = probe("probe_shared_descriptor_swap_and_restore");
    let output = run(&program, &bounds, b"clone-bounded").unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    for _ in 0..4 {
        let error = run(&program, &bounds, b"clone-overflow").unwrap_err().to_string();
        assert!(error.contains("logical output extent budget"), "{error}");
    }
}

#[test]
fn overlapping_unlinked_writes_deduplicate_aliases_and_enforce_the_sum() {
    let bounds = RunLimits { writable_bytes: 128 * 1024, stdin_bytes: 16, ..limits() };
    let program = probe("probe_overlapping_unlinked_writes");
    let output = run(&program, &bounds, b"bounded").unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    for _ in 0..4 {
        let error = run(&program, &bounds, b"overflow").unwrap_err().to_string();
        assert!(error.contains("logical output extent budget"), "{error}");
    }
}

#[test]
fn blocked_pipe_does_not_retain_unrelated_deleted_output() {
    let bounds = RunLimits { writable_bytes: 128 * 1024, ..limits() };
    let output = run(&probe("probe_blocked_pipe_and_temporary_cycles"), &bounds, b"").unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn independent_fork_descriptor_numbers_do_not_retain_other_process_files() {
    let bounds = RunLimits { writable_bytes: 128 * 1024, stdin_bytes: 16, ..limits() };
    let output = run(&probe("probe_blocked_pipe_and_temporary_cycles"), &bounds, b"independent").unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn close_range_unshare_separates_descriptor_numbers_within_a_thread_group() {
    let bounds = RunLimits { writable_bytes: 128 * 1024, stdin_bytes: 16, ..limits() };
    let output = run(&probe("probe_blocked_pipe_and_temporary_cycles"), &bounds, b"unshare").unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn exec_separates_previously_shared_descriptor_numbers() {
    let bounds = RunLimits { writable_bytes: 128 * 1024, stdin_bytes: 16, ..limits() };
    let output = run(&probe("probe_blocked_pipe_and_temporary_cycles"), &bounds, b"shared-exec").unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}

#[test]
fn nonleader_exec_preserves_aggregate_quota_observation() {
    let bounds = RunLimits { writable_bytes: 128 * 1024, stdin_bytes: 16, ..limits() };
    let program = probe("probe_nonleader_exec");
    let output = run(&program, &bounds, b"bounded").unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    for _ in 0..4 {
        let error = run(&program, &bounds, b"overflow").unwrap_err().to_string();
        assert!(error.contains("logical output extent budget"), "{error}");
    }
}

#[test]
#[ignore = "explicit repeated concurrent-observer regression"]
fn many_threaded_writes_and_exits_stress() {
    let attempts: usize =
        std::env::var("SM_CONFINEMENT_THREAD_STRESS").map(|value| value.parse().unwrap()).unwrap_or(32);
    for attempt in 0..attempts {
        let output = run(&probe("probe_many_threaded_writes_and_exits"), &limits(), b"")
            .unwrap_or_else(|error| panic!("concurrent observation attempt {attempt}: {error:#}"));
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn synthetic_self_exe_is_explicit_and_exposes_only_the_program() {
    let mut program = probe("probe_self_exe");
    let output = run(&program, &limits(), b"").unwrap();
    assert!(!output.status.success(), "ordinary producer unexpectedly sees proc/self/exe");
    program.runtime.push(ReadOnlyBind { source: program.executable.clone(), destination: "/proc/self/exe".into() });
    let output = run(&program, &limits(), b"").unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn synthetic_self_exe_rejects_aliases_and_other_sources() {
    for destination in
        ["/proc", "/proc/1/exe", "/proc/self/../self/exe", "/proc//self/exe", "/proc/self/./exe", "/proc/self/exe/"]
    {
        let mut program = probe("probe_self_exe");
        program.runtime.push(ReadOnlyBind { source: program.executable.clone(), destination: destination.into() });
        assert!(run(&program, &limits(), b"").is_err(), "accepted alias {destination}");
    }
    let mut program = probe("probe_self_exe");
    program.runtime.push(ReadOnlyBind { source: program.helper.clone(), destination: "/proc/self/exe".into() });
    assert!(run(&program, &limits(), b"").is_err(), "accepted a different executable");
    program.runtime.last_mut().unwrap().source =
        program.executable.parent().unwrap().join(".").join(program.executable.file_name().unwrap());
    assert!(run(&program, &limits(), b"").is_err(), "accepted an aliased source");
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_empty_child() {}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_read_only_input_opens() {
    // Linkers repeatedly open read-only inputs after earlier compiler outputs
    // have populated the writable tree. These opens cannot grow that tree.
    for index in 0..256 {
        fs::File::create(format!("/work/tmp/object-{index}")).unwrap();
    }
    println!("created 256 compiler-output placeholders");
    for _ in 0..4096 {
        drop(fs::File::open("/program").unwrap());
        let descriptor = unsafe { libc::open(c"/program".as_ptr(), libc::O_RDONLY | libc::O_CLOEXEC) };
        assert!(descriptor >= 0);
        assert_eq!(unsafe { libc::close(descriptor) }, 0);
    }
    println!("completed 8192 read-only input opens");
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_read_only_directory_opens() {
    for index in 0..256 {
        fs::File::create(format!("/work/tmp/object-{index}")).unwrap();
    }
    fs::create_dir("/work/tmp/existing-directory").unwrap();
    println!("created 256 compiler-output placeholders and an empty directory");
    for _ in 0..4096 {
        assert_eq!(fs::read_dir("/work/tmp/existing-directory").unwrap().count(), 0);
        let descriptor = unsafe {
            libc::open(c"/work/tmp/existing-directory".as_ptr(), libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC)
        };
        assert!(descriptor >= 0);
        assert_eq!(unsafe { libc::close(descriptor) }, 0);
    }
    println!("completed 8192 read-only directory opens");
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_pre_exec_handshake() {
    use std::os::unix::process::CommandExt;
    let mut child = Command::new("/program");
    child
        .args(["--ignored", "--exact", "probe_empty_child"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    // The empty hook forces Rust's fork/exec error-reporting handshake.
    unsafe {
        child.pre_exec(|| Ok(()));
    }
    assert!(child.status().unwrap().success());
    for (domain, kind, protocol) in [
        (libc::AF_INET, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC, 0),
        (libc::AF_INET6, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC, 0),
        (libc::AF_UNIX, libc::SOCK_STREAM | libc::SOCK_CLOEXEC, 0),
        (libc::AF_UNIX, libc::SOCK_SEQPACKET, 0),
        (libc::AF_UNIX, libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC, 1),
    ] {
        let mut descriptors = [-1; 2];
        assert_eq!(unsafe { libc::socketpair(domain, kind, protocol, descriptors.as_mut_ptr()) }, -1);
        assert_eq!(std::io::Error::last_os_error().raw_os_error(), Some(libc::EPERM));
    }
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_null_stdio() {
    let mut null = fs::OpenOptions::new().read(true).write(true).open("/dev/null").unwrap();
    null.write_all(b"discarded").unwrap();
    assert_eq!(null.read(&mut [0; 1]).unwrap(), 0);
    fs::File::open("/dev/urandom").unwrap().read_exact(&mut [0; 1]).unwrap();
    let mut devices = fs::read_dir("/dev").unwrap().map(|entry| entry.unwrap().file_name()).collect::<Vec<_>>();
    devices.sort();
    assert_eq!(devices, ["null", "urandom"]);
    assert!(
        Command::new("/program")
            .args(["--ignored", "--exact", "probe_empty_child"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success()
    );
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_threaded_writes_and_exits() {
    threaded_writes_and_exits(1, 32, 1);
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_many_threaded_writes_and_exits() {
    threaded_writes_and_exits(32, 512, 8);
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_parallel_exec_and_writes() {
    let mut parameters = String::new();
    std::io::stdin().read_to_string(&mut parameters).unwrap();
    let values = parameters.split_whitespace().map(|value| value.parse::<usize>().unwrap()).collect::<Vec<_>>();
    assert_eq!(values.len(), 3);
    let [threads, children, writes] = values[..] else { unreachable!() };
    assert!(threads <= 32 && children <= 64 && writes <= 4096);
    std::thread::scope(|scope| {
        for _ in 0..threads {
            scope.spawn(|| {
                for _ in 0..children {
                    assert!(
                        Command::new("/program")
                            .args(["--ignored", "--exact", "probe_empty_child"])
                            .stdin(Stdio::null())
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .status()
                            .unwrap()
                            .success()
                    );
                }
            });
        }
        for _ in 0..writes {
            fs::write("/work/tmp/exec-progress", b"x").unwrap();
        }
    });
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_shared_descriptor_swap_and_restore() {
    use std::{
        os::fd::{AsRawFd, FromRawFd, IntoRawFd},
        sync::atomic::{AtomicBool, Ordering},
    };
    let mut mode = String::new();
    std::io::stdin().read_to_string(&mut mode).unwrap();
    let base = fs::File::create("/work/tmp/base").unwrap();
    let transient = fs::File::create("/work/tmp/transient").unwrap();
    base.set_len(32 * 1024).unwrap();
    transient.set_len(32 * 1024).unwrap();
    fs::remove_file("/work/tmp/base").unwrap();
    fs::remove_file("/work/tmp/transient").unwrap();
    let transient = transient.into_raw_fd();
    let mut pipe = [-1; 2];
    assert_eq!(unsafe { libc::pipe2(pipe.as_mut_ptr(), libc::O_CLOEXEC) }, 0);
    let mut reader = unsafe { fs::File::from_raw_fd(pipe[0]) };
    let mut writer = unsafe { fs::File::from_raw_fd(pipe[1]) };
    let saved = writer.try_clone().unwrap();
    let descriptor = writer.as_raw_fd();
    let capacity = unsafe { libc::fcntl(descriptor, libc::F_GETPIPE_SZ) };
    assert!(capacity > 0);
    writer.write_all(&vec![0; capacity as usize]).unwrap();
    let started = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let blocked = scope.spawn(|| {
            started.store(true, Ordering::Release);
            // A full pipe keeps this write in flight while its shared FD is
            // replaced and restored. Both endpoint snapshots name the pipe.
            let count = unsafe { libc::write(descriptor, [1u8; 4096].as_ptr().cast(), 4096) };
            assert_eq!(count, 4096);
        });
        while !started.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        std::thread::sleep(Duration::from_millis(20));
        let extent = if mode.ends_with("overflow") { 112 * 1024 } else { 48 * 1024 };
        if mode.starts_with("clone-") {
            // CLONE_FILES without CLONE_VM shares descriptors across separate
            // processes. SIGCHLD makes ptrace report FORK, not CLONE.
            let child = unsafe { libc::syscall(libc::SYS_clone, libc::CLONE_FILES | libc::SIGCHLD, 0, 0, 0, 0) };
            assert!(child >= 0);
            if child == 0 {
                unsafe { libc::_exit(swap_descriptor_extent(transient, descriptor, saved.as_raw_fd(), extent)) };
            }
            let mut status = 0;
            assert_eq!(unsafe { libc::waitpid(child as i32, &mut status, 0) }, child as i32);
            assert_eq!(status, 0);
        } else {
            assert_eq!(swap_descriptor_extent(transient, descriptor, saved.as_raw_fd(), extent), 0);
        }
        let mut drained = vec![0; capacity as usize + 4096];
        reader.read_exact(&mut drained).unwrap();
        assert!(drained[capacity as usize..].iter().all(|byte| *byte == 1));
        blocked.join().unwrap();
    });
}

fn swap_descriptor_extent(source: i32, destination: i32, saved: i32, extent: i64) -> i32 {
    // Only async-signal-safe calls: also used after a multithreaded clone.
    // Immediately close the unlinked source and restore the pipe descriptor.
    if unsafe { libc::dup2(source, destination) } != destination
        || unsafe { libc::ftruncate(destination, extent) } != 0
        || unsafe { libc::close(source) } != 0
        || unsafe { libc::dup2(saved, destination) } != destination
    {
        return 7;
    }
    0
}

fn temporary_descriptor_cycles(descriptor: i32) -> i32 {
    // Only async-signal-safe calls after a multithreaded fork. Cumulative
    // allocation is 512 KiB; live output is one 16 KiB file under a 128 KiB cap.
    for _ in 0..32 {
        let temporary = unsafe {
            libc::open(
                c"/work/tmp/independent".as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_TRUNC | libc::O_CLOEXEC,
                0o600,
            )
        };
        if temporary < 0
            || unsafe { libc::dup2(temporary, descriptor) } != descriptor
            || unsafe { libc::write(descriptor, [0u8; 16 * 1024].as_ptr().cast(), 16 * 1024) } != 16 * 1024
            || unsafe { libc::unlink(c"/work/tmp/independent".as_ptr()) } != 0
        {
            return 7;
        }
        if temporary != descriptor {
            unsafe { libc::close(temporary) };
        }
        unsafe { libc::close(descriptor) };
    }
    0
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_independent_descriptor_cycles() {
    let descriptor: i32 = std::env::var("SM_CONFINEMENT_CYCLE_DESCRIPTOR").unwrap().parse().unwrap();
    assert_eq!(temporary_descriptor_cycles(descriptor), 0);
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_nonleader_exec() {
    use std::os::unix::process::CommandExt;
    std::thread::spawn(|| {
        assert_ne!(unsafe { libc::syscall(libc::SYS_gettid) }, unsafe { libc::getpid() } as libc::c_long);
        let error = Command::new("/program")
            .args(["--ignored", "--exact", "probe_overlapping_unlinked_writes", "--nocapture"])
            .exec();
        panic!("nonleader exec failed: {error}");
    })
    .join()
    .unwrap();
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_blocked_pipe_and_temporary_cycles() {
    use std::{
        os::fd::{AsRawFd, FromRawFd},
        sync::atomic::{AtomicBool, Ordering},
    };
    let mut mode = String::new();
    std::io::stdin().read_to_string(&mut mode).unwrap();
    let mut pipe = [-1; 2];
    assert_eq!(unsafe { libc::pipe2(pipe.as_mut_ptr(), libc::O_CLOEXEC) }, 0);
    let mut reader = unsafe { fs::File::from_raw_fd(pipe[0]) };
    let mut writer = unsafe { fs::File::from_raw_fd(pipe[1]) };
    let descriptor = writer.as_raw_fd();
    let capacity = unsafe { libc::fcntl(descriptor, libc::F_GETPIPE_SZ) };
    assert!(capacity > 0);
    writer.write_all(&vec![0; capacity as usize]).unwrap();
    let started = AtomicBool::new(false);
    std::thread::scope(|scope| {
        let blocked = scope.spawn(|| {
            started.store(true, Ordering::Release);
            assert_eq!(unsafe { libc::write(descriptor, [1u8; 4096].as_ptr().cast(), 4096) }, 4096);
        });
        while !started.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        std::thread::sleep(Duration::from_millis(20));
        if mode == "independent" || mode == "shared-exec" {
            let setting = std::ffi::CString::new(format!("SM_CONFINEMENT_CYCLE_DESCRIPTOR={descriptor}")).unwrap();
            let arguments = [
                c"/program".as_ptr(),
                c"--ignored".as_ptr(),
                c"--exact".as_ptr(),
                c"probe_independent_descriptor_cycles".as_ptr(),
                c"--nocapture".as_ptr(),
                std::ptr::null(),
            ];
            let environment = [setting.as_ptr(), std::ptr::null()];
            let child = if mode == "shared-exec" {
                unsafe { libc::syscall(libc::SYS_clone, libc::CLONE_FILES | libc::SIGCHLD, 0, 0, 0, 0) as i32 }
            } else {
                unsafe { libc::fork() }
            };
            assert!(child >= 0);
            if child == 0 {
                if mode == "shared-exec" {
                    unsafe { libc::execve(c"/program".as_ptr(), arguments.as_ptr(), environment.as_ptr()) };
                    unsafe { libc::_exit(8) };
                }
                unsafe { libc::_exit(temporary_descriptor_cycles(descriptor)) };
            }
            let mut status = 0;
            assert_eq!(unsafe { libc::waitpid(child, &mut status, 0) }, child);
            assert_eq!(status, 0);
        } else if mode == "unshare" {
            scope
                .spawn(|| {
                    // Privatize this thread's table without closing a live FD.
                    assert_eq!(
                        unsafe { libc::syscall(libc::SYS_close_range, u32::MAX, u32::MAX, libc::CLOSE_RANGE_UNSHARE) },
                        0
                    );
                    assert_eq!(temporary_descriptor_cycles(descriptor), 0);
                })
                .join()
                .unwrap();
        } else {
            // Cumulative allocation is 512 KiB, but only one 16 KiB temporary
            // is live at a time under the unchanged 128 KiB tmpfs limit.
            for index in 0..32 {
                let path = format!("/work/tmp/cycle-{index}");
                let mut temporary = fs::File::create(&path).unwrap();
                temporary.write_all(&[0u8; 16 * 1024]).unwrap();
                fs::remove_file(path).unwrap();
                drop(temporary);
            }
        }
        reader.read_exact(&mut vec![0; capacity as usize + 4096]).unwrap();
        blocked.join().unwrap();
    });
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_overlapping_unlinked_writes() {
    use std::sync::Barrier;
    let mut mode = String::new();
    std::io::stdin().read_to_string(&mut mode).unwrap();
    let mut files = Vec::new();
    for index in 0..2 {
        let path = format!("/work/tmp/unlinked-{index}");
        let alias = format!("/work/tmp/alias-{index}");
        let file = fs::File::create(&path).unwrap();
        fs::hard_link(&path, &alias).unwrap();
        file.set_len(16 * 1024).unwrap();
        // Keep both the inode alias and the descriptor alias during one
        // observation, then remove both names without closing either FD.
        let descriptor_alias = file.try_clone().unwrap();
        fs::remove_file(path).unwrap();
        fs::remove_file(alias).unwrap();
        files.push((file, descriptor_alias));
    }
    let barrier = Barrier::new(2);
    let extent = if mode == "overflow" { 72 * 1024 } else { 48 * 1024 };
    std::thread::scope(|scope| {
        for (file, alias) in &files {
            scope.spawn(|| {
                barrier.wait();
                file.set_len(extent).unwrap();
                assert_eq!(alias.metadata().unwrap().len(), extent);
                // Hold each unlinked extent until both writes completed so
                // per-file limits cannot masquerade as aggregate enforcement.
                barrier.wait();
            });
        }
    });
}

fn threaded_writes_and_exits(threads: usize, writes: usize, worker_writes: usize) {
    std::thread::scope(|scope| {
        for index in 0..threads {
            scope.spawn(move || {
                let path = format!("/work/tmp/thread-{index}");
                let mut output = fs::File::create(&path).unwrap();
                fs::remove_file(path).unwrap();
                for _ in 0..worker_writes {
                    output.write_all(b"x").unwrap();
                }
            });
        }
        for _ in 0..writes {
            fs::write("/work/tmp/progress", b"x").unwrap();
        }
    });
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_self_exe() {
    use std::os::unix::fs::MetadataExt;
    assert_eq!(std::env::current_exe().unwrap(), Path::new("/program"));
    assert_eq!(fs::read_link("/proc/self/exe").unwrap(), Path::new("/program"));
    let staged = fs::metadata("/program").unwrap();
    let alias = fs::metadata("/proc/self/exe").unwrap();
    assert_eq!((staged.dev(), staged.ino()), (alias.dev(), alias.ino()));
    for (directory, expected) in [("/proc", "self"), ("/proc/self", "exe")] {
        let entries = fs::read_dir(directory).unwrap().map(|entry| entry.unwrap().file_name()).collect::<Vec<_>>();
        assert_eq!(entries, [expected]);
    }
    for hidden in ["/proc/1/status", "/proc/self/status", "/proc/self/fd", "/quota-proc/self/exe"] {
        assert!(fs::File::open(hidden).is_err(), "exposed {hidden}");
    }
    assert!(fs::File::create("/proc/self/other").is_err());
}
