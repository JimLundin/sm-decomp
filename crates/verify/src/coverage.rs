//! Every State field with its decode status: the base of §11.2's coverage report, in which
//! each field has a decode rule or a named reason it has no equivalent.

use crate::source::{self, WramBlock, WramByte, WramWord};

/// Where a decoded field comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// A byte of work RAM.
    WramByte(WramByte),
    /// A word of work RAM.
    WramWord(WramWord),
    /// A run of work RAM bytes.
    WramBlock(WramBlock),
    /// A word of work RAM resolved through the content: a ROM pointer to an ID.
    WramPointer(WramWord),
    /// The main thread's wait site and return chain: the cut's interrupted PC and the stack
    /// from `cpu.s`, through `strand::WAIT_SITES` and `strand::CALL_SITES`, with a message box
    /// lag countdown from `cpu.x`.
    Stack,
    /// A field of the record itself, by its `FrameRecord` name.
    Record(&'static str),
}

/// A State field's decode status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// The decoder reads it from its source.
    Decoded(Source),
    /// The original has nothing it corresponds to, for the reason given: it keeps its
    /// default.
    NoEquivalent(&'static str),
    /// Not decoded yet: it keeps its default.
    NotDecoded,
}

/// A State field, by its `Canonical::diff` path below `state`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Field {
    /// The path, as `Canonical::diff` names it.
    pub path: &'static str,
    /// How it's decoded.
    pub status: Status,
}

const fn wram(path: &'static str, word: WramWord) -> Field {
    Field {
        path,
        status: Status::Decoded(Source::WramWord(word)),
    }
}

const fn wram_byte(path: &'static str, byte: WramByte) -> Field {
    Field {
        path,
        status: Status::Decoded(Source::WramByte(byte)),
    }
}

const fn record(path: &'static str, name: &'static str) -> Field {
    Field {
        path,
        status: Status::Decoded(Source::Record(name)),
    }
}

const fn none(path: &'static str, reason: &'static str) -> Field {
    Field {
        path,
        status: Status::NoEquivalent(reason),
    }
}

const fn open(path: &'static str) -> Field {
    Field {
        path,
        status: Status::NotDecoded,
    }
}

