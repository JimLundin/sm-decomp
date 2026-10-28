//! Movies cannot forge semantic request records or inspect an accepted audio delivery queue.
#![forbid(unsafe_code)]

use super::*;
use crate::audio_snapshot::ApuQueueSnapshot;
use crate::game_state::persistent_session::{
    AudioHost, AudioRequestCatalog, AudioRequests, MusicContinuation, MusicMeaning, MusicRequests,
    QueuedMusicMeaning, QueuedSoundMeaning, SessionStartContract, SoundMeaning, SoundRequests,
};

fn program() -> sc::AssetRef {
    sc::AssetRef {
        id: sc::AssetId(1),
        revision: sc::Revision([5; 32]),
    }
}

/// Independent declarations use the existing fixture's authored program1/bank2.
fn catalog() -> Arc<AudioRequestCatalog> {
    Arc::new(AudioRequestCatalog {
        music: [
            (0, MusicMeaning::Silence),
            (
                7,
                MusicMeaning::Program {
                    program: program(),
                    phase: 59,
                },
            ),
            (
                0xff06,
                MusicMeaning::Bank {
                    bank: sc::AssetRef {
                        id: sc::AssetId(2),
                        revision: sc::Revision([5; 32]),
                    },
                    after: MusicContinuation::Retain,
                },
            ),
            (
                0xff07,
                MusicMeaning::Bank {
                    bank: sc::AssetRef {
                        id: sc::AssetId(2),
                        revision: sc::Revision([5; 32]),
                    },
                    after: MusicContinuation::Program {
                        program: program(),
                        phase: 59,
                    },
                },
            ),
        ]
        .into(),
        sounds: (0..3)
            .flat_map(|channel| {
                [
                    ((channel, 0), SoundMeaning::Stop),
                    (
                        (channel, 3 + channel as u8),
                        SoundMeaning::Program {
                            program: program(),
                            phase: 17 + channel as u64,
                        },
                    ),
                ]
            })
            .collect(),
    })
}

fn music(request: u16, source_interval: u64, meaning: MusicMeaning) -> Option<QueuedMusicMeaning> {
    Some(QueuedMusicMeaning {
        request,
        source_tick: 0,
        source_interval,
        meaning,
    })
}

fn sounds() -> [SoundRequests; 3] {
    core::array::from_fn(|channel| {
        let request = 3 + channel as u8;
        let mut queue = [0; 16];
        queue[15] = request;
        let mut meanings = core::array::from_fn(|_| None);
        meanings[15] = Some(QueuedSoundMeaning {
            request,
            source_tick: 0,
            source_interval: 17 + channel as u64,
            meaning: SoundMeaning::Program {
                program: program(),
                phase: 17 + channel as u64,
            },
        });
        meanings[0] = Some(QueuedSoundMeaning {
            request: 0,
            source_tick: 0,
            source_interval: 18 + channel as u64,
            meaning: SoundMeaning::Stop,
        });
        SoundRequests {
            read: 15,
            write: 1,
            state: 1 + channel as u8,
            current: request,
            clear_delay: 4 + channel as u8,
            maximum_queued: 15,
            queue,
            meanings,
            current_meaning: Some(QueuedSoundMeaning {
                request,
                source_tick: 0,
                source_interval: 19 + channel as u64,
                meaning: SoundMeaning::Program {
                    program: program(),
                    phase: 17 + channel as u64,
                },
            }),
        }
    })
}

