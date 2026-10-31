//! Sole byte-compatible projection of the migrated aggregate and shared continuations.
use crate::player_transitions::state as transition_state;
use crate::ram::{word_arrays as arrays, words as ram};
use crate::room_entry::{effects::AssetSlot, world::WorldBinding};
use crate::{external_inputs::ExternalInputs, session::Session};
use std::sync::Arc;
// The camera (`crate::camera`): each axis's x word, then its y word.
macro_rules! camera_words {
    (($c:expr), $f:ident, ($($borrow:tt)*)) => {
        for axis in 0..2 {
            $f(ram::LAYER1_X_BLOCK+axis*2,$($borrow)* $c.layer1_block[axis]);
            $f(ram::LAYER2_X_BLOCK+axis*2,$($borrow)* $c.layer2_block[axis]);
            $f(ram::PREVIOUS_LAYER1_X_BLOCK+axis*2,$($borrow)* $c.previous_layer1_block[axis]);
            $f(ram::PREVIOUS_LAYER2_X_BLOCK+axis*2,$($borrow)* $c.previous_layer2_block[axis]);
            $f(ram::BG1_X_BLOCK+axis*2,$($borrow)* $c.bg1_block[axis]);
            $f(ram::BG2_X_BLOCK+axis*2,$($borrow)* $c.bg2_block[axis]);
            $f(ram::LAYER1_X_SUBPOS+axis*4,$($borrow)* $c.layer1_fraction[axis]);
            $f(ram::LAYER1_X_POS+axis*4,$($borrow)* $c.layer1[axis]);
            $f(ram::LAYER2_X_POS+axis*2,$($borrow)* $c.layer2[axis]);
            $f(ram::BG1_X_OFFSET+axis*2,$($borrow)* $c.bg1_offset[axis]);
            $f(ram::BG2_X_SCROLL+axis*2,$($borrow)* $c.bg2_scroll[axis]);
            $f(ram::IDEAL_LAYER1_XPOS+axis*4,$($borrow)* $c.ideal_layer1[axis]);
        }
    };
}
// The room's area and geometry (`crate::room_view`), in `room_view::Word` order.
macro_rules! room_geometry_words {
    (($c:expr), $f:ident, ($($borrow:tt)*)) => {
        $f(ram::AREA_INDEX,$($borrow)* $c.area);
        $f(ram::ROOM_WIDTH_IN_BLOCKS,$($borrow)* $c.width);
        $f(ram::ROOM_HEIGHT_IN_BLOCKS,$($borrow)* $c.height);
        $f(ram::ROOM_WIDTH_IN_SCROLLS,$($borrow)* $c.scroll_width);
        $f(ram::ROOM_HEIGHT_IN_SCROLLS,$($borrow)* $c.scroll_height);
    };
}
/// The room's area and geometry as `bytes` encode them.
pub fn room_geometry(bytes: &impl Fn(usize) -> u8) -> crate::room_view::RoomGeometry {
    let mut room = crate::room_view::RoomGeometry::default();
    let load = |at: usize, value: &mut u16| *value = read(bytes, at);
    room_geometry_words!((room), load, (&mut));
    room
}
/// Each room geometry word's address, in `room_view::Word` order.
pub fn room_geometry_addresses() -> [usize; 5] {
    let mut addresses = [0; 5];
    let mut index = 0;
    let room = crate::room_view::RoomGeometry::default();
    let mut visit = |at: usize, _: &u16| {
        addresses[index] = at;
        index += 1;
    };
    room_geometry_words!((room), visit, (&));
    addresses
}
// The room's effect and placement words (`crate::room_view::RoomEffects`),
// in `room_view::EffectWord` order. scalar_words projects each from the
// Session field named in its `EffectWord` documentation.
const ROOM_EFFECT_WORDS: [usize; 7] = [
    ram::EARTHQUAKE_TYPE,
    ram::EARTHQUAKE_TIMER,
    ram::TOURIAN_ENTRANCE_STATUE_ANIMSTATE,
    ram::TOURIAN_ENTRANCE_STATUE_FINISHED,
    ram::ROOM_X_COORDINATE_ON_MAP,
    ram::ROOM_Y_COORDINATE_ON_MAP,
    ram::ELEVATOR_DOOR_PROPERTIES_ORIENTATION,
];
/// The room's effect and placement words as `bytes` encode them.
pub fn room_effects(bytes: &impl Fn(usize) -> u8) -> crate::room_view::RoomEffects {
    let [w0, w1, w2, w3, w4, w5, w6] = ROOM_EFFECT_WORDS.map(|at| read(bytes, at));
    crate::room_view::RoomEffects {
        earthquake_kind: w0,
        earthquake_timer: w1,
        statue_animation: w2,
        statue_finished: w3,
        map_x: w4,
        map_y: w5,
        elevator_properties: w6,
    }
}
/// The address of `word`.
pub fn room_effect_address(word: crate::room_view::EffectWord) -> usize {
    ROOM_EFFECT_WORDS[word as usize]
}
// The game flow (`crate::flow`), in RAM order.
macro_rules! flow_words {
    (($c:expr), $f:ident, ($($borrow:tt)*)) => {
        $f(ram::CINEMATIC_FUNCTION,$($borrow)* $c.cinematic_function);
        $f(ram::DEMO_TIMER,$($borrow)* $c.demo_timer);
        $f(ram::DEMO_SET,$($borrow)* $c.demo_set);
        $f(ram::DEMO_SCENE,$($borrow)* $c.demo_scene);
        $f(ram::NUM_DEMO_SETS,$($borrow)* $c.num_demo_sets);
    };
}
// The menu state (`crate::menu_state`), in RAM order from $0727.
macro_rules! menu_words {
    (($c:expr), $f:ident, ($($borrow:tt)*)) => {
        $f(ram::MENU_INDEX,$($borrow)* $c.menu_index);
        $f(ram::PAUSEMENU_START_LR_PRESSED_HIGHLIGHT_TIMER,$($borrow)* $c.pausemenu_start_lr_pressed_highlight_timer);
        $f(ram::PAUSEMENU_LR_ANIMATION_TIMER,$($borrow)* $c.pausemenu_lr_animation_timer);
        $f(ram::PAUSEMENU_ITEM_SELECTOR_ANIMATION_TIMER,$($borrow)* $c.pausemenu_item_selector_animation_timer);
        $f(ram::PAUSEMENU_RESERVE_TANK_ANIMATION_TIMER,$($borrow)* $c.pausemenu_reserve_tank_animation_timer);
        $f(ram::UNUSED_WORD_7E0731,$($borrow)* $c.unused_word_7e0731);
        $f(ram::PAUSEMENU_UPARROW_ANIMATION_TIMER,$($borrow)* $c.pausemenu_uparrow_animation_timer);
        $f(ram::PAUSEMENU_DOWNARROW_ANIMATION_TIMER,$($borrow)* $c.pausemenu_downarrow_animation_timer);
        $f(ram::PAUSEMENU_RIGHTARROW_ANIMATION_TIMER,$($borrow)* $c.pausemenu_rightarrow_animation_timer);
        $f(ram::PAUSEMENU_LEFTARROW_ANIMATION_TIMER,$($borrow)* $c.pausemenu_leftarrow_animation_timer);
        $f(ram::PAUSEMENU_PALETTE_ANIMATION_TIMER,$($borrow)* $c.pausemenu_palette_animation_timer);
        $f(ram::UNUSED_WORD_7E073D,$($borrow)* $c.unused_word_7e073d);
        $f(ram::PAUSEMENU_LR_ANIMATION_FRAME,$($borrow)* $c.pausemenu_lr_animation_frame);
        $f(ram::PAUSEMENU_ITEM_SELECTOR_ANIMATION_FRAME,$($borrow)* $c.pausemenu_item_selector_animation_frame);
        $f(ram::PAUSEMENU_RESERVE_TANK_ANIMATION_FRAME,$($borrow)* $c.pausemenu_reserve_tank_animation_frame);
        $f(ram::UNUSED_WORD_7E0745,$($borrow)* $c.unused_word_7e0745);
        $f(ram::PAUSEMENU_UPARROW_ANIMATION_FRAME,$($borrow)* $c.pausemenu_uparrow_animation_frame);
        $f(ram::PAUSEMENU_DOWNARROW_ANIMATION_FRAME,$($borrow)* $c.pausemenu_downarrow_animation_frame);
        $f(ram::PAUSEMENU_RIGHTARROW_ANIMATION_FRAME,$($borrow)* $c.pausemenu_rightarrow_animation_frame);
        $f(ram::PAUSEMENU_LEFTARROW_ANIMATION_FRAME,$($borrow)* $c.pausemenu_leftarrow_animation_frame);
        $f(ram::PAUSEMENU_PALETTE_ANIMATION_FRAME,$($borrow)* $c.pausemenu_palette_animation_frame);
        $f(ram::PAUSEMENU_SHOULDER_BUTTON_HIGHLIGHT,$($borrow)* $c.pausemenu_shoulder_button_highlight);
        $f(ram::PAUSEMENU_BUTTON_LABEL_MODE,$($borrow)* $c.pausemenu_button_label_mode);
        $f(ram::PAUSEMENU_EQUIPMENT_CATEGORY_ITEM,$($borrow)* $c.pausemenu_equipment_category_item);
        $f(ram::PAUSEMENU_RESERVE_TANK_DELAY_CTR,$($borrow)* $c.pausemenu_reserve_tank_delay_ctr);
        $f(ram::UNUSED_WORD_7E0759,$($borrow)* $c.unused_word_7e0759);
        $f(ram::UNUSED_WORD_7E075B,$($borrow)* $c.unused_word_7e075b);
        $f(ram::UNUSED_WORD_7E075D,$($borrow)* $c.unused_word_7e075d);
        $f(ram::UNUSED_WORD_7E075F,$($borrow)* $c.unused_word_7e075f);
        $f(ram::UNUSED_WORD_7E0761,$($borrow)* $c.unused_word_7e0761);
        $f(ram::PAUSE_SCREEN_MODE,$($borrow)* $c.pause_screen_mode);
    };
}
/// The menu state as `bytes` encode it.
pub fn menu(bytes: &impl Fn(usize) -> u8) -> crate::menu_state::MenuState {
    let mut menu = crate::menu_state::MenuState::default();
    let load = |at: usize, value: &mut u16| *value = read(bytes, at);
    menu_words!((menu), load, (&mut));
    menu
}
/// RAM address of the first flow word, `cinematic_function`.
pub const FLOW: usize = ram::CINEMATIC_FUNCTION;
/// The camera as `bytes` encode it.
pub fn camera(bytes: &impl Fn(usize) -> u8) -> crate::camera::Camera {
    let mut camera = crate::camera::Camera::default();
    let load = |at: usize, value: &mut u16| *value = read(bytes, at);
    camera_words!((camera), load, (&mut));
    camera
}
/// The flow as `bytes` encode it.
pub fn flow(bytes: &impl Fn(usize) -> u8) -> crate::flow::Flow {
    let mut flow = crate::flow::Flow::default();
    let load = |at: usize, value: &mut u16| *value = read(bytes, at);
    flow_words!((flow), load, (&mut));
    flow
}
// Head0 owns only these ten bytes; full/odd accesses still project each lane.
macro_rules! botwoon_death_words {
    ($s:ident, $f:ident, ($($borrow:tt)*)) => {
        $f(0x801e, $($borrow)* $s.botwoon_death.pending_hit);
        $f(0x8020, $($borrow)* $s.botwoon_death.active);
        $f(0x803c, $($borrow)* $s.botwoon_death.tail_ready);
        $f(0x8818, $($borrow)* $s.botwoon_death.previous_health);
        $f(0x883e, $($borrow)* $s.botwoon_death.body_landed);
    };
}
fn botwoon_death_byte(s: &Session, at: usize) -> Option<u8> {
    let mut result = None;
    let mut select = |base: usize, word: &u16| {
        if (base..base + 2).contains(&at) {
            result = Some(word.to_le_bytes()[at - base]);
        }
    };
    botwoon_death_words!(s, select, (&));
    result
}
/// Adopt only this new cohort after a successful native Maw store. Existing
/// owned regions retain their former raw-write/Drop-overwrite behavior.
pub fn write_botwoon_death_byte(s: &mut Session, at: usize, value: u8) -> bool {
    let mut written = false;
    let mut store = |base: usize, word: &mut u16| {
        if (base..base + 2).contains(&at) {
            let mut bytes = word.to_le_bytes();
            bytes[at - base] = value;
            *word = u16::from_le_bytes(bytes);
            written = true;
        }
    };
    botwoon_death_words!(s, store, (&mut));
    written
}
// Finite typed words share one lossless projection; no raw-word mirrors.
macro_rules! transition_words {
    ($s:ident, $f:ident) => {
        $f!(
            0x0a28,
            $s.player.transitions.ordinary,
            transition_state::PoseRequestWord
        );
        $f!(
            0x0a2a,
            $s.player.transitions.interrupted,
            transition_state::PoseRequestWord
        );
        $f!(
            0x0a2c,
            $s.player.transitions.transitional,
            transition_state::PoseRequestWord
        );
        $f!(
            0x0a2e,
            $s.player.transitions.momentum,
            transition_state::MomentumAction
        );
        $f!(
            0x0a30,
            $s.player.transitions.interruption,
            transition_state::InterruptionAction
        );
        $f!(
            0x0a32,
            $s.player.transitions.hurt,
            transition_state::HurtAction
        );
        $f!(
            0x0a9a,
            $s.player.transitions.animation_skip,
            transition_state::AnimationSkip
        );
        $f!(
            0x0a58,
            $s.player.movement_handler,
            transition_state::MovementMode
        );
        $f!(
            0x0a5c,
            $s.player.transitions.draw_handler,
            transition_state::DrawMode
        );
        $f!(
            0x0a42,
            $s.player.transitions.frame_alpha,
            transition_state::FrameAlphaMode
        );
        $f!(
            0x0a44,
            $s.player.transitions.frame_beta,
            transition_state::FrameBetaMode
        );
        $f!(
            0x0a5a,
            $s.player.transitions.frame_gamma,
            transition_state::FrameGammaMode
        );
        $f!(
            0x0a56,
            $s.player.transitions.bomb_jump,
            transition_state::BombJump
        );
        $f!(
            0x0a60,
            $s.player.transitions.input_mode,
            transition_state::InputMode
        );
        $f!(
            0x0b20,
            $s.player.transitions.bounce,
            transition_state::LandingBounce
        );
    };
}
// Identities have only one mutable representation. Byte writes resolve each
// intermediate key through the same binding; no key mirror is retained.
macro_rules! room_keys {
    ($f:ident) => {
        $f!(0x078d, exit, exit_key, exit_identity);
        $f!(0x079b, room, room_key, room_identity);
        $f!(0x07b5, doors, door_list_key, door_list);
        $f!(0x07bb, state, state_key, state_identity);
    };
}
macro_rules! room_assets {
    ($f:ident) => {
        $f!(0x07bd, level, AssetSlot::Level);
        $f!(0x07c0, tile_table, AssetSlot::TileTable);
        $f!(0x07c3, tiles, AssetSlot::Tiles);
        $f!(0x07c6, palette, AssetSlot::Palette);
    };
}
// Direct projection for the 1,024-byte pool. Normal gameplay uses typed members;
// this is solely the active/tail compatibility mapping, shared by reads/writes.
macro_rules! sprite_projection {
    ($s:ident,$offset:ident,($($borrow:tt)*)) => {{
        let index = ($offset % 128) / 2;
        let column = $offset / 128;
        if index < 32 {
            let slot = $($borrow)* $s.sprite_effects.active[index];
            match column {
                0 => $($borrow)* slot.instruction,
                1 => $($borrow)* slot.timer,
                2 => $($borrow)* slot.appearance,
                3 => $($borrow)* slot.x.whole,
                4 => $($borrow)* slot.x.fraction,
                5 => $($borrow)* slot.y.whole,
                6 => $($borrow)* slot.y.fraction,
                7 => $($borrow)* slot.disable_flags,
                _ => unreachable!(),
            }
        } else {
            let index = index - 32;
            let tail = $($borrow)* $s.sprite_compatibility;
            match column {
                0 => $($borrow)* tail.instruction[index],
                1 => $($borrow)* tail.timer[index],
                2 => $($borrow)* tail.appearance[index],
                3 => $($borrow)* tail.x[index],
                4 => $($borrow)* tail.x_fraction[index],
                5 => $($borrow)* tail.y[index],
                6 => $($borrow)* tail.y_fraction[index],
                7 => $($borrow)* tail.disable_flags[index],
                _ => unreachable!(),
            }
        }
    }};
}
// Byte collections have one compatibility projection, shared by reads/writes.
macro_rules! flag_projection {
    ($s:ident,$offset:ident,($($borrow:tt)*)) => {
        match $offset {
            0..=7 => $($borrow)* $s.gameplay_flags.events[$offset],
            8..=15 => $($borrow)* $s.gameplay_flags.bosses_by_area[$offset - 8],
            16..=79 => $($borrow)* $s.gameplay_flags.used_room_objects[$offset - 16],
            80..=143 => $($borrow)* $s.gameplay_flags.collected_locations[$offset - 80],
            144..=207 => $($borrow)* $s.gameplay_flags.opened_doors[$offset - 144],
            _ => unreachable!(),
        }
    };
}
macro_rules! presentation_ref {
    ($s:ident, visual, (&)) => {
        $s.presentation.visual()
    };
    ($s:ident, visual, (&mut)) => {
        $s.presentation.visual_mut()
    };
    ($s:ident, palette, (&)) => {
        $s.presentation.palette()
    };
    ($s:ident, palette, (&mut)) => {
        $s.presentation.palette_mut()
    };
    ($s:ident, atmospheric, (&)) => {
        $s.presentation.atmospheric()
    };
    ($s:ident, atmospheric, (&mut)) => {
        $s.presentation.atmospheric_mut()
    };
}

