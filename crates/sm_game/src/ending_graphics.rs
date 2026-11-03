//! Ending graphics: logo palette steps, icon and beam transfers, and the item percentage.
use crate::cinematic::CinematicDestination;
use crate::cinematic::cinematic_native_record;
use crate::cinematic_state::Cinematic;
use crate::imported::{Imported, grid_word, load, word as table_word};
use crate::presentation_workspace::{Point, Role};
use crate::visual_state::EndingGraphics;
use crate::word_transfers::WordEndpoint;
use sm_data::credits::{
    BEAM_BIT_ADDRESS, BEAM_DESTINATION_ADDRESS, BEAM_SOURCE_ADDRESS, CAPACITY_ADDRESS,
    CAPACITY_DIVISOR_ADDRESS, DIGIT_TILE_ADDRESS, ICON_PALETTE_ADDRESS, ICON_TRANSFER_ADDRESS,
    ITEM_BIT_ADDRESS,
};

pub fn advance_ending_logo_palette<H: crate::menus::host::Host + ?Sized>(
    cine: &mut Cinematic,
    game: &mut H,
) -> bool {
    let pair = ((*cine.function_timer()).wrapping_mul(4) >> 1) as isize;
    let step = Imported::read(
        &game.visual_tables().icon_palettes,
        game,
        |steps| steps.row(crate::imported::row_of(pair, 2)?),
        |rom| sm_data::credits::IconPalettes {
            first: load(rom, ICON_PALETTE_ADDRESS, pair),
            second: load(rom, ICON_PALETTE_ADDRESS, pair + 1),
        },
    );
    let (first, second) = (step.first, step.second);
    for (mut source, offset) in [(first, 16usize), (second, 240)] {
        for index in (0..16).rev() {
            let color = cinematic_native_record::<0x8c>(source, game).word(0);
            // The second band originally wrote native RAM words.
            let color = if offset == 16 { color } else { color.to_le() };
            game.palette_write_word(false, 2 * (offset + index), color);
            source = source.wrapping_sub(2);
        }
    }
    ({
        *cine.function_timer() = cine.function_timer().wrapping_add(1);
        *cine.function_timer()
    } as i32
        - 16)
        & 0x8000
        == 0
}

pub fn advance_ending_graphics<H: crate::menus::host::Host + ?Sized>(
    graphics: EndingGraphics,
    cine: &mut Cinematic,
    game: &mut H,
) {
    use EndingGraphics::*;
    let limit = if matches!(graphics, Samus) { 16 } else { 6 };
    if (matches!(graphics, Background) && *cine.var17() != 0)
        || (i32::from(*cine.var14()) - limit) & 0x8000 == 0
    {
        return;
    }
    let frame = *cine.var14();
    let index = match graphics {
        Background => 4 * i32::from(frame),
        Samus => i32::from(frame.wrapping_mul(2) >> 1),
    };
    let tail = game.queue_tail();
    // Each ROM word stays between the preceding queue stores; bank indices wrap separately.
    for field in [0u16, 2, 4, 5] {
        let value = match (graphics, field) {
            (Background, _) => {
                let word = match field {
                    0 => index,
                    2 => index + 1,
                    4 => i32::from(frame.wrapping_mul(4)) + 2,
                    _ => index + 3,
                } as isize;
                Imported::read(
                    &game.visual_tables().icon_transfers,
                    game,
                    |transfers| {
                        let transfer = transfers.row(usize::from(frame))?;
                        Some(match field {
                            0 => transfer.size,
                            2 => transfer.source,
                            4 => transfer.bank,
                            _ => transfer.destination,
                        })
                    },
                    |rom| load(rom, ICON_TRANSFER_ADDRESS, word),
                )
            }
            (Samus, 0) => 2048,
            (Samus, 2) => table_word(
                &game.visual_tables().beam_sources,
                game,
                BEAM_SOURCE_ADDRESS,
                index as isize,
            ),
            (Samus, 4) => 127,
            (Samus, _) => table_word(
                &game.visual_tables().beam_destinations,
                game,
                BEAM_DESTINATION_ADDRESS,
                index as isize,
            ),
        };
        let destination = tail.wrapping_add(field);
        if field == 4 {
            crate::presentation_workspace::QueueAccess::queue_byte_at(
                game,
                usize::from(destination),
                value.to_le_bytes()[0],
            );
        } else {
            game.queue_native_word(destination, value);
        }
    }
    game.store_queue_tail(tail.wrapping_add(7));
    *cine.var14() = cine.var14().wrapping_add(1);
}

pub fn draw_cinematic_item_percentage<H: crate::menus::host::Host + ?Sized>(game: &mut H) {
    crate::probe::hit("ending_graphics.item_percentage");
    let mut count = 0u16;
    for index in (0..=4).rev() {
        let address = table_word(
            &game.visual_tables().capacities,
            game,
            CAPACITY_ADDRESS,
            index,
        );
        game.assert_ram_pointer(address);
        // The capacity table names the maximum health, ammunition and
        // reserve words.
        let capacity = crate::inventory::Word::from_address(usize::from(address))
            .expect("capacity address names an inventory word");
        count = count.wrapping_add(snes_divide(
            crate::gameplay_context::inventory(game).get(capacity),
            table_word(
                &game.visual_tables().capacity_divisors,
                game,
                CAPACITY_DIVISOR_ADDRESS,
                index,
            ) as u8,
        ));
    }
    for index in (0..=10).rev() {
        if table_word(
            &game.visual_tables().item_bits,
            game,
            ITEM_BIT_ADDRESS,
            index,
        ) & crate::gameplay_context::inventory(game).equipment.owned
            != 0
        {
            count = count.wrapping_add(1);
        }
    }
    for index in (0..=4).rev() {
        if table_word(
            &game.visual_tables().beam_bits,
            game,
            BEAM_BIT_ADDRESS,
            index,
        ) & crate::gameplay_context::inventory(game).beams.owned
            != 0
        {
            count = count.wrapping_add(1);
        }
    }
    for (column, digit, visible) in [
        (462, count / 100, count >= 100),
        (463, count / 10 % 10, count >= 10),
        (464, count % 10, true),
    ] {
        if visible {
            let index = (digit.wrapping_mul(4) >> 1) as isize;
            for row in 0..2 {
                crate::cinematic::transfer_cinematic_words(
                    1,
                    2,
                    &[(
                        CinematicDestination::Screen(Point::new(
                            Role::Pause,
                            usize::try_from(2 * (column + row * 32)).unwrap(),
                        )),
                        WordEndpoint::Value(grid_word(
                            &game.visual_tables().digit_tiles,
                            game,
                            DIGIT_TILE_ADDRESS,
                            index + row,
                        )),
                    )],
                    game,
                );
            }
        }
    }
    crate::cinematic::transfer_cinematic_words(
        1,
        2,
        &[
            (
                CinematicDestination::Screen(Point::new(Role::Pause, 930)),
                WordEndpoint::Value(14442),
            ),
            (
                CinematicDestination::Screen(Point::new(Role::Pause, 994)),
                WordEndpoint::Value(14458),
            ),
        ],
        game,
    );
}

/// The original's 16-by-8 division, with its zero-divisor result.
fn snes_divide(a: u16, b: u8) -> u16 {
    if b == 0 { 0xffff } else { a / u16::from(b) }
}
