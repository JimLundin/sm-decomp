//! The oracle projection: observations decoded from work RAM.
//!
//! Every address comes from the RAM map (`sm_game::ram`, also the verifier's
//! failure names) or the enemy record layout; none is written here.

use crate::game::prelude::EnemyData;
use core::mem::{offset_of, size_of};
use sm_game::ram::{RAM_VARS, bytes, word_arrays, words};
use sm_tools::observation::{
    Actor, Actors, Camera, Effects, Input, Music, Observation, Progress, Room, Samus, Sound,
};

/// The oracle's work RAM.
struct Ram<'a>(&'a [u8]);

impl Ram<'_> {
    fn byte(&self, at: usize) -> u8 {
        self.0[at]
    }
    fn word(&self, at: usize) -> u16 {
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    fn array<const N: usize>(&self, at: usize) -> [u8; N] {
        core::array::from_fn(|index| self.byte(at + index))
    }
}

/// The address of the RAM map entry `name`, and where the next entry starts.
fn ram_var_span(name: &str) -> (usize, usize) {
    let index = RAM_VARS
        .iter()
        .position(|var| var.name == name)
        .unwrap_or_else(|| panic!("RAM map has no {name}"));
    let start = RAM_VARS[index].addr;
    let end = RAM_VARS[index + 1..]
        .iter()
        .map(|var| var.addr)
        .find(|&addr| addr > start)
        .expect("RAM map entry has a successor");
    (
        usize::try_from(start).expect("RAM address"),
        usize::try_from(end).expect("RAM address"),
    )
}

/// The whole oracle observation of its work RAM `bytes`.
pub(crate) fn project(bytes: &[u8]) -> Observation {
    let ram = &Ram(bytes);
    Observation {
        input: input(ram),
        samus: samus(ram),
        camera: camera(ram),
        room: room(ram),
        progress: progress(ram),
        actors: actors(ram),
        effects: effects(ram),
    }
}

fn input(ram: &Ram<'_>) -> Input {
    Input {
        held: ram.word(words::JOYPAD1_LASTKEYS),
        pressed: ram.word(words::JOYPAD1_NEWKEYS),
        frame: ram.word(words::NMI_FRAME_COUNTER_WORD),
        frame_including_lag: ram.word(words::NMI_FRAME_COUNTER_INCLUDING_LAG),
        missed_frames: ram.byte(bytes::NMI_FRAMES_MISSED),
    }
}

fn samus(ram: &Ram<'_>) -> Samus {
    Samus {
        x: ram.word(words::SAMUS_X_POS),
        x_fraction: ram.word(words::SAMUS_X_SUBPOS),
        y: ram.word(words::SAMUS_Y_POS),
        y_fraction: ram.word(words::SAMUS_Y_SUBPOS),
        y_speed: ram.word(words::SAMUS_Y_SPEED),
        y_speed_fraction: ram.word(words::SAMUS_Y_SUBSPEED),
        pose: ram.word(words::SAMUS_POSE),
        // The facing byte and the movement type byte that follows it.
        movement_and_facing: ram.word(bytes::SAMUS_POSE_X_DIR),
        health: ram.word(words::SAMUS_HEALTH),
        missiles: ram.word(words::SAMUS_MISSILES),
        super_missiles: ram.word(words::SAMUS_SUPER_MISSILES),
        power_bombs: ram.word(words::SAMUS_POWER_BOMBS),
        equipment: ram.word(words::EQUIPPED_ITEMS),
        beams: ram.word(words::EQUIPPED_BEAMS),
        invincibility: ram.word(words::SAMUS_INVINCIBILITY_TIMER),
        shine_timer: ram.word(words::SAMUS_SHINE_TIMER),
        charge: ram.word(words::FLARE_COUNTER),
    }
}

fn camera(ram: &Ram<'_>) -> Camera {
    Camera {
        layer1_x: ram.word(words::LAYER1_X_POS),
        layer1_x_fraction: ram.word(words::LAYER1_X_SUBPOS),
        layer1_y: ram.word(words::LAYER1_Y_POS),
        layer1_y_fraction: ram.word(words::LAYER1_Y_SUBPOS),
        layer2_x: ram.word(words::LAYER2_X_POS),
        layer2_y: ram.word(words::LAYER2_Y_POS),
        distance_index: ram.word(words::CAMERA_DISTANCE_INDEX),
    }
}

