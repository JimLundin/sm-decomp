//! Fault injection through the study CLI; no production-format mutation.
use super::*;

#[derive(Serialize)]
struct Control {
    name: String,
    path: PathBuf,
    publication_sha256: String,
    result: String,
    stderr: String,
}

fn rejected(root: &Path, seal: &str, output: &Path, publication_sha: &str, name: &str) -> Result<Control> {
    let result = Command::new("timeout")
        .args(["--signal=TERM", "--kill-after=5", "60"])
        .arg(std::env::current_exe()?)
        .arg("verify")
        .arg(root)
        .arg(seal)
        .arg(output)
        .arg(publication_sha)
        .output()?;
    fs::write(output.join(format!("{name}.stdout")), &result.stdout)?;
    fs::write(output.join(format!("{name}.stderr")), &result.stderr)?;
    // A timeout, signal, or panic is not a clean authenticated refusal.
    ensure!(result.status.code() == Some(1), "control {name} did not fail with an ordinary error: {}", result.status);
    let stderr = String::from_utf8(result.stderr)?;
    ensure!(stderr.starts_with("Error:"), "control did not return an Interface error");
    Ok(Control {
        name: name.into(),
        path: output.into(),
        publication_sha256: publication_sha.into(),
        result: "refused".into(),
        stderr,
    })
}

fn replace_publication(root: &Path, publication: &Publication) -> Result<String> {
    fs::remove_file(root.join("publication.toml"))?;
    publish(&root.join("publication.toml"), publication)
}

fn replace_file(path: &Path, bytes: &[u8]) -> Result<()> {
    // Detach this logical member from its study-local hard-link peers.
    let temporary = path.with_extension("control-replacement");
    fs::write(&temporary, bytes)?;
    fs::set_permissions(&temporary, fs::metadata(path)?.permissions())?;
    fs::rename(temporary, path)?;
    Ok(())
}

fn varint(bytes: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        bytes.push(value as u8 | 128);
        value >>= 7;
    }
    bytes.push(value as u8);
}

// Deliberately malformed canonical objects enter through Store's generic
// content Interface, then must be refused by a fresh logical reader. The trusted
// original input seal never changes to make a malformed candidate agree.
fn malformed_store(root: &Path, original: &Publication, control: &str) -> Result<String> {
    let mut store = Store::create(&root.join("store"))?;
    let original_id = parse_id(original.run.as_deref().context("missing original run")?)?;
    let run = evidence::read_run(&store, &original_id)?;
    let observations = match control {
        "extra-logical-member" => {
            let mut entries = evidence::read_tree(&store, &run.observations)?;
            let id = evidence::put_file(&mut store, &mut b"unexpected producer output".as_slice())?;
            entries.push(Entry { name: "unexpected.bin".into(), kind: EntryKind::File, id });
            evidence::put_tree(&mut store, entries)?
        }
        "duplicate-logical-member" => {
            let entry = evidence::read_tree(&store, &run.observations)?.remove(0);
            let mut bytes = Vec::new();
            for _ in 0..2 {
                varint(&mut bytes, entry.name.len() as u64);
                bytes.extend_from_slice(entry.name.as_bytes());
                bytes.push(entry.kind as u8);
                bytes.extend_from_slice(&entry.id);
            }
            store.put(b'T', &bytes)?
        }
        "missing-referenced-chunk" => {
            let mut leaves = BTreeMap::new();
            collect_leaves(&store, &run.observations, "", &mut leaves)?;
            let leaf = leaves.values_mut().find(|l| l.path.ends_with("before.wram")).context("missing WRAM")?;
            ensure!(leaf.kind == EntryKind::File, "chunk control requires raw representation");
            let (kind, mut bytes) = store.get(&leaf.id)?;
            ensure!(kind == b'F' && bytes.len() >= 34, "expected nonempty file object");
            // The final 32 bytes are a chunk id in the documented F format.
            let offset = bytes.len() - 32;
            bytes[offset..].fill(0);
            leaf.id = store.put(b'F', &bytes)?;
            tree(&mut store, &leaves.into_values().collect::<Vec<_>>())?
        }
        _ => bail!("unknown malformed-object control"),
    };
    let (kind, mut bytes) = store.get(&original_id)?;
    ensure!(kind == b'R' && bytes[0] == 2, "unexpected run schema");
    bytes[1..33].copy_from_slice(&observations);
    let candidate = store.put(b'R', &bytes)?;
    store.commit()?;
    let mut publication = original.clone();
    publication.run = Some(hex(&candidate));
    publication.packs = packs(&root.join("store"))?;
    replace_publication(root, &publication)
}

