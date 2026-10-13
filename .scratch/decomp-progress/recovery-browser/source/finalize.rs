use std::{fs, path::Path, process::Command};

fn main() {
    let repo = Path::new("/home/jim/sm-decomp");
    let evidence = repo.join(".scratch/decomp-progress/recovery-browser");
    let selected = [
        "/tmp/sm-progress-validation",
        "/tmp/sm-progress-validation-six",
        "/tmp/sm-progress-validation-browser-v2",
        "/tmp/sm-progress-validation-browser-v3",
        "/tmp/sm-progress-browser/target",
    ];
    let sizes = fs::read_to_string(evidence.join("cleanup/before-bytes.txt")).unwrap();
    let reclaimed: u64 = sizes.lines().map(|line| line.split_whitespace().next().unwrap().parse::<u64>().unwrap()).sum();
    for selected in selected { fs::remove_dir_all(selected).unwrap(); assert!(!Path::new(selected).exists()); }
    let remaining = Command::new("df").args(["-B1", "--output=avail", "/home/jim/sm-decomp"]).output().unwrap();
    assert!(remaining.status.success());
    fs::write(evidence.join("cleanup/after-disk.txt"), &remaining.stdout).unwrap();
    let available: u64 = String::from_utf8(remaining.stdout).unwrap().split_whitespace().last().unwrap().parse().unwrap();
    let report = format!(r#"# Recovered progress browser verification

Status: verified in an isolated synthetic root; retained evidence and cleanup complete.

The Rust helper drives the current `sm-pipeline` CLI and installed Chromium through CDP. The successful run used the refreshed 6,053-definition source inventory (75,229 body lines); all 220 routines in the whole Ridley scope are present. This is progress tooling verification and makes no gameplay ownership claim. Maintained progress, HTML and source inventory implementation files were not edited by this verifier.

`verification.json` records six separate concurrent CLI update processes, 51 reads of complete valid JSON during publication, six retained observations and unchanged observation identity after an identical retry. All six writers succeeded within 5,645 milliseconds; observed completion times were 4,198–5,634 milliseconds. The earlier six-writer runs also passed publication in 6,908 and 4,734 milliseconds before helper-only browser setup failures.

Browser assertions cover symbol search; combined bank, agent and scope filters; independent translated/verified/integrated stage counts; remaining routines; expanded evidence SHA-256 details; theme switching; seven-second live refresh while preserving filters; changed evidence excluded from stage totals and displayed as stale; file picker loading; malformed JSON error; standalone offline embedded data; and a 390-pixel phone viewport without horizontal overflow. `desktop.png` and `phone.png` were inspected. `index.html` and `registry.json` retain the isolated synthetic snapshot; their old temporary root and deliberately changed evidence are not acceptance pointers.

The initial 24-process stress attempt reached the intentional ten-second registry lock bound. It retained 19 complete observations and valid JSON; five writers timed out. This remains a bounded contention/performance limitation, not a lost-update or malformed-publication finding, and no maintained source fix is claimed. Failure diagnostics are `concurrency-24-failure.json` and `concurrency-24-failure.stderr`. The existing six-writer unit test was preserved. Browser helper defects were corrected in retained `source/main.rs`: HTTP reads honor Content-Length, the Host header includes the CDP port, and CDP reads have a bounded timeout. These changes affected only the disposable verifier.

Exact retained helper, pipeline binary and maintained source digests are in `source-hashes.sha256`. Rebuild the verifier with `cargo build --manifest-path source/Cargo.toml --locked --offline` after placing `source/main.rs` under a conventional `src/` directory, then run with a fresh `SM_PROGRESS_BROWSER_ROOT`. Its current default repository and inventory paths are recorded in source; inventory bytes are retained as `source/source-inventory.toml`.

## Completion evidence and cleanup

All verifier, CLI server and browser processes had exited before cleanup. Both required project cleanup passes succeeded for each of the four isolated roots and the throwaway Cargo target, reclaiming zero bytes because these formats are outside recognized gameplay run cleanup. Ten receipts are under `cleanup/`.

After copying reports, screenshots, helper source, Cargo lockfile, inventory and hashes into this retained directory, `source/finalize.rs` removed the four isolated roots and `/tmp/sm-progress-browser/target`. Measured apparent bytes removed: {reclaimed}. No implementer Git worktree, shared evidence store or gameplay run was changed. Available filesystem bytes afterward: {available}. Exact before-size and after-disk outputs are retained under `cleanup/`.
"#);
    fs::write(evidence.join("report.md"), report).unwrap();
    println!("removed {reclaimed} measured apparent bytes; {available} filesystem bytes available; retained report {}", evidence.join("report.md").display());
    let executable = std::env::current_exe().unwrap();
    fs::remove_file(executable).unwrap();
}
