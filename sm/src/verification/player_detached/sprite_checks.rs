//! Actual sprite core and controller operations on the existing composed Session.
use super::game::{
    session::Point, session_encoding, sprite_effect_controllers as controllers,
    sprite_effect_state::*, sprite_objects as sprites,
};
use super::{Composed, Event, RomImage};
fn request() -> SpriteSpawn {
    SpriteSpawn {
        position: Point { x: 32, y: 48 },
        definition: LegacySpriteDefinition(0),
        appearance: 0xaffe,
    }
}
fn prepare(h: &Composed) {
    h.word(0xb4, 0xbda8, 0x9000);
    h.word(0xb4, 0x9000, 1);
    h.word(0xb4, 0x9002, 0xa000);
    h.word(0xb4, 0x9004, 0xbd07);
}
fn allocation() {
    let mut h = Composed::new();
    for slot in &mut h.world.player.sprite_effects.active {
        slot.instruction = 0x9000;
        slot.timer = 0x55;
        slot.x.fraction = 0x1234;
        slot.y.fraction = 0x5678;
    }
    h.world.player.sprite_effects.active[3].instruction = 0;
    h.world.player.sprite_effects.active[29].instruction = 0;
    let before = h.world.player.clone();
    h.word(0xb4, 0xbda8, 0x0010);
    h.word(0xb4, 0x0010, 3);
    h.on_sprite_record = Some(|h, cursor| {
        assert_eq!(cursor, 0x10, "prepared allocation diagnostic cursor");
        h.world.player.sprite_effects.selected = 12;
        h.image = RomImage::zeroed(0x300000);
        h.word(0xb4, 0x0010, 7);
    });
    let result = sprites::legacy_result(sprites::spawn(&mut h, request()));
    let mut expected = before;
    let slot = &mut expected.sprite_effects.active[29];
    slot.instruction = 0x10;
    slot.timer = 7;
    slot.appearance = 0xaffe;
    slot.x.whole = 32;
    slot.x.fraction = 0;
    slot.y.whole = 48;
    slot.y.fraction = 0;
    slot.disable_flags = 0;
    expected.sprite_effects.selected = 12;
    assert_eq!(
        (result.0, &h.world.player),
        (58, &expected),
        "sprite.mutation.allocation_captured_highest"
    );
    h.world.player.sprite_effects.active[3].instruction = 0x9000;
    let before = h.world.clone();
    let records = h.visual_records.len();
    h.image = RomImage::zeroed(0);
    assert_eq!(
        sprites::spawn(&mut h, request()),
        SpawnResult::Full,
        "sprite.allocation.full_result"
    );
    assert_eq!(
        (&h.world, h.visual_records.len()),
        (&before, records),
        "sprite.allocation.full_unchanged"
    );
}
fn live_selected() {
    let mut h = Composed::new();
    h.world.player.sprite_effects.active[31].instruction = 0xc;
    h.world.player.sprite_effects.active[31].timer = 1;
    for i in [30, 28] {
        h.world.player.sprite_effects.active[i].instruction = 0x9000;
        h.world.player.sprite_effects.active[i].timer = 2;
    }
    h.word(0xb4, 0x10, 3);
    h.on_sprite_record = Some(|h, cursor| {
        assert_eq!(cursor, 0x10, "prepared update diagnostic cursor");
        h.world.player.sprite_effects.selected = 58;
        h.image = RomImage::zeroed(0x300000);
        h.word(0xb4, 0x10, 7);
    });
    sprites::update(&mut h);
    let s = &h.world.player.sprite_effects;
    assert_eq!(
        (
            s.active[31].instruction,
            s.active[31].timer,
            s.active[30].timer,
            s.active[28].timer,
            s.selected
        ),
        (0x10, 0, 2, 1, 0xfffe),
        "sprite.update.live-selected"
    );
    assert_eq!(s.active[29].timer, 7, "sprite.content.post-diagnostic");
}
fn pool_bytes(h: &Composed) -> Vec<u8> {
    (0xef78..0xf378)
        .map(|a| {
            session_encoding::projected_byte(&h.world.player, a)
                .expect("actual sprite codec extent")
        })
        .collect()
}
fn clears() {
    let mut h = Composed::new();
    for a in 0xef78..0xf378 {
        assert!(session_encoding::write_projected_byte(
            &mut h.world.player,
            a,
            (a as u8).wrapping_mul(7).wrapping_add(1)
        ));
    }
    h.world.player.sprite_effects.selected = 0x32;
    h.world.player.room.enemy_processing_stage = 0x5678;
    let before = h.world.player.clone();
    let bytes = pool_bytes(&h);
    sprites::clear_for_population_load(&mut h.world.player);
    let mut expected = vec![0; 512 * 2];
    expected[..2].copy_from_slice(&bytes[..2]);
    assert_eq!(
        (
            pool_bytes(&h),
            h.world.player.sprite_effects.selected,
            h.world.player.room.enemy_processing_stage
        ),
        (expected, 0x32, 0x5678),
        "sprite.mutation.population_clear_extent"
    );
    h.world.player = before.clone();
    sprites::retire_all(&mut h.world.player);
    let mut expected = before;
    for slot in &mut expected.sprite_effects.active {
        slot.instruction = 0;
    }
    sprites::draw(&mut h);
    assert_eq!(
        (&h.world.player, &h.events),
        (&expected, &Vec::<Event>::new()),
        "sprite.mutation.retirement_preserves_columns"
    );
}
fn lifecycle() {
    let mut h = Composed::new();
    prepare(&h);
    assert_eq!(
        sprites::legacy_result(sprites::spawn(&mut h, request())).0,
        62,
        "sprite.lifecycle.spawn"
    );
    controllers::freeze_attachment(
        &mut h,
        LegacySpriteHandle(62),
        0xc00,
        LegacySpriteProgram(0x9000),
    );
    let before = h.world.player.sprite_effects.active[31];
    sprites::update(&mut h);
    sprites::draw(&mut h);
    assert_eq!(
        h.world.player.sprite_effects.active[31], before,
        "sprite.lifecycle.disabled_update"
    );
    assert_eq!(
        h.events,
        vec![Event::Sprite(0xa000, 32, 48, 0xc00, 0)],
        "sprite.lifecycle.disabled_draw"
    );
    h.write_sprite_bytes(0xf0f4, 40u16.to_ne_bytes());
    h.write_sprite_bytes(0xf0f8, 50u16.to_ne_bytes());
    controllers::attach_actor(&mut h, LegacySpriteHandle(62), 0xe17a, 0xaffe);
    sprites::update(&mut h);
    assert_eq!(
        h.world.player.sprite_effects.active[31].instruction, 0,
        "sprite.lifecycle.retire"
    );
    assert_eq!(
        sprites::legacy_result(sprites::spawn(&mut h, request())).0,
        62,
        "sprite.lifecycle.reuse"
    );
    h.world.player.sprite_effects.selected = 14;
    for (debug, frozen) in [(1, 0), (0, 1)] {
        h.world.player_inputs.debug_enemy_freeze = debug;
        h.world.player_inputs.frozen = frozen;
        let before = h.world.clone();
        sprites::update(&mut h);
        assert_eq!(
            h.world, before,
            "sprite.lifecycle.freeze_preserves_selection"
        );
    }
    h.world.player_inputs.debug_enemy_freeze = 0;
    h.world.player_inputs.frozen = 0;
    h.world.player.sprite_effects.active[31].timer = 0;
    sprites::update(&mut h);
    assert_eq!(
        (
            h.world.player.sprite_effects.active[31].timer,
            h.world.player.sprite_effects.active[31].instruction
        ),
        (0xffff, 0x9000),
        "sprite.lifecycle.zero_timer_deferred"
    );
}
fn draw_live() {
    let mut h = Composed::new();
    prepare(&h);
    for i in [30, 31] {
        h.world.player.sprite_effects.active[i].instruction = 0x9000;
        h.world.player.sprite_effects.active[i].x.whole = 32;
        h.world.player.sprite_effects.active[i].y.whole = 48;
    }
    h.world.player.sprite_effects.active[31].appearance = 0xaffe;
    h.on_sprite_draw = Some(|h| {
        h.on_sprite_draw = None;
        let s = &mut h.world.player.sprite_effects.active[30];
        s.instruction = 0x9100;
        s.x.whole = 40;
        s.y.whole = 60;
        s.appearance = 0x1234;
        h.word(0xb4, 0x9102, 0xa222);
    });
    sprites::draw(&mut h);
    assert_eq!(
        h.events,
        vec![
            Event::Sprite(0xa000, 32, 48, 0xe00, 0x1fe),
            Event::Sprite(0xa222, 40, 60, 0x200, 0x34)
        ],
        "sprite.mutation.draw_order_and_live_next"
    );
}
fn clips() {
    let mut h = Composed::new();
    prepare(&h);
    h.world.player_inputs.camera_x = 8;
    for (i, x, y) in [(31, 0xfff8, 271), (30, 0xfff7, 271), (29, 8, 272)] {
        let s = &mut h.world.player.sprite_effects.active[i];
        s.instruction = 0x9000;
        s.x.whole = x;
        s.y.whole = y;
    }
    sprites::draw(&mut h);
    assert_eq!(
        h.events,
        vec![Event::Sprite(0xa000, 0xfff0, 271, 0, 0)],
        "sprite.mutation.draw_clip_edges"
    );
}
fn alias() {
    let mut h = Composed::new();
    h.world.player.sprite_effects.active[0].x.whole = 0xf35d;
    h.world.player.sprite_effects.active[0].appearance = 0x7654;
    h.world.player.sprite_compatibility.appearance[0] = 0x8c3a;
    h.world.player.room.enemy_processing_stage = 0x5678;
    controllers::flash_attachment(&mut h, LegacySpriteHandle(0x80), 0xa00);
    controllers::flash_attachment(&mut h, LegacySpriteHandle(0x40), 0xa00);
    assert_eq!(
        (
            h.world.player.sprite_effects.active[0].x.whole,
            h.world.player.sprite_compatibility.appearance[0],
            h.world.player.sprite_effects.active[0].appearance,
            h.world.player.room.enemy_processing_stage
        ),
        (0xfb5d, 0x8a3a, 0x7654, 0x5678),
        "sprite.mutation.controller_live_alias"
    );
}
fn enemy_spawn() {
    let mut h = Composed::new();
    prepare(&h);
    h.word(0x86, 0x9200, 0);
    h.world.player.enemy_projectiles.slots[0].x.whole = 100;
    h.world.player.enemy_projectiles.slots[0].y.whole = 100;
    h.enemy_random.push_back(0x0102);
    h.on_enemy_random = Some(|h| {
        h.world.player.enemy_projectiles.slots[0].x.whole = 300;
        h.world.player.enemy_projectiles.slots[0].y.whole = 200;
    });
    super::game::enemy_projectiles::sprite_command(&mut h, 0, 0x9200);
    sprites::draw(&mut h);
    assert_eq!(
        h.events,
        vec![Event::EnemyRandom, Event::Sprite(0xa000, 70, 169, 0, 0)],
        "sprite.composition.enemy_spawn_draw"
    );
    assert_eq!(
        (
            h.world.player.enemy_projectiles.slots[0].x.whole,
            h.world.player.enemy_projectiles.slots[0].y.whole
        ),
        (300, 200),
        "sprite.composition.enemy_owner_live"
    );
    sprites::update(&mut h);
    assert_eq!(
        h.world.player.sprite_effects.active[31].instruction, 0,
        "sprite.composition.enemy_spawn_retired"
    );
}
fn draygon() {
    let mut h = Composed::new();
    for index in (0..=10).step_by(2) {
        h.word(0xa5, 0xa1df + index * 2, 0x40);
    }
    h.word(0xa5, 0xa1df + 20, 0x40);
    h.word(0xa5, 0xa1af + 20, 0x30);
    h.word(0xa5, 0xa1af + 22, 0x30);
    h.word(0xa5, 0xa1df + 16, 0xc0);
    h.word(0xa5, 0xa1af + 16, 0x30);
    h.word(0xa5, 0xa1af + 18, 0x30);
    for (i, fraction) in [(31, 0xfff0), (30, 0x10)] {
        let s = &mut h.world.player.sprite_effects.active[i];
        s.x.whole = 0x1234;
        s.y.whole = 0x1234;
        s.x.fraction = fraction;
        s.y.fraction = fraction;
    }
    h.world.player.sprite_effects.active[25].x.whole = 0x7654;
    controllers::draygon_motion(&mut h, 2);
    let s = &h.world.player.sprite_effects.active;
    assert_eq!(
        (
            s[31].x.whole,
            s[31].x.fraction,
            s[31].y.whole,
            s[31].y.fraction,
            s[30].x.whole,
            s[30].x.fraction,
            s[30].y.whole,
            s[30].y.fraction,
            s[25].x.whole
        ),
        (
            0x1235, 0x20, 0x1235, 0x20, 0x1231, 0xffe0, 0x1233, 0xffe0, 0x7654
        ),
        "sprite.draygon42.carry-borrow"
    );
}
fn actor_alias() {
    let mut h = Composed::new();
    h.write_sprite_bytes(0xf0f4, 0x1234u16.to_ne_bytes());
    h.write_sprite_bytes(0xf0f8, 0x5678u16.to_ne_bytes());
    controllers::attach_actor(&mut h, LegacySpriteHandle(0), 0xe17a, 0xaffe);
    assert_eq!(
        (
            h.world.player.sprite_effects.active[0].x.whole,
            h.world.player.sprite_effects.active[0].y.whole
        ),
        (0x1234, 0x1234),
        "sprite.actor.attach_live_y_alias"
    );
    h.world.player.sprite_effects.active[0].appearance = 0xa00;
    controllers::maw_appearance(&mut h, LegacySpriteHandle(0), 0, 0xe0da);
    assert_eq!(
        h.world.player.sprite_effects.active[0].appearance, 0,
        "sprite.actor.maw_live_first_timer"
    );
    h.world.player.sprite_effects.active[0].appearance = 1;
    controllers::maw_appearance(&mut h, LegacySpriteHandle(0), 0, 0xe0da);
    assert_eq!(
        h.world.player.sprite_effects.active[0].appearance, 0xc01,
        "sprite.actor.maw_live_second_timer"
    );
}
// The timer occupies another owned word, so appearance writes cannot change it.
// E0DC + F9E = F07A: slot1 appearance supplies the actual actor timer read.
fn maw_grouping(timer: u16, initial: u16, expected: u16, tile: u16, label: &str) {
    let mut h = Composed::new();
    prepare(&h);
    let slot = &mut h.world.player.sprite_effects.active[0];
    slot.instruction = 0x9000;
    slot.x.whole = 32;
    slot.y.whole = 48;
    slot.appearance = initial;
    h.world.player.sprite_effects.active[1].appearance = timer;
    controllers::maw_appearance(&mut h, LegacySpriteHandle(0), 0xa00, 0xe0dc);
    sprites::draw(&mut h);
    assert_eq!(
        (
            h.world.player.sprite_effects.active[0].appearance,
            h.world.player.sprite_effects.active[1].appearance,
            &h.events
        ),
        (
            expected,
            timer,
            &vec![Event::Sprite(0xa000, 32, 48, 0xa00, tile)]
        ),
        "{label}"
    );
}
pub(crate) fn run_selected(case: &str) {
    match case {
        "allocation" => allocation(),
        "live_selected" => live_selected(),
        "clears" => clears(),
        "lifecycle" => lifecycle(),
        "draw_live" => draw_live(),
        "clips" => clips(),
        "alias" => alias(),
        "enemy_spawn" => enemy_spawn(),
        "draygon" => draygon(),
        "actor_alias" => actor_alias(),
        "maw_first_grouping" => {
            maw_grouping(0, 0x509f, 0x5a9f, 0x9f, "sprite.maw.first_write_grouping")
        }
        "maw_final_grouping" => {
            maw_grouping(1, 0x50a0, 0x5aa0, 0xa0, "sprite.maw.final_restore_grouping")
        }
        _ => panic!("unknown sprite composition case: {case}"),
    }
}
pub(crate) fn run() {
    for case in [
        "allocation",
        "live_selected",
        "clears",
        "lifecycle",
        "draw_live",
        "clips",
        "alias",
        "enemy_spawn",
        "draygon",
        "actor_alias",
        "maw_first_grouping",
        "maw_final_grouping",
    ] {
        run_selected(case);
    }
}
