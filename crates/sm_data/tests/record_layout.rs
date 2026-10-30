//! Each imported record's layout: importing a synthetic byte buffer and laying
//! every field back out at the offset written here reproduces the buffer, and
//! a buffer of any other length is rejected.
//!
//! The offsets are spelled out independently of each importer, so a shifted
//! offset, two swapped fields, a row-major/column-major mix-up or a dropped
//! word breaks the comparison. No ROM is read.
use std::collections::BTreeMap;

use sm_data::*;

/// A buffer of `len` bytes in which every little-endian word differs from
/// every other, and none is $FFFF (the terminator some records use).
fn source(len: usize, seed: u16) -> Vec<u8> {
    (0..len.div_ceil(2))
        .flat_map(|k| {
            let word = (k as u16).wrapping_mul(0x9e37).wrapping_add(seed) & 0x7fff;
            word.to_le_bytes()
        })
        .take(len)
        .collect()
}

fn word_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}

/// Compare words placed at byte offsets with `source`: every placed word must
/// match, no byte may be placed twice, and every byte outside `gaps` must be
/// placed.
fn check(name: &str, source: &[u8], placed: &[(usize, u16)], gaps: &[usize]) {
    let mut claimed = BTreeMap::new();
    for &(at, word) in placed {
        assert!(
            at + 1 < source.len(),
            "{name}: word at {at:#x} is past the record"
        );
        assert_eq!(word, word_at(source, at), "{name}: word at {at:#x}");
        for byte in [at, at + 1] {
            assert!(
                claimed.insert(byte, at).is_none(),
                "{name}: byte {byte:#x} placed twice"
            );
        }
    }
    for byte in 0..source.len() {
        if !gaps.contains(&byte) {
            assert!(
                claimed.contains_key(&byte),
                "{name}: byte {byte:#x} is in no field"
            );
        }
    }
}

#[derive(Clone, Copy)]
enum Layout {
    /// Row `r`'s field `c` is word `fields * r + c`.
    Rows,
    /// Field `c` of row `r` is word `rows * c + r`: one list per field.
    Columns,
}

/// Lay out `rows` rows of a row-based record and check them against `source`;
/// row `rows` must be absent.
fn rows<T>(
    name: &str,
    source: &[u8],
    rows: usize,
    layout: Layout,
    get: impl Fn(usize) -> Option<T>,
    fields: impl Fn(T) -> Vec<u16>,
) {
    let mut placed = Vec::new();
    for row in 0..rows {
        let words = fields(get(row).unwrap_or_else(|| panic!("{name}: row {row} missing")));
        let count = words.len();
        for (column, word) in words.into_iter().enumerate() {
            let index = match layout {
                Layout::Rows => count * row + column,
                Layout::Columns => rows * column + row,
            };
            placed.push((2 * index, word));
        }
    }
    assert!(get(rows).is_none(), "{name}: row {rows} past the record");
    check(name, source, &placed, &[]);
}

/// The importer rejects buffers two bytes short or long, and an empty one.
fn rejects_other_lengths<R, E: PartialEq + std::fmt::Debug>(
    name: &str,
    len: usize,
    import: impl Fn(&[u8]) -> Result<R, E>,
    error: impl Fn(usize) -> E,
) {
    for other in [0, len - 2, len + 2] {
        let bytes = source(other, 1);
        assert_eq!(
            import(&bytes).err(),
            Some(error(other)),
            "{name}: {other} bytes"
        );
    }
}

/// A one-word-per-row table: `import`, then check `get` across `count` rows.
macro_rules! word_table {
    ($name:literal, $count:expr, $import:expr, $get:expr) => {{
        let bytes = source(2 * $count, 0x11);
        let record = $import(&bytes).expect($name);
        rows(
            $name,
            &bytes,
            $count,
            Layout::Rows,
            |row| $get(&record, row),
            |word| vec![word],
        );
        rejects_other_lengths($name, 2 * $count, $import, table::LengthError);
    }};
}

#[test]
fn table_words_and_grids_lay_out_row_major() {
    let bytes = source(2 * 9, 3);
    let words = table::words::<9>(&bytes).unwrap();
    rows(
        "table::words",
        &bytes,
        9,
        Layout::Rows,
        |r| words.row(r),
        |w| vec![w],
    );
    rejects_other_lengths("table::words", 18, table::words::<9>, table::LengthError);

    let bytes = source(2 * 3 * 5, 4);
    let grid = table::grid::<3, 5>(&bytes).unwrap();
    rows(
        "table::grid",
        &bytes,
        5,
        Layout::Rows,
        |r| grid.row(r),
        |row| row.to_vec(),
    );
    for index in 0..15 {
        assert_eq!(
            grid.word(index),
            Some(word_at(&bytes, 2 * index)),
            "grid word {index}"
        );
    }
    assert_eq!(grid.word(15), None, "grid word past the table");
    rejects_other_lengths("table::grid", 30, table::grid::<3, 5>, table::LengthError);
}

