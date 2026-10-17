#![cfg(all(target_os = "linux", target_arch = "x86_64"))]

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    os::fd::FromRawFd,
    os::unix::fs::MetadataExt,
    path::PathBuf,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
    time::{Duration, Instant},
};
use translation_execution::confinement::{Program, RunLimits, helper_runtime, run};

fn clock_seconds(clock: libc::clockid_t) -> f64 {
    let mut time = std::mem::MaybeUninit::<libc::timespec>::uninit();
    assert_eq!(unsafe { libc::clock_gettime(clock, time.as_mut_ptr()) }, 0);
    let time = unsafe { time.assume_init() };
    time.tv_sec as f64 + time.tv_nsec as f64 / 1e9
}

fn helper() -> PathBuf {
    std::env::var_os("SM_CONFINEMENT_TEST_HELPER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_BIN_EXE_sm-translation-sandbox")))
}

// /usr/bin/time around bubblewrap does not include all reparented namespace
// children. Sample only this invocation's descendants, never the whole host.
// This is diagnostic CPU measurement, entirely outside quota enforcement.
fn sample_helper_cpu(
    tid: i32,
    stop: &AtomicBool,
    expected_image: (u64, u64),
    image_seen: &AtomicBool,
) -> Vec<(f64, u64)> {
    let mut descendants = BTreeSet::new();
    let mut helper_ticks = BTreeMap::<i32, u64>::new();
    let mut samples = Vec::new();
    while !stop.load(Ordering::Acquire) {
        let mut parents = vec![(std::process::id() as i32, tid)];
        parents.extend(descendants.iter().map(|pid| (*pid, *pid)));
        for (pid, tid) in parents {
            if let Ok(children) = fs::read_to_string(format!("/proc/{pid}/task/{tid}/children")) {
                descendants.extend(children.split_whitespace().filter_map(|pid| pid.parse::<i32>().ok()));
            }
        }
        for pid in &descendants {
            let Ok(stat) = fs::read_to_string(format!("/proc/{pid}/stat")) else { continue };
            let Some((name, fields)) = stat.split_once(") ") else { continue };
            if !name.ends_with("(trusted-helper") {
                continue;
            }
            // Read the live image through procfs, not its supplied pathname.
            // A bind mount keeps the selected helper's device/inode identity.
            if let Ok(image) = fs::metadata(format!("/proc/{pid}/exe")) {
                if (image.dev(), image.ino()) == expected_image {
                    image_seen.store(true, Ordering::Release);
                }
            }
            let fields = fields.split_whitespace().collect::<Vec<_>>();
            if let (Some(user), Some(system)) = (
                fields.get(11).and_then(|value| value.parse::<u64>().ok()),
                fields.get(12).and_then(|value| value.parse::<u64>().ok()),
            ) {
                helper_ticks.insert(*pid, user + system);
            }
        }
        samples.push((clock_seconds(libc::CLOCK_MONOTONIC), helper_ticks.values().sum()));
        std::thread::sleep(Duration::from_millis(2));
    }
    samples
}

struct Measurement {
    setup: f64,
    steady: f64,
    total: f64,
    helper_cpu: f64,
    helper_steady_cpu: f64,
    producer_steady_cpu: f64,
    cpu_samples: usize,
    helper_image_seen: bool,
}