macro_rules! scalar_words {
    ($s:ident,$f:ident,($($borrow:tt)*))=>{
        $f(0x0da2,$($borrow)* $s.player.camera_distance[0].whole);
        $f(0x0da4,$($borrow)* $s.player.camera_distance[0].fraction);
        $f(0x0da6,$($borrow)* $s.player.camera_distance[1].whole);
        $f(0x0da8,$($borrow)* $s.player.camera_distance[1].fraction);
        $f(0x0a64,$($borrow)* $s.player.grapple_interaction_flags);
        $f(0x0cf4,$($borrow)* $s.player.grapple_flags);
        $f(0x0cf6,$($borrow)* $s.player.grapple_delay);
        $f(0x0cf8,$($borrow)* $s.player.grapple_slow_scrolling);
        $f(0x0d1e,$($borrow)* $s.player.grapple_extension_compatibility.x);
        $f(0x0d20,$($borrow)* $s.player.grapple_extension_compatibility.y);
        $f(0x0d34,$($borrow)* $s.player.grapple_direction);
        $f(0x0d36,$($borrow)* $s.player.grapple_direction_compatibility);
        $f(0x0cfa,$($borrow)* $s.player.grapple_angle);
        $f(0x0cfe,$($borrow)* $s.player.grapple_length);
        $f(0x0d02,$($borrow)* $s.player.grapple_origin_offset.x);
        $f(0x0d04,$($borrow)* $s.player.grapple_origin_offset.y);
        $f(0x0d06,$($borrow)* $s.player.grapple_endpoint_fraction.x);
        $f(0x0d08,$($borrow)* $s.player.grapple_endpoint.x);
        $f(0x0d0a,$($borrow)* $s.player.grapple_endpoint_fraction.y);
        $f(0x0d0e,$($borrow)* $s.player.grapple_offset[0].fraction);
        $f(0x0d10,$($borrow)* $s.player.grapple_offset[0].whole);
        $f(0x0d12,$($borrow)* $s.player.grapple_offset[1].fraction);
        $f(0x0d14,$($borrow)* $s.player.grapple_offset[1].whole);
        $f(0x0d22,$($borrow)* $s.player.grapple_velocity.x);
        $f(0x0d24,$($borrow)* $s.player.grapple_velocity.y);
        $f(0x0cfc,$($borrow)* $s.player.swing.mirrored_angle);
        $f(0x0d00,$($borrow)* $s.player.swing.length_delta);
        $f(0x0d3e,$($borrow)* $s.player.swing.start_animation_timer);
        $f(0x0d40,$($borrow)* $s.player.swing.start_animation_tiles);
        $f(0x0d16,$($borrow)* $s.player.swing.start.x);
        $f(0x0d18,$($borrow)* $s.player.swing.start.y);
        $f(0x0d26,$($borrow)* $s.player.swing.angular_velocity);
        $f(0x0d28,$($borrow)* $s.player.swing.angle_acceleration);
        $f(0x0d2a,$($borrow)* $s.player.swing.input_acceleration);
        $f(0x0d2c,$($borrow)* $s.player.swing.rising_deceleration);
        $f(0x0d2e,$($borrow)* $s.player.swing.extra_angular_velocity);
        $f(0x0d30,$($borrow)* $s.player.swing.kick_cooldown);
        $f(0x0d38,$($borrow)* $s.player.swing.failed_movement_counter);
        $f(0x0d3a,$($borrow)* $s.player.swing.start_marks[0]);
        $f(0x0d3c,$($borrow)* $s.player.swing.start_marks[1]);
        $f(0x0d90,$($borrow)* $s.player.swing.probe.x);
        $f(0x0d92,$($borrow)* $s.player.swing.probe.y);
        $f(0x0d94,$($borrow)* $s.player.swing.probe_block.x);
        $f(0x0d96,$($borrow)* $s.player.swing.probe_block.y);
        for i in 0..16 {
            $f(0x0d42 + 2*i,$($borrow)* $s.player.grapple_animation_delay[i]);
            $f(0x0d62 + 2*i,$($borrow)* $s.player.grapple_animation_program[i]);
        }
        $f(0x0d0c,$($borrow)* $s.player.grapple_endpoint.y);
        $f(0x0791,$($borrow)* $s.room.entry.direction);
        $f(ram::ELEVATOR_DOOR_PROPERTIES_ORIENTATION,$($borrow)* $s.room.entry.elevator_properties);
        $f(0x0795,$($borrow)* $s.room.entry.elevator_interlock);
        $f(0x0797,$($borrow)* $s.room.entry.enemy_interlock);
        $f(0x0799,$($borrow)* $s.room.entry.elevator_direction);
        $f(0x079d,$($borrow)* $s.room.entry.room_index);
        $f(ram::ROOM_X_COORDINATE_ON_MAP,$($borrow)* $s.room.entry.map_x);
        $f(ram::ROOM_Y_COORDINATE_ON_MAP,$($borrow)* $s.room.entry.map_y);
        $f(0x07ad,$($borrow)* $s.room.entry.scroll_up);
        $f(0x07af,$($borrow)* $s.room.entry.scroll_down);
        $f(0x07b1,$($borrow)* $s.room.entry.previous_cre);
        $f(0x07b3,$($borrow)* $s.room.entry.cre);
        $f(0x07c9,$($borrow)* $s.room.entry.music_track);
        $f(0x07cb,$($borrow)* $s.room.entry.music_data);
        $f(0x07cd,$($borrow)* $s.room.entry.fx_key);
        $f(0x07cf,$($borrow)* $s.room.entry.enemy_population_key);
        $f(0x07d1,$($borrow)* $s.room.entry.enemy_tilesets_key);
        $f(0x07df,$($borrow)* $s.room.entry.main_code_key);
        $f(0x07e9,$($borrow)* $s.room.entry.scrolling_hook);
        $f(0x0925,$($borrow)* $s.room.entry.frame);
        $f(0x0927,$($borrow)* $s.room.entry.arrival_x);
        $f(0x0929,$($borrow)* $s.room.entry.arrival_y);
        $f(0x092b,$($borrow)* $s.room.entry.speed_fraction);
        $f(0x092d,$($borrow)* $s.room.entry.speed_whole);
        $f(0x092f,$($borrow)* $s.room.entry.elevator_delay);
        $f(0x0931,$($borrow)* $s.room.entry.completion);
        $f(0x099c,$($borrow)* $s.room.entry.transition_selector);
        $f(0x0e16,$($borrow)* $s.room.entry.elevator_flags);
        $f(0x0e18,$($borrow)* $s.room.entry.elevator_status);
        $f(0x00a9,$($borrow)* $s.room.entry.irq_policy);
        $f(0x05e7,$($borrow)* $s.flag_compatibility.bitmask);
        $f(0x1844,$($borrow)* $s.sprite_effects.selected);
        for index in 0..32 {
            $f(0xef78+index*2,$($borrow)* $s.sprite_effects.active[index].instruction);
            $f(0xeff8+index*2,$($borrow)* $s.sprite_effects.active[index].timer);
            $f(0xf078+index*2,$($borrow)* $s.sprite_effects.active[index].appearance);
            $f(0xf0f8+index*2,$($borrow)* $s.sprite_effects.active[index].x.whole);
            $f(0xf178+index*2,$($borrow)* $s.sprite_effects.active[index].x.fraction);
            $f(0xf1f8+index*2,$($borrow)* $s.sprite_effects.active[index].y.whole);
            $f(0xf278+index*2,$($borrow)* $s.sprite_effects.active[index].y.fraction);
            $f(0xf2f8+index*2,$($borrow)* $s.sprite_effects.active[index].disable_flags);
            $f(0xefb8+index*2,$($borrow)* $s.sprite_compatibility.instruction[index]);
            $f(0xf038+index*2,$($borrow)* $s.sprite_compatibility.timer[index]);
            $f(0xf0b8+index*2,$($borrow)* $s.sprite_compatibility.appearance[index]);
            $f(0xf138+index*2,$($borrow)* $s.sprite_compatibility.x[index]);
            $f(0xf1b8+index*2,$($borrow)* $s.sprite_compatibility.x_fraction[index]);
            $f(0xf238+index*2,$($borrow)* $s.sprite_compatibility.y[index]);
            $f(0xf2b8+index*2,$($borrow)* $s.sprite_compatibility.y_fraction[index]);
            $f(0xf338+index*2,$($borrow)* $s.sprite_compatibility.disable_flags[index]);
        }
        $f(0xac8,$($borrow)* $s.player.top_sprite);
        $f(0xaca,$($borrow)* $s.player.bottom_sprite);
        $f(0xb04,$($borrow)* $s.player.body_position.x);
        $f(0xb06,$($borrow)* $s.player.body_position.y);
        $f(0xa62,$($borrow)* $s.player.scripted_movement_direction);
        $f(0xa7a,$($borrow)* $s.player.demo.pre_action);
        $f(0xa7c,$($borrow)* $s.player.demo.delay);
        $f(0xa7e,$($borrow)* $s.player.demo.program);
        $f(0xa84,$($borrow)* $s.player.demo.held);
        $f(0xa86,$($borrow)* $s.player.demo.pressed);
        $f(0xa88,$($borrow)* $s.player.demo.enabled);
        $f(0xa8c,$($borrow)* $s.player.demo.previous_held);
        $f(0xa8e,$($borrow)* $s.player.demo.previous_pressed);
        $f(0xa48,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).hurt_flash);
        $f(0xa72,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).visor_timer);
        $f(0xa74,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).suit_index);
        $f(0xa76,$($borrow)* $s.inventory.hyper_beam);
        $f(0xa80,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).xray_progress);
        $f(0xad0,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).timer);
        $f(0xddc,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).suit_beam_speed);
        $f(0xdf0,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).suit_red_green);
        $f(0xdf2,$($borrow)* presentation_ref!($s, palette, ($($borrow)*)).crystal_flash_delay);
        $f(0xa4e,$($borrow)* $s.player.periodic_damage_fraction);
        $f(ram::TOURIAN_ENTRANCE_STATUE_ANIMSTATE,$($borrow)* $s.mechanisms.statue_animation);
        for index in 0..256 {
            $f(0xc000+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).colors[index]);
        }
        for index in 0..256 {
            $f(0xc200+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).targets[index]);
        }
        $f(0xc400,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_fade.step);
        $f(0xc402,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_fade.duration);
        $f(0xc404,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_fade.cursor);
        $f(0x723,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).screen_fade.delay);
        $f(0x725,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).screen_fade.counter);
        for index in 0..8 {
            $f(0x1e7d+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].definition);
            $f(0x1e8d+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].color_offset);
            $f(0x1ead+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].pre_action);
            $f(0x1ebd+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].program);
            $f(0x1ecd+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].delay);
            $f(0x1edd+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.slots[index].counter);
        }
        for index in 0..6 {
            $f(0x1ef5+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].definition);
            $f(0x1f01+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].counter);
            $f(0x1f0d+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].program);
            $f(0x1f19+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].delay);
            $f(0x1f25+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].source);
            $f(0x1f31+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].size);
            $f(0x1f3d+index*2,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.slots[index].vram);
        }
        $f(0x1e79,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.flags);
        $f(0x1e7b,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.selected);
        $f(0x1eed,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.heat);
        $f(0x1eef,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).palette_effects.previous_heat);
        $f(0x1ef1,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.flags);
        $f(0x1ef3,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.selected);
        $f(0x1f49,$($borrow)* presentation_ref!($s, visual, ($($borrow)*)).animated_tiles.last_instruction);
        $f(0x0a18,$($borrow)* $s.player.transitions.landing_compatibility);
        $f(0x0a52,$($borrow)* $s.player.transitions.knockback);
        $f(0x0a82,$($borrow)* $s.player.transitions.xray_angle);
        $f(0x0a9c,$($borrow)* $s.player.transitions.animation_buffer);
        $f(0x0a9e,$($borrow)* $s.player.transitions.grapple_walljump_timer);
        $f(0x0aa0,$($borrow)* $s.player.transitions.elevator_fade_timer);
        $f(0x0aa2,$($borrow)* $s.player.transitions.shinespark_start_stop_timer);
        $f(0x0aa4,$($borrow)* $s.player.crouch_transition_work);
        $f(0x0ad2,$($borrow)* $s.player.transitions.liquid_physics);
        $f(0x0b1a,$($borrow)* $s.player.transitions.landing_motion_compatibility);
        $f(0x0b22,$($borrow)* $s.player.transitions.falling);
        $f(0x0b2a,$($borrow)* $s.player.transitions.vertical_motion_compatibility);
        $f(0x0b38,$($borrow)* $s.player.transitions.vertical_auxiliary_compatibility);
        for index in 0..4 {
            $f(0x0ad4+index*2,$($borrow)* presentation_ref!($s, atmospheric, ($($borrow)*)).slots[index].remaining);
            $f(0x0adc+index*2,$($borrow)* presentation_ref!($s, atmospheric, ($($borrow)*)).slots[index].x);
            $f(0x0ae4+index*2,$($borrow)* presentation_ref!($s, atmospheric, ($($borrow)*)).slots[index].y);
            $f(0x0aec+index*2,$($borrow)* presentation_ref!($s, atmospheric, ($($borrow)*)).slots[index].frame_and_kind);
        }
        $f(ram::PROJECTILE_COUNTER,$($borrow)* $s.projectiles.count);
        $f(ram::BOMB_COUNTER,$($borrow)* $s.projectiles.bomb_count);
        $f(ram::PROJECTILE_INDEX,$($borrow)* $s.projectiles.selected);
        $f(ram::COOLDOWN_TIMER,$($borrow)* $s.projectiles.controls.cooldown);
        $f(ram::BOMB_SPREAD_CHARGE_TIMEOUT_COUNTER,$($borrow)* $s.projectiles.controls.spread_charge);
        $f(ram::USED_FOR_SBA_ATTACKS_B60,$($borrow)* $s.projectiles.controls.orbit_step);
        $f(ram::FLARE_COUNTER,$($borrow)* $s.projectiles.controls.charge);
        $f(ram::PREV_BEAM_CHARGE_COUNTER,$($borrow)* $s.projectiles.controls.previous_charge);
        $f(ram::CHARGED_SHOT_GLOW_TIMER,$($borrow)* $s.projectiles.controls.glow);
        $f(ram::SAMUS_CHARGE_PALETTE_INDEX,$($borrow)* $s.projectiles.controls.charge_palette);
        $f(ram::PLAY_RESUME_CHARGING_BEAM_SFX,$($borrow)* $s.projectiles.controls.resume_sound);
        $f(ram::SAMUS_ECHOES_SOUND_FLAG,$($borrow)* $s.projectiles.controls.echo_sound);
        $f(ram::NEW_PROJECTILE_DIRECTION_CHANGED_POSE,$($borrow)* $s.projectiles.controls.pending_aim);
        $f(ram::PROJECTILE_INVINCIBILITY_TIMER,$($borrow)* $s.projectiles.controls.protection);
        $f(ram::HUD_ITEM_INDEX,$($borrow)* $s.projectiles.controls.selected_weapon);
        $f(ram::SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX,$($borrow)* $s.projectiles.controls.automatic_weapon);
        $f(ram::HUD_ITEM_CHANGED_THIS_FRAME,$($borrow)* $s.projectiles.controls.weapon_changed);
        $f(0xaa6,$($borrow)* $s.projectiles.controls.cannon_flags);
        $f(ram::ARM_CANNON_FRAME,$($borrow)* $s.projectiles.controls.cannon_frame);
        $f(ram::ARM_CANNON_DRAWING_MODE,$($borrow)* $s.projectiles.controls.cannon_drawing_mode);
        $f(ram::SAMUS_X_POS,$($borrow)* $s.player.x);
        $f(ram::SAMUS_X_SUBPOS,$($borrow)* $s.player.x_fraction);
        $f(ram::SAMUS_Y_POS,$($borrow)* $s.player.y);
        $f(ram::SAMUS_Y_SUBPOS,$($borrow)* $s.player.y_fraction);
        $f(ram::SAMUS_MISSILES,$($borrow)* $s.inventory.ammunition[0].current);
        $f(ram::SAMUS_SUPER_MISSILES,$($borrow)* $s.inventory.ammunition[1].current);
        $f(ram::SAMUS_POWER_BOMBS,$($borrow)* $s.inventory.ammunition[2].current);
        $f(ram::GRAPPLE_BEAM_FUNCTION,$($borrow)* $s.player.grapple);
        $f(ram::SAMUS_SHINE_TIMER,$($borrow)* $s.player.shine_timer);
        $f(ram::TIMER_FOR_SHINE_TIMER,$($borrow)* $s.player.shine_tick);
        $f(ram::SPECIAL_SAMUS_PALETTE_FRAME,$($borrow)* $s.player.palette_frame);
        $f(ram::SAMUS_SPECIAL_SUPER_PALETTE_FLAGS,$($borrow)* $s.player.special_palette_flags);
        $f(ram::CONTROLLER1_INPUT_FOR_DEMO,$($borrow)* $s.projectiles.preparation.demo_held);
        $f(ram::CONTROLLER1_NEW_INPUT_FOR_DEMO,$($borrow)* $s.projectiles.preparation.demo_pressed);
        $f(ram::DEMO_BACKUP_PREV_CONTROLLER_INPUT,$($borrow)* $s.projectiles.preparation.demo_filtered_held);
        $f(ram::DEMO_BACKUP_PREV_CONTROLLER_INPUT_NEW,$($borrow)* $s.projectiles.preparation.demo_filtered_pressed);
        $f(ram::UNUSED_WORD_7E0DFA,$($borrow)* $s.projectiles.preparation.aim_history);
        $f(0xa10,$($borrow)* $s.projectiles.preparation.previous_movement_and_facing);
        $f(ram::POWER_BOMB_FLAG,$($borrow)* $s.projectiles.power_bomb.flag);
        $f(ram::POWER_BOMB_EXPLOSION_X_POS,$($borrow)* $s.projectiles.power_bomb.x);
        $f(ram::POWER_BOMB_EXPLOSION_Y_POS,$($borrow)* $s.projectiles.power_bomb.y);
        $f(ram::CUR_BLOCK_INDEX,$($borrow)* $s.room.collision_cursor);
        $f(ram::EARTHQUAKE_TYPE,$($borrow)* $s.feedback.earthquake_kind);
        $f(ram::EARTHQUAKE_TIMER,$($borrow)* $s.feedback.earthquake_timer);
        $f(ram::SPEED_ECHOES_INDEX,$($borrow)* $s.projectiles.echoes.index);
        for index in 0..3 {
            $f(ram::FLARE_ANIMATION_FRAME+index*2,$($borrow)* $s.projectiles.controls.flare_frames[index]);
            $f(ram::FLARE_ANIMATION_TIMER+index*2,$($borrow)* $s.projectiles.controls.flare_timers[index]);
        }
        for index in 0..4 {
            $f(arrays::SPEED_ECHO_XPOS+index*2,$($borrow)* $s.projectiles.echoes.x[index]);
            $f(arrays::SPEED_ECHO_YPOS+index*2,$($borrow)* $s.projectiles.echoes.y[index]);
            $f(arrays::SPEED_ECHO_XSPEED+index*2,$($borrow)* $s.projectiles.echoes.x_speed[index]);
        }
        for index in 0..10 {
            $f(0xb64+index*2,$($borrow)* $s.projectiles.shots[index].x);
            $f(0xb78+index*2,$($borrow)* $s.projectiles.shots[index].y);
            $f(0xb8c+index*2,$($borrow)* $s.projectiles.shots[index].x_fraction);
            $f(0xba0+index*2,$($borrow)* $s.projectiles.shots[index].y_fraction);
            $f(0xbb4+index*2,$($borrow)* $s.projectiles.shots[index].x_radius);
            $f(0xbc8+index*2,$($borrow)* $s.projectiles.shots[index].y_radius);
            $f(0xbdc+index*2,$($borrow)* $s.projectiles.shots[index].x_speed);
            $f(0xbf0+index*2,$($borrow)* $s.projectiles.shots[index].y_speed);
            $f(0xc04+index*2,$($borrow)* $s.projectiles.shots[index].direction);
            $f(0xc18+index*2,$($borrow)* $s.projectiles.shots[index].kind);
            $f(0xc2c+index*2,$($borrow)* $s.projectiles.shots[index].damage);
            $f(0xc40+index*2,$($borrow)* $s.projectiles.shots[index].cursor);
            $f(0xc54+index*2,$($borrow)* $s.projectiles.shots[index].remaining);
            $f(0xc68+index*2,$($borrow)* $s.projectiles.shots[index].motion);
            $f(0xc7c+index*2,$($borrow)* $s.projectiles.shots[index].variable);
            $f(0xc90+index*2,$($borrow)* $s.projectiles.shots[index].trail_time);
            $f(0xca4+index*2,$($borrow)* $s.projectiles.shots[index].auxiliary);
            $f(0xcb8+index*2,$($borrow)* $s.projectiles.shots[index].sprite);
        }
        for index in 0..18 {
            $f(0xd658+index*2,$($borrow)* $s.projectiles.trails[index].left.remaining);
            $f(0xd67c+index*2,$($borrow)* $s.projectiles.trails[index].right.remaining);
            $f(0xd6a0+index*2,$($borrow)* $s.projectiles.trails[index].left.cursor);
            $f(0xd6c4+index*2,$($borrow)* $s.projectiles.trails[index].right.cursor);
            $f(0xd6e8+index*2,$($borrow)* $s.projectiles.trails[index].left.tile);
            $f(0xd70c+index*2,$($borrow)* $s.projectiles.trails[index].right.tile);
            $f(0xd730+index*2,$($borrow)* $s.projectiles.trails[index].left.x);
            $f(0xd754+index*2,$($borrow)* $s.projectiles.trails[index].right.x);
            $f(0xd778+index*2,$($borrow)* $s.projectiles.trails[index].left.y);
            $f(0xd79c+index*2,$($borrow)* $s.projectiles.trails[index].right.y);
        }
        $f(0x12,$($borrow)* $s.spawn_scratch.header);
        $f(0x14,$($borrow)* $s.spawn_scratch.coordinates);
        $f(0x16,$($borrow)* $s.spawn_scratch.argument);
        $f(0xe20,$($borrow)* $s.compatibility_enemy_scratch);
        $f(0x9a2,$($borrow)* $s.inventory.equipment.active);
        $f(0x9a4,$($borrow)* $s.inventory.equipment.owned);
        $f(0x9a6,$($borrow)* $s.inventory.beams.active);
        $f(0x9a8,$($borrow)* $s.inventory.beams.owned);
        $f(0x9c2,$($borrow)* $s.inventory.health.current);
        $f(0xa4c,$($borrow)* $s.inventory.health_fraction);
        $f(0x9c4,$($borrow)* $s.inventory.health.maximum);
        $f(0x9c8,$($borrow)* $s.inventory.ammunition[0].maximum);
        $f(0x9cc,$($borrow)* $s.inventory.ammunition[1].maximum);
        $f(0x9d0,$($borrow)* $s.inventory.ammunition[2].maximum);
        $f(0x9d4,$($borrow)* $s.inventory.reserve_capacity);
        $f(0x9c0,$($borrow)* $s.inventory.reserve_mode);
        $f(0xa1c,$($borrow)* $s.player.pose);
        $f(0xa12,$($borrow)* $s.player.previous_health_for_flash);
        $f(0xa24,$($borrow)* $s.player.history.last_different.pose);
        $f(0x5e9,$($borrow)* $s.multiply_scratch.left);
        $f(0x5eb,$($borrow)* $s.multiply_scratch.right);
        $f(0x0,$($borrow)* $s.pointer_scratch.address);
        $f(0x2,$($borrow)* $s.pointer_scratch.bank);
        $f(0xa20,$($borrow)* $s.player.history.previous.pose);
        $f(0xa22,$($borrow)* $s.player.history.previous.movement_and_facing);
        $f(0xa26,$($borrow)* $s.player.history.last_different.movement_and_facing);
        $f(0xa34,$($borrow)* $s.player.clearance.actor_flags);
        $f(0xa36,$($borrow)* $s.player.clearance.terrain_flags);
        $f(0xa38,$($borrow)* $s.player.clearance.terrain_up);
        $f(0xa3a,$($borrow)* $s.player.clearance.radius_difference);
        $f(0xa3c,$($borrow)* $s.player.clearance.terrain_down);
        $f(0xa3e,$($borrow)* $s.player.clearance.actor_up);
        $f(0xa40,$($borrow)* $s.player.clearance.actor_down);
        $f(0xa46,$($borrow)* $s.player.collision.horizontal_slope_flags);
        $f(0xa94,$($borrow)* $s.player.animation_clock.remaining);
        $f(0xa96,$($borrow)* $s.player.animation_clock.frame);
        $f(0xafe,$($borrow)* $s.player.contact.x_radius);
        $f(0xb10,$($borrow)* $s.player.previous_position.x.whole);
        $f(0xb12,$($borrow)* $s.player.previous_position.x.fraction);
        $f(0xb14,$($borrow)* $s.player.previous_position.y.whole);
        $f(0xb16,$($borrow)* $s.player.previous_position.y.fraction);
        $f(0xb4a,$($borrow)* $s.player.contact.acceleration_mode);
        $f(0xdba,$($borrow)* $s.player.collision.slope_adjusted);
        $f(0xdbc,$($borrow)* $s.player.contact.total_x.whole);
        $f(0xdbe,$($borrow)* $s.player.contact.total_x.fraction);
        $f(0xdc6,$($borrow)* $s.player.collision.pose_input);
        $f(0xdce,$($borrow)* $s.player.collision.solid_actor_flags);
        $f(0xdd0,$($borrow)* $s.player.collision.flags);
        for index in 0..4 {
            $f(0xe04+index*2,$($borrow)* $s.player.collision.grapple_ejection[index]);
        }
        $f(0xdea,$($borrow)* $s.player.transitions.crystal_flash_ammo);
        $f(0xdec,$($borrow)* $s.shared_sequence.stage);
        $f(0xdee,$($borrow)* $s.shared_sequence.value);
        $f(0xa1e,$($borrow)* $s.player.movement_and_facing);
        $f(0x18a8,$($borrow)* $s.player.invincibility);
        $f(0xa50,$($borrow)* $s.player.periodic_damage);
        $f(0xb00,$($borrow)* $s.player.contact.radius);
        $f(0xb4c,$($borrow)* $s.player.contact.deceleration);
        $f(0xb3c,$($borrow)* $s.player.contact.momentum);
        $f(0xb3e,$($borrow)* $s.player.contact.speed_boost);
        $f(0xb44,$($borrow)* $s.player.contact.extra_run.fraction);
        $f(0xb42,$($borrow)* $s.player.contact.extra_run.whole);
        $f(0xb48,$($borrow)* $s.player.contact.base_run.fraction);
        $f(0xb46,$($borrow)* $s.player.contact.base_run.whole);
        $f(0xb36,$($borrow)* $s.player.contact.y_direction);
        $f(0xb2c,$($borrow)* $s.player.contact.y_speed.fraction);
        $f(0xb2e,$($borrow)* $s.player.contact.y_speed.whole);
        $f(0xb56,$($borrow)* $s.player.contact.external_x.fraction);
        $f(0xb58,$($borrow)* $s.player.contact.external_x.whole);
        $f(0xb5a,$($borrow)* $s.player.contact.external_y.fraction);
        $f(0xb5c,$($borrow)* $s.player.contact.external_y.whole);
        $f(0xb32,$($borrow)* $s.player.contact.y_acceleration.fraction);
        $f(0xb34,$($borrow)* $s.player.contact.y_acceleration.whole);
        $f(0xaf4,$($borrow)* $s.player.contact.auto_jump);
        $f(0xb02,$($borrow)* $s.player.contact.collision_direction);
        $f(0xa6e,$($borrow)* $s.player.contact.contact_attack);
        $f(0x1e73,$($borrow)* $s.player.contact.reaction_point);
        $f(0x1e71,$($borrow)* $s.player.contact.sand_visits);
        $f(0xa6c,$($borrow)* $s.player.contact.movement_speed_table);
        room_geometry_words!(($s.room), $f, ($($borrow)*));
        $f(0x7b9,$($borrow)* $s.room.size_bytes);
        $f(0x197e,$($borrow)* $s.room.liquid_options);
        $f(0x1978,$($borrow)* $s.room.liquid_height);
        $f(0x197a,$($borrow)* $s.room.liquid_target);
        $f(0x197c,$($borrow)* $s.room.liquid_velocity);
        $f(0x1980,$($borrow)* $s.room.liquid_delay);
        $f(0x195c,$($borrow)* $s.room.liquid.surface.fraction);
        $f(0x195e,$($borrow)* $s.room.liquid.surface.whole);
        $f(0x1960,$($borrow)* $s.room.liquid.lava_acid.fraction);
        $f(0x1962,$($borrow)* $s.room.liquid.lava_acid.whole);
        $f(0x1964,$($borrow)* $s.room.liquid.tilemap);
        $f(0x196c,$($borrow)* $s.room.liquid.rising);
        $f(0x196e,$($borrow)* $s.room.liquid.kind);
        $f(0x1970,$($borrow)* $s.room.liquid.offset.fraction);
        $f(0x1972,$($borrow)* $s.room.liquid.offset.whole);
        $f(0x1974,$($borrow)* $s.room.liquid.tide_phase);
        $f(0x1976,$($borrow)* $s.room.liquid.base_fraction);
        $f(0xe50,$($borrow)* $s.room.enemy_deaths);
        $f(0xe52,$($borrow)* $s.room.enemy_quota);
        $f(0xd91a,$($borrow)* $s.room.loaded_items);
        $f(0xcea,$($borrow)* $s.room.power_bomb_radius);
        for index in 0..4 { $f(0x7e1+index*2,$($borrow)* $s.room.main_variables[index]); }
        camera_words!(($s.room.camera), $f, ($($borrow)*));
        flow_words!(($s.flow), $f, ($($borrow)*));
        menu_words!(($s.menu), $f, ($($borrow)*));
        $f(0x5f5,$($borrow)* $s.feedback.sound_disabled);
        $f(0x5f7,$($borrow)* $s.minimap.disabled);
        for row in 0..3 {
            for column in 0..5 {
                $f(0xc608+2*(26+column+32*row),$($borrow)* $s.minimap.tiles[row][column]);
            }
        }
        $f(0xdc8,$($borrow)* $s.messages.queued);
        $f(0x1e75,$($borrow)* $s.mechanisms.save_locked);
        $f(0x1c23,$($borrow)* $s.mechanisms.flags);
        $f(0x1c25,$($borrow)* $s.mechanisms.draw_cursor);
        $f(0x1c27,$($borrow)* $s.mechanisms.selected);
        $f(0x1c29,$($borrow)* $s.mechanisms.block_x);
        $f(0x1c2b,$($borrow)* $s.mechanisms.block_y);
        $f(0x1c2d,$($borrow)* $s.mechanisms.next_graphics);
        for index in 0..4 { $f(0x1c2f+index*2,$($borrow)* $s.mechanisms.saved_graphics[index]); }
        for index in 0..256 { $f(0xc6c8+index*2,$($borrow)* $s.mechanisms.tilemap[index]); }
        for index in 0..3 { $f(0x1e67+index*2,$($borrow)* $s.mechanisms.custom_draw[index]); }
        for index in 0..40 {
            $f(0x1c37+index*2,$($borrow)* $s.mechanisms.slots[index].header);
            $f(0x1c87+index*2,$($borrow)* $s.mechanisms.slots[index].block);
            $f(0x1cd7+index*2,$($borrow)* $s.mechanisms.slots[index].pre_action);
            $f(0x1d27+index*2,$($borrow)* $s.mechanisms.slots[index].program);
            $f(0x1d77+index*2,$($borrow)* $s.mechanisms.slots[index].counter);
            $f(0x1dc7+index*2,$($borrow)* $s.mechanisms.slots[index].room_argument);
            $f(0x1e17+index*2,$($borrow)* $s.mechanisms.slots[index].work0);
            $f(0xde1c+index*2,$($borrow)* $s.mechanisms.slots[index].instruction_delay);
            $f(0xde6c+index*2,$($borrow)* $s.mechanisms.slots[index].draw_program);
            $f(0xdebc+index*2,$($borrow)* $s.mechanisms.slots[index].return_program);
            $f(0xdf0c+index*2,$($borrow)* $s.mechanisms.slots[index].work1);
        }
        $f(0x18b0,$($borrow)* $s.hdma.enabled);
        $f(0x18b2,$($borrow)* $s.hdma.selected);
        for index in 0..crate::hdma::OBJECTS {
            $f(0x18b4+index*2,$($borrow)* $s.hdma.objects[index].channels);
            $f(0x18c0+index*2,$($borrow)* $s.hdma.objects[index].bank_slot);
            $f(0x18cc+index*2,$($borrow)* $s.hdma.objects[index].instruction_list);
            $f(0x18d8+index*2,$($borrow)* $s.hdma.objects[index].table);
            $f(0x18e4+index*2,$($borrow)* $s.hdma.objects[index].instruction_timer);
            $f(0x18f0+index*2,$($borrow)* $s.hdma.objects[index].pre_instruction);
            $f(0x18fc+index*2,$($borrow)* $s.hdma.objects[index].pre_instruction_bank);
            $f(0x1908+index*2,$($borrow)* $s.hdma.objects[index].timer);
            $f(0x1914+index*2,$($borrow)* $s.hdma.objects[index].a);
            $f(0x1920+index*2,$($borrow)* $s.hdma.objects[index].b);
            $f(0x192c+index*2,$($borrow)* $s.hdma.objects[index].c);
            $f(0x1938+index*2,$($borrow)* $s.hdma.objects[index].d);
            $f(0x1944+index*2,$($borrow)* $s.hdma.objects[index].e);
            $f(0x1950+index*2,$($borrow)* $s.hdma.objects[index].f);
        }
        $f(0x198d,$($borrow)* $s.enemy_projectiles.flags);
        $f(0x1991,$($borrow)* $s.enemy_projectiles.selected);
        $f(0x1993,$($borrow)* $s.enemy_projectiles.spawn.primary);
        $f(0x1995,$($borrow)* $s.enemy_projectiles.spawn.secondary);
        $f(ram::HEALTH_DROP_BIAS_FLAG,$($borrow)* $s.enemy_projectiles.drop_bias);
        $f(ram::SAMUS_RESERVE_HEALTH,$($borrow)* $s.inventory.reserve_health);
        $f(ram::SAMUS_RESERVE_MISSILES,$($borrow)* $s.inventory.reserve_missiles);
        $f(ram::SAMUS_X_SPEED_DIVISOR,$($borrow)* $s.player.speed_divisor);
        $f(ram::SAMUS_KNOCKBACK_TIMER,$($borrow)* $s.player.knockback_timer);
        $f(ram::KNOCKBACK_X_DIR,$($borrow)* $s.player.knockback_x_direction);
        $f(ram::ENEMY_PROCESSING_STAGE,$($borrow)* $s.room.enemy_processing_stage);
        $f(ram::COLLISION_DETECTION_INDEX,$($borrow)* $s.room.collision_detection_index);
        $f(ram::CURRENT_SLOPE_BTS,$($borrow)* $s.room.slope_behavior);
        $f(ram::TOURIAN_ENTRANCE_STATUE_FINISHED,$($borrow)* $s.mechanisms.statue_finished);
        for index in 0..18 {
            $f(0x1997+index*2,$($borrow)* $s.enemy_projectiles.slots[index].definition);
            $f(0x19bb+index*2,$($borrow)* $s.enemy_projectiles.slots[index].graphics);
            $f(0x19df+index*2,$($borrow)* $s.enemy_projectiles.slots[index].counter);
            $f(0x1a03+index*2,$($borrow)* $s.enemy_projectiles.slots[index].pre_action);
            $f(0x1a27+index*2,$($borrow)* $s.enemy_projectiles.slots[index].x.fraction);
            $f(0x1a4b+index*2,$($borrow)* $s.enemy_projectiles.slots[index].x.whole);
            $f(0x1a6f+index*2,$($borrow)* $s.enemy_projectiles.slots[index].y.fraction);
            $f(0x1a93+index*2,$($borrow)* $s.enemy_projectiles.slots[index].y.whole);
            $f(0x1ab7+index*2,$($borrow)* $s.enemy_projectiles.slots[index].velocity.x);
            $f(0x1adb+index*2,$($borrow)* $s.enemy_projectiles.slots[index].velocity.y);
            $f(0x1aff+index*2,$($borrow)* $s.enemy_projectiles.slots[index].behavior_a);
            $f(0x1b23+index*2,$($borrow)* $s.enemy_projectiles.slots[index].behavior_b);
            $f(0x1b47+index*2,$($borrow)* $s.enemy_projectiles.slots[index].program);
            $f(0x1b6b+index*2,$($borrow)* $s.enemy_projectiles.slots[index].spritemap);
            $f(0x1b8f+index*2,$($borrow)* $s.enemy_projectiles.slots[index].delay);
            $f(0x1bb3+index*2,$($borrow)* $s.enemy_projectiles.slots[index].radius);
            $f(0x1bd7+index*2,$($borrow)* $s.enemy_projectiles.slots[index].properties);
            $f(0x1bfb+index*2,$($borrow)* $s.enemy_projectiles.slots[index].behavior_c);
            $f(0x97dc+index*2,$($borrow)* $s.enemy_projectiles.slots[index].angle);
            $f(0xf380+index*2,$($borrow)* $s.enemy_projectiles.slots[index].drop.flags);
            $f(0xf3c8+index*2,$($borrow)* $s.enemy_projectiles.slots[index].drop.enemy_definition);
            $f(0xf410+index*2,$($borrow)* $s.enemy_projectiles.slots[index].drop.killed_enemy);
        }
    };
}

