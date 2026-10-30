//! Sequential actual Rust lifecycle for the independently captured CPU fixture.
#![forbid(unsafe_code)]
use crate::{
    game::sprite_effect_runtime as runtime,
    game_state::Game,
    sprite_pool_case::{self as case, Operation},
};
pub(crate) fn capture(id: u32, game: &mut Game) -> Result<Vec<u8>, String> {
    if id != 0 {
        return Err("unknown sprite pool lifecycle case".into());
    }
    case::seed(game);
    let mut output = b"PSOR1".to_vec();
    case::record(&mut output, 0, &id.to_le_bytes());
    for index in 0..case::PHASES {
        let (operation, tick) = case::phase(index).unwrap();
        case::prepare(operation, game);
        // CPU metadata is intentionally absent from Rust semantics. The shared
        // framing reserves these16zero bytes; the comparator verifies zeros.
        case::state(
            &mut output,
            1,
            index,
            operation,
            tick,
            0xffff,
            [0; 16],
            game,
        );
        let result = match operation {
            Operation::Create => {
                let x = case::word(game, 0x12);
                let y = case::word(game, 0x14);
                let definition = case::word(game, 0x16);
                let appearance = case::word(game, 0x18);
                runtime::create_sprite(x, y, definition, appearance, game)
            }
            Operation::Update => {
                runtime::update_sprites(game);
                0xffff
            }
            Operation::Draw => {
                runtime::draw_sprites(game);
                0xffff
            }
        };
        if game.g_fail {
            return Err(format!(
                "sprite.pool.oracle.phase{index}.unexpected-diagnostic"
            ));
        }
        if operation == Operation::Create && result != 62 {
            return Err("sprite.pool.oracle.allocate-highest".into());
        }
        if operation == Operation::Draw && (case::word(game, 0x590) != 0) != (tick < 24) {
            return Err(format!("sprite.pool.oracle.phase{index}.actual-oam"));
        }
        if operation == Operation::Update && (case::word(game, 0xefb6) != 0) != (tick < 24) {
            return Err(format!("sprite.pool.oracle.phase{index}.retirement"));
        }
        case::state(
            &mut output,
            2,
            index,
            operation,
            tick,
            result,
            [0; 16],
            game,
        );
    }
    Ok(output)
}