/// The State's fields, in declaration order, at the depth the decoder covers them.
pub const FIELDS: &[Field] = &[
    none(
        "clock.tick",
        "the Session's own Tick count; the original counts NMIs, which the counters hold",
    ),
    wram(
        "clock.counters.nmi",
        source::NMI_FRAME_COUNTER_INCLUDING_LAG,
    ),
    wram("clock.counters.completed", source::NMI_FRAME_COUNTER_WORD),
    wram_byte("clock.counters.lag", source::NMI_FRAMES_MISSED),
    wram_byte("clock.counters.max_lag", source::NMI_FRAMES_MISSED_MAX),
    wram_byte(
        "clock.counters.requested_byte",
        source::NMI_FRAME_COUNTER_BYTE,
    ),
    wram("clock.game_time.hours", source::GAME_TIME_HOURS),
    wram("clock.game_time.minutes", source::GAME_TIME_MINUTES),
    wram("clock.game_time.seconds", source::GAME_TIME_SECONDS),
    wram("clock.game_time.frames", source::GAME_TIME_FRAMES),
    wram("rng.word", source::RANDOM_NUMBER),
    none(
        "rng.calls",
        "the original advances its RNG word without counting the calls",
    ),
    record("controller.auto_read", "pad"),
    wram("controller.held", source::JOYPAD1_LASTKEYS),
    wram("controller.pressed", source::JOYPAD1_NEWKEYS),
    wram("controller.previous", source::JOYPAD1_PREV),
    wram("controller.repeat", source::JOYPAD1_NEWKEYS2_UNUSED),
    wram("controller.repeat_delay", source::JOYPAD1_KEYREPEAT_CTR),
    wram("mode", source::GAME_STATE),
    Field {
        path: "main.frames",
        status: Status::Decoded(Source::Stack),
    },
    record("main.waiting", "cut"),
    wram("irq.command", source::CUR_IRQ_HANDLER),
    record("irq.line", "ppu.vtime"),
    record("timing.last", "cut"),
    none(
        "timing.owed",
        "the cost model's accumulator; the original's CPU time is the record's timing itself",
    ),
    record("video.memory.vram", "vram"),
    record("video.memory.cgram", "cgram"),
    record("video.memory.oam", "oam"),
    record("video.committed", "ppu"),
    record("video.dma", "ppu"),
    wram_byte("video.batch.registers.forced_blank", source::REG_INIDISP),
    wram_byte("video.batch.registers.brightness", source::REG_INIDISP),
    wram_byte("video.batch.registers.main_screen", source::REG_TM),
    wram_byte("video.batch.registers.sub_screen", source::REG_TS),
    wram_byte("video.batch.registers.objects", source::REG_OBSEL),
    wram_byte("video.batch.registers.mode", source::REG_BGMODE),
    wram_byte("video.batch.registers.mosaic", source::REG_MOSAIC),
    wram_byte("video.batch.registers.tilemaps", source::REG_BG_SC[0]),
    wram_byte("video.batch.registers.tile_bases", source::REG_BG_NBA[0]),
    wram_byte("video.batch.registers.mode7.select", source::REG_M7SEL),
    wram("video.batch.registers.mode7.matrix", source::REG_M7[0]),
    wram("video.batch.registers.mode7.centre", source::REG_M7[4]),
    wram_byte(
        "video.batch.registers.windows.select",
        source::REG_WINDOW_SELECT[0],
    ),
    wram_byte(
        "video.batch.registers.windows.edges",
        source::REG_WINDOW_EDGES[0],
    ),
    wram_byte(
        "video.batch.registers.windows.background_logic",
        source::REG_WBGLOG,
    ),
    wram_byte(
        "video.batch.registers.windows.object_logic",
        source::REG_WOBJLOG,
    ),
    wram_byte("video.batch.registers.main_window_mask", source::REG_TMW),
    wram_byte("video.batch.registers.sub_window_mask", source::REG_TSW),
    wram_byte(
        "video.batch.registers.colour_math.select",
        source::REG_CGWSEL,
    ),
    wram_byte(
        "video.batch.registers.colour_math.layers",
        source::REG_CGADSUB,
    ),
    none(
        "video.batch.registers.colour_math.fixed",
        "the mirrors hold COLDATA's three writes, video.batch.fixed_colour",
    ),
    wram_byte("video.batch.registers.settings", source::REG_SETINI),
    wram_byte("video.batch.fixed_colour", source::REG_COLDATA[0]),
    Field {
        path: "video.batch.registers.scroll",
        status: Status::Decoded(Source::WramWord(source::REG_BG_SCROLL[0][0])),
    },
    Field {
        path: "video.batch.oam",
        status: Status::Decoded(Source::WramBlock(source::OAM)),
    },
    open("video.batch.sprites"),
    Field {
        path: "video.batch.cgram",
        status: Status::Decoded(Source::WramBlock(source::PALETTE_BUFFER)),
    },
    Field {
        path: "video.batch.vram",
        status: Status::Decoded(Source::WramBlock(source::VRAM_WRITE_QUEUE)),
    },
    wram_byte("video.nmi_requested", source::WAITING_FOR_NMI),
    wram("samus.position.x.pixels", source::SAMUS_X_POS),
    wram("samus.position.x.subpixels", source::SAMUS_X_SUBPOS),
    wram("samus.position.y.pixels", source::SAMUS_Y_POS),
    wram("samus.position.y.subpixels", source::SAMUS_Y_SUBPOS),
    wram("samus.pose", source::SAMUS_POSE),
    wram("samus.previous.pose", source::SAMUS_PREV_POSE),
    wram_byte("samus.previous.facing", source::SAMUS_PREV_POSE_X_DIR),
    wram_byte("samus.previous.movement", source::SAMUS_PREV_MOVEMENT_TYPE2),
    wram(
        "samus.last_different.pose",
        source::SAMUS_LAST_DIFFERENT_POSE,
    ),
    wram_byte(
        "samus.last_different.facing",
        source::SAMUS_LAST_DIFFERENT_POSE_X_DIR,
    ),
    wram_byte(
        "samus.last_different.movement",
        source::SAMUS_LAST_DIFFERENT_POSE_MOVEMENT_TYPE,
    ),
    wram_byte("samus.facing", source::SAMUS_POSE_X_DIR),
    open("samus.aim"),
    // The variant decodes from the movement type; the data it carries is C3's.
    wram_byte("samus.movement", source::SAMUS_MOVEMENT_TYPE),
    wram("samus.equipment.collected", source::COLLECTED_ITEMS),
    wram("samus.equipment.equipped", source::EQUIPPED_ITEMS),
    wram("samus.equipment.energy.current", source::SAMUS_HEALTH),
    wram("samus.equipment.energy.max", source::SAMUS_MAX_HEALTH),
    wram(
        "samus.equipment.reserve.current",
        source::SAMUS_RESERVE_HEALTH,
    ),
    wram(
        "samus.equipment.reserve.max",
        source::SAMUS_MAX_RESERVE_HEALTH,
    ),
    wram("samus.equipment.missiles.current", source::SAMUS_MISSILES),
    wram("samus.equipment.missiles.max", source::SAMUS_MAX_MISSILES),
    wram(
        "samus.equipment.super_missiles.current",
        source::SAMUS_SUPER_MISSILES,
    ),
    wram(
        "samus.equipment.super_missiles.max",
        source::SAMUS_MAX_SUPER_MISSILES,
    ),
    wram(
        "samus.equipment.power_bombs.current",
        source::SAMUS_POWER_BOMBS,
    ),
    wram(
        "samus.equipment.power_bombs.max",
        source::SAMUS_MAX_POWER_BOMBS,
    ),
    wram("samus.equipment.reserve_mode", source::RESERVE_HEALTH_MODE),
    open("samus.invincibility"),
    open("samus.animation"),
    Field {
        path: "room.room",
        status: Status::Decoded(Source::WramPointer(source::ROOM_PTR)),
    },
    open("room.entered_by"),
    open("room.blocks"),
    open("room.scroll_areas"),
    wram("room.camera.x.pixels", source::LAYER1_X_POS),
    wram("room.camera.x.subpixels", source::LAYER1_X_SUBPOS),
    wram("room.camera.y.pixels", source::LAYER1_Y_POS),
    wram("room.camera.y.subpixels", source::LAYER1_Y_SUBPOS),
    open("room.plms"),
    open("room.enemies"),
    Field {
        path: "hud.tilemap",
        status: Status::Decoded(Source::WramBlock(source::HUD_TILEMAP)),
    },
    wram("hud.selected", source::HUD_ITEM_INDEX),
    wram("hud.auto_cancel", source::SAMUS_AUTO_CANCEL_HUD_ITEM_INDEX),
    wram("hud.drawn.energy", source::SAMUS_PREV_HEALTH),
    wram("hud.drawn.missiles", source::SAMUS_PREV_MISSILES),
    wram(
        "hud.drawn.super_missiles",
        source::SAMUS_PREV_SUPER_MISSILES,
    ),
    wram("hud.drawn.power_bombs", source::SAMUS_PREV_POWER_BOMBS),
    wram("hud.drawn.selection", source::SAMUS_PREV_HUD_ITEM_INDEX),
    wram("hud.item_palette", source::HUD_ITEM_TILEMAP_PALETTE_BITS),
    wram("message", source::MESSAGE_BOX_INDEX),
];