#[test]
fn single_word_tables_round_trip() {
    word_table!(
        "beam_gradients",
        beam_gradients::ANGLES,
        beam_gradients::import,
        |t: &beam_gradients::BeamGradients, r| t.row(r)
    );
    word_table!(
        "hdma_objects::bg3_x_velocities",
        hdma_objects::BG3_X_VELOCITIES,
        hdma_objects::import_bg3_x_velocities,
        |t: &hdma_objects::Bg3XVelocities, r| t.row(r)
    );
    word_table!(
        "title_screen::gradients",
        title_screen::GRADIENTS,
        title_screen::import_gradients,
        |t: &title_screen::Gradients, r| t.row(r)
    );
    word_table!(
        "shooting_stars::tiles",
        shooting_stars::TILE_STEPS,
        shooting_stars::import_tiles,
        |t: &shooting_stars::Tiles, r| t.row(r)
    );
    word_table!(
        "mother_brain::health_palettes",
        mother_brain::HEALTH_LEVELS,
        mother_brain::import_health_palettes,
        |t: &mother_brain::HealthPalettes, r| t.row(r)
    );
    word_table!(
        "mother_brain::rainbow",
        mother_brain::RAINBOW_WORDS,
        mother_brain::import_rainbow,
        |t: &mother_brain::RainbowColors, r| t.row(r)
    );
    word_table!(
        "credits::words",
        credits::CAPACITIES,
        credits::import_words::<{ credits::CAPACITIES }>,
        |t: &credits::Capacities, r| t.row(r)
    );
    word_table!(
        "crocomire::words",
        crocomire::SCROLL_SPRITEMAPS,
        crocomire::import_words::<{ crocomire::SCROLL_SPRITEMAPS }>,
        |t: &crocomire::ScrollSpritemaps, r| t.row(r)
    );
    word_table!(
        "equipment_screen::words",
        equipment_screen::ITEMS,
        equipment_screen::import_words::<{ equipment_screen::ITEMS }>,
        |t: &equipment_screen::ItemBitmasks, r| t.row(r)
    );
    word_table!(
        "pause_menu::words",
        pause_menu::AREA_MAPS,
        pause_menu::import_words::<{ pause_menu::AREA_MAPS }>,
        |t: &table::Rows<u16, { pause_menu::AREA_MAPS }>, r| t.row(r)
    );
}

#[test]
fn single_field_records_round_trip() {
    word_table!(
        "baby_metroid::health_palettes",
        baby_metroid::HEALTH_PALETTE_ROWS,
        baby_metroid::import_health_palettes,
        |t: &baby_metroid::HealthPalettes, r| t.row(r).map(|p| p.colors)
    );
    word_table!(
        "baby_metroid::fade_palettes",
        baby_metroid::FADE_PALETTE_ROWS,
        baby_metroid::import_fade_palettes,
        |t: &baby_metroid::FadePalettes, r| t.row(r).map(|p| p.colors)
    );
    word_table!(
        "mother_brain::brain_attacks",
        mother_brain::BRAIN_ATTACK_ROWS,
        mother_brain::import_brain_attacks,
        |t: &mother_brain::BrainAttacks, r| t.row(r).map(|a| a.instruction_list)
    );
    word_table!(
        "mother_brain::brain_palette_sets",
        mother_brain::BRAIN_PALETTE_SET_ROWS,
        mother_brain::import_brain_palette_sets,
        |t: &mother_brain::BrainPaletteSets, r| t.row(r).map(|p| p.colors)
    );
    word_table!(
        "mother_brain::walks",
        mother_brain::WALK_ROWS,
        mother_brain::import_walks,
        |t: &mother_brain::Walks, r| t.row(r).map(|w| w.instruction_list)
    );
    word_table!(
        "mother_brain::falling_tubes",
        mother_brain::FALLING_TUBE_ROWS,
        mother_brain::import_falling_tubes,
        |t: &mother_brain::FallingTubes, r| t.row(r).map(|f| f.function)
    );
    word_table!(
        "mother_brain::rainbow_beam_palettes",
        mother_brain::RAINBOW_BEAM_PALETTE_ROWS,
        mother_brain::import_rainbow_beam_palettes,
        |t: &mother_brain::RainbowBeamPalettes, r| t.row(r).map(|p| p.colors)
    );
    word_table!(
        "shaktool::head_directions",
        shaktool::HEAD_DIRECTION_ROWS,
        shaktool::import_head_directions,
        |t: &shaktool::HeadDirections, r| t.row(r).map(|h| h.instruction_list)
    );
    word_table!(
        "shaktool::trig",
        shaktool::ANGLES,
        shaktool::import_trig,
        |t: &shaktool::TrigTable, r| t.row(r).map(|v| v.value)
    );
    word_table!(
        "statues::tourian",
        statues::TOURIAN_STATUE_ROWS,
        statues::import_tourian_statues,
        |t: &statues::TourianStatues, r| t.row(r).map(|s| s.instruction_list)
    );
    word_table!(
        "statues::chozo",
        statues::CHOZO_STATUE_ROWS,
        statues::import_chozo_statues,
        |t: &statues::ChozoStatues, r| t.row(r).map(|s| s.instruction_list)
    );
    word_table!(
        "torizo::torizos",
        torizo::TORIZO_ROWS,
        torizo::import_torizos,
        |t: &torizo::Torizos, r| t.row(r).map(|s| s.instruction_list)
    );
}

