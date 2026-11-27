use super::requests::{CorpusRequest, MovieRequest, RequestError};
use super::{RoomSeed, csv, movie, save_due};
use std::path::PathBuf;

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

#[test]
fn movie_options_parse_without_opening_paths() {
    let request = MovieRequest::parse(&args(&[
        "missing.rom",
        "missing.inputs",
        "--frames",
        "100",
        "--seeds",
        "missing.csv",
        "seeds",
        "--native-seeds",
        "missing-native.csv",
        "native",
        "--from",
        "missing.state",
        "40",
        "--ladder",
        "30",
        "ladder",
        "--from-ladder",
        "missing-ladder",
        "--lag-log",
        "missing-lag.csv",
        "--watch",
        "10",
        "5",
        "--trace",
        "20",
        "25",
        "808000,818001",
        "--positions-only",
        "--ranges",
        "--no-memories",
        "--ppu",
        "--streams",
    ]))
    .unwrap();
    assert_eq!(request.frames, Some(100));
    assert_eq!(
        request.start,
        super::requests::MovieStart::Saved {
            state: PathBuf::from("missing.state"),
            frame: 40
        }
    );
    assert_eq!(request.watch, Some((10, 5)));
    assert_eq!(request.trace, Some((20, 25, vec![0x808000, 0x818001])));
    assert_eq!(request.cutting, Some((30, PathBuf::from("ladder"))));
    assert_eq!(request.native.len(), 1);
    assert!(!request.memories);
    assert!(request.positions_only && request.ranges);
    assert_eq!(request.streams, Some(crate::Streams::PpuAndApuLog));
}

#[test]
fn movie_repeated_options_keep_last_values_and_accumulate_native_lists() {
    let request = MovieRequest::parse(&args(&[
        "rom",
        "pads",
        "--frames",
        "10",
        "--frames",
        "20",
        "--from",
        "one",
        "3",
        "--from",
        "two",
        "7",
        "--seeds",
        "one.csv",
        "one",
        "--seeds",
        "two.csv",
        "two",
        "--native-seeds",
        "one.csv",
        "one",
        "--native-seeds",
        "two.csv",
        "two",
        "--streams",
        "--ppu",
    ]))
    .unwrap();
    assert_eq!(request.frames, Some(20));
    assert_eq!(
        request.start,
        super::requests::MovieStart::Saved {
            state: PathBuf::from("two"),
            frame: 7
        }
    );
    assert_eq!(
        request.seeds,
        Some((PathBuf::from("two.csv"), PathBuf::from("two")))
    );
    assert_eq!(request.native.len(), 2);
    assert_eq!(request.streams, Some(crate::Streams::PpuAndApuLog));
}

