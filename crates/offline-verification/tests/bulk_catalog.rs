use anyhow::Result;
use offline_verification::{
    BatchAdmission, BatchVerification, CatalogRequest, FamilyEvidence, FamilyJob, HostInputs, OfflineFamilies,
    fixtures::{self, Case, Manifest, Routine},
};
use std::{fs, path::Path, sync::Mutex};
use tool_execution::{bounded, testing::Directory};

fn verifier() -> OfflineFamilies {
    OfflineFamilies::new(HostInputs::new(&[("host.rs", "// public interface test")], "1".repeat(64)).unwrap())
}

fn checkout(configuration: &str) -> Result<Directory> {
    let directory = Directory::new()?;
    fs::create_dir_all(directory.0.join("tools/port"))?;
    fs::write(directory.0.join("tools/port/batch.toml"), configuration)?;
    Ok(directory)
}

fn declaration(root: &Path, name: &str) -> Result<()> {
    fixtures::write(
        &root.join(format!("tools/port/{name}.smf")),
        &Manifest {
            family: "shared-content".into(),
            rust_module: "crates/port/src/step.rs".into(),
            bank: "sm_80.c".into(),
            routines: vec![Routine {
                symbol: "Step".into(),
                cases: vec![Case { name: "root".into(), ..Default::default() }],
                ..Default::default()
            }],
            ..Default::default()
        },
    )
}

#[test]
fn explicit_declarations_are_lazy_and_concurrent_aliases_decode_once() -> Result<()> {
    let root = checkout("[families]\na='family'\nb='family'\n[profiles]\nall=['b','a']\n")?;
    declaration(&root.0, "a")?;
    declaration(&root.0, "b")?;
    let snapshot = verifier().capture_catalog(CatalogRequest::new(&root.0)?)?;
    assert_eq!(snapshot.statistics().catalog_parses, 1);
    assert_eq!(snapshot.statistics().manifest_decodes, 0);
    assert_eq!(snapshot.statistics().manifest_reads, 2);
    let captured_bytes = snapshot.statistics().input_bytes;
    assert_eq!(snapshot.catalog().profiles["all"], ["b", "a"]);
    let pointers = std::thread::scope(|scope| {
        let a = scope.spawn(|| snapshot.family("a").map(|facts| facts.declaration() as *const Manifest as usize));
        let b = scope.spawn(|| snapshot.family("b").map(|facts| facts.declaration() as *const Manifest as usize));
        (a.join().unwrap(), b.join().unwrap())
    });
    assert_eq!(pointers.0?, pointers.1?, "aliases must share the complete decoded declaration");
    assert_eq!(snapshot.statistics().manifest_reads, 2);
    assert_eq!(snapshot.statistics().manifest_decodes, 1);
    assert_eq!(snapshot.statistics().shared_content_hits, 1);
    assert_eq!(snapshot.statistics().input_bytes, captured_bytes, "lazy decoding reread captured declarations");
    let retained = snapshot.statistics().retained_bytes;
    snapshot.validate_current()?;
    snapshot.family("a")?;
    assert_eq!(snapshot.statistics().retained_bytes, retained);
    assert_eq!(snapshot.statistics().manifest_decodes, 1);
    assert_eq!(snapshot.statistics().freshness_reads, 3);
    Ok(())
}

#[test]
fn discovery_retains_decodes_and_configuration_and_namespace_remain_bound() -> Result<()> {
    let root = checkout("[families]\n[profiles]\n")?;
    declaration(&root.0, "a")?;
    declaration(&root.0, "b")?;
    let snapshot = verifier().capture_catalog(CatalogRequest::new(&root.0)?)?;
    assert_eq!(snapshot.statistics().manifest_decodes, 1);
    assert_eq!(snapshot.catalog().profiles["offline-all"], ["a", "b"]);
    snapshot.family("a")?;
    snapshot.family("b")?;
    assert_eq!(snapshot.statistics().manifest_reads, 2);
    fs::write(root.0.join("tools/port/batch.toml"), "# same meaning, changed input\n[families]\n[profiles]\n")?;
    assert!(snapshot.validate_current().is_err());
    let snapshot = verifier().capture_catalog(CatalogRequest::new(&root.0)?)?;
    declaration(&root.0, "new")?;
    assert!(snapshot.validate_current().is_err());
    Ok(())
}