/// A multi-field row record: `import`, then lay its rows out with `fields`.
macro_rules! row_record {
    ($name:literal, $rows:expr, $bytes:expr, $layout:expr, $import:expr, $fields:expr) => {{
        let bytes = source($bytes, 0x23);
        let record = $import(&bytes).expect($name);
        rows(
            $name,
            &bytes,
            $rows,
            $layout,
            |row| record.row(row),
            $fields,
        );
        rejects_other_lengths($name, $bytes, $import, table::LengthError);
    }};
}

#[test]
fn row_major_records_round_trip() {
    use Layout::Rows as R;
    row_record!(
        "credits::icon_transfers",
        credits::ICON_TRANSFERS,
        credits::ICON_TRANSFER_BYTES,
        R,
        credits::import_icon_transfers,
        |t: credits::VramTransfer| vec![t.size, t.source, t.bank, t.destination]
    );
    row_record!(
        "credits::icon_palettes",
        credits::ICON_PALETTE_STEPS,
        credits::ICON_PALETTE_BYTES,
        R,
        credits::import_icon_palettes,
        |p: credits::IconPalettes| vec![p.first, p.second]
    );
    row_record!(
        "credits::digit_tiles",
        credits::DIGITS,
        4 * credits::DIGITS,
        R,
        credits::import_digit_tiles,
        |tiles: [u16; 2]| tiles.to_vec()
    );
    row_record!(
        "grapple_aim::connections",
        3 * grapple_aim::DIRECTIONS,
        grapple_aim::CONNECTION_BYTES,
        R,
        |b: &[u8]| grapple_aim::Connections::import(b).map(ConnectionRows),
        |c: grapple_aim::Connection| vec![c.beam_function, c.handler]
    );
    row_record!(
        "kraid::body_hitbox",
        kraid::BODY_HITBOX_ROWS,
        kraid::BODY_HITBOX_BYTES,
        R,
        kraid::import_body_hitbox,
        |b: kraid::BodyHitboxBand| vec![b.bottom, b.left]
    );
    row_record!(
        "kraid::fingernail_boundaries",
        kraid::FINGERNAIL_BOUNDARY_ROWS,
        kraid::FINGERNAIL_BOUNDARY_BYTES,
        R,
        kraid::import_fingernail_boundaries,
        |b: kraid::FingernailBoundary| vec![b.left, b.top]
    );
    row_record!(
        "kraid::foot_targets",
        kraid::FOOT_TARGET_ROWS,
        kraid::FOOT_TARGET_BYTES,
        R,
        kraid::import_foot_targets,
        |t: kraid::FootTarget| vec![t.x_position, t.movement]
    );
    row_record!(
        "kraid::bg_palettes",
        kraid::BG_PALETTE_ROWS,
        kraid::BG_PALETTE_BYTES,
        R,
        kraid::import_bg_palettes,
        |p: kraid::Palette| p.colors.to_vec()
    );
    row_record!(
        "kraid::sprite_palettes",
        kraid::SPRITE_PALETTE_ROWS,
        kraid::SPRITE_PALETTE_BYTES,
        R,
        kraid::import_sprite_palettes,
        |p: kraid::Palette| p.colors.to_vec()
    );
    row_record!(
        "magdollite::palettes",
        magdollite::PALETTES,
        magdollite::PALETTE_BYTES,
        R,
        magdollite::import_palettes,
        |colors: magdollite::Palette| colors.to_vec()
    );
    row_record!(
        "mother_brain::fake_death_explosions",
        mother_brain::FAKE_DEATH_EXPLOSION_ROWS,
        mother_brain::FAKE_DEATH_EXPLOSION_BYTES,
        R,
        mother_brain::import_fake_death_explosions,
        |p: mother_brain::ExplosionPosition| vec![p.x, p.y]
    );
    row_record!(
        "pause_menu::palette_animation",
        pause_menu::PALETTE_FRAMES,
        pause_menu::PALETTE_ANIMATION_BYTES,
        R,
        pause_menu::import_palette_animation,
        |colors: pause_menu::Palette| colors.to_vec()
    );
    row_record!(
        "torizo_corpse::tile_transfers",
        torizo_corpse::TRANSFER_ROWS,
        torizo_corpse::TRANSFER_BYTES,
        R,
        torizo_corpse::import_tile_transfers,
        |t: torizo_corpse::TileTransfer| vec![
            t.size,
            t.source_bank,
            t.source_address,
            t.vram_address
        ]
    );
}

/// `Connections` exposes rows through `(set, direction)`; this adapter gives
/// the row-index view the checker walks, in the importer's set order.
struct ConnectionRows(grapple_aim::Connections);
impl ConnectionRows {
    fn row(&self, index: usize) -> Option<grapple_aim::Connection> {
        use grapple_aim::{ConnectionSet, DIRECTIONS};
        let set = match index / DIRECTIONS {
            0 => ConnectionSet::Standard,
            1 => ConnectionSet::Airborne,
            2 => ConnectionSet::Crouching,
            _ => return None,
        };
        self.0.direction(set, index % DIRECTIONS)
    }
}

