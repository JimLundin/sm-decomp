use evidence::*;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

struct Temp(PathBuf);
impl Temp {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "sm-evidence-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn path(&self) -> &Path {
        &self.0
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn independent_stores_import_with_remapped_ordinals_and_verified_inventory() -> anyhow::Result<()> {
    let temp = Temp::new();
    let source_path = temp.path().join("source-store");
    let dest_path = temp.path().join("dest-store");
    let input = temp.path().join("input");
    fs::create_dir(&input)?;
    fs::write(input.join("frame.bin"), vec![7u8; 65537])?;
    fs::write(input.join("hardware.log"), b"ppu 1 0 24 7\napu 1 2 3\n")?;
    let mut source = Store::create(&source_path)?;
    let tree = ingest_tree(&mut source, &input, &mut |_| {})?;
    let run = put_run(&mut source, tree, &serde_json::json!({"subject":"c-0"}))?;
    source.commit()?;
    let source = Store::open(&source_path)?;
    let mut dest = Store::create(&dest_path)?;
    // Different preceding objects force every physical compact ordinal to differ.
    put_image(&mut dest, &[99u8; 1000])?;
    dest.import_store(&source)?;
    dest.commit()?;
    let dest = Store::open(&dest_path)?;
    let (verified, inventory) = verified_inventory(&dest, &run)?;
    assert_eq!(verified.observations, tree);
    assert_eq!(inventory.len(), 2);
    assert_eq!(inventory["frame.bin"].bytes, 65537);
    verify_run(&dest, &run, Some(&input))?;
    let mut dest = Store::create(&dest_path)?;
    dest.import_store(&source)?;
    assert_eq!(dest.stats().new_objects, 0);
    dest.commit()?;
    Ok(())
}

#[test]
fn images_roundtrip_diff_and_survive_reopen() -> anyhow::Result<()> {
    let temp = Temp::new();
    let mut store = Store::create(temp.path())?;
    let bytes: Vec<u8> = (0..256 * 257 + 13).map(|i| (i % 251) as u8).collect();
    let left = put_image(&mut store, &bytes)?;
    let mut changed = bytes.clone();
    changed[257] ^= 1;
    changed[65536] ^= 1;
    let right = put_image(&mut store, &changed)?;
    let empty = put_image(&mut store, &[])?;
    assert_eq!(put_image(&mut store, &bytes)?, left);
    store.commit()?;
    let store = Store::open(temp.path())?;
    store.verify_packs()?;
    let mut out = Vec::new();
    write_image(&store, &left, &mut out)?;
    assert_eq!(out, bytes);
    out.clear();
    write_image(&store, &right, &mut out)?;
    assert_eq!(out, changed);
    out.clear();
    write_image(&store, &empty, &mut out)?;
    assert!(out.is_empty());
    let (offsets, reads) = diff_images(&store, &left, &right)?;
    assert_eq!(offsets, [256, 65536]);
    assert!(reads <= 12);
    assert_eq!(diff_images(&store, &left, &left)?, (vec![], 0));
    assert!(diff_images(&store, &left, &empty).is_err());
    Ok(())
}

#[test]
fn exact_stream_with_bursts_interleaving_unknown_reset_and_tail() -> anyhow::Result<()> {
    let temp = Temp::new();
    let mut store = Store::create(temp.path())?;
    let mut events = Vec::new();
    for _ in 0..2 {
        for i in 0..40000 {
            events.push(Event::Ppu {
                frame: 9,
                device: 0,
                register: if i % 2 == 0 { 24 } else { 25 },
                value: (i % 256) as u64,
            });
        }
    }
    events.push(Event::Apu { frame: 9, port: 0, value: 7 });
    events.push(Event::Ppu { frame: 0, device: 1, register: 4, value: 4 });
    let mut writer = TraceWriter::new(&mut store);
    for event in events {
        writer.push(event)?;
    }
    writer.literal(b"room-plm 0 47083 2056\nppu 000 0 4 4\n", Some(0))?;
    writer.push(Event::Apu { frame: 1, port: 2, value: 3 })?;
    writer.literal(b"interrupted", None)?;
    let id = writer.finish()?;
    let mut bytes = Vec::new();
    write_stream(&store, &id, &mut bytes)?;
    let restored = ingest_trace(&mut store, &mut Cursor::new(&bytes))?;
    // Different literal grouping is allowed; the reconstructed observation bytes agree.
    let mut again = Vec::new();
    write_stream(&store, &restored, &mut again)?;
    assert_eq!(again, bytes);
    assert_eq!(ingest_trace(&mut store, &mut Cursor::new(&bytes))?, restored);
    let stats = store.stats();
    assert!(stats.shared_objects > 0);
    store.commit()?;
    let store = Store::open(temp.path())?;
    let info = read_stream(&store, &id)?;
    assert_eq!(info.events, 80003);
    assert!(info.segments > 4);
    let mut frame = Vec::new();
    assert_eq!(write_frame(&store, &id, 9, &mut frame)?, 80001);
    assert!(frame.ends_with(b"apu 9 0 7\n"));
    frame.clear();
    write_frame(&store, &restored, 1, &mut frame)?;
    assert_eq!(frame, b"apu 1 2 3\n");
    let mut out = Vec::new();
    write_stream(&store, &id, &mut out)?;
    assert_eq!(out, bytes);
    Ok(())
}

#[test]
fn arbitrary_literal_lines_are_bounded_and_lossless() -> anyhow::Result<()> {
    let temp = Temp::new();
    let mut store = Store::create(temp.path())?;
    let mut bytes = vec![b'x'; 3 * 65536 + 1];
    bytes.extend_from_slice(b"\nppu 18446744073709551615 0 4 3\nppu 0 0 4 3\napu 1 0 2\n\xff\x00tail");
    let id = ingest_trace(&mut store, &mut Cursor::new(&bytes))?;
    let mut out = Vec::new();
    write_stream(&store, &id, &mut out)?;
    assert_eq!(out, bytes);
    assert_eq!(read_stream(&store, &id)?.events, 3);
    Ok(())
}

#[test]
fn sealed_directory_and_provenance_are_independent() -> anyhow::Result<()> {
    let temp = Temp::new();
    let source = temp.path().join("source");
    fs::create_dir(&source)?;
    fs::create_dir(source.join("frames"))?;
    fs::write(source.join("frames/one.bin"), vec![7u8; 65537])?;
    fs::write(source.join("hardware.log"), b"ppu 1 0 4 3\nunknown 1\napu 2 0 4\n")?;
    fs::write(source.join("large.dat"), vec![17u8; (3 << 20) + 1])?;
    let root = temp.path().join("store");
    let mut store = Store::create(&root)?;
    let tree = ingest_tree(&mut store, &source, &mut |_| {})?;
    let a = put_run(&mut store, tree, &serde_json::json!({"compiler":"a","nested":{"z":1,"a":2}}))?;
    let b = put_run(&mut store, tree, &serde_json::json!({"compiler":"b"}))?;
    assert_ne!(a, b);
    assert_eq!(read_run(&store, &a)?.observations, read_run(&store, &b)?.observations);
    store.commit()?;
    let reader = Store::open(&root)?;
    verify_run(&reader, &a, Some(&source))?;
    let mut writer = Store::create(&root)?;
    assert_eq!(ingest_tree(&mut writer, &source, &mut |_| {})?, tree);
    assert!(writer.commit()?.is_none());
    fs::write(source.join("extra"), b"x")?;
    assert!(verify_run(&reader, &a, Some(&source)).is_err());
    fs::remove_file(source.join("extra"))?;
    fs::write(source.join("hardware.log"), b"changed")?;
    assert!(verify_run(&reader, &a, Some(&source)).is_err());
    Ok(())
}

#[test]
fn writer_ownership_abort_and_append() -> anyhow::Result<()> {
    let temp = Temp::new();
    let mut writer = Store::create(temp.path())?;
    assert!(Store::create(temp.path()).is_err());
    drop(Store::open(temp.path())?);
    assert!(temp.path().join("writer.lock").exists());
    let id = put_image(&mut writer, b"first")?;
    writer.commit()?;
    let mut writer = Store::create(temp.path())?;
    put_image(&mut writer, b"aborted")?;
    drop(writer);
    assert!(!temp.path().join("writer.lock").exists());
    let mut writer = Store::create(temp.path())?;
    let other = put_image(&mut writer, b"second")?;
    writer.commit()?;
    let reader = Store::open(temp.path())?;
    assert_eq!(reader.stats().packs, 2);
    reader.verify_packs()?;
    for (id, bytes) in [(id, b"first".as_slice()), (other, b"second".as_slice())] {
        let mut out = Vec::new();
        write_image(&reader, &id, &mut out)?;
        assert_eq!(out, bytes);
    }
    let writer = Store::create(temp.path())?;
    writer.commit()?;
    assert!(!temp.path().join("writer.lock").exists());
    Ok(())
}

#[test]
fn corrupt_body_index_and_truncation_fail_closed() -> anyhow::Result<()> {
    let temp = Temp::new();
    let mut store = Store::create(temp.path())?;
    let id = put_image(&mut store, b"abcdefg")?;
    store.commit()?;
    let pack = fs::read_dir(temp.path())?.next().unwrap()?.path();
    let original = fs::read(&pack)?;
    let mut bad = original.clone();
    bad[9] ^= 1;
    fs::write(&pack, &bad)?;
    let reader = Store::open(temp.path())?;
    assert!(reader.verify_packs().is_err());
    assert!(write_image(&reader, &id, &mut Vec::new()).is_err());
    drop(reader);
    bad = original.clone();
    let n = bad.len();
    bad[n - 90] ^= 1;
    fs::write(&pack, bad)?;
    assert!(Store::open(temp.path()).is_err());
    fs::write(&pack, &original[..original.len() - 1])?;
    assert!(Store::open(temp.path()).is_err());
    fs::write(&pack, original)?;
    fs::remove_file(pack)?;
    let reader = Store::open(temp.path())?;
    assert!(write_image(&reader, &id, &mut Vec::new()).is_err());
    Ok(())
}

#[test]
fn malformed_objects_and_ids_return_errors_without_panics() -> anyhow::Result<()> {
    let temp = Temp::new();
    let mut store = Store::create(temp.path())?;
    for bytes in [vec![], vec![255; 100], vec![128, 0], vec![1, 0], vec![1, 255, 255, 255]] {
        let image = store.put(b'I', &bytes)?;
        let stream = store.put(b'S', &bytes)?;
        let tree = store.put(b'T', &bytes)?;
        assert!(write_image(&store, &image, &mut Vec::new()).is_err());
        assert!(read_stream(&store, &stream).is_err());
        if !bytes.is_empty() {
            assert!(read_tree(&store, &tree).is_err());
        }
    }
    assert!(parse_id(&"é".repeat(32)).is_err());
    assert!(parse_id(&"x".repeat(64)).is_err());
    Ok(())
}

#[test]
fn large_frame_directory_and_reordered_events() -> anyhow::Result<()> {
    let temp = Temp::new();
    let mut store = Store::create(temp.path())?;
    let mut writer = TraceWriter::new(&mut store);
    for frame in 0..2050 {
        writer.push(Event::Apu { frame, port: 0, value: frame })?;
    }
    let root = writer.finish()?;
    let mut text = Vec::new();
    write_stream(&store, &root, &mut text)?;
    assert_eq!(ingest_trace(&mut store, &mut Cursor::new(&text))?, root);
    let original = ingest_trace(&mut store, &mut Cursor::new(b"ppu 1 0 4 1\napu 1 0 2\n"))?;
    let reordered = ingest_trace(&mut store, &mut Cursor::new(b"apu 1 0 2\nppu 1 0 4 1\n"))?;
    let duplicated = ingest_trace(&mut store, &mut Cursor::new(b"ppu 1 0 4 1\nppu 1 0 4 1\napu 1 0 2\n"))?;
    assert_ne!(original, reordered);
    assert_ne!(original, duplicated);
    store.commit()?;
    let store = Store::open(temp.path())?;
    let mut frame = Vec::new();
    write_frame(&store, &root, 2049, &mut frame)?;
    assert_eq!(frame, b"apu 2049 0 2049\n");
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlinks_and_stale_locks_are_not_followed_or_stolen() -> anyhow::Result<()> {
    let temp = Temp::new();
    let real = temp.path().join("real");
    fs::create_dir(&real)?;
    let link = temp.path().join("link");
    std::os::unix::fs::symlink(&real, &link)?;
    assert!(Store::create(&link).is_err());
    assert!(!real.join("writer.lock").exists());
    let mut store = Store::create(&real)?;
    let source = temp.path().join("source");
    fs::create_dir(&source)?;
    std::os::unix::fs::symlink("missing", source.join("file"))?;
    assert!(ingest_tree(&mut store, &source, &mut |_| {}).is_err());
    drop(store);
    fs::write(real.join("writer.lock"), b"99999999\n")?;
    assert!(Store::create(&real).is_err());
    drop(Store::open(&real)?);
    assert!(real.join("writer.lock").exists());
    fs::write(real.join("junk.pending"), b"partial")?;
    assert_eq!(Store::open(&real)?.stats().packs, 0);
    Ok(())
}

#[test]
fn cli_ingest_extract_frame_and_verify() -> anyhow::Result<()> {
    use std::process::Command;
    let temp = Temp::new();
    let source = temp.path().join("source");
    fs::create_dir(&source)?;
    fs::write(source.join("hardware.log"), b"ppu 2 0 4 1\nunknown 2 x\napu 3 0 2\n")?;
    fs::write(source.join("frame.bin"), b"exact state")?;
    let root = temp.path().join("store");
    let run_cmd = |args: &[&str]| Command::new(env!("CARGO_BIN_EXE_sm-evidence")).args(args).output().unwrap();
    let store_path = root.to_str().unwrap();
    let source_path = source.to_str().unwrap();
    let ingest = run_cmd(&["ingest", store_path, source_path]);
    assert!(ingest.status.success());
    let roots: serde_json::Value = serde_json::from_slice(&ingest.stdout)?;
    let id = roots["run"].as_str().unwrap();
    assert!(run_cmd(&["verify", store_path, id, source_path]).status.success());
    assert_eq!(run_cmd(&["cat", store_path, id, "frame.bin"]).stdout, b"exact state");
    assert_eq!(run_cmd(&["frame", store_path, id, "hardware.log", "2"]).stdout, b"ppu 2 0 4 1\nunknown 2 x\n");
    let dest = temp.path().join("restored");
    let dest_path = dest.to_str().unwrap();
    assert!(run_cmd(&["extract", store_path, id, dest_path]).status.success());
    assert_eq!(fs::read(dest.join("hardware.log"))?, fs::read(source.join("hardware.log"))?);
    assert!(!run_cmd(&["extract", store_path, id, dest_path]).status.success());
    Ok(())
}
