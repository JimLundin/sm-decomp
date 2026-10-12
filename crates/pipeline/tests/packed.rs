//! Native capture publishes only authenticated bytes and preserves failed staging.
use evidence::{Pointer, Provenance};
use serde::{Deserialize, Serialize};
#[derive(Clone, Serialize)]
struct CaptureRequest {
    directory: PathBuf,
    reader: PathBuf,
    command: Vec<String>,
    provenance: Provenance,
    root: PathBuf,
    workers: usize,
    timeout: f64,
}
#[derive(Serialize)]
struct PublishRequest {
    directory: PathBuf,
    store: PathBuf,
    root: PathBuf,
    workers: usize,
}
#[derive(Serialize)]
struct MaterializeRequest {
    directory: PathBuf,
    destination: PathBuf,
    member: String,
    frame: u64,
    root: PathBuf,
    workers: usize,
}
#[derive(Deserialize)]
struct Timing {
    completed: bool,
}
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};
struct Run(PathBuf);
impl Drop for Run {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
fn hash(path: &Path) -> String {
    format!("{:x}", Sha256::digest(fs::read(path).unwrap()))
}
fn invoke<T: Serialize>(command: &str, request: T, success: bool) -> Option<Pointer> {
    let mut process = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg(command)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    process.stdin.take().unwrap().write_all(toml::to_string(&request).unwrap().as_bytes()).unwrap();
    let output = process.wait_with_output().unwrap();
    assert_eq!(output.status.success(), success, "{}", String::from_utf8_lossy(&output.stderr));
    if !success || command == "packed-materialize" {
        None
    } else {
        Some(toml::from_str(std::str::from_utf8(&output.stdout).unwrap()).unwrap())
    }
}
#[test]
fn four_private_captures_share_after_publication_and_failed_producer_keeps_staging() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let reader = std::env::var_os("SM_EVIDENCE_TEST_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| checkout.join("target/debug/sm-evidence"));
    assert!(reader.is_file(), "build current native evidence reader before packed controls");
    let root = Run(std::env::temp_dir()
        .join(format!("sm-packed-test-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())));
    fs::create_dir(&root.0).unwrap();
    fs::copy(reader, root.0.join("sm-evidence")).unwrap();
    let reader = root.0.join("sm-evidence");
    fs::write(root.0.join("producer.c"),"#include <stdio.h>\n#include <stdlib.h>\n#include <unistd.h>\nint main(int argc,char **argv){if(chdir(getenv(\"SM_EVIDENCE_STAGING\")))return 2;FILE *f=fopen(getenv(\"SM_EVIDENCE_PIPE\"),\"wb\");if(!f)return 2;for(int i=0;i<40000;i++)fprintf(f,\"ppu 7 0 %u %u\\n\",i%2?24:25,i%256);fputs(\"apu 0 3 4\\nunknown 0 exact\\n\",f);if(fclose(f))return 2;if(argc>1){if(argv[1][0]=='s')sleep(60);return 3;}f=fopen(\"frame-000001.bin\",\"wb\");for(int i=0;i<1401335;i++)fputc(i%13,f);fclose(f);f=fopen(\"fixtures/capture.wram\",\"wb\");fputs(\"memory bytes\",f);return fclose(f);}").unwrap();
    let producer = root.0.join("producer");
    assert!(Command::new("cc").arg(root.0.join("producer.c")).arg("-o").arg(&producer).status().unwrap().success());
    let names = ["c-0", "c-1", "mixed-0", "mixed-1"];
    let requests = names
        .into_iter()
        .map(|name| {
            let directory = root.0.join(name);
            fs::create_dir(&directory).unwrap();
            CaptureRequest {
                directory,
                reader: reader.clone(),
                command: vec![producer.to_string_lossy().into_owned()],
                provenance: Provenance {
                    subject: Some(name.into()),
                    reader_sha256: Some(hash(&reader)),
                    binary_sha256: Some(hash(&producer)),
                    ..Default::default()
                },
                root: root.0.join("workers"),
                workers: 4,
                timeout: 60.0,
            }
        })
        .collect::<Vec<_>>();
    let pointers = std::thread::scope(|scope| {
        requests
            .iter()
            .map(|r| scope.spawn(|| invoke("packed-capture", r.clone(), true).unwrap()))
            .collect::<Vec<_>>()
            .into_iter()
            .map(|thread| thread.join().unwrap())
            .collect::<Vec<_>>()
    });
    assert!(pointers.iter().all(|pointer| pointer.observations == pointers[0].observations));
    for name in names {
        let metrics: toml::Value =
            toml::from_str(&fs::read_to_string(root.0.join(name).join("capture.metrics.toml")).unwrap()).unwrap();
        assert_eq!(metrics["format"].as_str(), Some("pipeline-metrics-v1"));
        let processes = metrics["processes"].as_array().unwrap();
        assert!(processes.iter().any(|p| p["role"].as_str() == Some("producer")));
        assert!(processes.iter().any(|p| p["role"].as_str() == Some("collector")));
        assert!(processes.iter().all(|p| p["reap_count"].as_integer() == Some(1)));
        let stages = metrics["stages"].as_array().unwrap();
        for stage in [
            "trace-commit",
            "staged-tree-ingest",
            "final-commit",
            "reopened-verification",
            "staged-comparison",
            "pointer-fsync",
            "subject-authentication",
            "staging-release",
        ] {
            assert!(stages.iter().any(|s| s["name"].as_str() == Some(stage)), "missing {stage}");
        }
        assert!(pointers[0].artifacts.keys().all(|key| !key.contains("metrics")));
    }
    for (name, pointer) in names.into_iter().zip(pointers) {
        let directory = root.0.join(name);
        assert!(!directory.join(".evidence-stage").exists());
        let updated = invoke(
            "packed-publish",
            PublishRequest {
                directory: directory.clone(),
                store: root.0.join("shared"),
                root: root.0.join("workers"),
                workers: 4,
            },
            true,
        )
        .unwrap();
        let publication: toml::Value =
            toml::from_str(&fs::read_to_string(directory.join("publication.metrics.toml")).unwrap()).unwrap();
        let stages = publication["stages"].as_array().unwrap();
        for name in
            ["store-wait", "shared-import", "import-commit", "import-destination-verification", "private-store-release"]
        {
            assert!(stages.iter().any(|s| s["name"].as_str() == Some(name)), "missing publication stage {name}");
        }
        assert_eq!(updated.run, pointer.run);
        assert!(!directory.join("store").exists());
        let output = Command::new(&reader)
            .arg("frame")
            .arg(root.0.join("shared"))
            .arg(&updated.run)
            .args(["hardware.log", "0"])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(output.stdout, b"apu 0 3 4\nunknown 0 exact\n");
        let extracted = root.0.join(format!("{name}-frame.log"));
        invoke(
            "packed-materialize",
            MaterializeRequest {
                directory: directory.clone(),
                destination: extracted.clone(),
                member: "hardware.log".into(),
                frame: 0,
                root: root.0.join("workers"),
                workers: 4,
            },
            true,
        );
        assert_eq!(fs::read(extracted).unwrap(), b"apu 0 3 4\nunknown 0 exact\n");
    }
    for (name, arg, timeout) in [("failed", "fail", 60.0), ("timeout", "sleep", 0.1)] {
        let directory = root.0.join(name);
        fs::create_dir(&directory).unwrap();
        let mut request = requests[0].clone();
        request.directory = directory.clone();
        request.command = vec![producer.to_string_lossy().into_owned(), arg.into()];
        request.timeout = timeout;
        invoke("packed-capture", request, false);
        assert!(directory.join(".evidence-stage").exists());
        assert!(!directory.join("evidence.toml").exists());
        assert!(!directory.join(".effects.pipe").exists());
        let timing: Timing =
            toml::from_str(&fs::read_to_string(directory.join("pipeline-timing.toml")).unwrap()).unwrap();
        assert!(!timing.completed);
        let metrics: toml::Value =
            toml::from_str(&fs::read_to_string(directory.join("capture.metrics.toml")).unwrap()).unwrap();
        assert_eq!(metrics["outcome"].as_str(), Some(if name == "timeout" { "timed-out" } else { "failed" }));
        assert!(metrics["error"].as_str().is_some_and(|s| !s.is_empty()));
        assert_eq!(metrics["processes"].as_array().unwrap().len(), 2);
        assert!(metrics["processes"].as_array().unwrap().iter().all(|p| p["reap_count"].as_integer() == Some(1)));
    }
}

#[test]
fn native_capture_workers_run_two_producers_simultaneously() {
    let checkout = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap();
    let reader = std::env::var_os("SM_EVIDENCE_TEST_BINARY")
        .map(PathBuf::from)
        .unwrap_or_else(|| checkout.join("target/debug/sm-evidence"));
    let root = Run(std::env::temp_dir()
        .join(format!("sm-capture-rendezvous-{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos())));
    fs::create_dir(&root.0).unwrap();
    fs::copy(reader, root.0.join("sm-evidence")).unwrap();
    fs::write(root.0.join("rendezvous.c"),"#include <stdio.h>\n#include <stdlib.h>\n#include <unistd.h>\nint main(int argc,char**argv){FILE*f=fopen(argv[1],\"w\");fputs(\"ready\",f);fclose(f);for(int i=0;access(argv[2],F_OK)&&i<100;i++)usleep(20000);if(access(argv[2],F_OK))return 7;f=fopen(getenv(\"SM_EVIDENCE_PIPE\"),\"wb\");if(!f)return 2;fputs(\"ppu 0 0 1 2\\n\",f);return fclose(f);}\n").unwrap();
    let producer = root.0.join("rendezvous");
    assert!(Command::new("cc").arg(root.0.join("rendezvous.c")).arg("-o").arg(&producer).status().unwrap().success());
    let mut captures = vec![];
    for (name, peer) in [("first", "second"), ("second", "first")] {
        let directory = root.0.join(name);
        fs::create_dir(&directory).unwrap();
        captures.push(CaptureRequest {
            directory,
            reader: root.0.join("sm-evidence"),
            command: vec![
                producer.to_string_lossy().into_owned(),
                root.0.join(format!("{name}.ready")).to_string_lossy().into_owned(),
                root.0.join(format!("{peer}.ready")).to_string_lossy().into_owned(),
            ],
            provenance: Provenance {
                reader_sha256: Some(hash(&root.0.join("sm-evidence"))),
                binary_sha256: Some(hash(&producer)),
                ..Default::default()
            },
            root: root.0.join("workers"),
            workers: 2,
            timeout: 5.0,
        });
    }
    #[derive(Serialize)]
    struct Request {
        captures: Vec<CaptureRequest>,
        jobs: usize,
    }
    let mut process = Command::new(env!("CARGO_BIN_EXE_sm-pipeline"))
        .arg("packed-capture-many")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    process
        .stdin
        .take()
        .unwrap()
        .write_all(toml::to_string(&Request { captures, jobs: 2 }).unwrap().as_bytes())
        .unwrap();
    let output = process.wait_with_output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}
