use std::{fs, path::Path, process::Command};
fn main() {
    let base = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013/current-root-20261013/ledger-directory-fix-20261013";
    let old = format!("{base}/production-validation-1");
    let diag = format!("{base}/prepare-permission-trace");
    fs::create_dir(&diag).unwrap();
    fs::write(Path::new(&diag).join("namespace-assignment.lock"), []).unwrap();
    let command = fs::read_to_string(Path::new(&old).join("namespace-command.toml")).unwrap();
    let args = command.lines().find(|line| line.starts_with("argv = ")).unwrap().strip_prefix("argv = [").unwrap().strip_suffix(']').unwrap();
    let mut args = args.split(", ").map(|value| value.strip_prefix('"').unwrap().strip_suffix('"').unwrap().replace(&old, &diag)).collect::<Vec<_>>();
    let separator = args.iter().position(|arg| arg == "--").unwrap(); args.truncate(separator + 1);
    args.extend(["/usr/bin/strace".to_owned(), "-f".into(), "-o".into(), format!("{diag}/strace.txt"), "-e".into(), "trace=file,flock".into(), "/home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/bin/sm-pipeline".into(), "decomp-authority".into(), "--request".into(), format!("{base}/prepare-request.toml")]);
    let result = Command::new("bwrap").args(&args).output().unwrap();
    fs::write(Path::new(&diag).join("stdout.txt"), result.stdout).unwrap();
    fs::write(Path::new(&diag).join("stderr.txt"), result.stderr).unwrap();
    fs::write(Path::new(&diag).join("exit.txt"), format!("{}\n", result.status.code().unwrap_or(-1))).unwrap();
}
