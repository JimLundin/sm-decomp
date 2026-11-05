//! The mutant catalogue: each mutant names its edits and the check that must
//! catch it. A mutant is one deliberate bug; the catalogue is data, and
//! `main.rs` is its interpreter.

/// One textual edit. `anchor` must occur exactly once in `file`.
pub struct Edit {
    pub file: &'static str,
    pub anchor: &'static str,
    pub replacement: &'static str,
}

/// What the mutant attacks, for the report.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// The oracle's RAM-to-Session decoder (`session_encoding::decode`).
    Decoder,
    /// The Session-space comparison (`sm_tools::session_compare`).
    SessionCompare,
    /// The digest lock: the digests and their running fold (`sm_tools::trace`).
    Lock,
    /// The engine, seen through one lock stream.
    Engine(Stream),
}

/// The lock stream a replay-level engine mutant is meant to reach.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stream {
    Ram,
    Video,
    ApuPorts,
    Session,
}

/// A cargo test target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tests {
    /// The package's library unit tests.
    Lib,
    /// One integration test file, by name (`tests/<name>.rs`).
    Integration(&'static str),
}

/// The check that must catch the mutant.
pub enum Proof {
    /// ROM-free: every named test must pass unmutated and fail mutated.
    Unit {
        package: &'static str,
        tests: Tests,
        names: &'static [&'static str],
    },
    /// Needs the ROM and the rows' bundles: engine-only replay against the
    /// committed digest lock must report a lock mismatch on some row.
    Lock {
        package: &'static str,
        rows: &'static [&'static str],
    },
}

impl Proof {
    pub fn package(&self) -> &'static str {
        match self {
            Self::Unit { package, .. } | Self::Lock { package, .. } => package,
        }
    }

    pub fn needs_rom(&self) -> bool {
        matches!(self, Self::Lock { .. })
    }
}

pub struct Mutant {
    pub name: &'static str,
    pub target: Target,
    pub edits: &'static [Edit],
    pub proof: Proof,
}

const COMPARE: &str = "crates/sm_tools/src/session_compare.rs";
const TRACE: &str = "crates/sm_tools/src/trace.rs";
const DIGEST: &str = "crates/sm_data/src/digest.rs";
const ENCODING: &str = "crates/sm_game/src/session_encoding.rs";
const ROUND_TRIP: &[&str] = &["session_round_trips_random_ram"];

