use super::requests::{CorpusRequest, CrossRequest, MovieRequest, RequestError};
use super::{FramePads, RoomSeed, corpus, cross, csv, movie, save_due};
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
        "--oracle",
        "lsnes",
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
        "--profile",
        "1",
        "2",
        "--spc",
        "--positions-only",
        "--ranges",
        "--no-picture",
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
    assert_eq!(request.profile, Some((1, 2)));
    assert_eq!(request.trace, Some((20, 25, vec![0x808000, 0x818001])));
    assert_eq!(request.cutting, Some((30, PathBuf::from("ladder"))));
    assert_eq!(request.native.len(), 1);
    assert!(!request.picture && !request.memories);
    assert!(request.spc && request.positions_only && request.ranges);
    assert!(request.streams.unwrap().picture);
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
        "--oracle",
        "lakesnes",
        "--oracle",
        "lsnes",
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
    assert!(request.streams.unwrap().picture);
    assert_eq!(request.backend.into_name(), "lsnes");
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
        vec!["--oracle"],
        vec!["--native-seeds", "csv"],
        vec!["--from", "state"],
        vec!["--from", "state", "no"],
        vec!["--ladder", "0", "dir"],
        vec!["--ladder", "1"],
        vec!["--from-ladder"],
        vec!["--watch", "1"],
        vec!["--profile", "1"],
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
fn movie_backend_requirements_are_validated_before_acquisition() {
    for tail in [
        vec!["--streams"],
        vec!["--ppu"],
        vec!["--ladder", "1", "dir"],
        vec!["--native-seeds", "csv", "dir"],
        vec!["--from-ladder", "dir"],
    ] {
        let mut values = vec!["missing.rom", "missing.inputs"];
        values.extend(tail);
        assert!(matches!(
            movie(&args(&values)),
            Err(RequestError::Compatibility)
        ));
    }
    let request = MovieRequest::parse(&args(&["rom", "pads", "--oracle", "custom"])).unwrap();
    assert_eq!(request.backend.into_name(), "custom");
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
        "--oracle",
        "lsnes",
        "--record",
        "missing.record",
        "--record-v2",
        "missing.v2",
        "--sidecar",
        "missing.sidecar",
        "--dump-frames",
        "one:1,3",
        "--native-seeds",
        "missing.native",
        "--watch-words",
        "$1234,abcd",
        "--watch-pcs",
        "$80:8000,818000",
        "--from-frame-pads",
        "missing.pads",
        "--cache",
        "missing.cache",
        "--cache-pictures",
    ]))
    .unwrap();
    assert_eq!(request.only, ["one", "two"]);
    assert_eq!(request.limit, 4);
    assert!(request.shard.includes(1) && request.shard.includes(4));
    assert!(!request.shard.includes(2));
    assert_eq!(request.watched, [0x1234, 0xabcd]);
    assert_eq!(request.watched_pcs, [0x808000, 0x818000]);
    assert_eq!(
        request.dump_frames,
        [("one".to_owned(), 1), ("one".to_owned(), 3)]
    );
}

#[test]
fn corpus_repeat_precedence_preserves_selection_and_effective_outputs() {
    let request = CorpusRequest::parse(&args(&[
        "rom",
        "lock",
        "data",
        "--record",
        "one",
        "--record",
        "two",
        "--limit",
        "3",
        "--limit",
        "4",
        "--shard",
        "1/2",
        "--shard",
        "0/3",
        "--frame-pads",
        "write",
        "--from-frame-pads",
        "play",
        "--watch-words",
        "1",
        "--watch-words",
        "2",
    ]))
    .unwrap();
    assert!(
        matches!(&request.outputs[1], super::requests::CorpusOutput::Record(path) if path == &PathBuf::from("two"))
    );
    assert_eq!(request.limit, 4);
    assert_eq!(request.watched, [1, 2]);
    assert!(request.shard.includes(0) && request.shard.includes(3));
    assert!(
        matches!(request.pads, Some(FramePads::Play(directory)) if directory.as_path() == std::path::Path::new("play"))
    );
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
        "--sidecar",
        "--dump-frames",
        "--oracle",
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
        vec!["--record-v2", "file", "--oracle", "lakesnes"],
        vec!["--record-v2", "file", "--oracle", "lsnes"],
        vec!["--sidecar", "dir"],
        vec!["--dump-frames", "row:1"],
        vec!["--cache", "dir"],
        vec![
            "--cache",
            "dir",
            "--from-frame-pads",
            "pads",
            "--no-memories",
        ],
    ] {
        let mut values = vec!["rom", "lock", "data"];
        values.extend(tail);
        assert!(CorpusRequest::parse(&args(&values)).is_err());
    }
}

