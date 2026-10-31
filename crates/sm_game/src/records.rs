//! The sm_data records imported from the loaded cartridge for bank-specific tables.
/// Imported by `register_rom`; each record serves only its source allocation.
#[derive(Default)]
pub struct RomRecords {
    pub sand_heap: Option<crate::sand_heap::ImportedSandHeap>,
    pub ceres_rotation: Option<crate::ceres_elevator::ImportedRotation>,
    pub shooting_stars: Option<crate::shooting_stars::ImportedShootingStars>,
    pub grapple_origins: Option<crate::grapple_origins::ImportedOrigins>,
    pub grapple_aim: crate::grapple_aim::ImportedAim,
    pub kraid_tables: crate::kraid_tables::ImportedKraid,
    pub shaktool_tables: crate::shaktool_tables::ImportedShaktool,
    pub statue_tables: crate::statue_tables::ImportedStatues,
    pub torizo_tables: crate::torizo_tables::ImportedTorizos,
    pub corpse_tables: crate::corpse_tables::ImportedCorpses,
    pub torizo_corpse_tables: crate::torizo_corpse_tables::ImportedTorizoCorpse,
    pub baby_metroid_tables: crate::baby_metroid_tables::ImportedBabyMetroid,
    pub mother_brain_tables: crate::mother_brain_tables::ImportedMotherBrain,
    pub flame_rain: Option<crate::phantoon_tables::ImportedFlameRain>,
    pub phantoon_timers: Option<crate::phantoon_tables::ImportedTimers>,
    pub ceres_ridley_tables: crate::ceres_ridley_tables::ImportedCeresRidley,
    pub ridley_tables: crate::ridley_data::ImportedRidley,
    pub ridley_explosion_tables: crate::ridley_explosion_data::ImportedRidleyExplosion,
    pub ceres_door_tables: crate::ceres_door_data::ImportedCeresDoors,
    pub zebetite_tables: crate::zebetite_data::ImportedZebetites,
    pub menu_lists: crate::menu_lists::ImportedMenuLists,
    pub boss_lists: crate::boss_lists::ImportedBossLists,
    pub gameplay_tables: crate::gameplay_tables::ImportedGameplayTables,
    pub enemy_lists: crate::enemy_lists::ImportedEnemyLists,
    pub samus_palettes: crate::samus_palette::ImportedPaletteTables,
    pub actor_tables: crate::actor_tables::ImportedActorTables,
    pub palette_programs: Option<crate::imported::Imported<sm_data::program_bank::ProgramBank>>,
    pub animated_programs: Option<crate::imported::Imported<sm_data::program_bank::ProgramBank>>,
    pub sprite_programs: Option<crate::imported::Imported<sm_data::program_bank::ProgramBank>>,
}

/// A host holding the loaded image and its imported records.
pub trait Records: crate::cartridge::LoadedImage {
    fn records(&self) -> &RomRecords;
}
