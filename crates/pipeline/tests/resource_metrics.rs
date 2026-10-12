//! Resource accounting through the public native execution receipt.
use std::{
    fs,
    io::{Seek, SeekFrom, Write},
    os::unix::fs::{MetadataExt, symlink},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

struct Run(PathBuf);
impl Run {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "sm-resource-control-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn compile(root: &Path, source: &str) -> PathBuf {
    let source_path = root.join("control.c");
    let binary = root.join("control");
    fs::write(&source_path, source).unwrap();
    assert!(Command::new("cc").arg("-O2").arg(&source_path).arg("-o").arg(&binary).status().unwrap().success());
    binary
}
fn execute(root: &Path, binary: &Path, label: &str, argument: &str, paths: &[(&Path, &str)]) -> toml::Value {
    #[derive(serde::Serialize)]
    struct Scope<'a> {
        path: &'a Path,
        kind: &'a str,
    }
    #[derive(serde::Serialize)]
    struct Request<'a> {
        command: Vec<String>,
        run: &'a Path,
        label: &'a str,
        root: PathBuf,
        workers: usize,
        sampler_paths: Vec<Scope<'a>>,
    }
    let request = Request {
        command: vec![binary.to_string_lossy().into_owned(), argument.into()],
        run: root,
        label,
        root: root.join("workers"),
        workers: 1,
        sampler_paths: paths.iter().map(|(path, kind)| Scope { path, kind }).collect(),
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("exec")
        .env_remove("SM_PIPELINE_METRICS_CONTEXT")
        .env("SM_PIPELINE_METRICS", "enabled")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(toml::to_string(&request).unwrap().as_bytes()).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    toml::from_str(&fs::read_to_string(root.join(format!("{label}.metrics.toml"))).unwrap()).unwrap()
}
fn measured(receipt: &toml::Value, field: &str) -> i64 {
    assert_eq!(receipt["resources"][field]["state"].as_str(), Some("measured"));
    receipt["resources"][field]["value"].as_integer().unwrap()
}

fn allocated(path: &Path) -> i64 {
    i64::try_from(fs::symlink_metadata(path).unwrap().blocks() * 512).unwrap()
}

fn unavailable(receipt: &toml::Value, field: &str) {
    assert_eq!(receipt["resources"][field]["state"].as_str(), Some("unavailable"));
    assert!(receipt["resources"][field].get("value").is_none(), "unavailable field claims a numeric value");
}

fn assert_incomplete(receipt: &toml::Value) {
    assert_eq!(receipt["outcome"].as_str(), Some("success"), "telemetry failure changed process outcome");
    assert_eq!(receipt["configuration"]["resource-samples-complete"].as_str(), Some("false"));
    assert!(receipt["resources"]["incomplete_samples"].as_integer().unwrap() > 0);
    assert!(!receipt["configuration"]["resource-limitations"].as_str().unwrap().is_empty());
}

#[test]
fn allocated_disk_counts_sparse_and_pending_files_once_and_does_not_follow_symlinks() {
    let run = Run::new();
    let staging = run.0.join("a-staging");
    let nested = staging.join("nested");
    let private = run.0.join("b-private-store");
    let packs = private.join("packs");
    let outside = run.0.join("outside");
    fs::create_dir_all(&nested).unwrap();
    fs::create_dir_all(&packs).unwrap();
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("excluded"), vec![1u8; 1024 * 1024]).unwrap();

    let sparse = nested.join("sparse.bin");
    let mut file = fs::File::create(&sparse).unwrap();
    file.set_len(64 * 1024 * 1024).unwrap();
    file.seek(SeekFrom::End(-4096)).unwrap();
    file.write_all(&[1u8; 4096]).unwrap();
    file.sync_all().unwrap();
    drop(file);
    assert!(allocated(&sparse) < 1024 * 1024, "control filesystem did not preserve sparse allocation");

    let shared = staging.join("shared.bin");
    fs::write(&shared, vec![2u8; 32 * 1024]).unwrap();
    fs::hard_link(&shared, nested.join("same-inode.bin")).unwrap();
    fs::hard_link(&shared, private.join("same-inode.bin")).unwrap();
    let pending = packs.join("0000000000000000-123.pending");
    fs::write(&pending, vec![3u8; 128 * 1024]).unwrap();
    symlink(&outside, staging.join("outside-directory")).unwrap();
    symlink(outside.join("excluded"), private.join("outside-file")).unwrap();
    symlink(run.0.join("missing"), private.join("dangling")).unwrap();

    // Known control entries give an independent allocated-block oracle. Repeated
    // inode names and symlinks deliberately do not add bytes to this total.
    let expected_staging = allocated(&staging) + allocated(&nested) + allocated(&sparse) + allocated(&shared);
    let expected_private = allocated(&private) + allocated(&packs) + allocated(&pending);
    let binary = compile(&run.0, "#include <unistd.h>\nint main(void){usleep(650000);return 0;}\n");
    let receipt = execute(
        &run.0,
        &binary,
        "allocated",
        "",
        &[(&staging, "staging"), (&nested, "staging"), (&private, "private-store")],
    );
    assert_eq!(measured(&receipt, "staging_allocated_bytes"), expected_staging);
    assert_eq!(measured(&receipt, "private_store_allocated_bytes"), expected_private);
    assert_eq!(measured(&receipt, "combined_allocated_bytes"), expected_staging + expected_private);
    assert!(measured(&receipt, "minimum_free_bytes") > 0);
    assert!(receipt["resources"]["disk_samples"].as_integer().unwrap() >= 2);
    assert_eq!(receipt["configuration"]["disk-missing-paths"].as_str(), Some("0"));
    assert_eq!(receipt["configuration"]["resource-samples-complete"].as_str(), Some("true"));
}

