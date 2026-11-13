use std::cell::Cell;
use std::convert::Infallible;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

use canonical::Canonical;

use super::*;

/// An invented record set.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct Notes(Vec<u16>);

impl Cached for Notes {
    const NAME: &'static str = "notes";
    const REVISION: Revision = Revision(3);
}

/// The same set and name written by a later importer.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct LaterNotes(Vec<u16>);

impl Cached for LaterNotes {
    const NAME: &'static str = "notes";
    const REVISION: Revision = Revision(4);
}

/// The same name in another shape of the format.
#[derive(Canonical, Clone, Debug, PartialEq, Eq)]
struct WideNotes(Vec<u32>);

impl Cached for WideNotes {
    const NAME: &'static str = "notes";
    const REVISION: Revision = Revision(3);
}

/// A directory of this test's own, removed when the test ends.
struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "sm-content-cache-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        Self(dir)
    }

    fn cache(&self) -> Cache {
        Cache::new(self.0.join("cache"))
    }

    /// The one file the cache holds.
    fn file(&self) -> PathBuf {
        let mut files = fs::read_dir(self.0.join("cache")).unwrap();
        let file = files.next().unwrap().unwrap().path();
        assert!(files.next().is_none(), "one set of one ROM is one file");
        file
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const IMAGE: &[u8] = b"an invented image";

fn notes() -> Notes {
    Notes(vec![7, 0x1234, 9])
}

/// Loads `Notes`, counting the builds.
fn load(cache: &Cache, image: &[u8], builds: &Cell<u32>) -> Loaded<Notes> {
    cache
        .load(image, || {
            builds.update(|count| count.saturating_add(1));
            Ok::<_, Infallible>(notes())
        })
        .unwrap()
}

fn built(miss: Miss) -> Source {
    Source::Built {
        miss,
        kept: Kept::Written,
    }
}

#[test]
fn a_set_is_built_once_then_read_with_the_same_records_and_fingerprint() {
    let scratch = Scratch::new();
    let builds = Cell::new(0);
    let first = load(&scratch.cache(), IMAGE, &builds);
    assert_eq!(first.source, built(Miss::Absent));
    assert_eq!(first.content, notes());
    assert_eq!(first.fingerprint, ContentFingerprint::of(&notes()));
    let second = load(&scratch.cache(), IMAGE, &builds);
    assert_eq!(second.source, Source::Cache);
    assert_eq!(
        (second.content, second.fingerprint),
        (first.content, first.fingerprint)
    );
    assert_eq!(builds.get(), 1);
}

#[test]
fn another_rom_has_its_own_file_and_doesnt_read_the_first() {
    let scratch = Scratch::new();
    let builds = Cell::new(0);
    load(&scratch.cache(), IMAGE, &builds);
    let other = load(&scratch.cache(), b"another image", &builds);
    assert_eq!(other.source, built(Miss::Absent));
    assert_eq!(load(&scratch.cache(), IMAGE, &builds).source, Source::Cache);
    assert_eq!(builds.get(), 2);
}

#[test]
fn a_file_written_for_another_rom_is_not_used_under_this_roms_name() {
    let scratch = Scratch::new();
    let builds = Cell::new(0);
    load(&scratch.cache(), b"another image", &builds);
    let others = scratch.file();
    let bytes = fs::read(&others).unwrap();
    fs::remove_file(&others).unwrap();
    load(&scratch.cache(), IMAGE, &builds);
    fs::write(scratch.file(), bytes).unwrap();
    assert_eq!(
        load(&scratch.cache(), IMAGE, &builds).source,
        built(Miss::OtherKey)
    );
    assert_eq!(load(&scratch.cache(), IMAGE, &builds).source, Source::Cache);
}

#[test]
fn a_later_importer_or_another_shape_rebuilds_and_replaces_the_file() {
    let scratch = Scratch::new();
    let builds = Cell::new(0);
    load(&scratch.cache(), IMAGE, &builds);
    let later = scratch
        .cache()
        .load(IMAGE, || Ok::<_, Infallible>(LaterNotes(vec![1])))
        .unwrap();
    assert_eq!(later.source, built(Miss::OtherKey));
    let wide = scratch
        .cache()
        .load(IMAGE, || Ok::<_, Infallible>(WideNotes(vec![1])))
        .unwrap();
    assert_eq!(wide.source, built(Miss::OtherKey));
    // The file is the last writer's, so the first set misses again.
    assert_eq!(
        load(&scratch.cache(), IMAGE, &builds).source,
        built(Miss::OtherKey)
    );
}

#[test]
fn a_file_with_a_changed_or_missing_byte_is_damaged_and_rebuilt() {
    let scratch = Scratch::new();
    let builds = Cell::new(0);
    load(&scratch.cache(), IMAGE, &builds);
    let file = scratch.file();
    let whole = fs::read(&file).unwrap();

    let mut changed = whole.clone();
    *changed.last_mut().unwrap() ^= 1;
    fs::write(&file, changed).unwrap();
    assert_eq!(
        load(&scratch.cache(), IMAGE, &builds).source,
        built(Miss::Damaged)
    );

    for length in [whole.len() - 1, Key::LEN + 3, Key::LEN - 1, 0] {
        fs::write(&file, &whole[..length]).unwrap();
        let loaded = load(&scratch.cache(), IMAGE, &builds);
        assert_eq!(loaded.source, built(Miss::Damaged), "cut to {length} bytes");
        assert_eq!(loaded.content, notes());
    }
    assert_eq!(load(&scratch.cache(), IMAGE, &builds).source, Source::Cache);
}

#[test]
fn records_that_carry_their_fingerprint_but_dont_decode_are_damaged() {
    let scratch = Scratch::new();
    let builds = Cell::new(0);
    load(&scratch.cache(), IMAGE, &builds);
    let file = scratch.file();
    let mut bytes = fs::read(&file).unwrap();
    // A length of three notes with only one behind it, under its own fingerprint.
    let encoding = [3, 0, 0, 0, 7, 0];
    bytes.truncate(Key::LEN);
    ContentFingerprint::of_encoding::<Notes>(&encoding).encode(&mut bytes);
    bytes.extend_from_slice(&encoding);
    fs::write(&file, bytes).unwrap();
    assert_eq!(
        load(&scratch.cache(), IMAGE, &builds).source,
        built(Miss::Damaged)
    );
}

#[test]
fn a_cache_that_cant_be_written_still_loads_and_builds_each_time() {
    let scratch = Scratch::new();
    fs::create_dir_all(&scratch.0).unwrap();
    // A file stands where the cache's directory would be.
    fs::write(scratch.0.join("cache"), b"").unwrap();
    let builds = Cell::new(0);
    for _ in 0..2 {
        let loaded = load(&scratch.cache(), IMAGE, &builds);
        assert_eq!(loaded.content, notes());
        assert!(matches!(
            loaded.source,
            Source::Built {
                kept: Kept::NotWritten(_),
                ..
            }
        ));
    }
    assert_eq!(builds.get(), 2);
}

#[test]
fn an_image_that_doesnt_import_is_the_importers_error_and_keeps_nothing() {
    let scratch = Scratch::new();
    let refused = scratch.cache().load::<Notes, _>(IMAGE, || Err("refused"));
    assert_eq!(refused, Err("refused"));
    assert!(!scratch.0.join("cache").exists());
    let builds = Cell::new(0);
    assert_eq!(
        load(&scratch.cache(), IMAGE, &builds).source,
        built(Miss::Absent)
    );
}

#[test]
fn no_part_file_is_left_beside_a_kept_set() {
    let scratch = Scratch::new();
    let builds = Cell::new(0);
    load(&scratch.cache(), IMAGE, &builds);
    load(&scratch.cache(), b"another image", &builds);
    let names: Vec<_> = fs::read_dir(scratch.0.join("cache"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(names.len(), 2);
    assert!(
        names
            .iter()
            .all(|name| name.starts_with("notes-") && name.ends_with(".content"))
    );
}