#[test]
fn captured_facts_stay_coherent_and_current_mutation_missing_or_redirected_inputs_refuse() -> Result<()> {
    let root = checkout("[families]\na='family'\nb='family'\n[profiles]\n")?;
    declaration(&root.0, "a")?;
    declaration(&root.0, "b")?;
    let snapshot = verifier().capture_catalog(CatalogRequest::new(&root.0)?)?;
    let path = root.0.join("tools/port/a.smf");
    let bytes = fs::read(&path)?;
    let captured_hash = snapshot.input_identities()[Path::new("tools/port/a.smf")].clone();
    fs::write(&path, b"changed declaration")?;
    assert_eq!(snapshot.family("a")?.manifest_sha256(), captured_hash);
    assert!(snapshot.validate_current().is_err());
    fs::write(&path, &bytes)?;
    let snapshot = verifier().capture_catalog(CatalogRequest::new(&root.0)?)?;
    fs::remove_file(&path)?;
    assert_eq!(snapshot.family("a")?.manifest_sha256(), captured_hash);
    assert!(snapshot.validate_current().is_err());
    std::os::unix::fs::symlink(root.0.join("tools/port/b.smf"), &path)?;
    assert_eq!(snapshot.family("a")?.manifest_sha256(), captured_hash);
    assert!(snapshot.validate_current().is_err());
    Ok(())
}

#[test]
fn capture_memory_io_and_deadline_bounds_refuse_through_the_interface() -> Result<()> {
    let root = checkout("[families]\na='family'\n[profiles]\n")?;
    declaration(&root.0, "a")?;
    let mut request = CatalogRequest::new(&root.0)?;
    request.deadline_unix_ms = 0;
    assert!(verifier().capture_catalog(request).err().unwrap().to_string().contains("deadline"));
    let mut request = CatalogRequest::new(&root.0)?;
    request.max_input_bytes = 1;
    assert!(verifier().capture_catalog(request).is_err());
    let mut request = CatalogRequest::new(&root.0)?;
    request.max_retained_bytes = 1;
    assert!(verifier().capture_catalog(request).err().unwrap().to_string().contains("memory"));
    let encoded_extent = verifier().capture_catalog(CatalogRequest::new(&root.0)?)?.statistics().retained_bytes;
    let mut request = CatalogRequest::new(&root.0)?;
    request.max_retained_bytes = encoded_extent;
    let snapshot = verifier().capture_catalog(request)?;
    assert!(snapshot.family("a").is_err());
    assert!(snapshot.check_budget().is_err());
    assert!(snapshot.statistics().retained_bytes <= encoded_extent);
    Ok(())
}

#[test]
fn batch_membership_refuses_omissions_duplicates_and_extra_bindings_without_runtime() -> Result<()> {
    let root = checkout("[families]\na='family'\nb='family'\n[profiles]\n")?;
    declaration(&root.0, "a")?;
    declaration(&root.0, "b")?;
    let offline = verifier();
    let snapshot = offline.capture_catalog(CatalogRequest::new(&root.0)?)?;
    let item = |name: &str| FamilyEvidence {
        family: name.into(),
        verdict: root.0.join("absent-verdict.toml"),
        manifest_sha256: "0".repeat(64),
        kind: "family".into(),
    };
    for (required, evidence) in [
        (vec!["a".into(), "b".into()], vec![item("a")]),
        (vec!["a".into()], vec![item("a"), item("a")]),
        (vec!["a".into()], vec![item("a"), item("b")]),
        (vec!["a".into(), "a".into()], vec![item("a")]),
    ] {
        assert!(
            offline
                .authenticate_batch(BatchAdmission {
                    catalog: &snapshot,
                    required: &required,
                    source: root.0.join("absent-source"),
                    evidence
                })
                .is_err()
        );
    }
    let published = Mutex::new(0);
    assert!(
        offline
            .verify_batch(
                BatchVerification {
                    catalog: &snapshot,
                    source: root.0.join("absent-source"),
                    root: root.0.join("capacity"),
                    workers: 4,
                    family_jobs: 2,
                    case_jobs: 1,
                    families: ["a", "a"]
                        .map(|name| FamilyJob { family: name.into(), output: root.0.join(name), previous: None })
                        .into(),
                    inputs: None,
                },
                |_| {
                    *published.lock().unwrap() += 1;
                    Ok(())
                }
            )
            .is_err()
    );
    assert_eq!(*published.lock().unwrap(), 0);
    assert_eq!(snapshot.statistics().manifest_decodes, 0);
    assert!(bounded::now_unix_ms()? > 0);
    Ok(())
}
