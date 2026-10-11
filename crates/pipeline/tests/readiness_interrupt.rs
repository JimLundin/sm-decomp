#![doc = " SIGINT must retain the active process outcome and stop later stages."]
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use toml::Value;
#[test]
fn interrupted_preflight_retains_logs_and_prevents_later_commands() {
    let root = std::env::temp_dir()
        .join(format!("sm-readiness-interrupt-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()));
    fs::create_dir_all(root.join("bin")).unwrap();
    assert!(Command::new("git").args(["init", "-q"]).arg(&root).status().unwrap().success());
    let just = root.join("bin/just");
    fs::write(&just, "#!/bin/sh\necho retained-stage-output\nsleep 60\n").unwrap();
    fs::set_permissions(&just, fs::Permissions::from_mode(0o755)).unwrap();
    let stderr = root.join("stderr");
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .args(["readiness-preflight", "--root"])
        .arg(&root)
        .env("PATH", format!("{}:{}", root.join("bin").display(), std::env::var("PATH").unwrap()))
        .stderr(fs::File::create(&stderr).unwrap())
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let started = Instant::now();
    let run = loop {
        if let Ok(entries) = fs::read_dir(root.join("out/runs"))
            && let Some(entry) = entries.into_iter().next()
        {
            let path = entry.unwrap().path();
            if fs::read_to_string(path.join("logs/doctor.stdout.log"))
                .is_ok_and(|s| s.contains("retained-stage-output"))
            {
                break path;
            }
        }
        assert!(started.elapsed() < Duration::from_secs(15), "{}", fs::read_to_string(&stderr).unwrap_or_default());
        std::thread::sleep(Duration::from_millis(10));
    };
    unsafe {
        assert_eq!(kill(child.id() as i32, 2), 0);
    }
    assert!(!child.wait().unwrap().success());
    let verdict: Value = toml::from_str(&fs::read_to_string(run.join("verdict.toml")).unwrap()).unwrap();
    assert_eq!(verdict["status"], toml::Value::try_from("incomplete").unwrap());
    assert_eq!(verdict["acceptance_eligible"], toml::Value::try_from(false).unwrap());
    assert_eq!(verdict["checks"]["doctor"]["reason"], toml::Value::try_from("stage interrupted").unwrap());
    assert_eq!(verdict["checks"]["doctor"]["complete"], toml::Value::try_from(false).unwrap());
    let manifest: Value = toml::from_str(&fs::read_to_string(run.join("manifest.toml")).unwrap()).unwrap();
    assert_eq!(manifest["commands"].as_array().unwrap().len(), 1);
    fs::remove_dir_all(root).unwrap();
}
unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
}
