//! Boss controls on an already-admitted scheduler lease; no Game implementation.
#![forbid(unsafe_code)]
use super::Runtime;
use sm_game::actors::boss_interaction::{self as interaction, Access, Array, Word};
use sm_game::actors::session_slots::SlotAccess as _;

fn address(word: Word) -> usize {
    match word {
        Word::QuakeSoundTimer => 0x9000,
        Word::CrocomireBridge1 => 0x9002,
        Word::CrocomireBridge2 => 0x9006,
        Word::CrocomireUnread => 0x900a,
        Word::CrocomireAcidSmoke => 0x9018,
        Word::PhantoonMaterialisationSound => 0x9032,
        Word::EnemyTilemapPending => 0xe1e,
        Word::EnemyTilemapSize => 0x179a,
        Word::BossId => 0x179c,
        Word::Unused179E => 0x179e,
        Word::MinimapDisabled => 0x5f7,
        Word::LavaAcidY => 0x1962,
        Word::DoorTransitionEnemies => 0x797,
        Word::CameraDistance => 0x941,
        Word::Random => 0x5e5,
        Word::FxLayerFlags => 0x1988,
        Word::MessageBoxScratch1 => 0x5a6,
        Word::RemovedR18 => 0x12,
        Word::RemovedR20 => 0x14,
        Word::RemovedR22 => 0x16,
        Word::AreaIndex => 0x79f,
        Word::RoomWidthInBlocks => 0x7a5,
        Word::ScrollingFinishedHook => 0x7e9,
        Word::CeresStatus => 0x93f,
        Word::TimerStatus => 0x943,
        Word::EquippedItems => 0x9a2,
        Word::CollectedItems => 0x9a4,
        Word::EquippedBeams => 0x9a6,
        Word::CollectedBeams => 0x9a8,
        Word::SamusHealth => 0x9c2,
        Word::SamusMaxHealth => 0x9c4,
        Word::SamusMissiles => 0x9c6,
        Word::SamusMaxMissiles => 0x9c8,
        Word::SamusSuperMissiles => 0x9ca,
        Word::SamusMaxSuperMissiles => 0x9cc,
        Word::SamusPowerBombs => 0x9ce,
        Word::SamusMaxPowerBombs => 0x9d0,
        Word::SamusMaxReserveHealth => 0x9d4,
        Word::SamusReserveHealth => 0x9d6,
        Word::JapaneseTextFlag => 0x9e2,
        Word::SamusPose => 0xa1c,
        Word::KnockbackXDir => 0xa54,
        Word::SamusGrappleFlags => 0xa64,
        Word::SamusXSpeedDivisor => 0xa66,
        Word::SamusXPos => 0xaf6,
        Word::SamusYPos => 0xafa,
        Word::SamusXRadius => 0xafe,
        Word::SamusYRadius => 0xb00,
        Word::ExtraSamusXSubdisplacement => 0xb56,
        Word::ExtraSamusXDisplacement => 0xb58,
        Word::ExtraSamusYDisplacement => 0xb5c,
        Word::ProjectileCounter => 0xcce,
        Word::PowerBombExplosionXPos => 0xce2,
        Word::PowerBombExplosionYPos => 0xce4,
        Word::PowerBombFlag => 0xcee,
        Word::GrappleBeamFunction => 0xd32,
        Word::RemovedVarE22 => 0xe22,
        Word::RemovedVarE2A => 0xe2a,
        Word::RandomEnemyCounter => 0xe44,
        Word::CurEnemyIndex => 0xe54,
        Word::FlagProcessAllEnemies => 0x183c,
        Word::EarthquakeType => 0x183e,
        Word::EarthquakeTimer => 0x1840,
        Word::CollisionDetectionIndex => 0x18a6,
        Word::SamusInvincibilityTimer => 0x18a8,
        Word::SamusKnockbackTimer => 0x18aa,
        Word::EprojInitParam2 => 0x1995,
        Word::MeltEraseTarget => 0x688,
        Word::MeltSubindex => 0x68a,
        Word::MeltWords => 0x68e,
        Word::MeltColumn => 0x690,
        Word::MeltDisplacement => 0x692,
        Word::MeltYOffset => 0x694,
        Word::MeltUnread => 0x696,
        Word::MeltMaxY => 0x698,
        Word::MeltIndex => 0x69a,
    }
}
fn collection_address(array: Array, index: usize) -> usize {
    let base = match array {
        Array::ProjectileXPos => 0xb64,
        Array::ProjectileYPos => 0xb78,
        Array::ProjectileXRadius => 0xbb4,
        Array::ProjectileYRadius => 0xbc8,
        Array::ProjectileDir => 0xc04,
        Array::ProjectileType => 0xc18,
        Array::PaletteBuffer => 0xc000,
    };
    base + 2 * index
}
impl Access for Runtime<'_> {
    fn boss_control(&self, word: Word) -> u16 {
        if self.persistent_is_current() {
            return *interaction::control(self.session(), word);
        }
        // The original SlotAccess byte view still serves a dirty lease.
        let at = address(word);
        u16::from_le_bytes([self.byte(at), self.byte(at + 1)])
    }
    fn set_boss_control(&mut self, word: Word, value: u16) {
        for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
            if !self.persistent_is_current() {
                self.set_byte(address(word) + lane, byte);
                continue;
            }
            let field = interaction::control_mut(self.session_mut(), word);
            let mut bytes = field.to_le_bytes();
            bytes[lane] = byte;
            *field = u16::from_le_bytes(bytes);
            self.refresh_written_inputs();
        }
    }
    fn boss_collection(&self, array: Array, index: usize) -> u16 {
        if self.persistent_is_current() {
            if let Some(value) = interaction::collection(self.session(), array, index) {
                return *value;
            }
        }
        // Full widened indices keep their original neighbouring-field aliases.
        let at = collection_address(array, index);
        u16::from_le_bytes([self.slot_byte(at), self.slot_byte(at + 1)])
    }
    fn set_boss_collection(&mut self, array: Array, index: usize, value: u16) {
        for (lane, byte) in value.to_le_bytes().into_iter().enumerate() {
            if !self.persistent_is_current()
                || interaction::collection(self.session(), array, index).is_none()
            {
                self.set_slot_byte(collection_address(array, index) + lane, byte);
                continue;
            }
            let field = interaction::collection_mut(self.session_mut(), array, index)
                .expect("admitted boss collection");
            let mut bytes = field.to_le_bytes();
            bytes[lane] = byte;
            *field = u16::from_le_bytes(bytes);
            self.refresh_written_inputs();
        }
    }
}