#[test]
fn missing_or_symlink_registered_roots_report_unavailable_incomplete_disk_measurements() {
    let run = Run::new();
    let outside = run.0.join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("excluded"), vec![1u8; 128 * 1024]).unwrap();
    let linked = run.0.join("linked-root");
    symlink(&outside, &linked).unwrap();
    let missing = run.0.join("missing");
    let binary = compile(&run.0, "int main(void){return 0;}\n");
    let receipt = execute(&run.0, &binary, "unreadable", "", &[(&missing, "staging"), (&linked, "private-store")]);
    assert_incomplete(&receipt);
    for field in
        ["staging_allocated_bytes", "private_store_allocated_bytes", "combined_allocated_bytes", "minimum_free_bytes"]
    {
        unavailable(&receipt, field);
    }
    assert!(receipt["configuration"]["disk-missing-paths"].as_str().unwrap().parse::<u64>().unwrap() >= 2);
}

#[test]
fn missing_scope_keeps_readable_allocated_bytes_as_an_explicit_lower_bound() {
    let run = Run::new();
    let staging = run.0.join("staging");
    fs::create_dir(&staging).unwrap();
    let data = staging.join("data");
    fs::write(&data, vec![1u8; 128 * 1024]).unwrap();
    let missing = run.0.join("missing-private-store");
    let expected = allocated(&staging) + allocated(&data);
    let binary = compile(&run.0, "int main(void){return 0;}\n");
    let receipt = execute(&run.0, &binary, "partial", "", &[(&staging, "staging"), (&missing, "private-store")]);
    assert_incomplete(&receipt);
    assert_eq!(measured(&receipt, "staging_allocated_bytes"), expected);
    assert_eq!(measured(&receipt, "combined_allocated_bytes"), expected);
    unavailable(&receipt, "private_store_allocated_bytes");
}

#[test]
fn bounded_disk_depth_reports_truncation_and_preserves_a_lower_bound() {
    let run = Run::new();
    let staging = run.0.join("staging");
    fs::create_dir(&staging).unwrap();
    let mut path = staging.clone();
    let mut directories = vec![path.clone()];
    for _ in 0..258 {
        path = path.join("d");
        fs::create_dir(&path).unwrap();
        directories.push(path.clone());
    }
    let unseen = path.join("beyond-depth-bound");
    fs::write(&unseen, vec![1u8; 128 * 1024]).unwrap();
    let directory_bytes: i64 = directories.iter().map(|directory| allocated(directory)).sum();
    let binary = compile(&run.0, "int main(void){return 0;}\n");
    let receipt = execute(&run.0, &binary, "bounded", "", &[(&staging, "staging")]);
    assert_incomplete(&receipt);
    assert!(measured(&receipt, "combined_allocated_bytes") <= directory_bytes);
    assert!(receipt["configuration"]["truncated-scans"].as_str().unwrap().parse::<u64>().unwrap() > 0);
    assert!(receipt["configuration"]["resource-limitations"].as_str().unwrap().contains("bound reached"));
}

#[test]
fn concurrent_descendant_memory_is_sampled_together_without_adding_separate_peaks() {
    let run = Run::new();
    let binary = compile(
        &run.0,
        r#"
#include <stdlib.h>
#include <unistd.h>
#include <sys/wait.h>
static void allocate(void) {
  volatile unsigned char *p = malloc(64*1024*1024);
  if (!p) _exit(2);
  for (unsigned i=0; i<64*1024*1024; i+=4096) p[i]=1;
  usleep(800000);
  free((void*)p);
  _exit(0);
}
int main(int argc, char **argv) {
  int status;
  pid_t first=fork();
  if (first==0) allocate();
  if (first<0) return 3;
  if (argv[1][0]=='s') {waitpid(first,&status,0); if(status) return 4;}
  pid_t second=fork();
  if (second==0) allocate();
  if (second<0) return 5;
  waitpid(second,&status,0); if(status) return 6;
  if (argv[1][0]!='s') {waitpid(first,&status,0); if(status) return 7;}
  return 0;
}
"#,
    );
    let sequential = execute(&run.0, &binary, "sequential", "sequential", &[]);
    let overlapping = execute(&run.0, &binary, "overlapping", "overlapping", &[]);
    let sequential_peak = measured(&sequential, "aggregate_sampled_rss_bytes");
    let overlapping_peak = measured(&overlapping, "aggregate_sampled_rss_bytes");
    assert!(sequential_peak >= 60 * 1024 * 1024, "descendant memory missing: {sequential_peak}");
    assert!(sequential_peak < 112 * 1024 * 1024, "separate maxima were added: {sequential_peak}");
    assert!(
        overlapping_peak >= sequential_peak + 48 * 1024 * 1024,
        "overlap missing: {sequential_peak} versus {overlapping_peak}"
    );
    assert!(overlapping["resources"]["rss_samples"].as_integer().unwrap() >= 4);
}
