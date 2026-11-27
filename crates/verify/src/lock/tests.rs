//! The reader against the shared fixture (`format2.toml`, which `oracle::relock`'s writer
//! test writes byte for byte) and the strict rejections.

use super::*;

const FIXTURE: &str = include_str!("format2.toml");

fn fixture() -> Lock {
    Lock::parse(FIXTURE).expect("the fixture parses")
}

fn fixture_oracle() -> Oracle {
    fixture().header.oracle
}

fn error(text: &str) -> ErrorKind {
    Lock::parse(text).expect_err("the text is rejected").kind
}

#[test]
fn the_fixture_reads_as_written() {
    let lock = fixture();
    assert_eq!(lock.header.mark_frames, FrameCount(1000));
    assert_eq!(lock.header.oracle.emulator, "synthetic");
    assert_eq!(lock.header.oracle.version, "test 1");
    assert_eq!(
        lock.header.oracle.server.to_string(),
        "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff"
    );
    assert_eq!(lock.header.oracle.fixes, ["fix-a"]);
    assert_eq!(lock.header.decoder, None);
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
    assert_eq!(alpha.ticks, Ticks(1490));
    assert_eq!(alpha.overruns, [Frame(254), Frame(256), Frame(257)]);
    assert!(alpha.nmi_off.is_empty() && alpha.still_running.is_empty());
    assert_eq!(
        alpha.facts,
        [
            ("room:91F8".to_owned(), FactResult::Holds(true)),
            ("probe:example".to_owned(), FactResult::Unevaluated)
        ]
    );
    assert_eq!(
        alpha.streams.get(&Stream::Ppu),
        Some(&StreamLock::Marks(vec![Digest(0x501), Digest(0x502)]))
    );
    assert_eq!(alpha.streams.get(&Stream::Samus), None);

    assert_eq!(beta.start, Start::PowerOn);
    assert_eq!(beta.seed_record, None);
    assert_eq!(beta.nmi_off, (5..=9).map(Frame).collect::<Vec<_>>());
    assert_eq!(beta.still_running, [Frame(12)]);
    assert_eq!(beta.facts, []);
    assert_eq!(beta.streams.get(&Stream::Video), Some(&StreamLock::Absent));
    assert_eq!(
        beta.streams.get(&Stream::Wram),
        Some(&StreamLock::Marks(vec![Digest(0xa_0901)]))
    );
}

#[test]
fn the_rest_of_the_format_reads() {
    let text = FIXTURE.replacen(
        "mark_frames = 1000\n",
        "mark_frames = 1000 # a comment\ndecoder = \"decode-1\"\nbundles = [{ name = \"b.tar.gz.enc\", sha256 = \"5555555555555555555555555555555555555555555555555555555555555555\" }]\n",
        1,
    )
    .replacen("nmi_off = []\n", "nmi_off = []\nrungs = [500, 1000]\n", 1);
    let lock = Lock::parse(&text).expect("parses");
    assert_eq!(lock.header.decoder.as_deref(), Some("decode-1"));
    assert_eq!(
        lock.header.bundles,
        [Bundle {
            name: "b.tar.gz.enc".to_owned(),
            sha256: Sha256([0x55; 32])
        }]
    );
    assert_eq!(lock.rows[0].rungs, [Frame(500), Frame(1000)]);
}

#[test]
fn the_reader_is_strict() {
    let cases: [(&str, &str, ErrorKind); 12] = [
        ("format = 2", "format = 3", ErrorKind::Format(3)),
        (
            "ticks = 1490\n",
            "ticks = 1490\nspeed = 1\n",
            ErrorKind::UnknownKey("speed".to_owned()),
        ),
        (
            "ticks = 1490\n",
            "ticks = 1490\nticks = 1490\n",
            ErrorKind::DuplicateKey("ticks".to_owned()),
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
        (
            "streams.ppu = [\"0000000000000501\", \"0000000000000502\"]\n",
            "",
            ErrorKind::MissingStream(Stream::Ppu),
        ),
        (
            "streams.ppu = [\"0000000000000501\", \"0000000000000502\"]",
            "streams.ppu = [\"0000000000000501\"]",
            ErrorKind::SpanCount {
                stream: Stream::Ppu,
                expected: 2,
                found: 1,
            },
        ),
        (
            "streams.vram = [\"0000000000000101\", ",
            "streams.vrom = [\"0000000000000101\", ",
            ErrorKind::UnknownStream("vrom".to_owned()),
        ),
        (
            "absent = [\"video\"]",
            "absent = [\"video\", \"vram\"]",
            ErrorKind::DuplicateStream(Stream::Vram),
        ),
        (
            "\"256-257\"",
            "\"255-257\"",
            ErrorKind::BadSchedule("255-257".to_owned()),
        ),
        (
            "id = \"row-beta\"",
            "id = \"row-alpha\"",
            ErrorKind::DuplicateRow(RowId("row-alpha".to_owned())),
        ),
        (
            "\"probe:example\" = \"unevaluated\"",
            "\"probe:example\" = \"maybe\"",
            ErrorKind::BadFact("probe:example".to_owned()),
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

#[test]
fn a_lock_of_another_oracle_version_is_rejected() {
    let oracle = fixture_oracle();
    assert!(Lock::parse_for(FIXTURE, &oracle).is_ok());
    let other = FIXTURE.replacen("version = \"test 1\"", "version = \"test 2\"", 1);
    assert_eq!(
        Lock::parse_for(&other, &oracle).map_err(|error| error.kind),
        Err(ErrorKind::OracleMismatch)
    );
}

/// The committed lock is what the gate reads, so a recorder or reader change that
/// the other side doesn't follow fails here, not at the first engine comparison.
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