#[cfg(test)]
mod tests {
    use super::{FIELDS, Source, Status};
    use crate::decode::Rules;
    use crate::source::WramByte;

    #[test]
    fn paths_are_unique() {
        for (index, field) in FIELDS.iter().enumerate() {
            assert!(
                FIELDS[..index].iter().all(|other| other.path != field.path),
                "{} listed twice",
                field.path
            );
        }
    }

    #[test]
    fn every_state_field_is_listed() {
        let top = [
            "clock",
            "rng",
            "controller",
            "mode",
            "main",
            "irq",
            "timing",
            "video",
            "samus",
            "room",
            "hud",
            "message",
        ];
        for name in top {
            assert!(
                FIELDS
                    .iter()
                    .any(|field| field.path.split('.').next() == Some(name)),
                "{name} has no row"
            );
        }
    }

    /// The byte sources the decoder reads.
    fn byte_sources(rules: &Rules<'_>) -> [WramByte; 29] {
        [
            rules.lag,
            rules.max_lag,
            rules.requested_byte,
            rules.nmi_requested,
            rules.inidisp,
            rules.main_screen,
            rules.sub_screen,
            rules.mirrors.objects,
            rules.mirrors.mode,
            rules.mirrors.mosaic,
            rules.mirrors.tilemaps[0],
            rules.mirrors.tile_bases[0],
            rules.mirrors.mode7_select,
            rules.mirrors.window_select[0],
            rules.mirrors.window_edges[0],
            rules.mirrors.background_logic,
            rules.mirrors.object_logic,
            rules.mirrors.main_window_mask,
            rules.mirrors.sub_window_mask,
            rules.mirrors.colour_math_select,
            rules.mirrors.colour_math_layers,
            rules.mirrors.fixed_colour[0],
            rules.mirrors.settings,
            rules.pose.facing,
            rules.pose.movement,
            rules.previous_pose.facing,
            rules.previous_pose.movement,
            rules.last_different_pose.facing,
            rules.last_different_pose.movement,
        ]
    }

