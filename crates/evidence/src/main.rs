use anyhow::{Context, Result, bail, ensure};
use evidence::*;
use serde::Serialize;
use std::fs::OpenOptions;
mod metrics;
fn emit<T: Serialize>(value: &T) -> Result<()> {
    print!("{}", toml::to_string_pretty(value)?);
    Ok(())
}
#[derive(Serialize)]
struct VerifyReceipt {
    files: u64,
    bytes: u64,
    seconds: f64,
}
#[derive(Serialize)]
struct StatsReceipt {
    objects: u64,
    packs: u64,
    stored_bytes: u64,
}
#[derive(Serialize)]
struct DifferenceReceipt {
    offsets: Vec<u64>,
    node_reads: u64,
}
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let start = Instant::now();
    let mut metrics = metrics::Metrics::start(args.first().map(String::as_str).unwrap_or("unknown"))?;
    match args.first().map(String::as_str) {
        Some("ingest") if (3..=4).contains(&args.len()) => {
            let mut store = Store::create(Path::new(&args[1]))?;
            let tree = ingest_tree(&mut store, Path::new(&args[2]), &mut |p| eprintln!("ingest {}", p.display()))?;
            let provenance = if let Some(path) = args.get(3) {
                toml::from_str(&std::fs::read_to_string(path)?)?
            } else {
                Provenance::default()
            };
            let run = put_run(&mut store, tree, &provenance)?;
            let stats = store.stats();
            let pack = store.commit()?;
            emit(&IngestReceipt {
                run: hex(&run),
                observations: hex(&tree),
                pack: pack.as_ref().map(hex),
                new_objects: stats.new_objects,
                shared_objects: stats.shared_objects,
                seconds: start.elapsed().as_secs_f64(),
            })?;
        }
        Some("capture") if args.len() == 6 => {
            // Pipeline holds its shared-store admission lock while recording and sealing.
            let mut store = Store::create(Path::new(&args[1]))?;
            println!("ready");
            std::io::stdout().flush()?;
            let stream = metrics.measure("trace-ingest", || ingest_trace(&mut store, &mut std::io::stdin().lock()))?;
            let info = read_stream(&store, &stream)?;
            ensure!(info.bytes > 0, "empty captured trace");
            metrics.measure("trace-commit", || store.commit())?;
            let temporary = TraceReceipt { stream: hex(&stream), bytes: info.bytes, sha256: hex(&info.legacy_sha256) };
            let mut out = OpenOptions::new().write(true).create_new(true).open(&args[2])?;
            out.write_all(toml::to_string(&temporary)?.as_bytes())?;
            out.sync_all()?;
            let mut store = Store::create(Path::new(&args[1]))?;
            let observations =
                metrics.measure("staged-tree-ingest", || ingest_tree(&mut store, Path::new(&args[3]), &mut |_| {}))?;
            let mut entries = read_tree(&store, &observations)?;
            entries.push(Entry { name: "hardware.log".into(), kind: EntryKind::Stream, id: stream });
            let tree = put_tree(&mut store, entries)?;
            let provenance = toml::from_str(&std::fs::read_to_string(&args[4])?)?;
            let run = put_run(&mut store, tree, &provenance)?;
            metrics.measure("final-commit", || store.commit())?;
            // Reopen sealed packs before allowing staging bytes to be released.
            let store = Store::open(Path::new(&args[1]))?;
            let (verified, inventory) =
                metrics.measure("reopened-verification", || verified_inventory(&store, &run))?;
            let artifacts = inventory.iter().map(|(name, info)| (name.clone(), hex(&info.sha256))).collect();
            let sizes = inventory.iter().map(|(name, info)| (name.clone(), info.bytes)).collect();
            let pointer = Pointer {
                format: "sm-evidence-v2".into(),
                run: hex(&run),
                observations: hex(&tree),
                files: verified.files,
                bytes: verified.bytes,
                artifacts,
                sizes,
                provenance: verified.provenance,
                store: None,
                pipeline_seconds: None,
                producer_seconds: None,
                seal_seconds: None,
                stored_added_bytes: None,
                worker_wait_seconds: None,
                store_wait_seconds: None,
                publication_seconds: None,
            };
            let mut out = OpenOptions::new().write(true).create_new(true).open(&args[5])?;
            out.write_all(toml::to_string(&pointer)?.as_bytes())?;
            out.sync_all()?;
        }
        Some("inventory") if args.len() == 3 => {
            let store = Store::open(Path::new(&args[1]))?;
            let (run, inventory) = verified_inventory(&store, &parse_id(&args[2])?)?;
            let artifacts = inventory.iter().map(|(name, info)| (name.clone(), hex(&info.sha256))).collect();
            emit(&Inventory {
                observations: hex(&run.observations),
                files: run.files,
                bytes: run.bytes,
                artifacts,
                provenance: run.provenance,
            })?;
        }
        Some("import") if args.len() == 4 => {
            let source = Store::open(Path::new(&args[1]))?;
            let id = parse_id(&args[3])?;
            let (original, inventory) =
                metrics.measure("import-source-verification", || verified_inventory(&source, &id))?;
            let mut destination = Store::create(Path::new(&args[2]))?;
            metrics.measure("shared-import", || destination.import_store(&source))?;
            metrics.measure("import-commit", || destination.commit())?;
            let destination = Store::open(Path::new(&args[2]))?;
            let (restored, actual) =
                metrics.measure("import-destination-verification", || verified_inventory(&destination, &id))?;
            ensure!(
                original.observations == restored.observations && original.provenance == restored.provenance,
                "imported run differs"
            );
            ensure!(
                inventory.len() == actual.len()
                    && inventory.iter().all(|(name, member)| actual
                        .get(name)
                        .is_some_and(|other| member.bytes == other.bytes && member.sha256 == other.sha256)),
                "imported inventory differs"
            );
            emit(&ImportReceipt {
                run: hex(&id),
                observations: hex(&restored.observations),
                files: restored.files,
                bytes: restored.bytes,
                seconds: start.elapsed().as_secs_f64(),
                provenance: restored.provenance,
                artifacts: actual.iter().map(|(name, member)| (name.clone(), hex(&member.sha256))).collect(),
            })?;
        }
        Some("verify") if (3..=4).contains(&args.len()) => {
            let store = Store::open(Path::new(&args[1]))?;
            store.verify_packs()?;
            let run = verify_run(&store, &parse_id(&args[2])?, args.get(3).map(Path::new))?;
            emit(&VerifyReceipt { files: run.files, bytes: run.bytes, seconds: start.elapsed().as_secs_f64() })?;
        }
        Some("stats") if args.len() == 2 => {
            let stats = Store::open(Path::new(&args[1]))?.stats();
            emit(&StatsReceipt { objects: stats.objects, packs: stats.packs, stored_bytes: stats.stored_bytes })?;
        }
        Some("cat") if args.len() == 4 => {
            let store = Store::open(Path::new(&args[1]))?;
            let run = read_run(&store, &parse_id(&args[2])?)?;
            let entry = find(&store, run.observations, &args[3])?;
            write_entry(&store, entry.kind, &entry.id, &mut BufWriter::new(std::io::stdout().lock()))?;
        }
        Some("frame") if args.len() == 5 => {
            let store = Store::open(Path::new(&args[1]))?;
            let run = read_run(&store, &parse_id(&args[2])?)?;
            let entry = find(&store, run.observations, &args[3])?;
            ensure!(entry.kind == EntryKind::Stream, "entry is not stream");
            write_frame(&store, &entry.id, args[4].parse()?, &mut BufWriter::new(std::io::stdout().lock()))?;
        }
        Some("extract") if args.len() == 4 => {
            let store = Store::open(Path::new(&args[1]))?;
            let run = verify_run(&store, &parse_id(&args[2])?, None)?;
            std::fs::create_dir(Path::new(&args[3]))?;
            extract(&store, &run.observations, Path::new(&args[3]), 0)?;
        }
        Some("diff-image") if args.len() == 4 => {
            let store = Store::open(Path::new(&args[1]))?;
            let (offsets, reads) = diff_images(&store, &parse_id(&args[2])?, &parse_id(&args[3])?)?;
            emit(&DifferenceReceipt { offsets, node_reads: reads })?;
        }
        _ => bail!(
            "usage: sm-evidence capture STORE TRACE_RECEIPT DIRECTORY PROVENANCE POINTER | inventory STORE RUN | import SOURCE_STORE DEST_STORE RUN | ingest STORE DIRECTORY [PROVENANCE.toml] | verify STORE RUN [DIRECTORY] | stats STORE | cat STORE RUN MEMBER | frame STORE RUN MEMBER FRAME | extract STORE RUN NEW_DIRECTORY | diff-image STORE LEFT RIGHT"
        ),
    }
    metrics.finish("success", None)?;
    Ok(())
}
fn find(store: &Store, mut tree: Id, path: &str) -> Result<Entry> {
    let components: Vec<_> = path.split('/').collect();
    ensure!(!components.is_empty(), "empty member path");
    for (i, name) in components.iter().enumerate() {
        let entry = read_tree(store, &tree)?.into_iter().find(|e| e.name == *name).context("missing member")?;
        if i + 1 == components.len() {
            return Ok(entry);
        }
        ensure!(entry.kind == EntryKind::Tree, "member is not directory");
        tree = entry.id;
    }
    bail!("empty member path")
}
fn extract(store: &Store, tree: &Id, path: &Path, depth: usize) -> Result<()> {
    ensure!(depth <= 64, "tree nesting exceeds limit");
    for entry in read_tree(store, tree)? {
        let dest = path.join(entry.name);
        if entry.kind == EntryKind::Tree {
            std::fs::create_dir(&dest)?;
            extract(store, &entry.id, &dest, depth + 1)?;
        } else {
            let mut out = BufWriter::new(OpenOptions::new().write(true).create_new(true).open(dest)?);
            write_entry(store, entry.kind, &entry.id, &mut out)?;
            out.flush()?;
        }
    }
    Ok(())
}