fn measure(mode: &str, retained: usize) -> Measurement {
    let program = Program {
        executable: fs::canonicalize(std::env::current_exe().unwrap()).unwrap(),
        arguments: vec!["--ignored".into(), "--exact".into(), "probe_streaming_io".into(), "--nocapture".into()],
        inputs: vec![],
        runtime: helper_runtime().unwrap(),
        helper: helper(),
        outputs: vec![],
    };
    let limits = RunLimits {
        timeout: Duration::from_secs(5),
        writable_bytes: 8 * 1024 * 1024,
        stdout_bytes: 65536,
        stderr_bytes: 65536,
        stdin_bytes: 128,
    };
    let stop = AtomicBool::new(false);
    let image_seen = AtomicBool::new(false);
    let image = fs::metadata(&program.helper).unwrap();
    let expected_image = (image.dev(), image.ino());
    let tid = unsafe { libc::syscall(libc::SYS_gettid) } as i32;
    let started = Instant::now();
    let (output, samples) = std::thread::scope(|scope| {
        let sampler = scope.spawn(|| sample_helper_cpu(tid, &stop, expected_image, &image_seen));
        let input = format!("{mode} {retained} 256");
        let output = run(&program, &limits, input.as_bytes());
        stop.store(true, Ordering::Release);
        (output.unwrap(), sampler.join().unwrap())
    });
    let total = started.elapsed().as_secs_f64();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(image_seen.load(Ordering::Acquire), "selected helper image was not observed executing");
    let stdout = String::from_utf8(output.stdout).unwrap();
    let timing = stdout.lines().find_map(|line| line.strip_prefix("timing ")).unwrap();
    let values = timing.split_whitespace().map(|value| value.parse::<f64>().unwrap()).collect::<Vec<_>>();
    let [setup, steady, begin, end, producer_steady_cpu] = values[..] else { panic!("unexpected timing {timing}") };
    let ticks_per_second = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
    assert!(ticks_per_second > 0.0);
    let ticks_at = |time| samples.iter().rev().find(|(sampled, _)| *sampled <= time).map_or(0, |(_, ticks)| *ticks);
    let helper_cpu = samples.last().map_or(0, |(_, ticks)| *ticks) as f64 / ticks_per_second;
    Measurement {
        setup,
        steady,
        total,
        helper_cpu,
        helper_steady_cpu: ticks_at(end).saturating_sub(ticks_at(begin)) as f64 / ticks_per_second,
        producer_steady_cpu,
        cpu_samples: samples.len(),
        helper_image_seen: image_seen.load(Ordering::Acquire),
    }
}

#[test]
#[ignore = "explicit before/after performance measurement"]
fn retained_files_do_not_multiply_streaming_io_cost() {
    let root = PathBuf::from(std::env::var_os("SM_CONFINEMENT_PERFORMANCE_ROOT").expect("performance root"));
    fs::create_dir_all(&root).unwrap();
    let repeats: usize =
        std::env::var("SM_CONFINEMENT_PERFORMANCE_REPEATS").map(|value| value.parse().unwrap()).unwrap_or(3);
    let mut report = format!(
        "helper = {:?}\nhelper_cpu_tick_seconds = {}\nhelper_cpu_poll_seconds = 0.002\n\n",
        fs::canonicalize(helper()).unwrap().to_str().unwrap(),
        1.0 / unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64
    );
    let mut measurements = Vec::new();
    for mode in ["file", "pipe"] {
        for retained in [0, 256] {
            for repeat in 0..repeats {
                let value = measure(mode, retained);
                report.push_str(&format!(
                    "[[measurement]]\nmode = {mode:?}\nretained_files = {retained}\noperations = 256\nrepeat = {repeat}\nsetup_seconds = {:.9}\nsteady_seconds = {:.9}\ntotal_seconds = {:.9}\nhelper_cpu_seconds = {:.6}\nhelper_steady_cpu_seconds = {:.6}\nproducer_steady_cpu_seconds = {:.9}\ncpu_samples = {}\nselected_helper_image_observed = {}\n\n",
                    value.setup, value.steady, value.total, value.helper_cpu, value.helper_steady_cpu, value.producer_steady_cpu, value.cpu_samples, value.helper_image_seen,
                ));
                fs::write(root.join("timings.toml"), &report).unwrap();
                println!(
                    "mode={mode} retained={retained} setup={:.6}s steady={:.6}s total={:.6}s helper_cpu={:.2}s helper_steady_cpu={:.2}s",
                    value.setup, value.steady, value.total, value.helper_cpu, value.helper_steady_cpu,
                );
                measurements.push((mode, retained, value.steady, value.helper_steady_cpu));
            }
        }
    }
    // Absolute headroom absorbs scheduler noise; retained unrelated files must
    // not turn the same 256 syscalls into a tree-sized operation per syscall.
    for mode in ["file", "pipe"] {
        let median = |retained, cpu| {
            let mut times = measurements
                .iter()
                .filter_map(|(kind, files, seconds, helper_cpu)| {
                    (*kind == mode && *files == retained).then_some(if cpu { *helper_cpu } else { *seconds })
                })
                .collect::<Vec<_>>();
            times.sort_by(f64::total_cmp);
            times[times.len() / 2]
        };
        let empty = median(0, false);
        let populated = median(256, false);
        assert!(
            populated < empty * 3.0 + 0.05,
            "{mode}: unrelated files multiply steady I/O cost: {empty:.6}s -> {populated:.6}s"
        );
        let empty_cpu = median(0, true);
        let populated_cpu = median(256, true);
        assert!(
            populated_cpu <= empty_cpu * 2.0 + 0.02,
            "{mode}: unrelated files multiply helper CPU: {empty_cpu:.3}s -> {populated_cpu:.3}s"
        );
    }
}