#[test]
fn column_major_records_round_trip() {
    use Layout::Columns as C;
    row_record!(
        "ceres_doors::steam",
        ceres_doors::STEAM_KINDS,
        ceres_doors::STEAM_BYTES,
        C,
        ceres_doors::import_steam,
        |k: ceres_doors::SteamKind| vec![k.instruction_list, k.function]
    );
    row_record!(
        "corpses::hops",
        corpses::HOP_ROWS,
        corpses::HOP_BYTES,
        C,
        corpses::import_hops,
        |h: corpses::Hop| vec![h.y_velocity, h.x_velocity]
    );
    row_record!(
        "corpses::variants",
        corpses::GEEMER_ROWS,
        4 * corpses::GEEMER_ROWS,
        C,
        corpses::import_variants::<{ corpses::GEEMER_ROWS }>,
        |v: corpses::CorpseVariant| vec![v.instruction_list, v.rotting]
    );
    row_record!(
        "ridley::tail_angles",
        ridley::FACINGS,
        ridley::TAIL_ANGLE_BYTES,
        C,
        ridley::import_tail_angles,
        |a: ridley::TailAngleLimits| vec![a.min_clockwise, a.max_anticlockwise]
    );
    row_record!(
        "ridley_explosion::part",
        ridley_explosion::PART_SIDES,
        ridley_explosion::PART_BYTES,
        C,
        ridley_explosion::import_part,
        |p: ridley_explosion::PartPlacement| vec![p.x_offset, p.instruction_list]
    );
    row_record!(
        "shaktool::pieces",
        shaktool::PIECE_ROWS,
        shaktool::PIECE_BYTES,
        C,
        shaktool::import_pieces,
        |p: shaktool::Piece| vec![
            p.properties,
            p.ram_offset,
            p.neighbour_angle,
            p.instruction_list,
            p.layer,
            p.function,
            p.curling_delta,
            p.zero,
            p.unused,
            p.head_bob
        ]
    );
    row_record!(
        "statues::movements",
        statues::MOVEMENT_ROWS,
        statues::MOVEMENT_BYTES,
        C,
        statues::import_movements,
        |m: statues::Movement| vec![m.x_velocity, m.samus_x_offset, m.samus_y_offset]
    );
    row_record!(
        "torizo_corpse::sand_heap_lines",
        torizo_corpse::SAND_HEAP_LINE_ROWS,
        torizo_corpse::SAND_HEAP_LINE_BYTES,
        C,
        torizo_corpse::import_sand_heap_lines,
        |l: torizo_corpse::SandHeapLine| vec![l.destination, l.source]
    );
    row_record!(
        "zebetites::placements",
        zebetites::PLACEMENTS,
        zebetites::PLACEMENT_BYTES,
        C,
        zebetites::import_placements,
        |p: zebetites::Placement| vec![
            p.multi_part,
            p.height,
            p.instruction_list,
            p.x_position,
            p.y_position,
            p.y_position_alternate
        ]
    );
    row_record!(
        "zebetites::health_lists",
        zebetites::HEALTH_STAGES,
        zebetites::HEALTH_LIST_BYTES,
        C,
        zebetites::import_health_lists,
        |h: zebetites::HealthLists| vec![h.big, h.small_pair]
    );

    // The Shaktool piece columns start where the bank log puts each list.
    let list = |column: u32| shaktool::PIECE_ADDRESS + column * 2 * shaktool::PIECE_ROWS as u32;
    assert_eq!(
        shaktool::RAM_OFFSET_ADDRESS,
        list(1),
        "Shaktool RAM offset list"
    );
    assert_eq!(
        shaktool::HEAD_BOB_ADDRESS,
        list(9),
        "Shaktool head bob list"
    );
}

#[test]
fn phantoon_flame_rain_skips_each_rows_fourth_word() {
    let bytes = source(phantoon::FLAME_RAIN_BYTES, 0x31);
    let record = phantoon::import_flame_rain(&bytes).unwrap();
    let mut placed = Vec::new();
    for row in 0..phantoon::FLAME_RAIN_ROWS {
        let p = record.row(row).unwrap();
        placed.extend([
            (8 * row, p.figure_eight_index),
            (8 * row + 2, p.x_position),
            (8 * row + 4, p.y_position),
        ]);
    }
    assert!(record.row(phantoon::FLAME_RAIN_ROWS).is_none());
    let unused: Vec<usize> = (0..phantoon::FLAME_RAIN_ROWS)
        .flat_map(|row| [8 * row + 6, 8 * row + 7])
        .collect();
    check("phantoon::flame_rain", &bytes, &placed, &unused);
    rejects_other_lengths(
        "phantoon::flame_rain",
        phantoon::FLAME_RAIN_BYTES,
        phantoon::import_flame_rain,
        table::LengthError,
    );
}

#[test]
fn phantoon_timers_round_trip() {
    use phantoon::Timer;
    let bytes = source(phantoon::TIMERS_BYTES, 0x37);
    let timers = phantoon::Timers::import(&bytes).unwrap();
    let mut placed = Vec::new();
    for (timer, base) in [
        (Timer::FigureEightVulnerable, 0),
        (Timer::EyeClosed, 0x12),
        (Timer::FlameRainHiding, 0x22),
    ] {
        for index in 0..8 {
            placed.push((base + 2 * index, timers.frames(timer, index).unwrap()));
        }
        assert_eq!(timers.frames(timer, 8), None);
    }
    check("phantoon::timers", &bytes, &placed, &[0x10, 0x11]);
    rejects_other_lengths(
        "phantoon::timers",
        phantoon::TIMERS_BYTES,
        phantoon::Timers::import,
        table::LengthError,
    );
}

