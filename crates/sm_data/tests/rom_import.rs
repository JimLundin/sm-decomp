//! Every record must import from the original cartridge; otherwise the game
//! silently keeps its original ROM reads and replays prove nothing about it.
//! Set SM_ROM to the original ROM's path to run.
use sm_data::*;

#[test]
fn records_import_from_the_original_rom() {
    let Some(path) = std::env::var_os("SM_ROM") else {
        eprintln!("SM_ROM unset; ROM record imports not checked");
        return;
    };
    let rom = std::fs::read(path).expect("SM_ROM must name a readable ROM");
    let skip = rom.len() % 0x8000;
    let at = |address: u32, bytes: usize| {
        let o = skip + ((((address >> 16) << 15) | (address & 0x7fff)) & 0x3fffff) as usize;
        rom[o..o + bytes].to_vec()
    };
    let mut failures = Vec::new();
    macro_rules! check {
        ($name:literal, $result:expr) => {
            if let Err(error) = $result {
                failures.push(format!("{}: {:?}", $name, error));
            }
        };
    }
    check!(
        "enemy_speeds",
        enemy_speeds::EnemySpeeds::import(&at(enemy_speeds::ADDRESS, enemy_speeds::BYTES))
    );
    check!(
        "speeds32",
        enemy_speeds::Speeds32::import(&at(enemy_speeds::ADDRESS32, enemy_speeds::BYTES32))
    );
    check!(
        "sand_heap",
        sand_heap::SandHeapTiles::import(&at(sand_heap::ADDRESS, sand_heap::BYTES))
    );
    check!(
        "shooting_stars",
        shooting_stars::ShootingStars::import(&at(shooting_stars::ADDRESS, shooting_stars::BYTES))
    );
    check!(
        "ceres_elevator",
        ceres_elevator::Rotation::import(&at(ceres_elevator::ADDRESS, ceres_elevator::BYTES))
    );
    check!(
        "save_slots",
        save_slots::SaveSlots::import(&at(save_slots::ADDRESS, save_slots::BYTES))
    );
    check!(
        "demo_sets",
        demo_rooms::DemoSets::import(&at(demo_rooms::SETS_ADDRESS, demo_rooms::SETS_BYTES))
    );
    check!(
        "demo_scenes",
        demo_rooms::DemoScenes::import(&at(demo_rooms::SCENES_ADDRESS, demo_rooms::SCENES_BYTES))
    );
    check!(
        "grapple_origins",
        grapple_origins::GrappleOrigins::import(&at(
            grapple_origins::ADDRESS,
            grapple_origins::BYTES
        ))
    );
    check!(
        "grapple_extensions",
        grapple_aim::Extensions::import(&at(
            grapple_aim::EXTENSION_ADDRESS,
            grapple_aim::EXTENSION_BYTES
        ))
    );
    check!(
        "grapple_connections",
        grapple_aim::Connections::import(&at(
            grapple_aim::CONNECTION_ADDRESS,
            grapple_aim::CONNECTION_BYTES
        ))
    );
    check!(
        "area_select_palettes",
        area_select_palettes::AreaSelectPalettes::import(&at(
            area_select_palettes::ADDRESS,
            area_select_palettes::BYTES
        ))
    );
    check!(
        "intro_palette",
        intro_palette::IntroPalette::import(&at(intro_palette::ADDRESS, intro_palette::BYTES))
    );
    check!(
        "botwoon_holes",
        botwoon_holes::BotwoonHoles::import(&at(botwoon_holes::ADDRESS, botwoon_holes::BYTES))
    );
    check!(
        "botwoon_speeds",
        botwoon_speeds::BotwoonSpeeds::import(&at(botwoon_speeds::ADDRESS, botwoon_speeds::BYTES))
    );
    check!(
        "kraid_body",
        kraid::import_body_hitbox(&at(kraid::BODY_HITBOX_ADDRESS, kraid::BODY_HITBOX_BYTES))
    );
    check!(
        "kraid_nail",
        kraid::import_fingernail_boundaries(&at(
            kraid::FINGERNAIL_BOUNDARY_ADDRESS,
            kraid::FINGERNAIL_BOUNDARY_BYTES
        ))
    );
    check!(
        "kraid_foot",
        kraid::import_foot_targets(&at(kraid::FOOT_TARGET_ADDRESS, kraid::FOOT_TARGET_BYTES))
    );
    check!(
        "kraid_bg",
        kraid::import_bg_palettes(&at(kraid::BG_PALETTE_ADDRESS, kraid::BG_PALETTE_BYTES))
    );
    check!(
        "kraid_sprite",
        kraid::import_sprite_palettes(&at(
            kraid::SPRITE_PALETTE_ADDRESS,
            kraid::SPRITE_PALETTE_BYTES
        ))
    );
    check!(
        "phantoon_flame_rain",
        phantoon::import_flame_rain(&at(
            phantoon::FLAME_RAIN_ADDRESS,
            phantoon::FLAME_RAIN_BYTES
        ))
    );
    assert!(
        failures.is_empty(),
        "rejected the original ROM: {failures:?}"
    );
    // Spot values against raw ROM words.
    let w = |address: u32| u16::from_le_bytes(at(address, 2).try_into().unwrap());
    let origins = grapple_origins::GrappleOrigins::import(&at(
        grapple_origins::ADDRESS,
        grapple_origins::BYTES,
    ))
    .unwrap();
    let o = origins
        .origin(grapple_origins::OriginSet::Running, 3)
        .unwrap();
    assert_eq!(
        (o.x, o.y, o.previous_x, o.previous_y),
        (
            w(0x9bc172 + 6),
            w(0x9bc186 + 6),
            w(0x9bc19a + 6),
            w(0x9bc1ae + 6)
        )
    );
    let e = grapple_aim::Extensions::import(&at(
        grapple_aim::EXTENSION_ADDRESS,
        grapple_aim::EXTENSION_BYTES,
    ))
    .unwrap()
    .direction(9)
    .unwrap();
    assert_eq!(
        (e.x_velocity, e.y_velocity, e.angle),
        (w(0x9bc0db + 18), w(0x9bc0ef + 18), w(0x9bc104 + 18))
    );
    let c = grapple_aim::Connections::import(&at(
        grapple_aim::CONNECTION_ADDRESS,
        grapple_aim::CONNECTION_BYTES,
    ))
    .unwrap()
    .direction(grapple_aim::ConnectionSet::Crouching, 4)
    .unwrap();
    assert_eq!(
        (c.beam_function, c.handler),
        (w(0x9bc416 + 16), w(0x9bc416 + 18))
    );
    let step =
        ceres_elevator::Rotation::import(&at(ceres_elevator::ADDRESS, ceres_elevator::BYTES))
            .unwrap()
            .step(68)
            .unwrap();
    assert_eq!(
        (step.frames, step.mode7_b, step.mode7_a),
        (w(0x89ad5f + 408), w(0x89ad5f + 410), w(0x89ad5f + 412))
    );
}
