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
        helper: PathBuf::from(env!("CARGO_BIN_EXE_sm-translation-sandbox")),
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
    threaded_writes_and_exits(1, 32);
}

#[test]
#[ignore = "candidate probe, invoked only by confined parent test"]
fn probe_many_threaded_writes_and_exits() {
    threaded_writes_and_exits(32, 512);
}

fn threaded_writes_and_exits(threads: usize, writes: usize) {
    std::thread::scope(|scope| {
        for index in 0..threads {
            scope.spawn(move || {
                let path = format!("/work/tmp/thread-{index}");
                let mut output = fs::File::create(&path).unwrap();
                fs::remove_file(path).unwrap();
                output.write_all(b"x").unwrap();
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