/// A record read through `get(index)` whose fields lie at `stride * index +
/// 2 * field`, for the importers that keep their own fixed arrays.
fn strided<T>(
    name: &str,
    source: &[u8],
    count: usize,
    stride: usize,
    get: impl Fn(usize) -> Option<T>,
    fields: impl Fn(T) -> Vec<u16>,
) {
    let mut placed = Vec::new();
    for index in 0..count {
        let words = fields(get(index).unwrap_or_else(|| panic!("{name}: entry {index} missing")));
        for (field, word) in words.into_iter().enumerate() {
            placed.push((stride * index + 2 * field, word));
        }
    }
    assert!(
        get(count).is_none(),
        "{name}: entry {count} past the record"
    );
    check(name, source, &placed, &[]);
}

#[test]
fn fixed_array_records_round_trip() {
    let bytes = source(botwoon_holes::BYTES, 0x41);
    let holes = botwoon_holes::BotwoonHoles::import(&bytes).unwrap();
    strided(
        "botwoon_holes",
        &bytes,
        botwoon_holes::HOLES,
        8,
        |i| holes.hole(i),
        |h| vec![h.left, h.right, h.top, h.bottom],
    );
    rejects_other_lengths(
        "botwoon_holes",
        botwoon_holes::BYTES,
        botwoon_holes::BotwoonHoles::import,
        botwoon_holes::LengthError,
    );

    let bytes = source(botwoon_speeds::BYTES, 0x43);
    let speeds = botwoon_speeds::BotwoonSpeeds::import(&bytes).unwrap();
    strided(
        "botwoon_speeds",
        &bytes,
        botwoon_speeds::LEVELS,
        4,
        |i| speeds.level(i),
        |s| vec![s.speed, s.body_travel_time],
    );
    rejects_other_lengths(
        "botwoon_speeds",
        botwoon_speeds::BYTES,
        botwoon_speeds::BotwoonSpeeds::import,
        botwoon_speeds::LengthError,
    );

    let bytes = source(ceres_elevator::BYTES, 0x47);
    let rotation = ceres_elevator::Rotation::import(&bytes).unwrap();
    strided(
        "ceres_elevator",
        &bytes,
        ceres_elevator::STEPS,
        6,
        |i| rotation.step(i),
        |s| vec![s.frames, s.mode7_b, s.mode7_a],
    );
    rejects_other_lengths(
        "ceres_elevator",
        ceres_elevator::BYTES,
        ceres_elevator::Rotation::import,
        ceres_elevator::LengthError,
    );

    let bytes = source(shooting_stars::BYTES, 0x49);
    let stars = shooting_stars::ShootingStars::import(&bytes).unwrap();
    strided(
        "shooting_stars",
        &bytes,
        shooting_stars::STARS,
        8,
        |i| stars.star(i),
        |s| {
            vec![
                s.x_acceleration,
                s.y_acceleration,
                s.frame_delay,
                s.start_delay,
            ]
        },
    );
    rejects_other_lengths(
        "shooting_stars",
        shooting_stars::BYTES,
        shooting_stars::ShootingStars::import,
        shooting_stars::LengthError,
    );

    let bytes = source(save_slots::BYTES, 0x4b);
    let slots = save_slots::SaveSlots::import(&bytes).unwrap();
    strided(
        "save_slots",
        &bytes,
        save_slots::SLOTS,
        2,
        |i| slots.offset(i),
        |o| vec![o],
    );
    rejects_other_lengths(
        "save_slots",
        save_slots::BYTES,
        save_slots::SaveSlots::import,
        save_slots::LengthError,
    );

    let bytes = source(intro_palette::BYTES, 0x4d);
    let palette = intro_palette::IntroPalette::import(&bytes).unwrap();
    strided(
        "intro_palette",
        &bytes,
        intro_palette::COLORS,
        2,
        |i| palette.color(i),
        |c| vec![c],
    );
    rejects_other_lengths(
        "intro_palette",
        intro_palette::BYTES,
        intro_palette::IntroPalette::import,
        intro_palette::LengthError,
    );

    let bytes = source(demo_rooms::SETS_BYTES, 0x4f);
    let sets = demo_rooms::DemoSets::import(&bytes).unwrap();
    strided(
        "demo_rooms::sets",
        &bytes,
        demo_rooms::SETS,
        2,
        |i| sets.start(i),
        |s| vec![s],
    );
    rejects_other_lengths(
        "demo_rooms::sets",
        demo_rooms::SETS_BYTES,
        demo_rooms::DemoSets::import,
        demo_rooms::ImportError::Length,
    );

    let bytes = source(sand_heap::BYTES, 0x51);
    let tiles = sand_heap::SandHeapTiles::import(&bytes).unwrap();
    strided(
        "sand_heap",
        &bytes,
        sand_heap::BYTES / 2,
        2,
        |i| tiles.word(i),
        |w| vec![w],
    );
    rejects_other_lengths(
        "sand_heap",
        sand_heap::BYTES,
        sand_heap::SandHeapTiles::import,
        sand_heap::LengthError,
    );
}