    #[test]
    fn every_wram_rule_is_a_decoder_source() {
        let rules = Rules::ORIGINAL;
        let words = [
            rules.nmi,
            rules.completed,
            rules.hours,
            rules.minutes,
            rules.seconds,
            rules.frames,
            rules.rng,
            rules.held,
            rules.pressed,
            rules.previous,
            rules.repeat,
            rules.repeat_delay,
            rules.game_state,
            rules.samus_x,
            rules.samus_x_subpixels,
            rules.samus_y,
            rules.samus_y_subpixels,
            rules.room,
            rules.collected,
            rules.equipped,
            rules.energy,
            rules.max_energy,
            rules.reserve,
            rules.max_reserve,
            rules.missiles,
            rules.max_missiles,
            rules.super_missiles,
            rules.max_super_missiles,
            rules.power_bombs,
            rules.max_power_bombs,
            rules.reserve_mode,
            rules.hud.selected,
            rules.hud.auto_cancel,
            rules.hud.energy,
            rules.hud.missiles,
            rules.hud.super_missiles,
            rules.hud.power_bombs,
            rules.hud.selection,
            rules.hud.item_palette,
            rules.camera_x,
            rules.camera_x_subpixels,
            rules.camera_y,
            rules.camera_y_subpixels,
            rules.irq_command,
            rules.message,
            rules.pose.pose,
            rules.previous_pose.pose,
            rules.last_different_pose.pose,
            // The scroll row names the first mirror; the queue's tail bounds the queue block.
            rules.scroll[0][0],
            // An array of mirrors is named by its first; mode 7's centre by its own first.
            rules.mirrors.mode7[0],
            rules.mirrors.mode7[4],
        ];
        let bytes = byte_sources(&rules);
        let blocks = [
            rules.oam_buffer,
            rules.palette_buffer,
            rules.vram_queue,
            rules.hud.tilemap,
        ];
        for field in FIELDS {
            match field.status {
                Status::Decoded(Source::WramWord(word) | Source::WramPointer(word)) => {
                    assert!(words.contains(&word), "{} isn't read", field.path);
                }
                Status::Decoded(Source::WramByte(byte)) => {
                    assert!(bytes.contains(&byte), "{} isn't read", field.path);
                }
                Status::Decoded(Source::WramBlock(block)) => {
                    assert!(blocks.contains(&block), "{} isn't read", field.path);
                }
                Status::Decoded(Source::Record(_) | Source::Stack)
                | Status::NoEquivalent(_)
                | Status::NotDecoded => {}
            }
        }
        let listed = |source: Source| {
            FIELDS
                .iter()
                .any(|field| field.status == Status::Decoded(source))
        };
        for word in words {
            assert!(
                listed(Source::WramWord(word)) || listed(Source::WramPointer(word)),
                "{} has no row",
                word.name
            );
        }
        for byte in bytes {
            assert!(listed(Source::WramByte(byte)), "{} has no row", byte.name);
        }
        for block in blocks {
            assert!(
                listed(Source::WramBlock(block)),
                "{} has no row",
                block.name
            );
        }
    }

    #[test]
    fn every_field_without_an_equivalent_says_why() {
        let reasons = FIELDS.iter().filter_map(|field| match field.status {
            Status::NoEquivalent(reason) => Some((field.path, reason)),
            Status::Decoded(_) | Status::NotDecoded => None,
        });
        let mut count = 0_usize;
        for (path, reason) in reasons {
            assert!(!reason.trim().is_empty(), "{path} has no reason");
            count = count.wrapping_add(1);
        }
        assert!(count >= 2, "clock.tick and rng.calls have no equivalent");
    }
}