#[test]
fn invalid_tail_preserves_existing_outputs_and_does_not_create_directories() {
    let directory =
        std::env::temp_dir().join(format!("oracle-cli-preservation-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let output = directory.join("sentinel.txt");
    let new_directory = directory.join("uncreated");
    for flag in ["--record", "--record-v2"] {
        std::fs::write(&output, b"synthetic sentinel").unwrap();
        let mut values = args(&["missing.rom", "missing.lock", "missing.data", flag]);
        values.push(output.to_string_lossy().into_owned());
        values.extend(args(&["--sidecar"]));
        values.push(new_directory.to_string_lossy().into_owned());
        values.extend(args(&["--unknown"]));
        assert!(corpus(&values).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), b"synthetic sentinel");
        assert!(!new_directory.exists());
        // A fully parsed but incompatible request must also preserve the file.
        values.truncate(5);
        values.extend(args(&["--oracle", "lakesnes", "--record-v2"]));
        values.push(output.to_string_lossy().into_owned());
        assert!(corpus(&values).is_err());
        assert_eq!(std::fs::read(&output).unwrap(), b"synthetic sentinel");
    }
    std::fs::remove_file(output).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn successful_repeated_outputs_are_acquired_in_option_order() {
    let directory =
        std::env::temp_dir().join(format!("oracle-cli-output-order-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let first = directory.join("first.txt");
    let last = directory.join("last.txt");
    let pads = directory.join("pads");
    for path in [&first, &last] {
        std::fs::write(path, b"synthetic sentinel").unwrap();
    }
    let values = vec![
        "rom".to_owned(),
        "lock".to_owned(),
        "data".to_owned(),
        "--record".to_owned(),
        first.to_string_lossy().into_owned(),
        "--frame-pads".to_owned(),
        pads.to_string_lossy().into_owned(),
        "--record".to_owned(),
        last.to_string_lossy().into_owned(),
        "--from-frame-pads".to_owned(),
        "play".to_owned(),
    ];
    let request = CorpusRequest::parse(&values).unwrap();
    let (mut record, record_v2) = super::acquire_corpus_outputs(request.outputs);
    std::io::Write::write_all(record.as_mut().unwrap(), b"active record").unwrap();
    drop(record);
    assert!(record_v2.is_none());
    assert_eq!(std::fs::read(&first).unwrap(), b"");
    assert_eq!(std::fs::read(&last).unwrap(), b"active record");
    assert!(pads.is_dir());
    std::fs::remove_file(first).unwrap();
    std::fs::remove_file(last).unwrap();
    std::fs::remove_dir(pads).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn cross_preserves_repeat_resume_and_unclamped_range_requests() {
    let request = CrossRequest::parse(&args(&[
        "missing.rom",
        "missing.inputs",
        "--frames",
        "1..4",
        "--frames",
        "8..3",
        "--ranges",
        "--streams",
        "--ranges-at",
        "5..100",
        "--right-from",
        "missing.one",
        "10",
        "--right-from",
        "missing.two",
        "20",
    ]))
    .unwrap();
    let range = request.range.unwrap();
    assert_eq!((*range.start(), *range.end()), (8, 3));
    assert_eq!(request.ranges_at, Some(5..=100));
    assert_eq!(
        request.right_start,
        super::requests::CrossStart::Saved {
            state: PathBuf::from("missing.two"),
            step: 20
        }
    );
    assert!(request.streams && request.first_ranges);
}

#[test]
fn cross_malformed_tail_returns_before_loading_state() {
    for tail in [
        vec!["--frames"],
        vec!["--frames", "1..no"],
        vec!["--ranges-at", "no"],
        vec!["--right-from", "state"],
        vec!["--right-from", "state", "no"],
        vec!["--unknown"],
    ] {
        let mut values = vec!["missing.rom", "missing.inputs"];
        values.extend(tail);
        assert!(cross(&args(&values)).is_err());
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
