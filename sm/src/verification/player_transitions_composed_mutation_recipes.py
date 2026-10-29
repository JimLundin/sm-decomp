"""Composed transition DLL mutation recipes, loaded by player_transitions_dll_mutations.py."""

RECIPES = [
    {
        'name': 'transitions_compose_order',
        'fault': 'COMPOSE_ORDER',
        'selected': 'transition:3700',
        'qualification': 'exact_completed_trace',
        'edits': [
            {
                'path': 'sm/src/game/player_transitions/mod.rs',
                'before': """\
    advance_animation(host, state::AnimationStep::Gameplay);
    requests::hit_and_bomb(host);
    requests::from_blocks(host);
    resolver::resolve_transitions(host);""",
                'after': """\
    requests::hit_and_bomb(host);
    requests::from_blocks(host);
    resolver::resolve_transitions(host);
    advance_animation(host, state::AnimationStep::Gameplay);""",
            },
            {
                'path': 'sm/src/verification/player_transitions_composed.rs',
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
                    "composed {id} full RAM differs at {at:05x}: actual {:02x}, expected {:02x}",
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
                'path': 'sm/src/verification/player_transitions_composed.rs',
                'before': '            observer::scalar(3702, 1);',
                'after': '            observer::scalar(3702, u16::from(completed_matches));',
            },
        ],
        'expected': 'Normal return; timer25 instead of11, detects phase order.',
    },
    {
        'name': 'transitions_compose_stale_request',
        'fault': 'COMPOSE_STALE_REQUEST',
        'selected': 'transition:3702',
        'qualification': 'exact_completed_trace',
        'edits': [
            {
                'path': 'sm/src/game/player_transitions/mod.rs',
                'before': '    advance_animation(host, state::AnimationStep::Gameplay);',
                'after': """\
    let stale = host.session().player.transitions.interrupted;
    advance_animation(host, state::AnimationStep::Gameplay);
    host.session_mut().player.transitions.interrupted = stale;""",
            },
            {
                'path': 'sm/src/verification/player_transitions_composed.rs',
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
                    "composed {id} full RAM differs at {at:05x}: actual {:02x}, expected {:02x}",
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
                'path': 'sm/src/verification/player_transitions_composed.rs',
                'before': '            observer::scalar(3702, 1);',
                'after': '            observer::scalar(3702, u16::from(completed_matches));',
            },
        ],
        'expected': 'Normal return; lost reimported interrupted37 keeps pose2 and old history instead of pose3/new history.',
    },
    {
        'name': 'transitions_compose_history',
        'fault': 'COMPOSE_HISTORY',
        'selected': 'transition:3700',
        'qualification': 'exact_completed_trace',
        'edits': [
            {
                'path': 'sm/src/game/player_transitions/pose.rs',
                'before': '    s.player.history.last_different = s.player.history.previous.clone();',
                'after': '    s.player.history.last_different = s.player.history.last_different.clone();',
            },
            {
                'path': 'sm/src/verification/player_transitions_composed.rs',
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
                    "composed {id} full RAM differs at {at:05x}: actual {:02x}, expected {:02x}",
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
                'path': 'sm/src/verification/player_transitions_composed.rs',
                'before': '            observer::scalar(3702, 1);',
                'after': '            observer::scalar(3702, u16::from(completed_matches));',
            },
        ],
        'expected': 'Normal return; last history stays5678/9abc instead of3/4.',
    },
]
