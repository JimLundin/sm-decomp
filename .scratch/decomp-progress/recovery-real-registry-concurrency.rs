use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Arc, Barrier,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
    time::{Duration, Instant},
};

fn main() {
    let repo = Path::new("/home/jim/sm-decomp");
    let cli = repo.join("target/release/sm-pipeline");
    let output = repo.join(".scratch/decomp-progress/recovery-real-registry-concurrency");
    fs::create_dir_all(&output).unwrap();
    let shared = repo.join(".scratch/decomp-progress/registry.json");
    let registry = output.join("registry.json");
    let html = output.join("index.html");
    fs::copy(&shared, &registry).unwrap();
    let before = fs::read(&registry).unwrap();
    let updates = [
        "ridley-whole-6053-recovered-translating-20261013",
        "ridley-whole-6053-recovered-translated-20261013",
        "projectile-trails-6053-recovered-translated-20261013",
        "projectile-trails-6053-recovered-verified-20261013",
        "samus-projectile-frame-6053-recovered-translated-20261013",
        "samus-projectile-frame-6053-recovered-verified-20261013",
    ];
    let barrier = Arc::new(Barrier::new(updates.len() + 1));
    let finished = Arc::new(AtomicUsize::new(0));
    let started = Instant::now();
    let mut writers = Vec::new();
    for (index, update) in updates.into_iter().enumerate() {
        let barrier = barrier.clone();
        let finished = finished.clone();
        let cli = cli.clone();
        let registry = registry.clone();
        let html = html.clone();
        let root: PathBuf = repo.into();
        let output = output.clone();
        writers.push(thread::spawn(move || {
            let stdout = fs::File::create(output.join(format!("writer-{index}.stdout"))).unwrap();
            let stderr = fs::File::create(output.join(format!("writer-{index}.stderr"))).unwrap();
            barrier.wait();
            let status = Command::new(cli)
                .args(["progress", "update", "--root"])
                .arg(root)
                .arg("--registry")
                .arg(registry)
                .arg("--html")
                .arg(html)
                .arg("--update")
                .arg(format!("/home/jim/sm-decomp/.scratch/decomp-port/progress-{update}.toml"))
                .stdout(stdout)
                .stderr(stderr)
                .status()
                .unwrap();
            finished.fetch_add(1, Ordering::SeqCst);
            (status.success(), started.elapsed().as_millis())
        }));
    }
    barrier.wait();
    let mut atomic_reads = 0;
    while finished.load(Ordering::SeqCst) < updates.len() {
        // An identical retry must preserve the entire authoritative JSON,
        // including observation timestamps, so exact bytes also detect any
        // torn read during publication without a synthetic data model.
        assert_eq!(fs::read(&registry).unwrap(), before, "non-atomic registry read");
        atomic_reads += 1;
        thread::sleep(Duration::from_millis(5));
    }
    let results = writers.into_iter().map(|writer| writer.join().unwrap()).collect::<Vec<_>>();
    assert_eq!(fs::read(&registry).unwrap(), before, "identical retries changed observations");
    assert_eq!(fs::read(&shared).unwrap(), before, "shared registry changed during isolated verification");
    assert!(html.is_file());
    let success = results.iter().all(|result| result.0);
    let times = results.iter().map(|result| result.1.to_string()).collect::<Vec<_>>().join(",");
    let json = format!(
        "{{\"success\":{success},\"writers\":6,\"atomic_exact_reads\":{atomic_reads},\"unchanged_observation_bytes\":true,\"unchanged_shared_registry\":true,\"registry_bytes\":{},\"completion_ms\":[{times}],\"total_ms\":{}}}\n",
        before.len(),
        started.elapsed().as_millis()
    );
    fs::write(output.join("verification.json"), &json).unwrap();
    print!("{json}");
    assert!(success, "a release CLI writer failed; inspect retained stderr");
}