/// Explicit pending commands at the fixture's Tick0/next input23; never a captured game start.
fn requests(catalog: Arc<AudioRequestCatalog>) -> AudioRequests {
    let mut meanings = core::array::from_fn(|_| None);
    meanings[7] = music(
        0xff06,
        20,
        MusicMeaning::Bank {
            bank: sc::AssetRef {
                id: sc::AssetId(2),
                revision: sc::Revision([5; 32]),
            },
            after: MusicContinuation::Retain,
        },
    );
    meanings[0] = music(
        0xff07,
        21,
        MusicMeaning::Bank {
            bank: sc::AssetRef {
                id: sc::AssetId(2),
                revision: sc::Revision([5; 32]),
            },
            after: MusicContinuation::Program {
                program: program(),
                phase: 59,
            },
        },
    );
    meanings[1] = music(0, 22, MusicMeaning::Silence);
    AudioRequests {
        music: MusicRequests {
            tracks: [0xff07, 0, 0, 0, 0, 0, 0, 0xff06],
            delays: [35, 1, 0, 0, 0, 0, 0, 17],
            write: 4,
            read: 14,
            entry: 7,
            timer: 9,
            current_track: 7,
            data_index: 6,
            track_index: 7,
            meanings,
            entry_meaning: music(
                7,
                22,
                MusicMeaning::Program {
                    program: program(),
                    phase: 59,
                },
            ),
            current_meaning: music(
                7,
                23,
                MusicMeaning::Program {
                    program: program(),
                    phase: 59,
                },
            ),
        },
        sounds: sounds(),
        uploading: 0,
        downtime: 9,
        catalog,
        pending_mutation: false,
    }
}

/// Reuse the complete prepared Session; the actual retained delivery owner stays outside G.
pub(super) fn run(owner: &GameOwner, original_contract: &SessionStartContract) {
    let (mut start, media) = owner.checkpoint_session().unwrap().into_start().unwrap();
    let audio = start.display.audio.clone().unwrap();
    assert_eq!(start.display.identity.unwrap().committed_tick, 0);
    assert_eq!(start.display.identity.unwrap().next_input_record, 23);
    let mut required = original_contract.requirements().clone();
    required.audio_catalog = catalog();
    let expected = requests(required.audio_catalog.clone());
    expected
        .validate(
            start.gameplay.as_ref().unwrap().control.metadata().unwrap(),
            &required.audio_catalog,
        )
        .unwrap();
    start.audio_requests = Some(expected.clone());
    *start
        .gameplay
        .as_mut()
        .unwrap()
        .control
        .requests_mut()
        .unwrap() = expected.clone();
    let accepted = owner.with(|game| {
        let mut queue = ApuQueueSnapshot::default();
        queue.reset();
        queue.write(0, 0x71);
        queue.write(2, 0x73);
        queue.push(false);
        queue.write(1, 0x72);
        queue.restore(game);
        ApuQueueSnapshot::capture(game).to_bytes()
    });
    owner
        .start_supplied_session(SuppliedSessionStart::new(
            start,
            media.clone(),
            required.clone(),
        ))
        .unwrap();
    let contract = SessionStartContract::new(required);
    let bytes = owner.encode_session_checkpoint().unwrap();
    assert_current(owner, &expected, &audio, &accepted);
    assert_rejections(owner, &bytes, media.clone(), &contract, &accepted);
    let prior = owner.with(|game| game.actor_identity().clone());
    owner
        .restore_session_checkpoint(&bytes, media.clone(), &contract)
        .unwrap();
    owner.with(|game| {
        crate::game::actors::identity::assert_installed_epoch(&prior, game.actor_identity())
    });
    assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
    assert_current(owner, &expected, &audio, &accepted);
    owner
        .reopen_supplied_session(owner.checkpoint_session().unwrap(), &contract)
        .unwrap();
    assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
    assert_current(owner, &expected, &audio, &accepted);
    assert_deliveries(owner, &bytes, &media, &contract, &accepted);
    owner.with(|game| {
        let epoch = game.compatibility_write_epoch();
        let mut session = game.lease_persistent_session().unwrap().unwrap();
        session
            .gameplay
            .control
            .requests_mut()
            .unwrap()
            .pending_mutation = true;
        game.return_persistent_session(session, true, epoch);
    });
    assert_eq!(
        owner.encode_session_checkpoint().err().as_deref(),
        Some("Session capture Incomplete")
    );
    owner
        .restore_session_checkpoint(&bytes, media, &contract)
        .unwrap();
    assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
    assert_current(owner, &expected, &audio, &accepted);
}

