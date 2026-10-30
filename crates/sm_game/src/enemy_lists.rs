//! Small per-enemy tables through the imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed load with its failures.
#![forbid(unsafe_code)]
use crate::imported::ImportedWords;
use crate::records::Records;
use sm_data::cartridge::RomImage;
use sm_data::enemy_lists::*;

#[derive(Default)]
pub struct ImportedEnemyLists {
    orb: ImportedWords<ORB_INSTRUCTION_LISTS, ORB_DIRECTIONS>,
    spark: ImportedWords<SPARK_INSTRUCTION_LISTS, SPARK_KINDS>,
    spark_functions: ImportedWords<SPARK_FUNCTIONS, SPARK_KINDS>,
    face_block_glow: ImportedWords<FACE_BLOCK_GLOW_COLOURS, FACE_BLOCK_GLOW_WORDS>,
    kihunter_wings: ImportedWords<KIHUNTER_WING_INSTRUCTION_LISTS, KIHUNTER_WING_LISTS>,
    floater_spikes: ImportedWords<FLOATER_SPIKE_INSTRUCTION_LISTS, FLOATER_SPIKE_LISTS>,
    eye: ImportedWords<EYE_INSTRUCTION_LISTS, EYE_FACINGS>,
    pipe_bug: ImportedWords<PIPE_BUG_INSTRUCTION_LISTS, PIPE_BUG_PHASES>,
    pipe_bug_alternate: ImportedWords<PIPE_BUG_ALTERNATE_INSTRUCTION_LISTS, PIPE_BUG_PHASES>,
}

pub fn import(image: &RomImage) -> ImportedEnemyLists {
    ImportedEnemyLists {
        orb: ImportedWords::import(image),
        spark: ImportedWords::import(image),
        spark_functions: ImportedWords::import(image),
        face_block_glow: ImportedWords::import(image),
        kihunter_wings: ImportedWords::import(image),
        floater_spikes: ImportedWords::import(image),
        eye: ImportedWords::import(image),
        pipe_bug: ImportedWords::import(image),
        pipe_bug_alternate: ImportedWords::import(image),
    }
}

fn lists(game: &(impl Records + ?Sized)) -> &ImportedEnemyLists {
    &game.records().enemy_lists
}

/// $A8:E380: the orb instruction list for a starting direction.
pub fn orb_instruction_list(game: &(impl Records + ?Sized), direction: isize) -> u16 {
    lists(game).orb.word(game.loaded_image(), direction)
}

/// $A8:E682: the spark instruction list for a spark kind.
pub fn spark_instruction_list(game: &(impl Records + ?Sized), kind: isize) -> u16 {
    lists(game).spark.word(game.loaded_image(), kind)
}

/// $A8:E688: the spark AI function for a spark kind.
pub fn spark_function(game: &(impl Records + ?Sized), kind: isize) -> u16 {
    lists(game).spark_functions.word(game.loaded_image(), kind)
}

/// $A8:E7CC: one glow colour; four consecutive words make a phase.
pub fn face_block_glow_colour(game: &(impl Records + ?Sized), index: isize) -> u16 {
    lists(game).face_block_glow.word(game.loaded_image(), index)
}

/// $A8:F3B0: a Kihunter wing instruction list.
pub fn kihunter_wing_instruction_list(game: &(impl Records + ?Sized), index: isize) -> u16 {
    lists(game).kihunter_wings.word(game.loaded_image(), index)
}

/// $A8:C599: a Maridia floater spike instruction list.
pub fn floater_spike_instruction_list(game: &(impl Records + ?Sized), index: isize) -> u16 {
    lists(game).floater_spikes.word(game.loaded_image(), index)
}

/// $A8:90DA: the morph ball eye instruction list for a facing.
pub fn eye_instruction_list(game: &(impl Records + ?Sized), facing: isize) -> u16 {
    lists(game).eye.word(game.loaded_image(), facing)
}

/// $B3:882B or $B3:8833: the pipe bug instruction list for a phase.
pub fn pipe_bug_instruction_list(
    game: &(impl Records + ?Sized),
    alternate: bool,
    phase: isize,
) -> u16 {
    let lists = lists(game);
    if alternate {
        lists.pipe_bug_alternate.word(game.loaded_image(), phase)
    } else {
        lists.pipe_bug.word(game.loaded_image(), phase)
    }
}