#[test]
fn superseded_missing_input_is_replaced_by_the_effective_input() {
    let directory =
        std::env::temp_dir().join(format!("oracle-cli-effective-input-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let missing = directory.join("missing.csv");
    let effective = directory.join("effective.csv");
    std::fs::write(&effective, "room_index,room,movie_frame\n1,1234,40\n").unwrap();
    let values = vec![
        "rom".to_owned(),
        "pads".to_owned(),
        "--seeds".to_owned(),
        missing.to_string_lossy().into_owned(),
        "unused".to_owned(),
        "--seeds".to_owned(),
        effective.to_string_lossy().into_owned(),
        "used".to_owned(),
    ];
    let request = MovieRequest::parse(&values).unwrap();
    let (csv, destination) = request.seeds.unwrap();
    assert_eq!(csv, effective);
    assert_eq!(destination, PathBuf::from("used"));
    assert!(!missing.exists());
    assert_eq!(super::room_seeds(&csv).len(), 1);
    std::fs::remove_file(effective).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn movie_missing_and_malformed_operands_return_errors() {
    for tail in [
        vec!["--frames"],
        vec!["--frames", "no"],
        vec!["--seeds", "csv"],
        vec!["--native-seeds", "csv"],
        vec!["--from", "state"],
        vec!["--from", "state", "no"],
        vec!["--ladder", "0", "dir"],
        vec!["--ladder", "1"],
        vec!["--from-ladder"],
        vec!["--watch", "1"],
        vec!["--trace", "1", "2"],
        vec!["--trace", "1", "2", "no"],
        vec!["--lag-log"],
        vec!["--unknown"],
    ] {
        let mut values = vec!["rom", "pads"];
        values.extend(tail);
        assert!(matches!(
            MovieRequest::parse(&args(&values)),
            Err(RequestError::Invalid(_))
        ));
    }
}

#[test]
fn movie_from_ladder_without_native_seeds_is_rejected_before_acquisition() {
    assert!(matches!(
        movie(&args(&[
            "missing.rom",
            "missing.inputs",
            "--from-ladder",
            "dir"
        ])),
        Err(RequestError::Compatibility)
    ));
}

#[test]
fn corpus_options_parse_without_acquiring_outputs() {
    let request = CorpusRequest::parse(&args(&[
        "missing.rom",
        "missing.lock",
        "missing.data",
        "--only",
        "one",
        "--only",
        "two",
        "--limit",
        "4",
        "--shard",
        "1/3",
        "--watch-words",
        "$1234,abcd",
        "--watch-pcs",
        "$80:8000,818000",
        "--cache",
        "missing.cache",
    ]))
    .unwrap();
    assert_eq!(request.only, ["one", "two"]);
    assert_eq!(request.limit, 4);
    assert!(request.shard.includes(1) && request.shard.includes(4));
    assert!(!request.shard.includes(2));
    assert_eq!(request.watched, [0x1234, 0xabcd]);
    assert_eq!(request.watched_pcs, [0x808000, 0x818000]);
}

#[test]
fn corpus_repeated_options_keep_the_last_selection_and_accumulate_watches() {
    let request = CorpusRequest::parse(&args(&[
        "rom",
        "lock",
        "data",
        "--limit",
        "3",
        "--limit",
        "4",
        "--shard",
        "1/2",
        "--shard",
        "0/3",
        "--watch-words",
        "1",
        "--watch-words",
        "2",
    ]))
    .unwrap();
    assert_eq!(request.limit, 4);
    assert_eq!(request.watched, [1, 2]);
    assert!(request.shard.includes(0) && request.shard.includes(3));
}

#[test]
fn corpus_invalid_shards_are_rejected_before_running_rows() {
    for shard in ["0/0", "1/0", "1/1", "3/2", "-1/2", "0/no", "0/2/3"] {
        assert!(matches!(
            CorpusRequest::parse(&args(&["rom", "lock", "data", "--shard", shard])),
            Err(RequestError::Shard)
        ));
    }
    let request = CorpusRequest::parse(&args(&["rom", "lock", "data", "--shard", "0/1"])).unwrap();
    assert!(request.shard.includes(usize::MAX));
}

#[test]
fn corpus_missing_operands_and_compatibility_are_rejected() {
    for option in [
        "--only",
        "--shard",
        "--limit",
        "--record",
        "--record-v2",
        "--native-seeds",
        "--watch-words",
        "--watch-pcs",
        "--frame-pads",
        "--from-frame-pads",
        "--cache",
    ] {
        assert!(CorpusRequest::parse(&args(&["rom", "lock", "data", option])).is_err());
    }
    for tail in [
        vec!["--watch-words", "$1ffff"],
        vec!["--watch-pcs", "$1000000"],
        vec!["--cache", "dir", "--no-memories"],
    ] {
        let mut values = vec!["rom", "lock", "data"];
        values.extend(tail);
        assert!(CorpusRequest::parse(&args(&values)).is_err());
    }
}

#[test]
fn room_csv_headers_can_be_reordered_and_blank_lines_keep_line_identity() {
    let rows = csv::room_seeds("movie_frame,ignored,room,room_index\n40,x,abcd,7\n\n80,x,1234,8\n")
        .unwrap();
    assert_eq!(
        rows,
        [
            RoomSeed {
                index: 7,
                room: 0xabcd,
                movie_frame: 40
            },
            RoomSeed {
                index: 8,
                room: 0x1234,
                movie_frame: 80
            }
        ]
    );
    let error = csv::room_seeds("movie_frame,ignored,room,room_index\n\n40,x,abcd\n").unwrap_err();
    assert_eq!(
        error,
        csv::CsvError {
            line: 3,
            column: 4,
            kind: csv::ErrorKind::MissingField
        }
    );
}

#[test]
fn room_csv_invalid_fields_report_coordinates_without_retaining_the_row() {
    let error =
        csv::room_seeds("room_index,room,movie_frame\n1,invalid-private-value,30\n").unwrap_err();
    assert_eq!(
        error,
        csv::CsvError {
            line: 2,
            column: 2,
            kind: csv::ErrorKind::InvalidNumber
        }
    );
    assert_eq!(
        csv::room_seeds("other\n").unwrap_err().kind,
        csv::ErrorKind::MissingColumn("room_index")
    );
}

#[test]
fn lag_csv_short_rows_are_errors_and_unknown_numeric_fields_keep_the_sentinel() {
    assert_eq!(
        csv::lag_log("header\n0,1,2,3,4,5,6,7,8,9,10\n").unwrap(),
        [[3, 5, 6, 7, 8, 9, 10]]
    );
    assert_eq!(
        csv::lag_log("header\n0,1,2,unknown,4,5,6,7,8,9,10\n").unwrap(),
        [[-1, 5, 6, 7, 8, 9, 10]]
    );
    assert_eq!(
        csv::lag_log("header\n0,1,2,3,4,5\n").unwrap_err(),
        csv::CsvError {
            line: 2,
            column: 7,
            kind: csv::ErrorKind::MissingField
        }
    );
}

#[test]
fn native_save_uses_absolute_movie_step_when_resumed() {
    let lists = vec![(
        vec![RoomSeed {
            index: 1,
            room: 1,
            movie_frame: 41,
        }],
        PathBuf::from("native"),
    )];
    assert!(save_due(&lists, None, (40, 100)));
    assert!(!save_due(&lists, None, (0, 100)));
    assert!(!save_due(&lists, None, (41, 100)));
}

#[test]
fn movie_lag_log_and_positions_only_report_the_compared_fields() {
    use super::movie_report::lag_difference;
    use crate::Beam;

    let mut record = crate::record::lsnes::blank();
    record.wram = vec![0; 0x1000].into_boxed_slice();
    record.wram[0x05b6..0x05b8].copy_from_slice(&41267_u16.to_le_bytes());
    record.wram[0x0af6..0x0af8].copy_from_slice(&54128_u16.to_le_bytes());
    record.wait_entry = Some(Beam { v: 283, h: 1179 });
    let row = csv::lag_log("header\n0,0,0,49712,0,0,1,0,0,31805,0\n").unwrap()[0];
    let payloads = ["41267", "54128", "49712", "31805", "283", "1179"];

    for positions_only in [false, true] {
        let mut command = args(&["rom", "inputs", "--lag-log", "lag.csv"]);
        if positions_only {
            command.push("--positions-only".to_owned());
        }
        let request = MovieRequest::parse(&command).unwrap();
        assert!(request.lag_log.is_some());
        let report = lag_difference(47, &record, &row, request.positions_only).unwrap();
        assert!(report.contains("frame 47"));
        assert!(report.contains("Samus X"));
        assert_eq!(report.contains("frame counter"), !positions_only);
        assert_omits(&report, &payloads);
        let mut same_position = row;
        same_position[5] = 54128;
        assert_eq!(
            lag_difference(47, &record, &same_position, request.positions_only).is_none(),
            positions_only
        );
    }
}

#[test]
fn movie_seed_comparisons_report_room_status_and_optional_range_locations() {
    use super::movie_report::seed_comparison;
    use std::collections::VecDeque;

    let seed = RoomSeed {
        index: 17,
        room: 0xadbe,
        movie_frame: 10,
    };
    let mut capture = vec![0; 0x1000];
    capture[0x40..0x42].copy_from_slice(&[0xb2, 0xa1]);
    capture[0x079b..0x079d].copy_from_slice(&seed.room.to_le_bytes());
    let mut earlier = capture.clone().into_boxed_slice();
    earlier[0x40..0x42].copy_from_slice(&[0xd4, 0xc3]);
    earlier[0x079b..0x079d].copy_from_slice(&[0xfa, 0xce]);
    let mut later = earlier.clone();
    later[0x12] = 1;
    let history = VecDeque::from([earlier, later]);

    for ranges in [false, true] {
        let mut command = args(&["rom", "inputs", "--seeds", "rooms.csv", "captures"]);
        if ranges {
            command.push("--ranges".to_owned());
        }
        let request = MovieRequest::parse(&command).unwrap();
        assert!(request.seeds.is_some());
        let (matched, report) = seed_comparison(&seed, 14, &history, &capture, request.ranges);
        assert!(!matched);
        let seed_line = report
            .lines()
            .find(|line| line.starts_with("seed"))
            .unwrap();
        let mut words = seed_line.split_whitespace();
        assert_eq!(words.next(), Some("seed"));
        assert_eq!(words.next(), Some("17"));
        assert!(seed_line.contains("10:"));
        assert!(seed_line.contains("room DIFFERS"));
        assert!(seed_line.contains("differing 4"));
        assert!(seed_line.contains("+2"));
        assert_eq!(report.contains("[4, 5]"), ranges);
        assert_eq!(report.contains("2 ranges"), ranges);
        assert_eq!(report.contains("00040-00041 2 bytes"), ranges);
        assert_eq!(report.contains("0079B-0079C 2 bytes"), ranges);
        assert_omits(
            &report,
            &["adbe", "cefa", "b2, a1", "d4, c3", "be, ad", "fa, ce"],
        );
    }
}

fn assert_omits(text: &str, payloads: &[&str]) {
    let text = text.to_ascii_lowercase();
    for payload in payloads {
        assert!(!text.contains(payload));
    }
}

#[test]
fn derived_seed_write_comparison_reports_offset_and_status_without_words() {
    use super::seed_write_report;
    use crate::lsnes::WramWord;

    let word = WramWord {
        offset: 2,
        value: 0xface,
    };
    let plain = [0, 0, 0x1d, 0xb0];
    for (derived, expected) in [
        ([0, 0, 0xce, 0xfa], true),
        ([0, 0, 0x6a, 0xd2], true),
        (plain, false),
    ] {
        let (kept, report) = seed_write_report(word, &plain, &derived);
        assert_eq!(kept, expected);
        assert!(report.contains("$00002"));
        assert_eq!(report.contains("LOST"), !expected);
        assert_omits(&report, &["face", "b01d", "d26a"]);
    }
}