fn assert_current(
    owner: &GameOwner,
    expected: &AudioRequests,
    audio: &ss::InitialAudioIntent,
    accepted: &[u8; 71],
) {
    let (start, _) = owner.checkpoint_session().unwrap().into_start().unwrap();
    assert_eq!(start.audio_requests.as_ref().unwrap(), expected);
    assert_eq!(
        start.gameplay.as_ref().unwrap().control.requests().unwrap(),
        expected
    );
    assert_eq!(start.display.audio.as_ref().unwrap(), audio);
    owner.with(|game| {
        assert_eq!(ApuQueueSnapshot::capture(game).to_bytes(), *accepted);
        let mut delivery = ApuQueueSnapshot::capture(game);
        assert_eq!(delivery.pop(), Some([0x71, 0xff, 0x73, 0xff]));
        assert_eq!(delivery.pop(), None);
        assert_eq!(delivery.pending, [0xff, 0x72, 0xff, 0xff]);
        for (at, word) in [
            (0x619, 0xff07u16),
            (0x627, 0xff06),
            (0x639, 4),
            (0x63b, 14),
            (0x63d, 7),
            (0x63f, 9),
            (0x686, 9),
        ] {
            assert_eq!(
                [game.compatibility_byte(at), game.compatibility_byte(at + 1)],
                word.to_le_bytes()
            );
        }
        assert_eq!(game.compatibility_byte(0x64c), 7);
        assert_eq!(game.compatibility_byte(0x665), 3);
        assert_eq!(game.compatibility_byte(0x675), 4);
        assert_eq!(game.compatibility_byte(0x685), 5);
    });
}

fn unique(bytes: &[u8], needle: &[u8]) -> usize {
    let found: Vec<_> = bytes
        .windows(needle.len())
        .enumerate()
        .filter_map(|(at, value)| (value == needle).then_some(at))
        .collect();
    assert_eq!(
        found.len(),
        1,
        "independently authored audio record must occur once"
    );
    found[0]
}

/// No movie supplies malformed semantic records, catalog bindings or unfinished publication.
fn assert_rejections(
    owner: &GameOwner,
    bytes: &[u8],
    media: Arc<sc::SceneMedia>,
    contract: &SessionStartContract,
    accepted: &[u8; 71],
) {
    let identity = owner.with(|game| game.actor_identity().clone());
    let ram = owner.with(|game| {
        (0..game.ram().len())
            .map(|at| game.compatibility_byte(at))
            .collect::<Vec<_>>()
    });
    let reject = |rejected: &[u8], required: &SessionStartContract, error: &str| {
        assert_eq!(
            owner
                .restore_session_checkpoint(rejected, media.clone(), required)
                .err()
                .as_deref(),
            Some(error)
        );
        assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
        owner.with(|game| {
            assert_eq!(*game.actor_identity(), identity);
            assert!(
                ram.iter()
                    .enumerate()
                    .all(|(at, value)| game.compatibility_byte(at) == *value)
            );
            assert_eq!(ApuQueueSnapshot::capture(game).to_bytes(), *accepted);
        });
    };
    for sound in [false, true] {
        let mut changed = contract.requirements().clone();
        let catalog = Arc::make_mut(&mut changed.audio_catalog);
        if sound {
            catalog.sounds.insert(
                (2, 5),
                SoundMeaning::Program {
                    program: program(),
                    phase: 20,
                },
            );
        } else {
            catalog.music.insert(
                7,
                MusicMeaning::Program {
                    program: program(),
                    phase: 60,
                },
            );
        }
        reject(
            bytes,
            &SessionStartContract::new(changed),
            "audio checkpoint catalog declarations changed",
        );
    }
    assert_malformed(bytes, |rejected, error| reject(rejected, contract, error));
}