/// A record whose fields come from one list per field: field `f` of entry
/// `i` is word `count * f + i`.
fn columns<T>(
    name: &str,
    source: &[u8],
    count: usize,
    get: impl Fn(usize) -> Option<T>,
    fields: impl Fn(T) -> Vec<u16>,
) {
    rows(name, source, count, Layout::Columns, get, fields);
}

#[test]
fn column_list_records_round_trip() {
    let bytes = source(equipment_screen::CATEGORY_BYTES, 0x53);
    let categories = equipment_screen::Categories::import(&bytes).unwrap();
    columns(
        "equipment_screen::categories",
        &bytes,
        equipment_screen::CATEGORIES,
        |i| categories.category(i),
        |c| {
            vec![
                c.tilemap_offsets,
                c.bitmasks,
                c.equipped_address,
                c.tilemaps,
            ]
        },
    );
    rejects_other_lengths(
        "equipment_screen::categories",
        equipment_screen::CATEGORY_BYTES,
        equipment_screen::Categories::import,
        table::LengthError,
    );

    let bytes = source(escape_etecoons::BYTES, 0x55);
    let etecoons = escape_etecoons::EscapeEtecoons::import(&bytes).unwrap();
    columns(
        "escape_etecoons",
        &bytes,
        escape_etecoons::ETECOONS,
        |i| etecoons.etecoon(i),
        |e| {
            vec![
                e.x_position,
                e.y_position,
                e.function,
                e.instruction_list,
                e.x_velocity,
            ]
        },
    );
    rejects_other_lengths(
        "escape_etecoons",
        escape_etecoons::BYTES,
        escape_etecoons::EscapeEtecoons::import,
        escape_etecoons::LengthError,
    );

    let bytes = source(magdollite::ARM_BYTES, 0x57);
    let arm = magdollite::ArmSteps::import(&bytes).unwrap();
    columns(
        "magdollite::arm_steps",
        &bytes,
        magdollite::ARM_STEPS,
        |i| arm.step(i),
        |s| vec![s.growth, s.instruction_list, s.y_offset],
    );
    rejects_other_lengths(
        "magdollite::arm_steps",
        magdollite::ARM_BYTES,
        magdollite::ArmSteps::import,
        table::LengthError,
    );
    assert_eq!(
        [
            magdollite::ARM_GROWTH_ADDRESS,
            magdollite::ARM_INSTRUCTION_ADDRESS,
            magdollite::ARM_Y_OFFSET_ADDRESS
        ],
        [0, 1, 2]
            .map(|list| magdollite::ARM_GROWTH_ADDRESS + (2 * magdollite::ARM_STEPS * list) as u32),
        "Magdollite arm lists sit one after another"
    );
}

#[test]
fn ceres_ridley_getaway_round_trips() {
    use ceres_ridley::{GETAWAY_BYTES, GETAWAY_STEPS};
    let bytes = source(GETAWAY_BYTES, 0x59);
    let getaway = ceres_ridley::Getaway::import(&bytes).unwrap();
    let zoom = 2 * (GETAWAY_STEPS + 1);
    let mut placed = Vec::new();
    for step in 0..=GETAWAY_STEPS {
        placed.push((2 * step, getaway.zoom(step).unwrap()));
    }
    assert_eq!(getaway.zoom(GETAWAY_STEPS + 1), None);
    for step in 0..GETAWAY_STEPS {
        let velocity = getaway.velocity(step).unwrap();
        placed.push((zoom + 2 * step, velocity.y));
        placed.push((zoom + 2 * (GETAWAY_STEPS + step), velocity.x));
    }
    assert_eq!(getaway.velocity(GETAWAY_STEPS), None);
    check("ceres_ridley::getaway", &bytes, &placed, &[]);
    rejects_other_lengths(
        "ceres_ridley::getaway",
        GETAWAY_BYTES,
        ceres_ridley::Getaway::import,
        table::LengthError,
    );
}

#[test]
fn grapple_records_round_trip() {
    use grapple_aim::{DIRECTIONS, EXTENSION_BYTES};
    let bytes = source(EXTENSION_BYTES, 0x5b);
    let extensions = grapple_aim::Extensions::import(&bytes).unwrap();
    let mut placed = Vec::new();
    for direction in 0..DIRECTIONS {
        let e = extensions.direction(direction).unwrap();
        placed.extend([
            (2 * direction, e.x_velocity),
            (0x14 + 2 * direction, e.y_velocity),
            (0x29 + 2 * direction, e.angle),
        ]);
    }
    assert!(extensions.direction(DIRECTIONS).is_none());
    check("grapple_aim::extensions", &bytes, &placed, &[0x28]);
    for column in 0..3 {
        assert_eq!(
            grapple_aim::Extension::column_offset(column),
            [0, 0x14, 0x29][column],
            "extension column {column}"
        );
    }
    rejects_other_lengths(
        "grapple_aim::extensions",
        EXTENSION_BYTES,
        grapple_aim::Extensions::import,
        table::LengthError,
    );

    let bytes = source(grapple_aim::POINT_ANIMATION_BYTES, 0x5d);
    let animation = grapple_aim::PointAnimation::import(&bytes).unwrap();
    check(
        "grapple_aim::point_animation",
        &bytes,
        &[(0, animation.first), (2, animation.end)],
        &[],
    );
    rejects_other_lengths(
        "grapple_aim::point_animation",
        4,
        grapple_aim::PointAnimation::import,
        table::LengthError,
    );

    use grapple_origins::{BYTES, OriginSet};
    let bytes = source(BYTES, 0x5f);
    let origins = grapple_origins::GrappleOrigins::import(&bytes).unwrap();
    let mut placed = Vec::new();
    for (set, base) in [(OriginSet::Standard, 0), (OriginSet::Running, 0x50)] {
        for direction in 0..grapple_origins::DIRECTIONS {
            let o = origins.origin(set, direction).unwrap();
            for (column, word) in [o.x, o.y, o.previous_x, o.previous_y]
                .into_iter()
                .enumerate()
            {
                placed.push((base + 0x14 * column + 2 * direction, word));
            }
        }
        assert!(origins.origin(set, grapple_origins::DIRECTIONS).is_none());
        assert_eq!(
            set.column_address(3),
            grapple_origins::ADDRESS + (base + 0x3c) as u32,
            "grapple origin previous-y column"
        );
    }
    check("grapple_origins", &bytes, &placed, &[]);
    rejects_other_lengths(
        "grapple_origins",
        BYTES,
        grapple_origins::GrappleOrigins::import,
        grapple_origins::LengthError,
    );
}

