use anyhow::{Result, bail};
use evidence::*;
use std::io::Write;
use std::path::Path;

const USAGE: &str = "usage:
  sm-evidence ingest <store> <dir>...        store each directory as a run tree, print its id
  sm-evidence verify <store> <tree-id> <dir> check every file re-materializes byte-identically
  sm-evidence cat <store> <kind> <id>        write an entry (blob|image|stream) to stdout
  sm-evidence ls <store> <tree-id>           list a tree
  sm-evidence diff-image <store> <id> <id>   list differing page offsets";

fn kind_name(kind: u8) -> &'static str {
    match kind {
        BLOB => "blob",
        NODE => "node",
        IMAGE => "image",
        SEGMENT => "segment",
        STREAM => "stream",
        TREE => "tree",
        _ => "?",
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = args.first() else { bail!(USAGE) };
    match (command.as_str(), &args[1..]) {
        ("ingest", [store, directories @ ..]) if !directories.is_empty() => {
            let mut store = Store::open(Path::new(store))?;
            for directory in directories {
                let started = std::time::Instant::now();
                let id = put_tree(&mut store, Path::new(directory), &mut |path| {
                    if path.metadata().map(|m| m.len() > 64 << 20).unwrap_or(false) {
                        eprintln!("  large file {}", path.display());
                    }
                })?;
                println!("{} {}  ({:.1}s)", hex(&id), directory, started.elapsed().as_secs_f64());
            }
            println!("{:<8} {:>10} {:>14} {:>10} {:>14}", "kind", "new", "new bytes", "shared", "shared bytes");
            let (mut new, mut shared) = (0, 0);
            for (kind, s) in &store.stats {
                println!("{:<8} {:>10} {:>14} {:>10} {:>14}", kind_name(*kind), s.new_objects, s.new_bytes, s.shared_objects, s.shared_bytes);
                new += s.new_bytes;
                shared += s.shared_bytes;
            }
            println!("{:<8} {:>10} {:>14} {:>10} {:>14}", "total", "", new, "", shared);
        }
        ("verify", [store, id, directory]) => {
            let store = Store::open(Path::new(store))?;
            let (files, bytes) = verify_tree(&store, &parse_id(id)?, Path::new(directory))?;
            println!("verified {files} files, {bytes} bytes");
        }
        ("cat", [store, kind, id]) => {
            let store = Store::open(Path::new(store))?;
            let kind = match kind.as_str() {
                "blob" => ENTRY_BLOB,
                "image" => ENTRY_IMAGE,
                "stream" => ENTRY_STREAM,
                _ => bail!(USAGE),
            };
            let stdout = std::io::stdout().lock();
            let mut out = std::io::BufWriter::with_capacity(1 << 20, stdout);
            write_entry(&store, kind, &parse_id(id)?, &mut out)?;
            out.flush()?;
        }
        ("ls", [store, id]) => {
            let store = Store::open(Path::new(store))?;
            for entry in read_tree(&store, &parse_id(id)?)? {
                println!("{} {} {}", entry.kind as char, hex(&entry.id), entry.name);
            }
        }
        ("diff-image", [store, left, right]) => {
            let store = Store::open(Path::new(store))?;
            let (offsets, reads) = diff_images(&store, &parse_id(left)?, &parse_id(right)?)?;
            println!("{} differing pages ({} node reads)", offsets.len(), reads);
            for offset in offsets {
                println!("{offset:#07x}");
            }
        }
        _ => bail!(USAGE),
    }
    Ok(())
}
