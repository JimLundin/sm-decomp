use std::{fs, path::{Path, PathBuf}, process::Command};
const OLD: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013/current-root-20261013";
const NEW: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/preparation/readiness-final-20261013/current-root-20261013/ledger-directory-fix-20261013";
const ENGINE: &str = "/home/jim/sm-decomp/.scratch/decomp-parallel/drill-retry/bin/sm-pipeline";
fn hash(path: impl AsRef<Path>) -> String {
    let result = Command::new("sha256sum").arg(path.as_ref()).output().unwrap(); assert!(result.status.success());
    String::from_utf8(result.stdout).unwrap().split_whitespace().next().unwrap().to_owned()
}
fn write(name: &str, body: impl AsRef<[u8]>) { fs::write(Path::new(NEW).join(name), body).unwrap(); }
fn command(program: &str, args: &[&str], name: &str) {
    let result = Command::new(program).args(args).output().unwrap();
    write(&format!("{name}.stdout"), result.stdout); write(&format!("{name}.stderr"), result.stderr); assert!(result.status.success(), "{name} failed");
}
fn visit(path: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(path).unwrap() { let entry = entry.unwrap();
        if ["target", "package", "artifacts.sha256"].iter().any(|name| entry.file_name() == *name) { continue; }
        let path = entry.path(); if path.is_dir() { visit(&path, files); } else { files.push(path); }
    }
}
fn main() {
    assert!(!Path::new(NEW).join("artifacts.sha256").exists());
    assert!(!Path::new(OLD).join("production-validation-1").exists());
    assert_eq!(hash(format!("{OLD}/artifacts.sha256")), "f83d696c5cadd898dce86c9f24ad5eda1e89db166c432d392a013f52ce695ec1");
    command("sha256sum", &["-c", &format!("{OLD}/artifacts.sha256")], "predecessor-authentication");
    let source = fs::read_to_string(format!("{OLD}/main.rs")).unwrap();
    let marker = "        let entry = entry.map_err(|error| error.to_string())?;\n        let path = entry.path().join(\"ledger.toml\");";
    assert_eq!(source.matches(marker).count(), 1);
    let fixed = "        let entry = entry.map_err(|error| error.to_string())?;\n        let entry_type = entry.file_type().map_err(|error| error.to_string())?;\n        ensure(!entry_type.is_symlink(), \"effort root must not be a symlink\")?;\n        if !entry_type.is_dir() { continue; }\n        let path = entry.path().join(\"ledger.toml\");";
    write("main.rs", source.replace(marker, fixed).replace(&format!("const READINESS: &str = \"{OLD}\";"), &format!("const READINESS: &str = \"{NEW}\";")));
    for name in ["Cargo.toml", "Cargo.lock", "prepare-request.toml"] { fs::copy(Path::new(OLD).join(name), Path::new(NEW).join(name)).unwrap(); }
    let manifest = format!("{NEW}/Cargo.toml"); let target = format!("{NEW}/target");
    command("cargo", &["build", "--locked", "--offline", "--manifest-path", &manifest, "--target-dir", &target], "build");
    fs::create_dir(Path::new(NEW).join("bin")).unwrap();
    fs::copy(Path::new(NEW).join("target/debug/decomp-readiness"), Path::new(NEW).join("bin/decomp-readiness")).unwrap();
    write("helper-build.toml", format!("schema = \"decomp-readiness-helper-build-v1\"\nsource_sha256 = \"{}\"\nmanifest_sha256 = \"{}\"\nlock_sha256 = \"{}\"\nbinary_sha256 = \"{}\"\nactual_caller_required = \"01a13559-e517-72d1-88b5-e554fed61da4\"\nnamespace_validation_run = false\nlimitations = [\"Predecessor actual root invocation returned os20 before output/namespace creation because assignment.lock is a regular file\", \"This helper skips nondirectory entries, rejects symlink effort roots and retains regular nonsymlink ledger enforcement\", \"Same real caller and prepared request observation evidence; frozen source/engine/drill stay unchanged\"]\n", hash(format!("{NEW}/main.rs")), hash(&manifest), hash(format!("{NEW}/Cargo.lock")), hash(format!("{NEW}/bin/decomp-readiness"))));
    write("predecessor-failure.toml", "schema = \"decomp-readiness-predecessor-failure-v1\"\nactual_caller = \"01a13559-e517-72d1-88b5-e554fed61da4\"\nexit_code = 1\nstderr = \"Not a directory (os error 20)\"\noutput_directory_absent = true\nnamespace_created = false\nobservation_source = \"Root reported actual tool command failure; no matching documented output exists\"\npredecessor_manifest_sha256 = \"f83d696c5cadd898dce86c9f24ad5eda1e89db166c432d392a013f52ce695ec1\"\n");
    for (name, dedup) in [("cleanup1", false), ("cleanup2", true)] {
        let mut args = vec!["cleanup", "--root", NEW]; if dedup { args.push("--deduplicate"); } args.push("--apply"); command(ENGINE, &args, name);
    }
    let measured = Command::new("du").args(["-sb", &target]).output().unwrap(); assert!(measured.status.success());
    let bytes = String::from_utf8(measured.stdout).unwrap().split_whitespace().next().unwrap().to_owned();
    command("cargo", &["clean", "--manifest-path", &manifest, "--target-dir", &target], "cargo-clean");
    let result = Command::new("df").args(["-B1", "--output=avail", NEW]).output().unwrap();
    let available = String::from_utf8(result.stdout).unwrap().split_whitespace().last().unwrap().to_owned();
    write("cleanup.toml", format!("schema = \"decomp-readiness-package-cleanup-v1\"\nstatus = \"complete\"\nprivate_target_logical_bytes_before_clean = {bytes}\navailable_filesystem_bytes = {available}\nhelper_binary_retained = true\nnamespace_validation_run = false\n"));
    let mut files = Vec::new(); visit(Path::new(NEW), &mut files); files.sort();
    write("artifacts.sha256", files.iter().map(|path| format!("{}  {}\n", hash(path), path.display())).collect::<String>());
}