fn room(ram: &Ram<'_>) -> Room {
    Room {
        area: ram.word(words::AREA_INDEX),
        room: ram.word(words::ROOM_PTR),
        state: ram.word(words::ROOMDEFROOMSTATE_PTR),
        door: ram.word(words::DOOR_DEF_PTR),
        width: ram.word(words::ROOM_WIDTH_IN_BLOCKS),
        height: ram.word(words::ROOM_HEIGHT_IN_BLOCKS),
    }
}

fn progress(ram: &Ram<'_>) -> Progress {
    Progress {
        events: ram.array(bytes::EVENTS_THAT_HAPPENED),
        bosses: ram.array(bytes::BOSS_BITS_FOR_AREA),
        items: ram.array(bytes::ITEM_BIT_ARRAY),
        doors: ram.array(bytes::OPENED_DOOR_BIT_ARRAY),
        collected_equipment: ram.word(words::COLLECTED_ITEMS),
        collected_beams: ram.word(words::COLLECTED_BEAMS),
        max_health: ram.word(words::SAMUS_MAX_HEALTH),
        max_missiles: ram.word(words::SAMUS_MAX_MISSILES),
    }
}

fn actors(ram: &Ram<'_>) -> Actors {
    let (first, end) = ram_var_span("enemy_data");
    let stride = size_of::<EnemyData>();
    let enemies = (first..end)
        .step_by(stride)
        .enumerate()
        .filter_map(|(slot, record)| {
            let word = |offset: usize| ram.word(record + offset);
            let species = word(offset_of!(EnemyData, enemy_ptr));
            (species != 0).then(|| Actor {
                slot: u16::try_from(slot).expect("enemy slot"),
                species,
                x: word(offset_of!(EnemyData, x_pos)),
                x_fraction: word(offset_of!(EnemyData, x_subpos)),
                y: word(offset_of!(EnemyData, y_pos)),
                y_fraction: word(offset_of!(EnemyData, y_subpos)),
                health: word(offset_of!(EnemyData, health)),
                ai_handlers: word(offset_of!(EnemyData, ai_handler_bits)),
                instruction: word(offset_of!(EnemyData, current_instruction)),
                frozen: word(offset_of!(EnemyData, frozen_timer)),
                invincibility: word(offset_of!(EnemyData, invincibility_timer)),
                ai_a: word(offset_of!(EnemyData, ai_var_a)),
                ai_b: word(offset_of!(EnemyData, ai_var_b)),
                ai_c: word(offset_of!(EnemyData, ai_var_c)),
                ai_d: word(offset_of!(EnemyData, ai_var_d)),
                ai_e: word(offset_of!(EnemyData, ai_var_e)),
            })
        })
        .collect();
    Actors { enemies }
}

fn effects(ram: &Ram<'_>) -> Effects {
    // Each library has a read and a write position and a ring of requests.
    let libraries = bytes::SFX_WRITEPOS - bytes::SFX_READPOS;
    let ring = bytes::SFX2_QUEUE - bytes::SFX1_QUEUE;
    let mut sounds = Vec::new();
    for library in 0..libraries {
        let queue = bytes::SFX1_QUEUE + ring * library;
        let mut read = usize::from(ram.byte(bytes::SFX_READPOS + library)) % ring;
        let write = usize::from(ram.byte(bytes::SFX_WRITEPOS + library)) % ring;
        while read != write {
            sounds.push(Sound {
                library: u8::try_from(library).expect("sound library"),
                sound: ram.byte(queue + read),
            });
            read = (read + 1) % ring;
        }
    }
    // The music queue's positions are byte offsets into its word arrays; the
    // game keeps them even, and an entry is observed at its word.
    let word = size_of::<u16>();
    let entries = (word_arrays::MUSIC_QUEUE_DELAY - word_arrays::MUSIC_QUEUE_TRACK) / word;
    let mut read = usize::from(ram.word(words::MUSIC_QUEUE_READ_POS)) / word % entries;
    let write = usize::from(ram.word(words::MUSIC_QUEUE_WRITE_POS)) / word % entries;
    let mut music = Vec::new();
    while read != write {
        music.push(Music {
            track: ram.word(word_arrays::MUSIC_QUEUE_TRACK + read * word),
            delay: ram.word(word_arrays::MUSIC_QUEUE_DELAY + read * word),
        });
        read = (read + 1) % entries;
    }
    Effects { sounds, music }
}