fn assert_malformed(bytes: &[u8], reject: impl Fn(&[u8], &str)) {
    let audio = unique(bytes, b"SMAUDIO\x02");
    let reference_size =
        u64::from_le_bytes(bytes[audio + 8..audio + 16].try_into().unwrap()) as usize;
    let pending = audio + 16 + reference_size;
    let mut malformed = bytes.to_vec();
    malformed[pending] = 1;
    reject(&malformed, "incomplete audio checkpoint publication");
    malformed = bytes.to_vec();
    malformed[pending + 17] ^= 1; // First independently declared music selector, after length/count.
    reject(&malformed, "audio checkpoint catalog declarations changed");
    let mut entry = vec![1, 7, 0];
    entry.extend_from_slice(&0u64.to_le_bytes());
    entry.extend_from_slice(&22u64.to_le_bytes());
    entry.push(2);
    entry.extend_from_slice(&1u32.to_le_bytes());
    entry.extend_from_slice(&[5; 32]);
    entry.extend_from_slice(&59u64.to_le_bytes());
    let entry = unique(bytes, &entry);
    for (offset, value, error) in [
        (19, 0xff, "unknown audio checkpoint music meaning"),
        (20, 2, "audio checkpoint Incomplete"),
        (3, 1, "audio checkpoint Incomplete"),
    ] {
        malformed = bytes.to_vec();
        malformed[entry + offset] = value;
        reject(&malformed, error);
    }
    let mut current = vec![1, 5];
    current.extend_from_slice(&0u64.to_le_bytes());
    current.extend_from_slice(&21u64.to_le_bytes());
    current.push(1);
    current.extend_from_slice(&1u32.to_le_bytes());
    current.extend_from_slice(&[5; 32]);
    current.extend_from_slice(&19u64.to_le_bytes());
    let current = unique(bytes, &current);
    for offset in [1, 23] {
        malformed = bytes.to_vec();
        malformed[current + offset] ^= 1;
        reject(&malformed, "audio checkpoint Identity");
    }
}

// Enter the existing owned production interval; no synthetic AudioHost or queue.
fn producing<R>(
    owner: &GameOwner,
    prepare: impl FnOnce(&mut AudioRequests),
    operation: impl FnOnce(&mut crate::game_state::Game) -> R,
) -> R {
    owner.with(|game| {
        let epoch = game.compatibility_write_epoch();
        let mut session = game.lease_persistent_session().unwrap().unwrap();
        session.begin_interval(true).unwrap();
        prepare(session.gameplay.control.requests_mut().unwrap());
        game.return_persistent_session(session, true, epoch);
        operation(game)
    })
}

fn inspect_audio(
    game: &mut crate::game_state::Game,
    inspect: impl FnOnce(&AudioRequests, &ss::InitialAudioIntent),
) {
    let epoch = game.compatibility_write_epoch();
    let session = game.lease_persistent_session().unwrap().unwrap();
    inspect(
        session.gameplay.control.requests().unwrap(),
        &session.metadata().audio,
    );
    game.return_persistent_session(session, true, epoch);
}