#[test]
fn area_select_palettes_round_trip_every_run() {
    use area_select_palettes::{
        AREAS, AreaSelectPalettes, BYTES, INACTIVE_OFFSET, ImportError, RUN_OFFSETS,
    };
    let mut bytes = source(BYTES, 0x61);
    let starts: Vec<usize> = RUN_OFFSETS
        .iter()
        .copied()
        .chain(RUN_OFFSETS.iter().map(|o| o + INACTIVE_OFFSET))
        .chain([BYTES])
        .collect();
    for run in starts.windows(2) {
        bytes[run[1] - 2..run[1]].copy_from_slice(&0xffffu16.to_le_bytes());
    }
    let palettes = AreaSelectPalettes::import(&bytes).unwrap();
    let mut placed = Vec::new();
    let mut ends = Vec::new();
    for (run, window) in starts.windows(2).enumerate() {
        let copies = palettes.copies(run % AREAS, run < AREAS).unwrap();
        assert_eq!(
            copies.len(),
            (window[1] - window[0] - 2) / 4,
            "run {run} length"
        );
        for (index, copy) in copies.iter().enumerate() {
            placed.push((window[0] + 4 * index, copy.source));
            placed.push((window[0] + 4 * index + 2, copy.destination));
        }
        ends.extend([window[1] - 2, window[1] - 1]);
    }
    assert!(palettes.copies(AREAS, true).is_none());
    check("area_select_palettes", &bytes, &placed, &ends);

    let mut early = bytes.clone();
    early[4..6].copy_from_slice(&0xffffu16.to_le_bytes());
    assert_eq!(
        AreaSelectPalettes::import(&early),
        Err(ImportError::Layout { offset: 6 }),
        "early run end"
    );
    rejects_other_lengths(
        "area_select_palettes",
        BYTES,
        AreaSelectPalettes::import,
        ImportError::Length,
    );
}

#[test]
fn crocomire_melt_tilemap_round_trips() {
    use crocomire::{MELT_TILEMAP_BYTES, MELT_TILES, MeltTilemap, MeltTilemapError};
    let mut bytes = source(MELT_TILEMAP_BYTES, 0x63);
    bytes[2 * MELT_TILES..].copy_from_slice(&0xffffu16.to_le_bytes());
    let tilemap = MeltTilemap::import(&bytes).unwrap();
    let placed: Vec<(usize, u16)> = tilemap
        .tiles()
        .iter()
        .enumerate()
        .map(|(i, &w)| (2 * i, w))
        .collect();
    check(
        "crocomire::melt_tilemap",
        &bytes,
        &placed,
        &[2 * MELT_TILES, 2 * MELT_TILES + 1],
    );

    let mut early = bytes.clone();
    early[20..22].copy_from_slice(&0xffffu16.to_le_bytes());
    assert_eq!(
        MeltTilemap::import(&early),
        Err(MeltTilemapError::End { at: 10 }),
        "early end"
    );
    let mut missing = bytes.clone();
    missing[2 * MELT_TILES] = 0;
    assert_eq!(
        MeltTilemap::import(&missing),
        Err(MeltTilemapError::End { at: MELT_TILES + 1 }),
        "no end"
    );
    rejects_other_lengths(
        "crocomire::melt_tilemap",
        MELT_TILEMAP_BYTES,
        MeltTilemap::import,
        MeltTilemapError::Length,
    );
}