#[test]
#[ignore = "explicit independent-sandbox throughput measurement"]
fn independent_sandbox_workers() {
    let root = PathBuf::from(std::env::var_os("SM_CONFINEMENT_PERFORMANCE_ROOT").expect("performance root"));
    fs::create_dir_all(&root).unwrap();
    let mut report = String::new();
    for mode in ["file", "pipe"] {
        for workers in [1, 2, 4] {
            let next = AtomicUsize::new(0);
            let started = Instant::now();
            let measurements = std::thread::scope(|scope| {
                let tasks = (0..workers)
                    .map(|_| {
                        scope.spawn(|| {
                            let mut completed = Vec::new();
                            while next.fetch_add(1, Ordering::Relaxed) < 8 {
                                completed.push(measure(mode, 256));
                            }
                            completed
                        })
                    })
                    .collect::<Vec<_>>();
                tasks.into_iter().flat_map(|task| task.join().unwrap()).collect::<Vec<_>>()
            });
            assert_eq!(measurements.len(), 8);
            let elapsed = started.elapsed().as_secs_f64();
            let helper_cpu: f64 = measurements.iter().map(|value| value.helper_cpu).sum();
            let steady: f64 = measurements.iter().map(|value| value.steady).sum();
            report.push_str(&format!("[[batch]]\nmode = {mode:?}\nworkers = {workers}\ncompleted_sandboxes = 8\nretained_files_per_sandbox = 256\noperations_per_sandbox = 256\nwall_seconds = {elapsed:.9}\nhelper_cpu_seconds = {helper_cpu:.6}\nsum_steady_seconds = {steady:.9}\n\n"));
            fs::write(root.join("parallel-timings.toml"), &report).unwrap();
            println!("mode={mode} workers={workers} completed=8 elapsed={elapsed:.6}s helper_cpu={helper_cpu:.2}s");
        }
    }
}

#[test]
#[ignore = "Rust producer invoked by the confined benchmark"]
fn probe_streaming_io() {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).unwrap();
    let values = input.split_whitespace().collect::<Vec<_>>();
    let [mode, retained, operations] = values[..] else { panic!("unexpected input") };
    let retained: usize = retained.parse().unwrap();
    let operations: usize = operations.parse().unwrap();
    let started = Instant::now();
    for index in 0..retained {
        fs::write(format!("/work/tmp/retained-{index}"), b"retained").unwrap();
    }
    let mut regular = File::create("/work/tmp/stream").unwrap();
    let mut descriptors = [-1; 2];
    assert_eq!(unsafe { libc::pipe2(descriptors.as_mut_ptr(), libc::O_CLOEXEC) }, 0);
    let mut pipe_read = unsafe { File::from_raw_fd(descriptors[0]) };
    let mut pipe_write = unsafe { File::from_raw_fd(descriptors[1]) };
    let setup = started.elapsed();
    let producer_cpu = clock_seconds(libc::CLOCK_PROCESS_CPUTIME_ID);
    let begin = clock_seconds(libc::CLOCK_MONOTONIC);
    let started = Instant::now();
    let bytes = [0x5a; 64];
    for _ in 0..operations {
        match mode {
            "file" => regular.write_all(&bytes).unwrap(),
            "pipe" => {
                pipe_write.write_all(&bytes).unwrap();
                let mut received = [0; 64];
                pipe_read.read_exact(&mut received).unwrap();
                assert_eq!(received, bytes);
            }
            _ => panic!("unknown mode"),
        }
    }
    let steady = started.elapsed();
    let end = clock_seconds(libc::CLOCK_MONOTONIC);
    let producer_cpu = clock_seconds(libc::CLOCK_PROCESS_CPUTIME_ID) - producer_cpu;
    println!("timing {:.9} {:.9} {begin:.9} {end:.9} {producer_cpu:.9}", setup.as_secs_f64(), steady.as_secs_f64());
}