pub fn read(bytes: &impl Fn(usize) -> u8, at: usize) -> u16 {
    u16::from_le_bytes([bytes(at), bytes(at + 1)])
}
pub fn decode(bytes: &impl Fn(usize) -> u8, byte_len: usize, world: Arc<WorldBinding>) -> Session {
    let mut s = Session::with_world(world);
    assert!(0x8f7 <= byte_len, "short minimap import");
    for (index, value) in s.minimap.explored.iter_mut().enumerate() {
        *value = bytes(0x7f7 + index);
    }
    macro_rules! load_key {
        ($at:expr, $field:ident, $get:ident, $set:ident) => {
            s.room.entry.$field = s.world.$set(read(bytes, $at));
        };
    }
    room_keys!(load_key);
    macro_rules! load_transition {
        ($at:expr, $field:expr, $ty:path) => {
            $field = <$ty>::from_raw(read(bytes, $at));
        };
    }
    transition_words!(s, load_transition);
    macro_rules! load_asset {
        ($at:expr, $field:ident, $slot:expr) => {
            assert!($at + 3 <= byte_len, "short room asset import");
            s.room.entry.$field = s
                .world
                .asset($slot, [bytes($at), bytes($at + 1), bytes($at + 2)]);
        };
    }
    room_assets!(load_asset);
    *s.control.primary_mut() = bytes(0x612);
    *s.control.secondary_mut() = bytes(0x613);
    s.room.entry.layer2_x = bytes(0x91b);
    s.room.entry.layer2_y = bytes(0x91c);
    for (base, flags) in [
        (0xd820, s.gameplay_flags.events.as_mut_slice()),
        (0xd828, s.gameplay_flags.bosses_by_area.as_mut_slice()),
        (0xd830, s.gameplay_flags.used_room_objects.as_mut_slice()),
        (0xd870, s.gameplay_flags.collected_locations.as_mut_slice()),
        (0xd8b0, s.gameplay_flags.opened_doors.as_mut_slice()),
    ] {
        let end = base + flags.len();
        assert!(end <= byte_len, "short gameplay flags import");
        for (index, value) in flags.iter_mut().enumerate() {
            *value = bytes(base + index);
        }
    }
    s.enemy_projectiles.spawn.angle = bytes(0x60b);
    let load = |at: usize, value: &mut u16| *value = read(bytes, at);
    scalar_words!(s,load,(&mut));
    for (index, value) in s.projectiles.launch.displacements.iter_mut().enumerate() {
        *value = ((u32::from(read(bytes, 0xdaa + index * 4)) << 16)
            | u32::from(read(bytes, 0xdac + index * 4))) as i32;
    }
    for (index, cell) in s.room.terrain.cells_mut().iter_mut().enumerate() {
        cell.level = read(bytes, 0x10002 + index * 2);
        cell.behavior = bytes(0x16402 + index);
    }
    for (index, tile) in s.room.terrain.tiles_mut().iter_mut().enumerate() {
        for (corner, value) in tile.iter_mut().enumerate() {
            *value = read(bytes, 0xa000 + index * 8 + corner * 2);
        }
    }
    crate::actors::codec::import_common(&mut s, bytes, byte_len);
    assert!(byte_len >= 0x8840, "short Botwoon death import");
    let load = |at, word: &mut u16| *word = read(bytes, at);
    botwoon_death_words!(s, load, (&mut));
    s
}
pub fn visit(s: &Session, mut visit: impl FnMut(usize, u16)) {
    // These signals formerly published immediately, before a fallible export.
    let mut emit = |at, word: &u16| visit(at, *word);
    botwoon_death_words!(s, emit, (&));
    for (index, pair) in s.minimap.explored.as_chunks::<2>().0.iter().enumerate() {
        visit(0x7f7 + 2 * index, u16::from_le_bytes([pair[0], pair[1]]));
    }
    crate::actors::codec::visit_common(s, &mut visit);
    macro_rules! emit_key {
        ($at:expr, $field:ident, $get:ident, $set:ident) => {
            visit($at, s.world.$get(&s.room.entry.$field));
        };
    }
    room_keys!(emit_key);
    macro_rules! emit_transition {
        ($at:expr, $field:expr, $ty:path) => {
            visit($at, $field.to_raw());
        };
    }
    transition_words!(s, emit_transition);
    for at in (0x7bd..0x7c9).step_by(2) {
        visit(
            at,
            u16::from_le_bytes([
                projected_byte(s, at).unwrap(),
                projected_byte(s, at + 1).unwrap(),
            ]),
        );
    }
    visit(
        0x612,
        u16::from_le_bytes([*s.control.primary(), *s.control.secondary()]),
    );
    visit(
        0x91b,
        u16::from_le_bytes([s.room.entry.layer2_x, s.room.entry.layer2_y]),
    );
    for (base, flags) in [
        (0xd820, s.gameplay_flags.events.as_slice()),
        (0xd828, s.gameplay_flags.bosses_by_area.as_slice()),
        (0xd830, s.gameplay_flags.used_room_objects.as_slice()),
        (0xd870, s.gameplay_flags.collected_locations.as_slice()),
        (0xd8b0, s.gameplay_flags.opened_doors.as_slice()),
    ] {
        for (index, pair) in flags.as_chunks::<2>().0.iter().enumerate() {
            visit(base + index * 2, u16::from_le_bytes([pair[0], pair[1]]));
        }
    }
    let mut emit = |at: usize, value: &u16| visit(at, *value);
    scalar_words!(s,emit,(&));
    for (index, &value) in s.projectiles.launch.displacements.iter().enumerate() {
        visit(0xdaa + index * 4, ((value as u32) >> 16) as u16);
        visit(0xdac + index * 4, value as u16);
    }
    for (index, cell) in s.room.terrain.cells().iter().enumerate() {
        visit(0x10002 + index * 2, cell.level);
    }
    for (index, tile) in s.room.terrain.tiles().iter().enumerate() {
        for (corner, &value) in tile.iter().enumerate() {
            visit(0xa000 + index * 8 + corner * 2, value);
        }
    }
}
pub fn encode(s: &Session, mut span: impl FnMut(usize, &[u8])) {
    span(0x60b, &[s.enemy_projectiles.spawn.angle]);
    visit(s, |at, value| span(at, &value.to_le_bytes()));
    for (index, cell) in s.room.terrain.cells().iter().enumerate() {
        span(0x16402 + index, &[cell.behavior]);
    }
    s.presentation.encode_projection(&mut span);
    s.control.encode_projection(span);
}
pub fn projected_byte(s: &Session, at: usize) -> Option<u8> {
    if let Some(value) = s.control.projected_byte(at) {
        return Some(value);
    }
    if let Some(value) = s.presentation.projected_byte(at) {
        return Some(value);
    }
    if let Some(value) = botwoon_death_byte(s, at) {
        return Some(value);
    }
    if (0x7f7..0x8f7).contains(&at) {
        return Some(s.minimap.explored[at - 0x7f7]);
    }
    if let Some(value) = crate::actors::codec::projected_byte(s, at) {
        return Some(value);
    }
    macro_rules! key_byte {
        ($base:expr, $field:ident, $get:ident, $set:ident) => {
            if ($base..$base + 2).contains(&at) {
                return Some(s.world.$get(&s.room.entry.$field).to_le_bytes()[at - $base]);
            }
        };
    }
    room_keys!(key_byte);
    macro_rules! transition_byte {
        ($base:expr, $field:expr, $ty:path) => {
            if ($base..$base + 2).contains(&at) {
                return Some($field.to_raw().to_le_bytes()[at - $base]);
            }
        };
    }
    transition_words!(s, transition_byte);
    macro_rules! asset_byte {
        ($base:expr, $field:ident, $slot:expr) => {
            if ($base..$base + 3).contains(&at) {
                return Some(s.world.asset_bytes(&s.room.entry.$field)[at - $base]);
            }
        };
    }
    room_assets!(asset_byte);
    match at {
        0x612 => return Some(*s.control.primary()),
        0x613 => return Some(*s.control.secondary()),
        0x91b => return Some(s.room.entry.layer2_x),
        0x91c => return Some(s.room.entry.layer2_y),
        _ => {}
    }
    if (0xd820..0xd8f0).contains(&at) {
        let offset = at - 0xd820;
        return Some(flag_projection!(s, offset, ()));
    }
    if (0xef78..0xf378).contains(&at) {
        let offset = at - 0xef78;
        let word = sprite_projection!(s,offset,(&));
        return Some(word.to_le_bytes()[offset % 2]);
    }
    if (0xc000..0xc400).contains(&at) {
        let offset = at - 0xc000;
        let colors = if offset < 512 {
            &s.presentation.visual().colors
        } else {
            &s.presentation.visual().targets
        };
        return Some(colors[(offset % 512) / 2].to_le_bytes()[offset % 2]);
    }
    if at == 0x60b {
        return Some(s.enemy_projectiles.spawn.angle);
    }
    if (0x10002..0x16402).contains(&at) {
        let offset = at - 0x10002;
        return Some(s.room.terrain.cells()[offset / 2].level.to_le_bytes()[offset % 2]);
    }
    if (0x16402..0x19602).contains(&at) {
        return Some(s.room.terrain.cells()[at - 0x16402].behavior);
    }
    if (0xa000..0xc000).contains(&at) {
        let offset = at - 0xa000;
        return Some(s.room.terrain.tiles()[offset / 8][offset % 8 / 2].to_le_bytes()[offset % 2]);
    }
    if (0xdaa..0xdba).contains(&at) {
        let index = (at - 0xdaa) / 4;
        let lane = (at - 0xdaa) % 4;
        let shift = if lane < 2 {
            16 + lane * 8
        } else {
            (lane - 2) * 8
        };
        return Some(((s.projectiles.launch.displacements[index] as u32) >> shift) as u8);
    }
    let mut result = None;
    let mut select = |base: usize, value: &u16| {
        if at == base || at == base + 1 {
            result = Some(value.to_le_bytes()[at - base]);
        }
    };
    scalar_words!(s,select,(&));
    result
}
pub fn write_projected_byte(s: &mut Session, at: usize, value: u8) -> bool {
    if s.control.write_projected_byte(at, value) {
        return true;
    }
    if s.presentation.write_projected_byte(at, value) {
        return true;
    }
    if write_botwoon_death_byte(s, at, value) {
        return true;
    }
    if (0x7f7..0x8f7).contains(&at) {
        s.minimap.explored[at - 0x7f7] = value;
        return true;
    }
    if crate::actors::codec::write_projected_byte(s, at, value) {
        return true;
    }
    macro_rules! write_key {
        ($base:expr, $field:ident, $get:ident, $set:ident) => {
            if ($base..$base + 2).contains(&at) {
                let mut bytes = s.world.$get(&s.room.entry.$field).to_le_bytes();
                bytes[at - $base] = value;
                s.room.entry.$field = s.world.$set(u16::from_le_bytes(bytes));
                return true;
            }
        };
    }
    room_keys!(write_key);
    macro_rules! write_transition {
        ($base:expr, $field:expr, $ty:path) => {
            if ($base..$base + 2).contains(&at) {
                let mut bytes = $field.to_raw().to_le_bytes();
                bytes[at - $base] = value;
                $field = <$ty>::from_raw(u16::from_le_bytes(bytes));
                return true;
            }
        };
    }
    transition_words!(s, write_transition);
    macro_rules! write_asset {
        ($base:expr, $field:ident, $slot:expr) => {
            if ($base..$base + 3).contains(&at) {
                let mut bytes = s.world.asset_bytes(&s.room.entry.$field);
                bytes[at - $base] = value;
                s.room.entry.$field = s.world.asset($slot, bytes);
                return true;
            }
        };
    }
    room_assets!(write_asset);
    let byte = match at {
        0x612 => Some(s.control.primary_mut()),
        0x613 => Some(s.control.secondary_mut()),
        0x91b => Some(&mut s.room.entry.layer2_x),
        0x91c => Some(&mut s.room.entry.layer2_y),
        _ => None,
    };
    if let Some(byte) = byte {
        *byte = value;
        return true;
    }
    if (0xd820..0xd8f0).contains(&at) {
        let offset = at - 0xd820;
        *flag_projection!(s,offset,(&mut)) = value;
        return true;
    }
    if (0xef78..0xf378).contains(&at) {
        let offset = at - 0xef78;
        let word = sprite_projection!(s,offset,(&mut));
        let mut bytes = word.to_le_bytes();
        bytes[offset % 2] = value;
        *word = u16::from_le_bytes(bytes);
        return true;
    }
    if (0xc000..0xc400).contains(&at) {
        let offset = at - 0xc000;
        let colors = if offset < 512 {
            &mut s.presentation.visual_mut().colors
        } else {
            &mut s.presentation.visual_mut().targets
        };
        let color = &mut colors[(offset % 512) / 2];
        let mut bytes = color.to_le_bytes();
        bytes[offset % 2] = value;
        *color = u16::from_le_bytes(bytes);
        return true;
    }
    if at == 0x60b {
        s.enemy_projectiles.spawn.angle = value;
        return true;
    }
    if (0x16402..0x19602).contains(&at) {
        s.room.terrain.cells_mut()[at - 0x16402].behavior = value;
        return true;
    }
    let word = if (0x10002..0x16402).contains(&at) {
        Some((
            &mut s.room.terrain.cells_mut()[(at - 0x10002) / 2].level,
            (at - 0x10002) % 2,
        ))
    } else if (0xa000..0xc000).contains(&at) {
        let offset = at - 0xa000;
        Some((
            &mut s.room.terrain.tiles_mut()[offset / 8][offset % 8 / 2],
            offset % 2,
        ))
    } else {
        None
    };
    if let Some((word, lane)) = word {
        let mut bytes = word.to_le_bytes();
        bytes[lane] = value;
        *word = u16::from_le_bytes(bytes);
        return true;
    }
    if (0xdaa..0xdba).contains(&at) {
        let index = (at - 0xdaa) / 4;
        let lane = (at - 0xdaa) % 4;
        let shift = if lane < 2 {
            16 + lane * 8
        } else {
            (lane - 2) * 8
        };
        let old = s.projectiles.launch.displacements[index] as u32;
        s.projectiles.launch.displacements[index] =
            ((old & !(0xff << shift)) | (u32::from(value) << shift)) as i32;
        return true;
    }
    let mut changed = false;
    let mut write = |base: usize, word: &mut u16| {
        if at == base || at == base + 1 {
            let mut bytes = word.to_le_bytes();
            bytes[at - base] = value;
            *word = u16::from_le_bytes(bytes);
            changed = true;
        }
    };
    scalar_words!(s,write,(&mut));
    changed
}
pub fn inputs(bytes: &impl Fn(usize) -> u8) -> ExternalInputs {
    let w = |at| read(bytes, at);
    ExternalInputs {
        transitions: crate::player_transitions::state::TransitionObservations {
            frame_gamma: transition_state::FrameGammaMode::from_raw(w(0xa5a)),
            debug_invincibility: w(0xde0),
            moonwalk_enabled: w(0x9e4),
            turn_request: w(0xdf8),
            game_state: w(0x998),
            game_time_frames: w(0x9da),
            fx_kind: w(0x196e),
        },
        debug_enemy_freeze: w(ram::DEBUG_TIME_FROZEN_FOR_ENEMIES),
        enemy_zero_health: u16::from_ne_bytes([bytes(0xf8c), bytes(0xf8d)]),
        blending_config: w(0x1982),
        lava_y: w(0x1962),
        current_enemy: w(ram::CUR_ENEMY_INDEX),
        random: w(ram::RANDOM_NUMBER),
        random_enemy_counter: w(ram::RANDOM_ENEMY_COUNTER),
        fx_y: w(ram::FX_Y_POS),
        drop_origin_x: w(ram::SPECIAL_DEATH_ITEM_DROP_X_ORIGIN_POS),
        drop_origin_y: w(ram::SPECIAL_DEATH_ITEM_DROP_Y_ORIGIN_POS),
        nmi_counter_low: bytes(0x5b5),

        shoot_button: w(ram::BUTTON_CONFIG_SHOOT_X),
        jump_button: w(ram::BUTTON_CONFIG_JUMP_A),
        cancel_button: w(ram::BUTTON_CONFIG_ITEMCANCEL_Y),
        switch_button: w(ram::BUTTON_CONFIG_ITEMSWITCH),
        run_button: w(ram::BUTTON_CONFIG_RUN_B),
        aim_up_button: w(ram::BUTTON_CONFIG_AIM_UP_R),
        aim_down_button: w(ram::BUTTON_CONFIG_AIM_DOWN_L),
        held: w(ram::JOYPAD1_LASTKEYS),
        pressed: w(ram::JOYPAD1_NEWKEYS),
        filtered_held: w(ram::JOYPAD1_INPUT_SAMUSFILTER),
        filtered_pressed: w(ram::JOYPAD1_NEWINPUT_SAMUSFILTER),
        frozen: w(ram::TIME_IS_FROZEN_FLAG),
        cinematic: w(ram::CINEMATIC_FUNCTION),
        hud_auto_cancel: w(ram::HUD_AUTO_CANCEL_FLAG),
        background_page_x: w(0x91d),
        power_bomb_active: w(0x592),
        camera_x: w(ram::LAYER1_X_POS),
        camera_y: w(ram::LAYER1_Y_POS),
        frame: w(ram::NMI_FRAME_COUNTER_WORD),
        ceres: w(ram::CERES_STATUS),
        m7_x: w(ram::REG_M7X),
        m7_y: w(ram::REG_M7Y),
        m7_a: w(ram::REG_M7A),
        m7_b: w(ram::REG_M7B),
        m7_c: w(ram::REG_M7C),
        grapple_x: w(ram::X_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME),
        grapple_y: w(ram::Y_POS_OF_START_OF_GRAPPLE_BEAM_PREVFRAME),
    }
}