#[test]
fn demo_scenes_round_trip_every_entry() {
    use demo_rooms::{DemoEntry, DemoScenes, ImportError, SCENES_ADDRESS, SCENES_BYTES};
    // Four sets of 6, 6, 6 and 5 scenes, each ended by $FFFF: 23 * 18 + 4 * 2.
    let mut bytes = source(SCENES_BYTES, 0x65);
    let mut layout = Vec::new();
    let mut offset = 0;
    for scenes in [6, 6, 6, 5] {
        for _ in 0..scenes {
            layout.push((offset, true));
            offset += 18;
        }
        bytes[offset..offset + 2].copy_from_slice(&0xffffu16.to_le_bytes());
        layout.push((offset, false));
        offset += 2;
    }
    assert_eq!(offset, SCENES_BYTES);
    let scenes = DemoScenes::import(&bytes).unwrap();
    let mut placed = Vec::new();
    let mut ends = Vec::new();
    for &(at, scene) in &layout {
        let address = (SCENES_ADDRESS as u16).wrapping_add(at as u16);
        match scenes.entry(address) {
            Some(DemoEntry::Scene(s)) if scene => {
                let words = [
                    s.room,
                    s.door,
                    s.door_slot,
                    s.layer1_x,
                    s.layer1_y,
                    s.samus_y_offset,
                    s.samus_x_offset,
                    s.timer,
                    s.hook,
                ];
                placed.extend(words.into_iter().enumerate().map(|(i, w)| (at + 2 * i, w)));
                assert_eq!(
                    scenes.entry(address.wrapping_add(2)),
                    None,
                    "no entry inside the scene at {at:#x}"
                );
            }
            Some(DemoEntry::End) if !scene => ends.extend([at, at + 1]),
            other => panic!("entry at {at:#x}: {other:?}"),
        }
    }
    check("demo_rooms::scenes", &bytes, &placed, &ends);

    // Without the last set's end, its final two bytes cannot hold a scene.
    let mut truncated = bytes.clone();
    truncated[SCENES_BYTES - 2..].copy_from_slice(&[0, 0]);
    assert_eq!(
        DemoScenes::import(&truncated),
        Err(ImportError::Truncated {
            offset: SCENES_BYTES - 2
        }),
        "truncated scene"
    );
    rejects_other_lengths(
        "demo_rooms::scenes",
        SCENES_BYTES,
        DemoScenes::import,
        ImportError::Length,
    );
}

#[test]
fn enemy_speed_tables_round_trip_every_word() {
    use enemy_speeds::{BYTES, BYTES32, EnemySpeeds, ImportError, SpeedTable, Speeds32};
    // Linear entries store speed high then low and its negation the same way;
    // quadratic entries, from $208, store each as low then high.
    let mut bytes = Vec::new();
    for entry in 0..BYTES / 8 {
        let speed = (entry as u32).wrapping_mul(0x0001_2345) ^ 0x8000;
        let negated = speed.wrapping_neg();
        let halves = |value: u32| {
            if 8 * entry < 0x208 {
                [(value >> 16) as u16, value as u16]
            } else {
                [value as u16, (value >> 16) as u16]
            }
        };
        for word in halves(speed).into_iter().chain(halves(negated)) {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
    }
    let speeds = EnemySpeeds::import(&bytes).unwrap();
    let mut placed = Vec::new();
    for index in 0..0x208 / 2 {
        placed.push((
            2 * index,
            speeds.word(SpeedTable::Linear, index as isize).unwrap(),
        ));
    }
    for index in 0..(BYTES - 0x208) / 2 {
        placed.push((
            0x208 + 2 * index,
            speeds.word(SpeedTable::Quadratic, index as isize).unwrap(),
        ));
    }
    check("enemy_speeds", &bytes, &placed, &[]);
    assert_eq!(
        speeds.word(SpeedTable::Quadratic, ((BYTES - 0x208) / 2) as isize),
        None,
        "past the quadratic table"
    );
    assert_eq!(
        speeds.word_at(SpeedTable::Quadratic, -2),
        Some(word_at(&bytes, 0x206)),
        "quadratic offsets reach back into linear"
    );
    assert_eq!(
        SpeedTable::Quadratic.address(),
        enemy_speeds::ADDRESS + 0x208
    );

    let mut bad = bytes.clone();
    bad[8 * 5 + 6] ^= 1;
    assert_eq!(
        EnemySpeeds::import(&bad),
        Err(ImportError::Negation {
            table: SpeedTable::Linear,
            entry: 5
        })
    );
    let mut bad = bytes.clone();
    bad[0x208 + 8 * 2 + 4] ^= 1;
    assert_eq!(
        EnemySpeeds::import(&bad),
        Err(ImportError::Negation {
            table: SpeedTable::Quadratic,
            entry: 2
        })
    );
    rejects_other_lengths(
        "enemy_speeds",
        BYTES,
        EnemySpeeds::import,
        ImportError::Length,
    );

    let mut bytes = Vec::new();
    for entry in 0..BYTES32 / 8 {
        let speed = (entry as u32).wrapping_mul(0x0003_1415) | 1;
        bytes.extend_from_slice(&speed.to_le_bytes());
        bytes.extend_from_slice(&speed.wrapping_neg().to_le_bytes());
    }
    let speeds = Speeds32::import(&bytes).unwrap();
    for offset in (0..BYTES32).step_by(4) {
        assert_eq!(
            speeds.word32_at(offset),
            Some(u32::from_le_bytes(
                bytes[offset..offset + 4].try_into().unwrap()
            )),
            "speeds32 at {offset:#x}"
        );
    }
    assert_eq!(speeds.word32_at(BYTES32 - 3), None);
    rejects_other_lengths("speeds32", BYTES32, Speeds32::import, ImportError::Length);
}