pub const MUTANTS: &[Mutant] = &[
    // The decoder. The engine's Session is compared directly and the oracle's
    // is decoded, so a decoder that reads the wrong byte must not be silent.
    Mutant {
        name: "decoder_minimap_reads_even_bytes",
        target: Target::Decoder,
        edits: &[Edit {
            file: ENCODING,
            anchor: "        *value = bytes(0x7f7 + index);",
            replacement: "        *value = bytes(0x7f7 + (index & !1));",
        }],
        proof: Proof::Unit {
            package: "sm_game",
            tests: Tests::Integration("state_layout"),
            names: ROUND_TRIP,
        },
    },
    Mutant {
        name: "decoder_countdown_repeats_first_digit",
        target: Target::Decoder,
        edits: &[Edit {
            file: ENCODING,
            anchor: "        [bytes(0x945), bytes(0x946), bytes(0x947)],",
            replacement: "        [bytes(0x945), bytes(0x945), bytes(0x947)],",
        }],
        proof: Proof::Unit {
            package: "sm_game",
            tests: Tests::Integration("state_layout"),
            names: ROUND_TRIP,
        },
    },
    // Session compare.
    Mutant {
        name: "compare_skips_the_last_field",
        target: Target::SessionCompare,
        edits: &[Edit {
            file: COMPARE,
            anchor: "                if engine.$field != oracle.$field {",
            replacement: "                if engine.$field != oracle.$field && index + 1 != FIELDS.len() {",
        }],
        proof: Proof::Unit {
            package: "sm_tools",
            tests: Tests::Lib,
            names: &["session_compare::tests::every_field_is_compared"],
        },
    },
    Mutant {
        name: "compare_reports_no_field",
        target: Target::SessionCompare,
        edits: &[Edit {
            file: COMPARE,
            anchor: "                    differing.push(index);",
            replacement: "                    let _ = index;",
        }],
        proof: Proof::Unit {
            package: "sm_tools",
            tests: Tests::Lib,
            names: &[
                "session_compare::tests::uncarried_fields_take_the_oracles_value",
                "session_compare::tests::a_changed_field_changes_only_its_group",
            ],
        },
    },
    Mutant {
        name: "compare_path_stops_at_the_root",
        target: Target::SessionCompare,
        edits: &[Edit {
            file: COMPARE,
            anchor: "        } else if depth > 0 {",
            replacement: "        } else if depth > 99 {",
        }],
        proof: Proof::Unit {
            package: "sm_tools",
            tests: Tests::Lib,
            names: &["session_compare::tests::the_path_names_fields_and_indexes_but_no_values"],
        },
    },
    Mutant {
        name: "compare_counts_a_divergent_tick_as_agreement",
        target: Target::SessionCompare,
        edits: &[Edit {
            file: COMPARE,
            anchor: "            (true, false) => (&mut self.session_only.0, &mut self.session_only.1),",
            replacement: "            (true, false) => {\n                self.both_passed += 1;\n                return;\n            }",
        }],
        proof: Proof::Unit {
            package: "sm_tools",
            tests: Tests::Lib,
            names: &["session_compare::tests::agreement_counts_each_side"],
        },
    },
    Mutant {
        name: "streams_put_a_field_in_the_wrong_group",
        target: Target::SessionCompare,
        edits: &[Edit {
            file: COMPARE,
            anchor: "    player => Player,",
            replacement: "    player => Scratch,",
        }],
        proof: Proof::Unit {
            package: "sm_tools",
            tests: Tests::Lib,
            names: &["session_compare::tests::a_changed_field_changes_only_its_group"],
        },
    },
    Mutant {
        name: "streams_keep_a_changed_fields_old_digest",
        target: Target::SessionCompare,
        edits: &[Edit {
            file: COMPARE,
            anchor: "                    fields[index] = field_digest(&session, index);",
            replacement: "                    let _ = index;",
        }],
        proof: Proof::Unit {
            package: "sm_tools",
            tests: Tests::Lib,
            names: &["session_compare::tests::a_changed_field_changes_only_its_group"],
        },
    },
    // The digest lock.
    Mutant {
        name: "lock_fold_drops_a_streams_tick",
        target: Target::Lock,
        edits: &[Edit {
            file: TRACE,
            anchor: "                Some(digest) => *running = fold(*running, digest),",
            replacement: "                Some(digest) => *running = fold(*running, digest & !1),",
        }],
        proof: Proof::Unit {
            package: "sm_tools",
            tests: Tests::Lib,
            names: &["trace::tests::one_changed_tick_changes_its_stream_and_the_combined_digest"],
        },
    },
    Mutant {
        name: "lock_combined_skips_a_machine_stream",
        target: Target::Lock,
        edits: &[Edit {
            file: TRACE,
            anchor: "        for &digest in &tick.machine {",
            replacement: "        for &digest in &tick.machine[1..] {",
        }],
        proof: Proof::Unit {
            package: "sm_tools",
            tests: Tests::Lib,
            names: &["trace::tests::one_changed_tick_changes_its_stream_and_the_combined_digest"],
        },
    },
    Mutant {
        name: "lock_writes_a_partial_stream",
        target: Target::Lock,
        edits: &[Edit {
            file: TRACE,
            anchor: "            if !missing {",
            replacement: "            if !missing || stream >= MACHINE.len() {",
        }],
        proof: Proof::Unit {
            package: "sm_tools",
            tests: Tests::Lib,
            names: &["trace::tests::a_stream_some_tick_lacks_has_no_running_digest"],
        },
    },
    Mutant {
        name: "digest_drops_the_rotation",
        target: Target::Lock,
        edits: &[Edit {
            file: DIGEST,
            anchor: "            self.0 = (self.0.rotate_left(23) ^ u64::from_le_bytes(word))",
            replacement: "            self.0 = (self.0 ^ u64::from_le_bytes(word))",
        }],
        proof: Proof::Unit {
            package: "sm_data",
            tests: Tests::Lib,
            names: &["digest::tests::the_digest_is_the_one_the_lock_recorded"],
        },
    },
    // Replay-level engine mutants, one per stream: the lock itself must fail.
    Mutant {
        name: "ram_botwoon_spit_spread",
        target: Target::Engine(Stream::Ram),
        edits: &[Edit {
            file: "crates/sm_game/src/actors/botwoon.rs",
            anchor: "const SPIT_SPREAD: u8 = 16;",
            replacement: "const SPIT_SPREAD: u8 = 17;",
        }],
        proof: Proof::Lock {
            package: "sm_game",
            rows: &["coverage-botwoon-spit", "boss-botwoon"],
        },
    },
    Mutant {
        name: "apu_ports_sound_off_by_one",
        target: Target::Engine(Stream::ApuPorts),
        edits: &[Edit {
            file: "crates/sm_game/src/music_requests.rs",
            anchor: "                        write(channel as u8, sound, record.as_ref(), host)",
            replacement: "                        write(channel as u8, sound.wrapping_add(1), record.as_ref(), host)",
        }],
        proof: Proof::Lock {
            package: "sm_game",
            rows: &["item-speed-booster", "pickup-missile"],
        },
    },
    Mutant {
        name: "video_brightness_over_sixteen",
        target: Target::Engine(Stream::Video),
        edits: &[Edit {
            file: "crates/sm_runtime/src/rendering/records.rs",
            anchor: "        let value = (((color << 3 | color >> 2) * u16::from(brightness)) / 15) as u8;",
            replacement: "        let value = (((color << 3 | color >> 2) * u16::from(brightness)) / 16) as u8;",
        }],
        proof: Proof::Lock {
            package: "sm_runtime",
            rows: &["area-crateria", "door-transition"],
        },
    },
    Mutant {
        name: "session_streams_regroup_player",
        target: Target::Engine(Stream::Session),
        edits: &[Edit {
            file: COMPARE,
            anchor: "    player => Player,",
            replacement: "    player => Scratch,",
        }],
        proof: Proof::Lock {
            package: "sm_tools",
            rows: &["samus-input-poses", "door-transition"],
        },
    },
];