#[cfg(test)]
mod player_lane_tests {
    //! The grapple writes these words as named `Session::player` fields inside a
    //! lease, so the projection must serve each word from exactly that field: no
    //! earlier owner may claim the address, and writing it touches no other lane.
    use super::{projected_byte, write_projected_byte};
    use crate::player_transitions::state::{
        DrawMode, HurtAction, InterruptionAction, MovementMode, PoseRequestWord,
    };
    use crate::room_entry::world::WorldBinding;
    use crate::session::Session;

    type Lane = (usize, fn(&mut Session) -> &mut u16);
    const LANES: &[Lane] = &[
        (0xd32, |s| &mut s.player.grapple),
        (0xcfe, |s| &mut s.player.grapple_length),
        (0xcf4, |s| &mut s.player.grapple_flags),
        (0xcfa, |s| &mut s.player.grapple_angle),
        (0xa1c, |s| &mut s.player.pose),
        (0xd08, |s| &mut s.player.grapple_endpoint.x),
        (0xd0c, |s| &mut s.player.grapple_endpoint.y),
        (0xd34, |s| &mut s.player.grapple_direction),
        (0xa96, |s| &mut s.player.animation_clock.frame),
        (0xa94, |s| &mut s.player.animation_clock.remaining),
        (0xa64, |s| &mut s.player.grapple_interaction_flags),
        (0xcf8, |s| &mut s.player.grapple_slow_scrolling),
        (0xa9e, |s| &mut s.player.transitions.grapple_walljump_timer),
        (0x18a8, |s| &mut s.player.invincibility),
        (0x18aa, |s| &mut s.player.knockback_timer),
        (0xb36, |s| &mut s.player.contact.y_direction),
        (0xb4a, |s| &mut s.player.contact.acceleration_mode),
        (0xb00, |s| &mut s.player.contact.radius),
        (0xaf6, |s| &mut s.player.x),
        (0xaf8, |s| &mut s.player.x_fraction),
        (0xafa, |s| &mut s.player.y),
        (0xafc, |s| &mut s.player.y_fraction),
        (0xb10, |s| &mut s.player.previous_position.x.whole),
        (0xb12, |s| &mut s.player.previous_position.x.fraction),
        (0xb14, |s| &mut s.player.previous_position.y.whole),
        (0xb16, |s| &mut s.player.previous_position.y.fraction),
        (0xcfc, |s| &mut s.player.swing.mirrored_angle),
        (0xd00, |s| &mut s.player.swing.length_delta),
        (0xd3e, |s| &mut s.player.swing.start_animation_timer),
        (0xd40, |s| &mut s.player.swing.start_animation_tiles),
        (0xd16, |s| &mut s.player.swing.start.x),
        (0xd18, |s| &mut s.player.swing.start.y),
        (0xd26, |s| &mut s.player.swing.angular_velocity),
        (0xd28, |s| &mut s.player.swing.angle_acceleration),
        (0xd2a, |s| &mut s.player.swing.input_acceleration),
        (0xd2c, |s| &mut s.player.swing.rising_deceleration),
        (0xd2e, |s| &mut s.player.swing.extra_angular_velocity),
        (0xd30, |s| &mut s.player.swing.kick_cooldown),
        (0xd38, |s| &mut s.player.swing.failed_movement_counter),
        (0xd3a, |s| &mut s.player.swing.start_marks[0]),
        (0xd3c, |s| &mut s.player.swing.start_marks[1]),
        (0xd90, |s| &mut s.player.swing.probe.x),
        (0xd92, |s| &mut s.player.swing.probe.y),
        (0xd94, |s| &mut s.player.swing.probe_block.x),
        (0xd96, |s| &mut s.player.swing.probe_block.y),
        (0xcd0, |s| &mut s.projectiles.controls.charge),
        (0x9c0, |s| &mut s.inventory.reserve_mode),
        (0x9c2, |s| &mut s.inventory.health.current),
        (0x9c4, |s| &mut s.inventory.health.maximum),
        (0x9c6, |s| &mut s.inventory.ammunition[0].current),
        (0x9c8, |s| &mut s.inventory.ammunition[0].maximum),
        (0x9ca, |s| &mut s.inventory.ammunition[1].current),
        (0x9cc, |s| &mut s.inventory.ammunition[1].maximum),
        (0x9ce, |s| &mut s.inventory.ammunition[2].current),
        (0x9d0, |s| &mut s.inventory.ammunition[2].maximum),
        (0x9d6, |s| &mut s.inventory.reserve_health),
        (0x9a2, |s| &mut s.inventory.equipment.active),
        (0x9d2, |s| &mut s.projectiles.controls.selected_weapon),
        (0xa04, |s| &mut s.projectiles.controls.automatic_weapon),
        (0xcd6, |s| &mut s.projectiles.controls.flare_frames[0]),
        (0xcd8, |s| &mut s.projectiles.controls.flare_frames[1]),
        (0xcda, |s| &mut s.projectiles.controls.flare_frames[2]),
        (0xcdc, |s| &mut s.projectiles.controls.flare_timers[0]),
        (0xcde, |s| &mut s.projectiles.controls.flare_timers[1]),
        (0xce0, |s| &mut s.projectiles.controls.flare_timers[2]),
    ];
    type RawLane = (usize, fn(&Session) -> u16, fn(&mut Session, u16));
    /// Lossless raw-word transition requests the grapple stores by name.
    const RAW_LANES: &[RawLane] = &[
        (
            0xa2a,
            |s| s.player.transitions.interrupted.to_raw(),
            |s, v| s.player.transitions.interrupted = PoseRequestWord::from_raw(v),
        ),
        (
            0xa2c,
            |s| s.player.transitions.transitional.to_raw(),
            |s, v| s.player.transitions.transitional = PoseRequestWord::from_raw(v),
        ),
        (
            0xa30,
            |s| s.player.transitions.interruption.to_raw(),
            |s, v| s.player.transitions.interruption = InterruptionAction::from_raw(v),
        ),
        (
            0xa32,
            |s| s.player.transitions.hurt.to_raw(),
            |s, v| s.player.transitions.hurt = HurtAction::from_raw(v),
        ),
        (
            0xa58,
            |s| s.player.movement_handler.to_raw(),
            |s, v| s.player.movement_handler = MovementMode::from_raw(v),
        ),
        (
            0xa5c,
            |s| s.player.transitions.draw_handler.to_raw(),
            |s, v| s.player.transitions.draw_handler = DrawMode::from_raw(v),
        ),
    ];

