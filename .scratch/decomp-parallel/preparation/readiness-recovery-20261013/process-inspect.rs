//! Read-only process identity/path inspection for retained Ceres ownership after coordinator recovery.
use std::{env, fs, path::Path, process::Command};

fn q(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r"))
}
fn main() {
    let checked = [
        "/tmp/sm-ceres-continuation-compact-controller-1",
        "/tmp/sm-ceres-continuation-shared-game-host-1",
        "/home/jim/sm-decomp/out/port/ceres-continuation",
        "/home/jim/sm-decomp/.scratch/ceres-continuation",
        "/tmp/sm-ceres-coordinator-target",
        "/tmp/sm-prefix-native-",
    ];
    let drill_paths = [
        "/tmp/sm-decomp-concurrency-drill-final-20261013",
        "/tmp/sm-concurrency-drill-final-",
        "/tmp/sm-decomp-drill-final-harness-build",
        "/home/jim/sm-decomp/.scratch/decomp-parallel/drill-final",
    ];
    let observed = Command::new("date").args(["-u", "+%Y-%m-%dT%H:%M:%SZ"]).output().unwrap();
    println!("schema = \"decomp-readiness-process-inspection-v1\"");
    println!("observed_at = {}", q(String::from_utf8_lossy(&observed.stdout).trim()));
    println!("observer_thread = {}", q(&env::var("CODEX_THREAD_ID").unwrap()));
    println!("boot_id = {}", q(fs::read_to_string("/proc/sys/kernel/random/boot_id").unwrap().trim()));
    println!("checked_path_prefixes = [{}]", checked.iter().map(|path| q(path)).collect::<Vec<_>>().join(", "));
    println!(
        "limitations = [\"A transient /proc disappearance is retained as a race. Generic shared app/REA processes do not prove original-owner liveness. Read-only reporting server may remain after reporting Git handoff.\"]"
    );
    let mut processes = Vec::new();
    for entry in fs::read_dir("/proc").unwrap().flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else { continue };
        if pid == std::process::id() {
            continue;
        }
        let raw = entry.path();
        let Ok(bytes) = fs::read(raw.join("cmdline")) else { continue };
        let argv: Vec<String> = bytes
            .split(|byte| *byte == 0)
            .filter(|arg| !arg.is_empty())
            .map(|arg| String::from_utf8_lossy(arg).into_owned())
            .collect();
        if argv.is_empty() {
            continue;
        }
        let cwd = fs::read_link(raw.join("cwd")).ok().map(|path| path.display().to_string());
        let mut matches = Vec::new();
        let mut drill_matches = Vec::new();
        if let Some(path) = &cwd {
            if checked.iter().any(|prefix| path.starts_with(prefix)) {
                matches.push(format!("cwd:{path}"));
            }
            if drill_paths.iter().any(|prefix| path.starts_with(prefix)) {
                drill_matches.push(format!("cwd:{path}"));
            }
        }
        for argument in &argv {
            if checked.iter().any(|prefix| argument.contains(prefix)) {
                matches.push(format!("argv:{argument}"));
            }
            if drill_paths.iter().any(|prefix| argument.contains(prefix)) {
                drill_matches.push(format!("argv:{argument}"));
            }
        }
        if let Ok(fds) = fs::read_dir(raw.join("fd")) {
            for fd in fds.flatten() {
                if let Ok(path) = fs::read_link(fd.path()) {
                    let path = path.display().to_string();
                    if checked.iter().any(|prefix| path.starts_with(prefix)) {
                        matches.push(format!("fd{}:{path}", fd.file_name().to_string_lossy()));
                    }
                    if drill_paths.iter().any(|prefix| path.starts_with(prefix)) {
                        drill_matches.push(format!("fd{}:{path}", fd.file_name().to_string_lossy()));
                    }
                }
            }
        }
        let reporting = argv.iter().any(|arg| arg == "progress") && argv.iter().any(|arg| arg == "serve");
        let main_root = cwd.as_deref() == Some("/home/jim/sm-decomp");
        if matches.is_empty() && drill_matches.is_empty() && !reporting && !main_root {
            continue;
        }
        let stat = fs::read_to_string(raw.join("stat")).ok();
        let tail = stat.as_deref().and_then(|value| value.rsplit_once(") ").map(|(_, tail)| tail));
        let fields = tail.map(|value| value.split_whitespace().collect::<Vec<_>>());
        let kernel = fields.as_ref().and_then(|fields| fields.first()).copied().unwrap_or("disappeared");
        let start_ticks = fields.as_ref().and_then(|fields| fields.get(19)).and_then(|value| value.parse::<u64>().ok());
        let ppid = fields.as_ref().and_then(|fields| fields.get(1)).and_then(|value| value.parse::<u32>().ok());
        let mut value = format!(
            "\n[[processes]]\npid = {pid}\nargv = [{}]\nmain_checkout_cwd = {main_root}\nreporting_server = {reporting}\nclaim_path_match = {}\nmatch_details = [{}]\nkernel_state = {}\n",
            argv.iter().map(|arg| q(arg)).collect::<Vec<_>>().join(", "),
            !matches.is_empty(),
            matches.iter().map(|entry| q(entry)).collect::<Vec<_>>().join(", "),
            q(kernel)
        );
        value.push_str(&format!(
            "fresh_drill_path_match = {}\nfresh_drill_match_details = [{}]\n",
            !drill_matches.is_empty(),
            drill_matches.iter().map(|entry| q(entry)).collect::<Vec<_>>().join(", ")
        ));
        if let Some(cwd) = cwd {
            value.push_str(&format!("cwd = {}\n", q(&cwd)));
        }
        if let Some(start_ticks) = start_ticks {
            value.push_str(&format!("start_ticks = {start_ticks}\n"));
        }
        if let Some(ppid) = ppid {
            value.push_str(&format!("ppid = {ppid}\n"));
        }
        processes.push((pid, value));
    }
    processes.sort_by_key(|entry| entry.0);
    for (_, process) in processes {
        print!("{process}");
    }
    let _ = Path::new("/proc");
}
