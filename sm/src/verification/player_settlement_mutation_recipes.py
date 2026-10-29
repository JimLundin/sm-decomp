"""Settlement DLL mutation recipes, loaded by player_transitions_dll_mutations.py."""

RECIPES = [
    {
        'name': 'transitions_settlement_early_consume',
        'fault': 'SETTLEMENT_EARLY_CONSUME',
        'selected': 'transition:3900',
        'qualification': 'exact_completed_trace',
        'edits': [
            {
                'path': 'sm/src/game/player_update/mod.rs',
                'before': """\
        transitions::finish_movement_and_palette(h);
        if mode == FrameBetaMode::Gameplay {
            // Gamma and settlement effects refresh this observation on return.
            let frozen = h.inputs().frozen;
            return samus_periodic_damage::apply(h.session_mut(), frozen);
        }
""",
                'after': """\
        if mode == FrameBetaMode::Gameplay {
            // Gamma and settlement effects refresh this observation on return.
            let frozen = h.inputs().frozen;
            let result = samus_periodic_damage::apply(h.session_mut(), frozen);
            transitions::finish_movement_and_palette(h);
            return result;
        }
        transitions::finish_movement_and_palette(h);
""",
            },
            {
                'path': 'sm/src/verification/player_settlement.rs',
                'before': """\
            if (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .ne(expected.iter().copied())
            {
                let at = (0..game.ram().len())
                    .map(|at| game.ram().read_byte(at))
                    .zip(&expected)
                    .position(|(a, b)| a != *b)
                    .unwrap();
                return Err(format!(
                    "settlement {id} RAM mismatch at {at:05x}: {:02x} != {:02x}",
                    game.ram().read_byte(at),
                    expected[at]
                ));
            }
""",
                'after': """\
            let completed_matches = (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .eq(expected.iter().copied());
""",
            },
            {
                'path': 'sm/src/verification/player_settlement.rs',
                'before': '            observer::scalar(3903, 1);',
                'after': '            observer::scalar(3903, u16::from(completed_matches));',
            },
        ],
        'expected': 'Normal return; consume seeded damage before actual acid producer, leaves generated damage pending.',
    },
    {
        'name': 'transitions_settlement_lost_fraction',
        'fault': 'SETTLEMENT_LOST_FRACTION',
        'selected': 'transition:3900',
        'qualification': 'exact_completed_trace',
        'edits': [
            {
                'path': 'sm/src/game/samus_periodic_damage.rs',
                'before': '    session.inventory.health_fraction = u16::from_le_bytes((health as u16).to_ne_bytes());',
                'after': '    let _ = u16::from_le_bytes((health as u16).to_ne_bytes());',
            },
            {
                'path': 'sm/src/verification/player_settlement.rs',
                'before': """\
            if (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .ne(expected.iter().copied())
            {
                let at = (0..game.ram().len())
                    .map(|at| game.ram().read_byte(at))
                    .zip(&expected)
                    .position(|(a, b)| a != *b)
                    .unwrap();
                return Err(format!(
                    "settlement {id} RAM mismatch at {at:05x}: {:02x} != {:02x}",
                    game.ram().read_byte(at),
                    expected[at]
                ));
            }
""",
                'after': """\
            let completed_matches = (0..game.ram().len())
                .map(|at| game.ram().read_byte(at))
                .eq(expected.iter().copied());
""",
            },
            {
                'path': 'sm/src/verification/player_settlement.rs',
                'before': '            observer::scalar(3903, 1);',
                'after': '            observer::scalar(3903, u16::from(completed_matches));',
            },
        ],
        'expected': 'Normal return; computed whole health97 with original fraction2000 instead of9fff.',
    },
]
