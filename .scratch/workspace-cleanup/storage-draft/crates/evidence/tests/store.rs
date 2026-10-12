use evidence::*;
use std::fs;
use std::path::PathBuf;

fn scratch(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("evidence-test-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}

fn trace() -> Vec<u8> {
    let mut text = String::new();
    for frame in [0u64, 0, 1, 63, 64, 65, 200] {
        text += &format!("ppu {frame} 0 21 128\n");
        for i in 0..40u64 {
            text += &format!("ppu {frame} 0 {} {}\n", 24 + i % 2, (i * 7) % 256);
        }
        text += &format!("apu {frame} 8512 0\n");
        // Short data-port run stays scalar.
        text += &format!("ppu {frame} 0 34 1\nppu {frame} 0 34 2\n");
    }
    text.into_bytes()
}

#[test]
fn run_tree_round_trips_and_shares_identical_content() {
    let root = scratch("roundtrip");
    let run = root.join("run");
    fs::create_dir_all(run.join("fixtures")).unwrap();
    fs::write(run.join("hardware.log"), trace()).unwrap();
    let mut wram = vec![0u8; 0x20000];
    wram[0x1234] = 7;
    fs::write(run.join("fixtures/a-before.wram"), &wram).unwrap();
    wram[0x1f000] = 9;
    fs::write(run.join("fixtures/a-after.wram"), &wram).unwrap();
    fs::write(run.join("fixtures/odd.rom"), vec![3u8; 1000]).unwrap();
    fs::write(run.join("fixtures/empty.bin"), b"").unwrap();
    fs::write(run.join("verdict.json"), b"{}\n").unwrap();
    // A file named like a trace that is not one falls back to a blob.
    fs::write(run.join("fixtures/hardware.log"), b"not a trace").unwrap();

    let mut store = Store::open(&root.join("store")).unwrap();
    let id = put_tree(&mut store, &run, &mut |_| {}).unwrap();
    assert_eq!(verify_tree(&store, &id, &run).unwrap().0, 7);

    // Same content again, anywhere: same id and nothing new written.
    let before: u64 = store.stats.values().map(|s| s.new_objects).sum();
    let again = put_tree(&mut store, &run, &mut |_| {}).unwrap();
    assert_eq!(id, again);
    assert_eq!(store.stats.values().map(|s| s.new_objects).sum::<u64>(), before);

    let entries = read_tree(&store, &id).unwrap();
    let kinds: Vec<_> = entries.iter().map(|e| (e.name.as_str(), e.kind)).collect();
    assert_eq!(kinds, [("fixtures", ENTRY_TREE), ("hardware.log", ENTRY_STREAM), ("verdict.json", ENTRY_BLOB)]);
    let fixtures = read_tree(&store, &entries[0].id).unwrap();
    let image = |name: &str| fixtures.iter().find(|e| e.name == name).unwrap().id;
    let (offsets, reads) = diff_images(&store, &image("a-before.wram"), &image("a-after.wram")).unwrap();
    assert_eq!(offsets, [0x1f000]);
    // 512 pages, fanout 16: height 3, so one path of 3 node pairs.
    assert_eq!(reads, 6);
    assert_eq!(fixtures.iter().find(|e| e.name == "hardware.log").unwrap().kind, ENTRY_BLOB);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn bursts_become_shared_payload_blobs() {
    let root = scratch("bursts");
    fs::write(root.join("hardware.log"), trace()).unwrap();
    let mut store = Store::open(&root.join("store")).unwrap();
    put_stream(&mut store, &root.join("hardware.log")).unwrap();
    let blobs = &store.stats[&BLOB];
    // Seven identical 40-write bursts: one payload stored, six shared.
    assert_eq!((blobs.new_objects, blobs.shared_objects), (1, 6));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn non_canonical_first_line_is_not_a_stream() {
    let root = scratch("canonical");
    let mut store = Store::open(&root.join("store")).unwrap();
    for bad in ["ppu 01 0 1 2\n", "ppu 1  0 1 2\n", "ppu 1 0 1 2", "xyz 1 2\n"] {
        fs::write(root.join("log"), bad).unwrap();
        assert!(put_stream(&mut store, &root.join("log")).is_err(), "{bad:?}");
    }
    // An empty log is a valid stream with no events.
    fs::write(root.join("log"), "").unwrap();
    let id = put_stream(&mut store, &root.join("log")).unwrap();
    let mut out = Vec::new();
    write_entry(&store, ENTRY_STREAM, &id, &mut out).unwrap();
    assert!(out.is_empty());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn non_canonical_remainder_is_kept_verbatim_as_a_tail() {
    let root = scratch("tail");
    let mut store = Store::open(&root.join("store")).unwrap();
    let prefix = String::from_utf8(trace()).unwrap();
    // Interrupted run, backwards frame, non-canonical number, garbage.
    for tail in ["ppu 300 0 34 1", "ppu 2 0 1 2\nppu 300 0 1 2\n", "ppu 300 0 01 2\n", "\u{0}\u{ff}junk"] {
        let text = prefix.clone() + tail;
        fs::write(root.join("log"), &text).unwrap();
        let id = put_stream(&mut store, &root.join("log")).unwrap();
        let mut out = Vec::new();
        write_entry(&store, ENTRY_STREAM, &id, &mut out).unwrap();
        assert_eq!(out, text.as_bytes(), "{tail:?}");
    }
    fs::remove_dir_all(root).unwrap();
}
