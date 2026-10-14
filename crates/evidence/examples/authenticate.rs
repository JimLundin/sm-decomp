//! Bounded synthetic warm-store authentication probe. Build this same source
//! against both revisions and run them against the same prepared immutable data.
//! Timings include restoration and all logical authentication, not preparation
//! or store opening. They are not an end-to-end Live replay performance claim.
use anyhow::{Result, ensure};
use evidence::*;
use std::io::Cursor;
use std::path::Path;
use std::time::Instant;

const WORKLOADS: [&str; 3] = ["typed", "literal", "mixed"];

fn prepare(root: &Path) -> Result<()> {
    ensure!(!root.exists(), "preparation requires a new directory");
    std::fs::create_dir_all(root)?;
    for workload in WORKLOADS {
        let mut store = Store::create(&root.join(workload))?;
        let mut writer = TraceWriter::new(&mut store);
        if workload == "literal" {
            let block: Vec<u8> = (0..65536).map(|i| if i % 80 == 79 { b'\n' } else { b'x' }).collect();
            for _ in 0..256 {
                writer.literal(&block, None)?;
            }
        } else {
            let count = if workload == "typed" { 1_048_576 } else { 262_144 };
            for i in 0..count {
                writer.push(Event::Ppu {
                    frame: i / 65536,
                    device: 0,
                    register: if i % 2 == 0 { 24 } else { 25 },
                    value: i % 256,
                })?;
            }
            writer.literal(b"unknown 0 marker\napu 000 0 2\n\xff\x00tail", None)?;
        }
        let stream = writer.finish()?;
        let mut entries = vec![Entry { name: "hardware.log".into(), kind: EntryKind::Stream, id: stream }];
        if workload == "mixed" {
            let bytes: Vec<u8> = (0..4 * 1024 * 1024).map(|i| (i % 251) as u8).collect();
            entries.push(Entry {
                name: "capture.dat".into(),
                kind: EntryKind::File,
                id: put_file(&mut store, &mut Cursor::new(&bytes))?,
            });
            entries.push(Entry {
                name: "frame.bin".into(),
                kind: EntryKind::Image,
                id: put_image(&mut store, &bytes[..131073])?,
            });
        }
        let tree = put_tree(&mut store, entries)?;
        let run = put_run(&mut store, tree, &Provenance::default())?;
        store.commit()?;
        verify_run(&Store::open(&root.join(workload))?, &run, None)?;
        std::fs::write(root.join(format!("{workload}.run")), hex(&run))?;
    }
    Ok(())
}

fn measure(root: &Path, repetitions: usize) -> Result<()> {
    ensure!((1..=25).contains(&repetitions), "repetitions must be 1..=25");
    println!("format = \"evidence-authentication-probe-v1\"");
    println!("repetitions = {repetitions}");
    for workload in WORKLOADS {
        let store = Store::open(&root.join(workload))?;
        let run_id = parse_id(std::fs::read_to_string(root.join(format!("{workload}.run")))?.trim())?;
        // Authenticate physical seals and warm the bounded decoded-block cache
        // before measuring. Retain exact digests to check every measured result.
        store.verify_packs()?;
        let (run, expected) = verified_inventory(&store, &run_id)?;
        let mut inventory_seconds = Vec::new();
        let mut verification_seconds = Vec::new();
        for _ in 0..repetitions {
            let start = Instant::now();
            let (actual_run, actual) = verified_inventory(&store, &run_id)?;
            inventory_seconds.push(start.elapsed().as_secs_f64());
            ensure!(actual_run.files == run.files && actual_run.bytes == run.bytes, "run counts changed");
            ensure!(actual.len() == expected.len(), "member count changed");
            for (name, member) in &expected {
                let found = &actual[name];
                ensure!(found.bytes == member.bytes && found.sha256 == member.sha256, "member changed");
            }
            let start = Instant::now();
            let counts = verify_tree(&store, &run.observations, None)?;
            verification_seconds.push(start.elapsed().as_secs_f64());
            ensure!(counts == (run.files, run.bytes), "tree counts changed");
        }
        println!("\n[[workloads]]\nname = \"{workload}\"");
        println!("run = \"{}\"\nobservations = \"{}\"", hex(&run_id), hex(&run.observations));
        println!("files = {}\nbytes = {}", run.files, run.bytes);
        println!("inventory_seconds = {inventory_seconds:?}");
        println!("verification_seconds = {verification_seconds:?}");
        for (name, member) in &expected {
            println!("[[workloads.members]]\nname = \"{name}\"");
            println!("bytes = {}\nsha256 = \"{}\"", member.bytes, hex(&member.sha256));
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [command, root] if command == "prepare" => prepare(Path::new(root)),
        [command, root, repetitions] if command == "measure" => measure(Path::new(root), repetitions.parse()?),
        _ => anyhow::bail!("usage: authenticate prepare NEW_ROOT | authenticate measure ROOT REPETITIONS"),
    }
}
