//! The bosses' and enemies' small word tables through imported `sm_data` records.
//!
//! Each accessor serves the record while the loaded image is the imported
//! allocation; any other image, and indices past a table, keep the original
//! indexed load with its failures.
use crate::imported::ImportedWords;
use crate::records::Records;
use sm_data::actor_tables::*;
use sm_data::cartridge::RomImage;

#[derive(Default)]
pub struct ImportedActorTables {
    pub botwoon_head_lists: ImportedWords<BOTWOON_HEAD_INSTRUCTION_LISTS, EIGHTHS>,
    pub botwoon_spit_lists: ImportedWords<BOTWOON_SPIT_INSTRUCTION_LISTS, EIGHTHS>,
    pub draygon_health_palettes:
        ImportedWords<DRAYGON_HEALTH_PALETTES, DRAYGON_HEALTH_PALETTE_WORDS>,
    pub draygon_health_thresholds: ImportedWords<DRAYGON_HEALTH_THRESHOLDS, DRAYGON_HEALTH_STAGES>,
    pub draygon_turret_positions:
        ImportedWords<DRAYGON_TURRET_POSITIONS, DRAYGON_TURRET_POSITION_WORDS>,
    pub ridley_x_velocities: ImportedWords<RIDLEY_X_VELOCITY_ROWS, RIDLEY_VELOCITY_ROWS>,
    pub ridley_y_velocities: ImportedWords<RIDLEY_Y_VELOCITY_ROWS, RIDLEY_VELOCITY_ROWS>,
    pub ridley_upwards: ImportedWords<RIDLEY_UPWARDS_ACCELERATIONS, RIDLEY_ACCELERATION_STEPS>,
    pub ridley_downwards: ImportedWords<RIDLEY_DOWNWARDS_ACCELERATIONS, RIDLEY_ACCELERATION_STEPS>,
    pub maw_attack_lists: ImportedWords<MAW_ATTACK_INSTRUCTION_LISTS, EIGHTHS>,
}

pub fn import(image: &RomImage) -> ImportedActorTables {
    ImportedActorTables {
        botwoon_head_lists: ImportedWords::import(image),
        botwoon_spit_lists: ImportedWords::import(image),
        draygon_health_palettes: ImportedWords::import(image),
        draygon_health_thresholds: ImportedWords::import(image),
        draygon_turret_positions: ImportedWords::import(image),
        ridley_x_velocities: ImportedWords::import(image),
        ridley_y_velocities: ImportedWords::import(image),
        ridley_upwards: ImportedWords::import(image),
        ridley_downwards: ImportedWords::import(image),
        maw_attack_lists: ImportedWords::import(image),
    }
}

/// Word `index` of the selected table in the loaded image.
pub fn word<const ADDRESS: u32, const ROWS: usize>(
    game: &(impl Records + ?Sized),
    table: impl Fn(&ImportedActorTables) -> &ImportedWords<ADDRESS, ROWS>,
    index: isize,
) -> u16 {
    table(&game.records().actor_tables).word(game.loaded_image(), index)
}
