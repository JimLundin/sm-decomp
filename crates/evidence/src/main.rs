use anyhow::{Context, Result, bail, ensure};
use evidence::*;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::time::Instant;

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let start = Instant::now();
    match args.first().map(String::as_str) {
        Some("ingest") if (3..=4).contains(&args.len()) => {
            let mut store = Store::create(Path::new(&args[1]))?;
            let tree = ingest_tree(&mut store, Path::new(&args[2]), &mut |p| eprintln!("ingest {}", p.display()))?;
            let provenance = if let Some(path) = args.get(3) {
                serde_json::from_reader(File::open(path)?)?
            } else {
                serde_json::json!({})
            };
            let run = put_run(&mut store, tree, &provenance)?;
            let stats = store.stats();
            let pack = store.commit()?;
            println!(
                "{}",
                serde_json::json!({"run":hex(&run), "observations":hex(&tree), "pack":pack.as_ref().map(hex), "new_objects":stats.new_objects, "shared_objects":stats.shared_objects, "seconds":start.elapsed().as_secs_f64()})
            );
        }
        Some("verify") if (3..=4).contains(&args.len()) => {
            let store = Store::open(Path::new(&args[1]))?;
            store.verify_packs()?;
            let run = verify_run(&store, &parse_id(&args[2])?, args.get(3).map(Path::new))?;
            println!(
                "{}",
                serde_json::json!({"files":run.files, "bytes":run.bytes, "seconds":start.elapsed().as_secs_f64()})
            );
        }
        Some("stats") if args.len() == 2 => {
            let stats = Store::open(Path::new(&args[1]))?.stats();
            println!(
                "{}",
                serde_json::json!({"objects":stats.objects,"packs":stats.packs,"stored_bytes":stats.stored_bytes})
            );
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
            println!("{}", serde_json::json!({"offsets":offsets,"node_reads":reads}));
        }
        _ => bail!(
            "usage: sm-evidence ingest STORE DIRECTORY [PROVENANCE.json] | verify STORE RUN [DIRECTORY] | stats STORE | cat STORE RUN MEMBER | frame STORE RUN MEMBER FRAME | extract STORE RUN NEW_DIRECTORY | diff-image STORE LEFT RIGHT"
        ),
    }
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