pub(super) fn crash_writer(root: &Path, input_sha: &str) -> Result<()> {
    let publication: Publication = toml::from_str(&fs::read_to_string(root.join("publication.toml"))?)?;
    let mut store = Store::create(&root.join("store"))?;
    let mut entries = Vec::new();
    for i in 0..3 {
        let bytes = vec![i as u8; 1 << 20];
        entries.push(Entry {
            name: format!("uncommitted-{i}"),
            kind: EntryKind::File,
            id: evidence::put_file(&mut store, &mut bytes.as_slice())?,
        });
    }
    let observations = evidence::put_tree(&mut store, entries)?;
    let run = evidence::put_run(&mut store, observations, &provenance(input_sha))?;
    let candidate = Publication { run: Some(hex(&run)), ..publication };
    // Intentionally violate the real writer's commit-before-publication order,
    // then exit without Drop. Pending bytes must never become admitted output.
    replace_publication(root, &candidate)?;
    std::process::exit(23);
}

pub(super) fn controls(root: &Path, seal: &str, matrix_sha: &str) -> Result<()> {
    let input = input(root, seal)?;
    let matrix: Matrix = read_document(&root.join("matrix.toml"), matrix_sha)?;
    ensure!(matrix.input_sha256 == seal, "control matrix inputs differ");
    let directory = matrix.samples.iter().find(|s| s.mode == Mode::Directory).context("missing directory sample")?;
    let packed = matrix.samples.iter().find(|s| s.mode == Mode::Raw1m).context("missing raw sample")?;
    let parent = root.join("controls");
    ensure!(!parent.exists(), "controls already exist");
    fs::create_dir(&parent)?;
    let mut results = Vec::new();
    for (name, source) in [
        ("corrupt-member", directory),
        ("missing-member", directory),
        ("extra-member", directory),
        ("equal-wrong-outputs", directory),
        ("missing-producer-record", directory),
        ("truncated-member", directory),
        ("corrupt-pack", packed),
        ("missing-pack", packed),
        ("truncated-pack", packed),
        ("extra-private-pack-member", packed),
        ("extra-logical-member", packed),
        ("duplicate-logical-member", packed),
        ("missing-referenced-chunk", packed),
        ("interrupted-before-pointer-rename", packed),
        ("interrupted-before-store-commit", packed),
    ] {
        let output = parent.join(name);
        capture(&source.path, &output, &inventory(&source.path)?)?;
        let original: Publication = read_document(&output.join("publication.toml"), &source.publication_sha256)?;
        let mut publication_sha = source.publication_sha256.clone();
        let wram = input.members.iter().find(|m| m.path.ends_with("before.wram")).context("missing input WRAM")?;
        let target = output.join("staging").join(&wram.path);
        match name {
            "corrupt-member" => {
                let mut bytes = bounded_bytes(&target, MAX_FILE)?;
                bytes[0] ^= 1;
                replace_file(&target, &bytes)?;
            }
            "missing-member" => fs::remove_file(target)?,
            "extra-member" => fs::write(output.join("staging/unexpected.bin"), b"unexpected")?,
            "equal-wrong-outputs" => {
                let case = Path::new(&wram.path).parent().unwrap();
                let c = output.join("staging").join(case).join("c-after.wram");
                let rust = output.join("staging").join(case).join("rust-after.wram");
                let mut bytes = bounded_bytes(&c, MAX_FILE)?;
                bytes[0] ^= 1;
                replace_file(&c, &bytes)?;
                replace_file(&rust, &bytes)?;
            }
            "missing-producer-record" => {
                let member = input
                    .members
                    .iter()
                    .find(|m| m.path.ends_with("c.fork.toml"))
                    .context("missing producer record")?;
                fs::remove_file(output.join("staging").join(&member.path))?;
            }
            "truncated-member" => {
                let bytes = bounded_bytes(&target, MAX_FILE)?;
                replace_file(&target, &bytes[..bytes.len() - 1])?;
            }
            "corrupt-pack" | "truncated-pack" | "missing-pack" => {
                let path = output.join("store").join(&original.packs[0]);
                if name == "missing-pack" {
                    fs::remove_file(path)?;
                } else if name == "truncated-pack" {
                    let length = fs::metadata(&path)?.len();
                    OpenOptions::new().write(true).open(path)?.set_len(length / 2)?;
                } else {
                    let mut bytes = bounded_bytes(&path, MAX_FILE)?;
                    bytes[8] ^= 1;
                    replace_file(&path, &bytes)?;
                }
            }
            "extra-private-pack-member" => fs::write(output.join("store/unclaimed.pending"), b"incomplete")?,
            "extra-logical-member" | "duplicate-logical-member" | "missing-referenced-chunk" => {
                publication_sha = malformed_store(&output, &original, name)?;
            }
            "interrupted-before-pointer-rename" => {
                fs::rename(output.join("publication.toml"), output.join("publication.pending"))?
            }
            "interrupted-before-store-commit" => {
                let status = Command::new(std::env::current_exe()?)
                    .arg("crash-writer")
                    .arg(&output)
                    .arg(seal)
                    .stdout(Stdio::null())
                    .stderr(Stdio::inherit())
                    .status()?;
                ensure!(status.code() == Some(23), "crash writer did not reach declared interruption");
                publication_sha = file_hash(&output.join("publication.toml"))?;
            }
            _ => unreachable!(),
        }
        results.push(rejected(root, seal, &output, &publication_sha, name)?);
        if name == "interrupted-before-store-commit" {
            // The actual child has exited and been reaped. Remove only its
            // identified unsealed pending file and writer lock, keeping logs.
            let mut removed = BTreeMap::new();
            for entry in fs::read_dir(output.join("store"))? {
                let path = entry?.path();
                if path.extension().is_some_and(|x| x == "pending")
                    || path.file_name().is_some_and(|x| x == "writer.lock")
                {
                    removed
                        .insert(path.file_name().unwrap().to_string_lossy().into_owned(), fs::metadata(&path)?.len());
                    fs::remove_file(path)?;
                }
            }
            fs::write(output.join("interruption-cleanup.toml"), toml::to_string_pretty(&removed)?)?;
            results.push(rejected(root, seal, &output, &publication_sha, "uncommitted-root-after-pending-cleanup")?);
            verify_publication(&output, &original, &input, seal)?;
            fs::write(
                output.join("previous-published-closure-survives.txt"),
                "complete original closure authenticated after interrupted writer cleanup\n",
            )?;
        }
    }
    cleanup(&input.cleanup_reader.path, &parent, false)?;
    cleanup(&input.cleanup_reader.path, &parent, true)?;
    #[derive(Serialize)]
    struct Report {
        format: &'static str,
        input_sha256: String,
        controls: Vec<Control>,
    }
    let report = Report { format: FORMAT, input_sha256: seal.into(), controls: results };
    println!(
        "controls_sha256 = {:?}\nrefusals = {}",
        publish(&root.join("controls.toml"), &report)?,
        report.controls.len()
    );
    Ok(())
}
