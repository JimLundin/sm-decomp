//! The reader against the fixture (`format2.toml`) and the strict rejections.

use super::*;

const FIXTURE: &str = include_str!("format2.toml");

fn fixture() -> Lock {
    Lock::parse(FIXTURE).expect("the fixture parses")
}

fn error(text: &str) -> ErrorKind {
    Lock::parse(text).expect_err("the text is rejected").kind
}

#[test]
fn the_fixture_reads_as_written() {
    let lock = fixture();
    assert_eq!(lock.header.oracle.emulator, "synthetic");
    assert_eq!(lock.header.oracle.version, "test 1");
    assert_eq!(
        lock.header.oracle.server.to_string(),
        "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
    );
    assert_eq!(lock.header.oracle.fixes, ["fix-a"]);
    assert_eq!(lock.header.bundles, []);

    let [alpha, beta] = lock.rows.as_slice() else {
        panic!("two rows");
    };
    assert_eq!(alpha.id, RowId("row-alpha".to_owned()));
    assert_eq!(
        alpha.start,
        Start::Seed {
            capture: "capture-a".to_owned(),
            index: 3
        }
    );
    assert_eq!(alpha.seed_record.map(|sha| sha.0), Some([0x11; 32]));
    assert_eq!(alpha.seed_words.map(|sha| sha.0), Some([0x22; 32]));
    assert_eq!(alpha.pads.0, [0x33; 32]);
    assert_eq!(alpha.frames, FrameCount(1500));

    assert_eq!(beta.start, Start::PowerOn);
    assert_eq!(beta.seed_record, None);
    assert_eq!(beta.seed_words, None);
    assert_eq!(beta.frames, FrameCount(1000));
}

#[test]
fn the_header_reads_its_bundles_and_comments() {
    let text = FIXTURE.replacen(
        "format = 2\n",
        "format = 2 # a comment\nbundles = [{ name = \"b.tar.gz.enc\", sha256 = \"5555555555555555555555555555555555555555555555555555555555555555\" }]\n",
        1,
    );
    let lock = Lock::parse(&text).expect("parses");
    assert_eq!(
        lock.header.bundles,
        [Bundle {
            name: "b.tar.gz.enc".to_owned(),
            sha256: Sha256([0x55; 32])
        }]
    );
}

#[test]
fn the_reader_is_strict() {
    let cases: [(&str, &str, ErrorKind); 8] = [
        ("format = 2", "format = 3", ErrorKind::Format(3)),
        (
            "frames = 1500\n",
            "frames = 1500\nticks = 1490\n",
            ErrorKind::UnknownKey("ticks".to_owned()),
        ),
        (
            "frames = 1500\n",
            "frames = 1500\nframes = 1500\n",
            ErrorKind::DuplicateKey("frames".to_owned()),
        ),
        (
            "fixes = [\"fix-a\"] }",
            "fixes = [\"fix-a\"], patches = [] }",
            ErrorKind::UnknownKey("patches".to_owned()),
        ),
        (
            "pads = \"3333",
            "pads = \"333X",
            ErrorKind::BadSha256(format!("333X{}", "3".repeat(60))),
        ),
        ("frames = 1500\n", "", ErrorKind::MissingKey("frames")),
        (
            "start = \"capture-a:3\"",
            "start = \"capture-a\"",
            ErrorKind::BadStart("capture-a".to_owned()),
        ),
        (
            "id = \"row-beta\"",
            "id = \"row-alpha\"",
            ErrorKind::DuplicateRow(RowId("row-alpha".to_owned())),
        ),
    ];
    for (from, to, expected) in cases {
        assert!(FIXTURE.contains(from), "{from}");
        assert_eq!(error(&FIXTURE.replacen(from, to, 1)), expected, "{to}");
    }
    assert_eq!(
        error(&FIXTURE.replacen("[[row]]", "[[scenario]]", 1)),
        ErrorKind::UnknownSection("scenario".to_owned())
    );
    assert!(matches!(
        error(&FIXTURE.replacen("start = \"power-on\"", "start = \"power\\on\"", 1)),
        ErrorKind::Syntax(_)
    ));
}

/// The committed lock is what replay reads, so a reader change it doesn't follow fails
/// here, not at the first replay case.
#[test]
fn the_committed_lsnes_lock_reads() {
    let lock = Lock::parse(include_str!("../../../../scenarios/locks/lsnes.toml"))
        .expect("the committed lock parses");
    assert_eq!(lock.header.oracle.emulator, "lsnes-bsnes");
    assert_eq!(lock.header.bundles.len(), 1);
    assert_eq!(lock.rows.len(), 651);
    assert!(lock.row(&RowId("area-brinstar".to_owned())).is_some());
    assert!(lock.row(&RowId("item-morph-ball".to_owned())).is_some());
}
