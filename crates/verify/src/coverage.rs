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
    open("video.committed"),
    open("video.batch.registers"),
    Field {
        path: "video.batch.oam",
        status: Status::Decoded(Source::WramBlock(source::OAM)),
    },
    open("video.batch.sprites"),
    open("video.batch.vram"),
    wram_byte("video.nmi_requested", source::WAITING_FOR_NMI),
    wram("samus.position.x.pixels", source::SAMUS_X_POS),
    wram("samus.position.x.subpixels", source::SAMUS_X_SUBPOS),
    wram("samus.position.y.pixels", source::SAMUS_Y_POS),
    wram("samus.position.y.subpixels", source::SAMUS_Y_SUBPOS),
    open("samus.facing"),
    open("samus.aim"),
    open("samus.movement"),
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
];

#[cfg(test)]
mod tests {
    use super::{FIELDS, Source, Status};
    use crate::decode::Rules;

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
            rules.camera_x,
            rules.camera_x_subpixels,
            rules.camera_y,
            rules.camera_y_subpixels,
            rules.irq_command,
        ];
        let bytes = [rules.lag, rules.max_lag, rules.nmi_requested];
        let blocks = [rules.oam_buffer];
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
