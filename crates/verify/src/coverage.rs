//! Every State field with its decode status: the base of §11.2's coverage report, in which
//! each field has a decode rule or a named reason it has no equivalent.

use crate::source::{self, SlotField, WramArray, WramBlock, WramByte, WramTable, WramWord};

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
    /// A word of every slot in a work RAM table.
    WramSlots(WramTable, SlotField),
    /// A word of every slot in a work RAM array of one word per slot.
    WramArray(WramArray),
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

const fn block(path: &'static str, block: WramBlock) -> Field {
    Field {
        path,
        status: Status::Decoded(Source::WramBlock(block)),
    }
}

const fn enemy(path: &'static str, field: SlotField) -> Field {
    Field {
        path,
        status: Status::Decoded(Source::WramSlots(source::ENEMY_DATA, field)),
    }
}

const fn plm(path: &'static str, array: WramArray) -> Field {
    Field {
        path,
        status: Status::Decoded(Source::WramArray(array)),
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
    wram_byte("video.playfield.main_screen", source::GAMEPLAY_TM),
    wram_byte("video.playfield.bg3_tilemap", source::GAMEPLAY_BG3SC),
    wram_byte(
        "video.playfield.colour_math.select",
        source::GAMEPLAY_COLOUR_MATH[0],
    ),
    wram_byte(
        "video.playfield.colour_math.layers",
        source::GAMEPLAY_COLOUR_MATH[1],
    ),
    wram_byte(
        "video.batch.playfield_colour_math.select",
        source::NEXT_GAMEPLAY_COLOUR_MATH[0],
    ),
    wram_byte(
        "video.batch.playfield_colour_math.layers",
        source::NEXT_GAMEPLAY_COLOUR_MATH[1],
    ),
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
    // The halves' flags; each half's pointer is only meaningful with its flag set.
    wram("video.batch.samus_tiles", source::NMI_COPY_SAMUS_HALVES),
    wram(
        "video.batch.samus_tiles.top",
        source::NMI_COPY_SAMUS_TOP_HALF_SRC,
    ),
    wram(
        "video.batch.samus_tiles.bottom",
        source::NMI_COPY_SAMUS_BOTTOM_HALF_SRC,
    ),
    block(
        "video.batch.level.bg1.column.left",
        source::BG1_COLUMN_UPDATE.first_halves,
    ),
    block(
        "video.batch.level.bg1.column.right",
        source::BG1_COLUMN_UPDATE.second_halves,
    ),
    wram(
        "video.batch.level.bg1.column.first.to",
        source::BG1_COLUMN_UPDATE.unwrapped_dst,
    ),
    wram(
        "video.batch.level.bg1.column.first.bytes",
        source::BG1_COLUMN_UPDATE.unwrapped_size,
    ),
    wram(
        "video.batch.level.bg1.column.wrapped.to",
        source::BG1_COLUMN_UPDATE.wrapped_dst,
    ),
    wram(
        "video.batch.level.bg1.column.wrapped.bytes",
        source::BG1_COLUMN_UPDATE.wrapped_size,
    ),
    wram(
        "video.batch.level.bg1.column.requests",
        source::BG1_COLUMN_UPDATE.enable,
    ),
    block(
        "video.batch.level.bg1.row.top",
        source::BG1_ROW_UPDATE.first_halves,
    ),
    block(
        "video.batch.level.bg1.row.bottom",
        source::BG1_ROW_UPDATE.second_halves,
    ),
    wram(
        "video.batch.level.bg1.row.first.to",
        source::BG1_ROW_UPDATE.unwrapped_dst,
    ),
    wram(
        "video.batch.level.bg1.row.first.bytes",
        source::BG1_ROW_UPDATE.unwrapped_size,
    ),
    wram(
        "video.batch.level.bg1.row.wrapped.to",
        source::BG1_ROW_UPDATE.wrapped_dst,
    ),
    wram(
        "video.batch.level.bg1.row.wrapped.bytes",
        source::BG1_ROW_UPDATE.wrapped_size,
    ),
    wram(
        "video.batch.level.bg1.row.requests",
        source::BG1_ROW_UPDATE.enable,
    ),
    block(
        "video.batch.level.bg2.column.left",
        source::BG2_COLUMN_UPDATE.first_halves,
    ),
    block(
        "video.batch.level.bg2.column.right",
        source::BG2_COLUMN_UPDATE.second_halves,
    ),
    wram(
        "video.batch.level.bg2.column.first.to",
        source::BG2_COLUMN_UPDATE.unwrapped_dst,
    ),
    wram(
        "video.batch.level.bg2.column.first.bytes",
        source::BG2_COLUMN_UPDATE.unwrapped_size,
    ),
    wram(
        "video.batch.level.bg2.column.wrapped.to",
        source::BG2_COLUMN_UPDATE.wrapped_dst,
    ),
    wram(
        "video.batch.level.bg2.column.wrapped.bytes",
        source::BG2_COLUMN_UPDATE.wrapped_size,
    ),
    wram(
        "video.batch.level.bg2.column.requests",
        source::BG2_COLUMN_UPDATE.enable,
    ),
    block(
        "video.batch.level.bg2.row.top",
        source::BG2_ROW_UPDATE.first_halves,
    ),
    block(
        "video.batch.level.bg2.row.bottom",
        source::BG2_ROW_UPDATE.second_halves,
    ),
    wram(
        "video.batch.level.bg2.row.first.to",
        source::BG2_ROW_UPDATE.unwrapped_dst,
    ),
    wram(
        "video.batch.level.bg2.row.first.bytes",
        source::BG2_ROW_UPDATE.unwrapped_size,
    ),
    wram(
        "video.batch.level.bg2.row.wrapped.to",
        source::BG2_ROW_UPDATE.wrapped_dst,
    ),
    wram(
        "video.batch.level.bg2.row.wrapped.bytes",
        source::BG2_ROW_UPDATE.wrapped_size,
    ),
    wram(
        "video.batch.level.bg2.row.requests",
        source::BG2_ROW_UPDATE.enable,
    ),
    wram_byte("video.nmi_requested", source::WAITING_FOR_NMI),
    wram("samus.position.x.pixels", source::SAMUS_X_POS),
    wram("samus.position.x.subpixels", source::SAMUS_X_SUBPOS),
    wram("samus.position.y.pixels", source::SAMUS_Y_POS),
    wram("samus.position.y.subpixels", source::SAMUS_Y_SUBPOS),
    wram("samus.previous_position.x.pixels", source::SAMUS_PREV_X_POS),
    wram(
        "samus.previous_position.x.subpixels",
        source::SAMUS_PREV_X_SUBPOS,
    ),
    wram("samus.previous_position.y.pixels", source::SAMUS_PREV_Y_POS),
    wram(
        "samus.previous_position.y.subpixels",
        source::SAMUS_PREV_Y_SUBPOS,
    ),
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
    // The horizontal motion of the movements that keep one: standing, running, turning, the
    // jumps and the fall.
    wram(
        "samus.movement.speed.base.pixels",
        source::SAMUS_X_BASE_SPEED,
    ),
    wram(
        "samus.movement.speed.base.subpixels",
        source::SAMUS_X_BASE_SUBSPEED,
    ),
    wram(
        "samus.movement.speed.extra_run.pixels",
        source::SAMUS_X_EXTRA_RUN_SPEED,
    ),
    wram(
        "samus.movement.speed.extra_run.subpixels",
        source::SAMUS_X_EXTRA_RUN_SUBSPEED,
    ),
    wram(
        "samus.movement.speed.acceleration",
        source::SAMUS_X_ACCEL_MODE,
    ),
    wram(
        "samus.movement.speed.momentum",
        source::SAMUS_HAS_MOMENTUM_FLAG,
    ),
    // The jumps' and the fall's vertical motion.
    wram(
        "samus.movement.vertical.speed.pixels",
        source::SAMUS_Y_SPEED,
    ),
    wram(
        "samus.movement.vertical.speed.subpixels",
        source::SAMUS_Y_SUBSPEED,
    ),
    wram("samus.movement.vertical.direction", source::SAMUS_Y_DIR),
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
    wram("samus.animation.frame", source::SAMUS_ANIM_FRAME),
    wram("samus.animation.timer", source::SAMUS_ANIM_FRAME_TIMER),
    wram(
        "samus.animation.new_pose_frame",
        source::SAMUS_ANIM_FRAME_SKIP,
    ),
    wram(
        "samus.super_special.pose",
        source::SAMUS_NEW_POSE_TRANSITIONAL,
    ),
    wram(
        "samus.super_special.command",
        source::SAMUS_HURT_SWITCH_INDEX,
    ),
    wram("samus.visor.shade", source::SAMUS_VISOR_PALETTE_TIMER_INDEX),
    wram("samus.visor.timer", source::SAMUS_VISOR_PALETTE_TIMER_INDEX),
    wram("samus.drawing", source::SAMUS_DRAW_HANDLER),
    wram("samus.y_radius", source::SAMUS_Y_RADIUS),
    wram("samus.slope_adjusted", source::SAMUS_SLOPE_ADJUSTED),
    Field {
        path: "room.room",
        status: Status::Decoded(Source::WramPointer(source::ROOM_PTR)),
    },
    open("room.entered_by"),
    Field {
        path: "room.blocks",
        status: Status::Decoded(Source::WramBlock(source::LEVEL_DATA)),
    },
    wram("room.width", source::ROOM_WIDTH_IN_BLOCKS),
    // Present when the room state's level data has layer 2, which the content says.
    block("room.layer2", source::CUSTOM_BACKGROUND),
    block("room.level_tail", source::LEVEL_DATA),
    wram("room.area", source::AREA_INDEX),
    Field {
        path: "room.tileset",
        status: Status::Decoded(Source::WramPointer(source::ROOMDEFROOMSTATE_PTR)),
    },
    wram("room.tilemap_gap", source::SIZE_OF_BG2),
    wram("room.layer_blending", source::FX_LAYER_BLENDING_CONFIG_A),
    Field {
        path: "room.scroll_areas",
        status: Status::Decoded(Source::WramBlock(source::SCROLLS)),
    },
    Field {
        path: "room.stale_scrolls",
        status: Status::Decoded(Source::WramBlock(source::SCROLLS)),
    },
    wram("room.camera.x.pixels", source::LAYER1_X_POS),
    wram("room.camera.x.subpixels", source::LAYER1_X_SUBPOS),
    wram("room.camera.y.pixels", source::LAYER1_Y_POS),
    wram("room.camera.y.subpixels", source::LAYER1_Y_SUBPOS),
    wram("room.camera.layer2.x", source::LAYER2_X_POS),
    wram("room.camera.layer2.y", source::LAYER2_Y_POS),
    wram("room.camera.bg1_offset.x", source::BG1_X_OFFSET),
    wram("room.camera.bg1_offset.y", source::BG1_Y_OFFSET),
    wram("room.camera.bg2_offset.x", source::BG2_X_OFFSET),
    wram("room.camera.bg2_offset.y", source::BG2_Y_OFFSET),
    wram("room.camera.distance", source::CAMERA_DISTANCE_INDEX),
    wram("room.camera.scrollers.up", source::UP_SCROLLER),
    wram("room.camera.scrollers.down", source::DOWN_SCROLLER),
    wram_byte("room.camera.layer2_scroll.x", source::LAYER2_SCROLL_X),
    wram_byte("room.camera.layer2_scroll.y", source::LAYER2_SCROLL_Y),
    wram(
        "room.camera.drawn.layer1.x",
        source::PREVIOUS_LAYER1_X_BLOCK,
    ),
    wram(
        "room.camera.drawn.layer1.y",
        source::PREVIOUS_LAYER1_Y_BLOCK,
    ),
    wram(
        "room.camera.drawn.layer2.x",
        source::PREVIOUS_LAYER2_X_BLOCK,
    ),
    wram(
        "room.camera.drawn.layer2.y",
        source::PREVIOUS_LAYER2_Y_BLOCK,
    ),
    none(
        "room.plms.slots.generation",
        "the original keeps no count of a slot's fills; its ID word is the occupant",
    ),
    plm("room.plms.slots.common.block", source::PLM_BLOCK_INDICES),
    // open: the label stays at its default until the PLM programs decode (B7).
    open("room.plms.slots.common.cursor.at"),
    plm(
        "room.plms.slots.common.cursor.delay",
        source::PLM_INSTRUCTION_TIMER,
    ),
    open("room.plms.slots.common.cursor.link"),
    open("room.plms.slots.common.cursor.timer"),
    // The variant and the item from the ID word, through the room's population.
    plm("room.plms.slots.occupancy", source::PLM_HEADER_PTR),
    plm(
        "room.plms.slots.occupancy.changes",
        source::PLM_ROOM_ARGUMENTS,
    ),
    plm("room.plms.slots.occupancy.triggered", source::PLM_VARIABLE),
    plm(
        "room.plms.slots.occupancy.step",
        source::PLM_INSTR_LIST_PTRS,
    ),
    plm("room.plms.slots.occupancy.slot", source::PLM_ITEM_GFX_SLOT),
    plm(
        "room.plms.slots.occupancy.collected_bit",
        source::PLM_ROOM_ARGUMENTS,
    ),
    plm(
        "room.plms.slots.occupancy.crumble",
        source::PLM_INSTR_LIST_PTRS,
    ),
    wram("room.item_gfx.next", source::PLM_ITEM_GFX_INDEX),
    // Each slot's pointer names its item while the tile table's entries for the slot are
    // the ones that item's load wrote.
    block("room.item_gfx.loaded", source::PLM_ITEM_GFX_PTRS),
    none(
        "room.enemies.slots.generation",
        "the original keeps no count of a slot's fills; its species word is the occupant",
    ),
    enemy("room.enemies.slots.common.species", source::ENEMY_SPECIES),
    enemy(
        "room.enemies.slots.common.position.x.pixels",
        source::ENEMY_X_POS,
    ),
    enemy(
        "room.enemies.slots.common.position.x.subpixels",
        source::ENEMY_X_SUBPOS,
    ),
    enemy(
        "room.enemies.slots.common.position.y.pixels",
        source::ENEMY_Y_POS,
    ),
    enemy(
        "room.enemies.slots.common.position.y.subpixels",
        source::ENEMY_Y_SUBPOS,
    ),
    enemy("room.enemies.slots.common.radius_x", source::ENEMY_X_RADIUS),
    enemy("room.enemies.slots.common.radius_y", source::ENEMY_Y_RADIUS),
    enemy(
        "room.enemies.slots.common.properties",
        source::ENEMY_PROPERTIES,
    ),
    enemy(
        "room.enemies.slots.common.extra_properties",
        source::ENEMY_EXTRA_PROPERTIES,
    ),
    enemy(
        "room.enemies.slots.common.ai_handlers",
        source::ENEMY_AI_HANDLER_BITS,
    ),
    enemy("room.enemies.slots.common.health", source::ENEMY_HEALTH),
    enemy("room.enemies.slots.common.timer", source::ENEMY_TIMER),
    enemy(
        "room.enemies.slots.common.cursor.at",
        source::ENEMY_INSTRUCTION,
    ),
    enemy(
        "room.enemies.slots.common.cursor.delay",
        source::ENEMY_INSTRUCTION_TIMER,
    ),
    open("room.enemies.slots.common.cursor.link"),
    open("room.enemies.slots.common.cursor.timer"),
    enemy(
        "room.enemies.slots.common.spritemap",
        source::ENEMY_SPRITEMAP,
    ),
    enemy(
        "room.enemies.slots.common.palette",
        source::ENEMY_PALETTE_INDEX,
    ),
    enemy(
        "room.enemies.slots.common.tiles",
        source::ENEMY_VRAM_TILES_INDEX,
    ),
    enemy("room.enemies.slots.common.layer", source::ENEMY_LAYER),
    enemy("room.enemies.slots.common.flash", source::ENEMY_FLASH_TIMER),
    enemy(
        "room.enemies.slots.common.frozen",
        source::ENEMY_FROZEN_TIMER,
    ),
    enemy(
        "room.enemies.slots.common.invincibility",
        source::ENEMY_INVINCIBILITY_TIMER,
    ),
    enemy("room.enemies.slots.common.shake", source::ENEMY_SHAKE_TIMER),
    enemy(
        "room.enemies.slots.common.frame_counter",
        source::ENEMY_FRAME_COUNTER,
    ),
    enemy(
        "room.enemies.slots.common.parameter_1",
        source::ENEMY_PARAMETER_1,
    ),
    enemy(
        "room.enemies.slots.common.parameter_2",
        source::ENEMY_PARAMETER_2,
    ),
    // The variant from the species word, `Deleting` from the properties' deleted bit.
    enemy("room.enemies.slots.occupancy", source::ENEMY_SPECIES),
    // The AI words, named by the first.
    enemy(
        "room.enemies.slots.occupancy.variables",
        source::ENEMY_AI_VARIABLES[0],
    ),
    // Empty at every Tick's end: the drawing loop empties what the main enemy routine queued.
    open("room.drawing.queues"),
    open("room.drawing.routine_count"),
    wram("room.drawing.hook", source::ENEMY_GFX_DRAWN_HOOK),
    wram(
        "enemy_palette_cycle.palette",
        source::ENEMY_PALETTE_CYCLE_PALETTE,
    ),
    wram("enemy_palette_cycle.set", source::ENEMY_PALETTE_CYCLE_SET),
    wram(
        "enemy_palette_cycle.timer",
        source::ENEMY_PALETTE_CYCLE_TIMER,
    ),
    wram("elevator.status", source::ELEVATOR_STATUS),
    wram("elevator.armed", source::ELEVATOR_FLAGS),
    wram("elevator.direction", source::ELEVATOR_DIRECTION),
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
    Field {
        path: "hud.mini_map.explored",
        status: Status::Decoded(Source::WramBlock(source::MAP_TILES_EXPLORED)),
    },
    wram("hud.mini_map.area", source::AREA_INDEX),
    wram("hud.mini_map.room.x", source::ROOM_X_COORDINATE_ON_MAP),
    wram("hud.mini_map.room.y", source::ROOM_Y_COORDINATE_ON_MAP),
    wram("hud.mini_map.map_collected", source::HAS_AREA_MAP),
    wram("hud.mini_map.disabled", source::DEBUG_DISABLE_MINIMAP),
    wram("message", source::MESSAGE_BOX_INDEX),
    Field {
        path: "collected_items",
        status: Status::Decoded(Source::WramBlock(source::ITEM_BIT_ARRAY)),
    },
    record("sram", "sram"),
];

#[cfg(test)]
mod tests {
    use super::{FIELDS, Source, Status};
    use crate::decode::Rules;
    use crate::source::{SlotField, WramArray, WramByte, WramTable, WramWord};

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
            "elevator",
            "enemy_palette_cycle",
            "hud",
            "message",
            "collected_items",
            "sram",
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
    fn byte_sources(rules: &Rules<'_>) -> [WramByte; 37] {
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
            rules.playfield.main_screen,
            rules.playfield.bg3_tilemap,
            rules.playfield.colour_math[0],
            rules.playfield.colour_math[1],
            rules.playfield.next_colour_math[0],
            rules.playfield.next_colour_math[1],
            rules.pose.facing,
            rules.pose.movement,
            rules.previous_pose.facing,
            rules.previous_pose.movement,
            rules.last_different_pose.facing,
            rules.last_different_pose.movement,
            rules.scrolling.layer2_scroll[0],
            rules.scrolling.layer2_scroll[1],
        ]
    }

    /// The word sources the decoder reads.
    fn word_sources(rules: &Rules<'_>) -> Vec<WramWord> {
        vec![
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
            rules.level.width,
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
            rules.hud.mini_map.area,
            rules.hud.mini_map.room_x,
            rules.hud.mini_map.room_y,
            rules.hud.mini_map.map_collected,
            rules.hud.mini_map.disabled,
            rules.camera_x,
            rules.camera_x_subpixels,
            rules.camera_y,
            rules.camera_y_subpixels,
            rules.scrolling.layer2[0],
            rules.scrolling.layer2[1],
            rules.scrolling.bg1_offset[0],
            rules.scrolling.bg1_offset[1],
            rules.scrolling.bg2_offset[0],
            rules.scrolling.bg2_offset[1],
            rules.scrolling.distance,
            rules.scrolling.scrollers[0],
            rules.scrolling.scrollers[1],
            rules.previous_position[0],
            rules.previous_position[1],
            rules.previous_position[2],
            rules.previous_position[3],
            rules.irq_command,
            rules.message,
            rules.animation.frame,
            rules.animation.timer,
            rules.animation.new_pose_frame,
            rules.super_special.pose,
            rules.super_special.command,
            rules.visor,
            rules.samus_tiles.halves,
            rules.samus_tiles.top,
            rules.samus_tiles.bottom,
            rules.layer_blending,
            rules.drawing,
            rules.drawn_hook.pointer,
            rules.palette_cycle.palette,
            rules.palette_cycle.set,
            rules.palette_cycle.timer,
            rules.y_radius,
            rules.slope_adjusted,
            rules.elevator.status,
            rules.elevator.armed,
            rules.elevator.direction,
            // The scroll row names the first mirror; the queue's tail bounds the queue block.
            rules.scroll[0][0],
            // An array of mirrors is named by its first; mode 7's centre by its own first.
            rules.mirrors.mode7[0],
            rules.mirrors.mode7[4],
        ]
    }

    /// Samus's pose and motion word sources.
    fn motion_word_sources(rules: &Rules<'_>) -> Vec<WramWord> {
        vec![
            rules.pose.pose,
            rules.previous_pose.pose,
            rules.last_different_pose.pose,
            rules.horizontal.base,
            rules.horizontal.base_subpixels,
            rules.horizontal.extra_run,
            rules.horizontal.extra_run_subpixels,
            rules.horizontal.acceleration,
            rules.horizontal.momentum,
            rules.vertical.speed,
            rules.vertical.subpixels,
            rules.vertical.direction,
        ]
    }

    /// The BG graphics update's word sources, and the room state's.
    fn scrolling_word_sources(rules: &Rules<'_>) -> Vec<WramWord> {
        vec![
            rules.level.state,
            rules.area,
            rules.tilemap_gap,
            rules.item_gfx.next,
            rules.scrolling.drawn[0][0],
            rules.scrolling.drawn[0][1],
            rules.scrolling.drawn[1][0],
            rules.scrolling.drawn[1][1],
            rules.uploads[0].column.unwrapped_size,
            rules.uploads[0].column.wrapped_size,
            rules.uploads[0].column.unwrapped_dst,
            rules.uploads[0].column.wrapped_dst,
            rules.uploads[0].column.enable,
            rules.uploads[0].row.unwrapped_size,
            rules.uploads[0].row.wrapped_size,
            rules.uploads[0].row.unwrapped_dst,
            rules.uploads[0].row.wrapped_dst,
            rules.uploads[0].row.enable,
            rules.uploads[1].column.unwrapped_size,
            rules.uploads[1].column.wrapped_size,
            rules.uploads[1].column.unwrapped_dst,
            rules.uploads[1].column.wrapped_dst,
            rules.uploads[1].column.enable,
            rules.uploads[1].row.unwrapped_size,
            rules.uploads[1].row.wrapped_size,
            rules.uploads[1].row.unwrapped_dst,
            rules.uploads[1].row.wrapped_dst,
            rules.uploads[1].row.enable,
        ]
    }

    /// The slot sources the decoder reads; an array of words is named by its first.
    fn slot_sources(rules: &Rules<'_>) -> Vec<(WramTable, SlotField)> {
        let enemies = &rules.enemies;
        [
            enemies.species,
            enemies.x,
            enemies.x_subpixels,
            enemies.y,
            enemies.y_subpixels,
            enemies.radius_x,
            enemies.radius_y,
            enemies.properties,
            enemies.extra_properties,
            enemies.ai_handlers,
            enemies.health,
            enemies.timer,
            enemies.spritemap,
            enemies.instruction,
            enemies.instruction_timer,
            enemies.palette,
            enemies.tiles,
            enemies.layer,
            enemies.flash,
            enemies.frozen,
            enemies.invincibility,
            enemies.shake,
            enemies.frame_counter,
            enemies.variables[0],
            enemies.parameter_1,
            enemies.parameter_2,
        ]
        .into_iter()
        .map(|field| (enemies.table, field))
        .collect()
    }

    /// The per-slot arrays the decoder reads.
    const fn array_sources(rules: &Rules<'_>) -> [WramArray; 7] {
        let plms = &rules.plms;
        [
            plms.id,
            plms.block,
            plms.timer,
            plms.instruction,
            plms.argument,
            plms.variable,
            plms.item_gfx,
        ]
    }

    #[test]
    fn every_wram_rule_is_a_decoder_source() {
        let rules = Rules::ORIGINAL;
        let mut words = word_sources(&rules);
        words.extend(motion_word_sources(&rules));
        words.extend(scrolling_word_sources(&rules));
        let bytes = byte_sources(&rules);
        let slots = slot_sources(&rules);
        let arrays = array_sources(&rules);
        let blocks = [
            rules.oam_buffer,
            rules.palette_buffer,
            rules.vram_queue,
            rules.hud.tilemap,
            rules.hud.mini_map.explored,
            // The BTS and the room's size, in blocks, go with layer 1.
            rules.level.blocks,
            rules.level.scrolls,
            rules.level.layer2,
            rules.collected_items,
            rules.item_gfx.pointers,
            rules.uploads[0].column.first_halves,
            rules.uploads[0].column.second_halves,
            rules.uploads[0].row.first_halves,
            rules.uploads[0].row.second_halves,
            rules.uploads[1].column.first_halves,
            rules.uploads[1].column.second_halves,
            rules.uploads[1].row.first_halves,
            rules.uploads[1].row.second_halves,
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
                Status::Decoded(Source::WramSlots(table, slot)) => {
                    assert!(slots.contains(&(table, slot)), "{} isn't read", field.path);
                }
                Status::Decoded(Source::WramArray(array)) => {
                    assert!(arrays.contains(&array), "{} isn't read", field.path);
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
        for (table, field) in slots {
            assert!(
                listed(Source::WramSlots(table, field)),
                "{}.{} has no row",
                table.name,
                field.name
            );
        }
        for array in arrays {
            assert!(
                listed(Source::WramArray(array)),
                "{} has no row",
                array.name
            );
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