/// The actual owned callback consumes authored records; compatibility numbers stay separate.
fn assert_deliveries(
    owner: &GameOwner,
    bytes: &[u8],
    media: &Arc<sc::SceneMedia>,
    contract: &SessionStartContract,
    accepted: &[u8; 71],
) {
    use crate::game::music::requests::{self, MusicEffect};
    let restore = || {
        owner
            .restore_session_checkpoint(bytes, media.clone(), contract)
            .unwrap()
    };
    restore();
    producing(
        owner,
        |requests| requests.music.timer = 1,
        |game| {
            let expected = QueuedMusicMeaning {
                request: 7,
                source_tick: 0,
                source_interval: 22,
                meaning: MusicMeaning::Program {
                    program: program(),
                    phase: 59,
                },
            };
            let mut delivered = None;
            requests::advance_music(game, |effect, meaning, game| {
                assert!(matches!(effect, MusicEffect::WriteTrack(7)));
                let meaning = meaning.unwrap();
                assert_eq!(meaning, &expected);
                assert_eq!(game.audio_word(0x7f5), 7);
                game.audio_mutation(|requests, _, _| {
                    let live = requests.music.entry_meaning.as_mut().unwrap();
                    live.source_tick = 1;
                    live.source_interval = 23;
                    Ok(())
                })
                .unwrap();
                game.audio_store_word(0x7f5, 0x33);
                assert_eq!(meaning.source_tick, 0);
                assert_eq!(meaning.source_interval, 22);
                delivered = Some(meaning.clone());
                Ok(())
            })
            .unwrap();
            assert_eq!(delivered, Some(expected.clone()));
            assert_eq!(game.audio_word(0x7f5), 0x33);
            inspect_audio(game, |requests, audio| {
                assert_eq!(
                    requests.music.current_meaning,
                    Some(QueuedMusicMeaning {
                        source_tick: 1,
                        source_interval: 23,
                        ..expected
                    })
                );
                let ss::OngoingMusic::Playing(anchor) = audio.music else {
                    panic!("accepted program")
                };
                assert_eq!(
                    (
                        anchor.program,
                        anchor.revision,
                        anchor.phase,
                        anchor.source_interval
                    ),
                    (sc::AssetId(1), sc::Revision([5; 32]), 59, 23)
                );
            });
        },
    );
    restore();
    producing(
        owner,
        |requests| {
            requests.music.timer = 1;
            requests.music.entry = 0xff06;
            requests.music.entry_meaning = music(
                0xff06,
                20,
                MusicMeaning::Bank {
                    bank: sc::AssetRef {
                        id: sc::AssetId(2),
                        revision: sc::Revision([5; 32]),
                    },
                    after: MusicContinuation::Retain,
                },
            );
        },
        |game| {
            let prior_music = game
                .audio_mutation(|_, audio, _| Ok(audio.music))
                .unwrap()
                .unwrap();
            let mut delivered = None;
            requests::advance_music(game, |effect, meaning, game| {
                assert!(matches!(effect, MusicEffect::UploadBank));
                assert_eq!(game.audio_word(0x7f3), 6);
                assert_eq!(game.audio_byte(0x64c), 0xff);
                let meaning = meaning.unwrap();
                assert_eq!(
                    meaning,
                    &QueuedMusicMeaning {
                        request: 0xff06,
                        source_tick: 0,
                        source_interval: 20,
                        meaning: MusicMeaning::Bank {
                            bank: sc::AssetRef {
                                id: sc::AssetId(2),
                                revision: sc::Revision([5; 32])
                            },
                            after: MusicContinuation::Retain,
                        },
                    }
                );
                delivered = Some(meaning.clone());
                game.audio_store_word(0x7f3, 9);
                Ok(())
            })
            .unwrap();
            assert_eq!(game.audio_word(0x7f3), 9);
            assert_eq!(game.audio_byte(0x64c), 0);
            inspect_audio(game, |requests, audio| {
                let mut expected = delivered.unwrap();
                expected.source_tick = 1;
                expected.source_interval = 23;
                assert_eq!(requests.music.current_meaning, Some(expected));
                assert_eq!(audio.music, prior_music);
            });
        },
    );
    // A rejecting or unwinding typed consumer leaves the original bank-upload prefix.
    for unwind in [false, true] {
        restore();
        producing(
            owner,
            |requests| {
                requests.music.timer = 1;
                requests.music.entry = 0xff06;
                requests.music.entry_meaning = music(
                    0xff06,
                    20,
                    MusicMeaning::Bank {
                        bank: sc::AssetRef {
                            id: sc::AssetId(2),
                            revision: sc::Revision([5; 32]),
                        },
                        after: MusicContinuation::Retain,
                    },
                );
            },
            |game| {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    requests::advance_music(game, |effect, meaning, _| {
                        assert!(matches!(effect, MusicEffect::UploadBank));
                        assert_eq!(meaning.unwrap().source_interval, 20);
                        assert_eq!(meaning.unwrap().source_tick, 0);
                        if unwind {
                            panic!("typed music consumer unwind");
                        }
                        Err(DisplayError::Source)
                    })
                }));
                if unwind {
                    assert_eq!(
                        result.unwrap_err().downcast_ref::<&str>(),
                        Some(&"typed music consumer unwind")
                    );
                } else {
                    assert_eq!(result.unwrap(), Err(DisplayError::Source));
                }
                assert_eq!(game.audio_word(0x63f), 0);
                assert_eq!(game.audio_word(0x63b), 14);
                assert_eq!(game.audio_word(0x7f3), 6);
                assert_eq!(game.audio_byte(0x64c), 0xff);
                assert_eq!(ApuQueueSnapshot::capture(game).to_bytes(), *accepted);
            },
        );
        assert!(owner.encode_session_checkpoint().is_err());
    }
    restore();
    producing(
        owner,
        |requests| requests.downtime = 0,
        |game| {
            let expected = QueuedSoundMeaning {
                request: 5,
                source_tick: 0,
                source_interval: 19,
                meaning: SoundMeaning::Program {
                    program: program(),
                    phase: 19,
                },
            };
            let mut delivered = None;
            requests::advance_sound_effects(game, |channel, value, meaning, game| {
                assert_eq!((channel, value), (2, 5));
                let meaning = meaning.unwrap();
                assert_eq!(meaning, &expected);
                game.audio_mutation(|requests, _, _| {
                    let live = requests.sounds[2].meanings[15].as_mut().unwrap();
                    live.source_tick = 1;
                    live.source_interval = 23;
                    Ok(())
                })
                .unwrap();
                game.audio_store_byte(0x64b, 3);
                assert_eq!((meaning.source_tick, meaning.source_interval), (0, 19));
                delivered = Some(meaning.clone());
                Ok(())
            })
            .unwrap();
            assert_eq!(delivered, Some(expected.clone()));
            inspect_audio(game, |requests, _| {
                assert_eq!((requests.sounds[2].read, requests.sounds[2].state), (0, 4));
                assert_eq!(
                    requests.sounds[2].current_meaning,
                    Some(QueuedSoundMeaning {
                        source_tick: 1,
                        source_interval: 23,
                        ..expected
                    })
                );
            });
        },
    );
    restore();
    producing(
        owner,
        |requests| requests.downtime = 0,
        |game| {
            assert_eq!(
                requests::advance_sound_effects(game, |channel, value, meaning, _| {
                    assert_eq!((channel, value), (2, 5));
                    assert_eq!(
                        meaning.unwrap(),
                        &QueuedSoundMeaning {
                            request: 5,
                            source_tick: 0,
                            source_interval: 19,
                            meaning: SoundMeaning::Program {
                                program: program(),
                                phase: 19
                            },
                        }
                    );
                    Err(DisplayError::Source)
                }),
                Err(DisplayError::Source)
            );
            assert_eq!(game.audio_byte(0x645), 15);
            assert_eq!(game.audio_byte(0x64b), 0);
        },
    );
    assert!(owner.encode_session_checkpoint().is_err());
    restore();
    producing(
        owner,
        |_| {},
        |game| {
            let mut delivered = Vec::new();
            requests::advance_sound_effects(game, |channel, value, meaning, _| {
                assert_eq!(value, 0);
                assert_eq!(meaning, None);
                delivered.push(channel);
                Ok(())
            })
            .unwrap();
            assert_eq!(delivered, [0, 1, 2]);
            inspect_audio(game, |requests, _| {
                assert_eq!(requests.downtime, 8);
                assert!(
                    requests
                        .sounds
                        .iter()
                        .all(|sound| sound.current_meaning.is_none())
                );
            });
        },
    );
    restore();
    producing(
        owner,
        |requests| {
            requests.downtime = 0;
            requests.sounds[2].read = 0;
            requests.sounds[2].state = 0;
        },
        |game| {
            requests::advance_sound_effects(game, |channel, value, meaning, _| {
                assert_eq!((channel, value), (2, 0));
                assert_eq!(
                    meaning.unwrap(),
                    &QueuedSoundMeaning {
                        request: 0,
                        source_tick: 0,
                        source_interval: 20,
                        meaning: SoundMeaning::Stop,
                    }
                );
                Ok(())
            })
            .unwrap();
            inspect_audio(game, |requests, _| {
                assert_eq!(
                    requests.sounds[2].current_meaning,
                    Some(QueuedSoundMeaning {
                        request: 0,
                        source_tick: 1,
                        source_interval: 23,
                        meaning: SoundMeaning::Stop,
                    })
                );
            });
        },
    );
    restore();
    assert_eq!(owner.encode_session_checkpoint().unwrap(), bytes);
    owner.with(|game| assert_eq!(ApuQueueSnapshot::capture(game).to_bytes(), *accepted));
}
