//! Finite combat-family providers on the current aggregate.
#![forbid(unsafe_code)]
use crate::actors::content_records::ContentSource as _;
use crate::actors::owned_execution::PreparedActorState;
use crate::actors::session_slots::SlotAccess as _;
use crate::enemy_projectiles::host::Graphics as _;
use crate::geometry::{Point, Rectangle};
use crate::session_access::SessionAccess as _;
use crate::sprite_effect_state::{LegacySpriteDefinition, SpriteSpawn};

pub(super) fn create_sprite(
    h: &mut PreparedActorState<'_>,
    x: u16,
    y: u16,
    kind: u16,
    appearance: u16,
) -> u16 {
    crate::sprite_objects::legacy_result(crate::sprite_objects::spawn(
        h,
        SpriteSpawn {
            position: Point { x, y },
            definition: LegacySpriteDefinition(kind),
            appearance,
        },
    ))
    .0
}
impl PreparedActorState<'_> {
    /// Only the combat families' extended species words use this port. Ordinary
    /// aligned extended words are the canonical typed block. Odd/broad aliases
    /// use the aggregate's selected image and retain native byte interpretation.
    pub(crate) fn combat_extended_word(&self, at: usize) -> u16 {
        if (0x7800..0x9000).contains(&at)
            && at.is_multiple_of(2)
            && !matches!(at, 0x801e | 0x8020 | 0x803c | 0x8818 | 0x883e)
        {
            u16::from_ne_bytes(self.slot_word(at).to_le_bytes())
        } else {
            self.current_native_word(at)
        }
    }
    pub(crate) fn combat_set_extended_word(&mut self, at: usize, value: u16) {
        if (0x7800..0x9000).contains(&at)
            && at.is_multiple_of(2)
            && !matches!(at, 0x801e | 0x8020 | 0x803c | 0x8818 | 0x883e)
        {
            self.set_slot_word(at, u16::from_le_bytes(value.to_ne_bytes()));
        } else {
            self.write_current_slice(at, &value.to_ne_bytes());
        }
    }
}
pub(super) fn fire_laser(h: &mut PreparedActorState<'_>, x: u16, y: u16, right: bool) {
    h.session_mut().enemy_projectiles.spawn.point = Point { x, y };
    h.session_mut().enemy_projectiles.spawn.variant = u16::from(right);
    crate::enemy_projectiles::lifecycle::spawn_room(h, 0xa17b, u16::from(right));
}
pub(super) fn spawn_claw(
    h: &mut PreparedActorState<'_>,
    x: u16,
    y: u16,
    width: u16,
    height: u16,
    parameter: u16,
) {
    h.session_mut().enemy_projectiles.spawn.rectangle = Rectangle {
        origin: Point { x, y },
        extent: Point {
            x: width,
            y: height,
        },
    };
    let selected = h.session().actors.controls.selected;
    crate::enemy_projectiles::lifecycle::spawn_from_enemy(h, parameter, selected, 0xa189);
}
pub(super) fn load_ninja_palette(h: &mut PreparedActorState<'_>) {
    // The original transfer captures the loaded allocation before its first
    // source read, then takes each native-order word immediately before storing.
    let image = h.content_image();
    let base =
        usize::try_from(sm_data::cartridge::lorom_offset(0xb28727)).expect("ninja palette source");
    for index in 0..16usize {
        let at = base + index * 2;
        let color = u16::from_ne_bytes([image.read(at), image.read(at + 1)]);
        // C3E0 aliases target palette F0 through the palette buffer's full
        // 1F0 index; do not truncate it to the active palette's F0 lane.
        h.set_palette_buffer(
            isize::try_from(0x1f0 + index).expect("ninja palette index"),
            u16::from_le_bytes(color.to_ne_bytes()),
        );
    }
}
