//! Mother Brain controls use the original typed Session owners.
#![forbid(unsafe_code)]
pub trait State: crate::actors::boss_fields_owned::State {
    fn store_control_colour(&mut self, index: usize, value: u16);
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OwnedWord {
    SamusHealth,
    MaxReserveHealth,
    ReserveHealth,
    Invincibility,
    KnockbackTimer,
    KnockbackDirection,
    SpeedDivisor,
    SamusYSpeed,
    SamusPaletteFrame,
    Cooldown,
    BombCounter,
    PowerBomb,
    ShotX(usize),
    ShotY(usize),
    ShotKind(usize),
    ShotDamage(usize),
    CollisionIndex,
    EarthquakeType,
    EarthquakeTimer,
    Color(usize),
    RandomNumber,
    RandomEnemyCounter,
    TimeFrozen,
    CollidingEnemy(usize),
    Bg2TilemapSize,
    Bg2TilemapPending,
    PaletteFadeStep,
    TimerStatus,
    JapaneseText,
    FxBlending,
}
impl OwnedWord {
    /// Address metadata is used solely to preserve exceptional full indices.
    pub fn address(self) -> usize {
        match self {
            Self::SamusHealth => 0x9c2,
            Self::MaxReserveHealth => 0x9d4,
            Self::ReserveHealth => 0x9d6,
            Self::Invincibility => 0x18a8,
            Self::KnockbackTimer => 0x18aa,
            Self::KnockbackDirection => 0xa54,
            Self::SpeedDivisor => 0xa66,
            Self::SamusYSpeed => 0xb2e,
            Self::SamusPaletteFrame => 0xace,
            Self::Cooldown => 0xccc,
            Self::BombCounter => 0xcd2,
            Self::PowerBomb => 0xcee,
            Self::ShotX(index) => 0xb64 + 2 * index,
            Self::ShotY(index) => 0xb78 + 2 * index,
            Self::ShotKind(index) => 0xc18 + 2 * index,
            Self::ShotDamage(index) => 0xc2c + 2 * index,
            Self::CollisionIndex => 0x18a6,
            Self::EarthquakeType => 0x183e,
            Self::EarthquakeTimer => 0x1840,
            Self::Color(index) => 0xc000 + 2 * index,
            Self::RandomNumber => 0x5e5,
            Self::RandomEnemyCounter => 0xe44,
            Self::TimeFrozen => 0xa78,
            Self::CollidingEnemy(direction) => 0x182c + 2 * direction,
            Self::Bg2TilemapSize => 0x179a,
            Self::Bg2TilemapPending => 0xe1e,
            Self::PaletteFadeStep => 0xc400,
            Self::TimerStatus => 0x943,
            Self::JapaneseText => 0x9e2,
            Self::FxBlending => 0x1982,
        }
    }
    fn exceptional(self) -> bool {
        match self {
            Self::ShotX(i) | Self::ShotY(i) | Self::ShotKind(i) | Self::ShotDamage(i) => i >= 10,
            Self::Color(i) => i >= 256,
            Self::CollidingEnemy(i) => i >= 4,
            _ => false,
        }
    }
}
pub trait OwnedWords: State {
    fn owned(&self, word: OwnedWord) -> u16 {
        if word.exceptional() {
            let at = word.address();
            return u16::from_le_bytes([
                crate::actors::boss_fields_owned::read_byte(self, at),
                crate::actors::boss_fields_owned::read_byte(self, at + 1),
            ]);
        }
        if word == OwnedWord::FxBlending {
            return crate::fx_encoding::control_word(self.session(), 0x1982);
        }
        let session = self.session();
        match word {
            OwnedWord::SamusHealth => session.inventory.health.current,
            OwnedWord::MaxReserveHealth => session.inventory.reserve_capacity,
            OwnedWord::ReserveHealth => session.inventory.reserve_health,
            OwnedWord::Invincibility => session.player.invincibility,
            OwnedWord::KnockbackTimer => session.player.knockback_timer,
            OwnedWord::KnockbackDirection => session.player.knockback_x_direction,
            OwnedWord::SpeedDivisor => session.player.speed_divisor,
            OwnedWord::SamusYSpeed => session.player.contact.y_speed.whole,
            OwnedWord::SamusPaletteFrame => session.player.palette_frame,
            OwnedWord::Cooldown => session.projectiles.controls.cooldown,
            OwnedWord::BombCounter => session.projectiles.bomb_count,
            OwnedWord::PowerBomb => session.projectiles.power_bomb.flag,
            OwnedWord::ShotX(index) => session.projectiles.shots[index].x,
            OwnedWord::ShotY(index) => session.projectiles.shots[index].y,
            OwnedWord::ShotKind(index) => session.projectiles.shots[index].kind,
            OwnedWord::ShotDamage(index) => session.projectiles.shots[index].damage,
            OwnedWord::CollisionIndex => session.room.collision_detection_index,
            OwnedWord::EarthquakeType => session.feedback.earthquake_kind,
            OwnedWord::EarthquakeTimer => session.feedback.earthquake_timer,
            OwnedWord::Color(index) => session.presentation.visual().colors[index],
            OwnedWord::RandomNumber => session.timing.random_number,
            OwnedWord::RandomEnemyCounter => session.actors.controls.phase_counter,
            OwnedWord::TimeFrozen => session.timing.time_frozen,
            OwnedWord::CollidingEnemy(direction) => session.actors.schedule.colliding[direction],
            OwnedWord::Bg2TilemapSize => session.actors.drawing.tilemap_size,
            OwnedWord::Bg2TilemapPending => session.actors.drawing.tilemap_pending,
            OwnedWord::PaletteFadeStep => session.presentation.visual().palette_fade.step,
            OwnedWord::TimerStatus => session.timing.timer_status,
            OwnedWord::JapaneseText => session.options.japanese_text,
            OwnedWord::FxBlending => session.room.fx_blending,
        }
    }
    fn set_owned(&mut self, word: OwnedWord, value: u16) {
        if word.exceptional() {
            let at = word.address();
            let [first, second] = value.to_le_bytes();
            crate::actors::boss_fields_owned::store_byte(self, at, first);
            crate::actors::boss_fields_owned::store_byte(self, at + 1, second);
            return;
        }
        if word == OwnedWord::FxBlending {
            crate::fx_encoding::write_control_word(self.session_mut(), 0x1982, value);
            return;
        }
        if let OwnedWord::Color(index) = word {
            self.store_control_colour(index, value);
            return;
        }
        let session = self.session_mut();
        match word {
            OwnedWord::SamusHealth => session.inventory.health.current = value,
            OwnedWord::MaxReserveHealth => session.inventory.reserve_capacity = value,
            OwnedWord::ReserveHealth => session.inventory.reserve_health = value,
            OwnedWord::Invincibility => session.player.invincibility = value,
            OwnedWord::KnockbackTimer => session.player.knockback_timer = value,
            OwnedWord::KnockbackDirection => session.player.knockback_x_direction = value,
            OwnedWord::SpeedDivisor => session.player.speed_divisor = value,
            OwnedWord::SamusYSpeed => session.player.contact.y_speed.whole = value,
            OwnedWord::SamusPaletteFrame => session.player.palette_frame = value,
            OwnedWord::Cooldown => session.projectiles.controls.cooldown = value,
            OwnedWord::BombCounter => session.projectiles.bomb_count = value,
            OwnedWord::PowerBomb => session.projectiles.power_bomb.flag = value,
            OwnedWord::ShotX(index) => session.projectiles.shots[index].x = value,
            OwnedWord::ShotY(index) => session.projectiles.shots[index].y = value,
            OwnedWord::ShotKind(index) => session.projectiles.shots[index].kind = value,
            OwnedWord::ShotDamage(index) => session.projectiles.shots[index].damage = value,
            OwnedWord::CollisionIndex => session.room.collision_detection_index = value,
            OwnedWord::EarthquakeType => session.feedback.earthquake_kind = value,
            OwnedWord::EarthquakeTimer => session.feedback.earthquake_timer = value,
            OwnedWord::Color(index) => session.presentation.visual_mut().colors[index] = value,
            OwnedWord::RandomNumber => session.timing.random_number = value,
            OwnedWord::RandomEnemyCounter => session.actors.controls.phase_counter = value,
            OwnedWord::TimeFrozen => session.timing.time_frozen = value,
            OwnedWord::CollidingEnemy(direction) => {
                session.actors.schedule.colliding[direction] = value
            }
            OwnedWord::Bg2TilemapSize => session.actors.drawing.tilemap_size = value,
            OwnedWord::Bg2TilemapPending => session.actors.drawing.tilemap_pending = value,
            OwnedWord::PaletteFadeStep => {
                session.presentation.visual_mut().palette_fade.step = value
            }
            OwnedWord::TimerStatus => session.timing.timer_status = value,
            OwnedWord::JapaneseText => session.options.japanese_text = value,
            OwnedWord::FxBlending => session.room.fx_blending = value,
        }
    }
}
impl<T: State + ?Sized> OwnedWords for T {}
impl State for crate::actors::owned_execution::PreparedActorState<'_> {
    fn store_control_colour(&mut self, index: usize, value: u16) {
        crate::enemy_projectiles::host::Graphics::set_palette_buffer(
            self,
            isize::try_from(index).expect("control colour selector"),
            value,
        );
    }
}