    fn session() -> Session {
        Session::with_world(WorldBinding::canonical_legacy())
    }
    fn projected_word(s: &Session, at: usize) -> Option<u16> {
        Some(u16::from_le_bytes([
            projected_byte(s, at)?,
            projected_byte(s, at + 1)?,
        ]))
    }

    #[test]
    fn player_lanes_project_from_their_fields() {
        for (index, &(at, field)) in LANES.iter().enumerate() {
            let value = 0x1357u16.wrapping_mul(index as u16 + 1) ^ 0x8001;
            let mut s = session();
            *field(&mut s) = value;
            assert_eq!(projected_word(&s, at), Some(value), "read {at:#x}");

            let mut s = session();
            let [low, high] = value.to_le_bytes();
            assert!(write_projected_byte(&mut s, at, low), "write {at:#x}");
            assert!(
                write_projected_byte(&mut s, at + 1, high),
                "write {:#x}",
                at + 1
            );
            assert_eq!(*field(&mut s), value, "write {at:#x}");
            for &(other, other_field) in LANES {
                if other != at {
                    assert_eq!(*other_field(&mut s), 0, "write {at:#x} touched {other:#x}");
                }
            }
        }
    }

    #[test]
    fn raw_transition_lanes_project_from_their_fields() {
        for (index, &(at, get, set)) in RAW_LANES.iter().enumerate() {
            let value = 0x2468u16.wrapping_mul(index as u16 + 1) ^ 0x8003;
            let mut s = session();
            set(&mut s, value);
            assert_eq!(projected_word(&s, at), Some(value), "read {at:#x}");

            let mut s = session();
            let [low, high] = value.to_le_bytes();
            assert!(write_projected_byte(&mut s, at, low), "write {at:#x}");
            assert!(write_projected_byte(&mut s, at + 1, high), "write {at:#x}");
            assert_eq!(get(&s), value, "write {at:#x}");
        }
    }
}
