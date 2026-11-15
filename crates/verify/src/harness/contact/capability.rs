//! C1-only marker producer exclusions from preceding source inputs.
use super::{Defect, word};
use data::pose::animation::{AnimationDelays, AnimationFrame, RunPaces};
use data::pose::{MovementType, PoseId, Poses};
use data::rom::{Rom, RomAddress};
use game::samus_appearance_bridge::Clips;
use std::collections::BTreeSet;

pub(super) struct Content {
    rom: Rom,
    poses: Poses,
    clips: Clips,
    doors: Vec<BTreeSet<u16>>,
}

impl Content {
    pub(super) fn load(image: &[u8]) -> Result<Self, Defect> {
        let rom = Rom::new(image.to_vec()).map_err(|_| Defect("C1 capability ROM import"))?;
        let door = data::enemy::species(rom.lorom(), data::enemy::SpeciesKey(0xe23f))
            .map_err(|_| Defect("C1 door header import"))?;
        require(
            door.ai_bank == 0xa6
                && door.routines.main.0 == RomAddress::new(0xa6, 0xf765)
                && door.routines.touch.0 == RomAddress::new(0xa6, 0xf920)
                && door.routines.shot.0 == RomAddress::new(0xa6, 0xf920),
            "C1 unsupported imported door callbacks",
        )?;
        let poses = Poses::load(rom.lorom()).map_err(|_| Defect("C1 pose import"))?;
        let animations = AnimationDelays::load(rom.lorom(), &poses)
            .map_err(|_| Defect("C1 animation import"))?;
        let paces = RunPaces::load(rom.lorom()).map_err(|_| Defect("C1 run pace import"))?;
        let clips = Clips::import(&animations, &paces).map_err(|_| Defect("C1 clip import"))?;
        let mut result = Self {
            rom,
            poses,
            clips,
            doors: Vec::new(),
        };
        result.doors = result.graphs(0xf52c, 7)?;
        Ok(result)
    }

    fn short(&self, bank: u8, at: usize) -> Result<u16, Defect> {
        let address = RomAddress::new(
            bank,
            u16::try_from(at).map_err(|_| Defect("C1 content address overflow"))?,
        );
        let bytes = self
            .rom
            .lorom()
            .read(address, 2)
            .map_err(|_| Defect("C1 content address"))?;
        word(bytes, 0)
    }

    fn graphs(&self, table: usize, count: usize) -> Result<Vec<BTreeSet<u16>>, Defect> {
        (0..count)
            .map(|index| {
                self.graph(self.short(0xa6, table.saturating_add(index.saturating_mul(2)))?)
            })
            .collect()
    }

    fn graph(&self, root: u16) -> Result<BTreeSet<u16>, Defect> {
        let mut pending = vec![root];
        let mut visited = BTreeSet::new();
        while let Some(pointer) = pending.pop() {
            require(
                pointer >= 0x8000,
                "C1 instruction target outside content bank",
            )?;
            if !visited.insert(pointer) {
                continue;
            }
            let at = usize::from(pointer);
            let op = self.short(0xa6, at)?;
            let advance = |size: u16| {
                pointer
                    .checked_add(size)
                    .ok_or(Defect("C1 instruction crosses bank"))
            };
            match op {
                1..=0x7fff => {
                    self.short(0xa6, at.saturating_add(2))?;
                    pending.push(advance(4)?);
                }
                0x80ed => pending.push(self.short(0xa6, at.saturating_add(2))?),
                0xf63e | 0xf66a | 0xf678 => {
                    pending.push(self.short(0xa6, at.saturating_add(2))?);
                    pending.push(advance(4)?);
                }
                0xf68b | 0xf695 | 0xf69f | 0xf6a6 | 0xf6b0 | 0xf6b3 | 0xf6bd => {
                    pending.push(advance(2)?);
                }
                _ => return Err(Defect("C1 unsupported actor instruction content")),
            }
        }
        Ok(visited)
    }

    pub(super) fn actors(&self, wram: &[u8], active: &[usize], live: bool) -> Result<(), Defect> {
        require(
            word(wram, 0x185e)? == 0,
            "C1 unsupported frozen actor dispatch",
        )?;
        for &offset in active {
            let w = |at: usize| word(wram, at.saturating_add(offset));
            require(
                w(0x0fa6)?.to_le_bytes()[0] == 0xa6 && w(0x0f8a)? == 0,
                "C1 unsupported actor AI callback",
            )?;
            require(
                w(0x0f88)? & 4 == 0,
                "C1 unsupported extended actor hitboxes",
            )?;
            let parameter = usize::from(w(0x0fb4)?);
            let function = w(0x0fa8)?;
            let graph = match w(0x0f78)? {
                0xe1ff => return Err(Defect("C1 steam touch continuation unsupported")),
                0xe23f => {
                    require(
                        !live || matches!(parameter, 0 | 1 | 4),
                        "C1 unsupported live door mode",
                    )?;
                    require(
                        match parameter {
                            0 | 1 | 4 => function == 0xf76b,
                            3 => function == 0xf770,
                            5 | 6 => function == 0xf7a5,
                            _ => false,
                        },
                        "C1 unsupported door function or producer mode",
                    )?;
                    self.doors.get(parameter)
                }
                _ => return Err(Defect("C1 unsupported actor species capability")),
            }
            .ok_or(Defect("C1 unknown actor parameter content"))?;
            require(
                graph.contains(&w(0x0f92)?),
                "C1 unsupported actor instruction cursor",
            )?;
        }
        Ok(())
    }

    pub(super) fn ordinary(&self, wram: &[u8]) -> Result<(), Defect> {
        ordinary_inputs(wram)?;
        require(
            word(wram, 0x0a6c)? == 0x9f55,
            "C1 unsupported retained air-speed table",
        )?;
        for (pose_at, definition_at) in [(0x0a1c, 0x0a1e), (0x0a20, 0x0a22)] {
            let pose = PoseId(word(wram, pose_at)?);
            let definition = self
                .poses
                .get(pose)
                .ok_or(Defect("C1 unknown ordinary pose"))?;
            require(
                ordinary_movement(definition.movement),
                "C1 unsupported pose movement family",
            )?;
            require(
                matches!(definition.y_radius, 16 | 19 | 21),
                "C1 pose radius exceeds envelope",
            )?;
            let raw = word(wram, definition_at)?.to_le_bytes();
            require(
                raw == [u8::from(definition.facing), u8::from(definition.movement)],
                "C1 restored pose definition differs",
            )?;
        }
        let pose = PoseId(word(wram, 0x0a1c)?);
        let definition = self
            .poses
            .get(pose)
            .ok_or(Defect("C1 unknown current pose"))?;
        require(
            word(wram, 0x0afe)? == 5 && word(wram, 0x0b00)? == u16::from(definition.y_radius),
            "C1 restored collision radii differ",
        )?;
        let frame = AnimationFrame(word(wram, 0x0a96)?);
        let animation = self
            .clips
            .of(pose)
            .ok_or(Defect("C1 missing pose animation"))?;
        require(
            animation.step_of(frame).is_some(),
            "C1 invalid pose animation cursor",
        )?;
        u8::try_from(word(wram, 0x0a94)?).map_err(|_| Defect("C1 invalid animation timer"))?;
        for code in 0..28_u8 {
            let movement = MovementType::try_from(code)
                .map_err(|_| Defect("C1 unknown speed-table movement"))?;
            if !ordinary_movement(movement) {
                continue;
            }
            let index = usize::from(code);
            let at = 0x9f55_usize.saturating_add(index.saturating_mul(12));
            require(
                self.short(0x90, at)? == 0,
                "C1 imported acceleration exceeds envelope",
            )?;
            require(
                self.short(0x90, at.saturating_add(4))? <= 8,
                "C1 imported maximum speed exceeds envelope",
            )?;
        }
        for index in (0..6).step_by(2) {
            require(
                self.short(0x90, 0x9f01_usize.saturating_add(index))? == 0,
                "C1 imported extra acceleration exceeds envelope",
            )?;
            let maximum = data::movement::Speed::new(
                self.short(0x90, 0x9f19_usize.saturating_add(index))?,
                self.short(0x90, 0x9f1f_usize.saturating_add(index))?,
            );
            require(
                maximum <= data::movement::Speed::new(2, 0),
                "C1 imported extra maximum exceeds envelope",
            )?;
        }
        for index in (0..6).step_by(2) {
            require(
                self.short(0x90, 0x9eb9_usize.saturating_add(index))? <= 8,
                "C1 imported jump launch exceeds envelope",
            )?;
        }
        Ok(())
    }
}

const fn ordinary_movement(movement: MovementType) -> bool {
    matches!(
        movement,
        MovementType::Standing
            | MovementType::Running
            | MovementType::NormalJumping
            | MovementType::SpinJumping
            | MovementType::Crouching
            | MovementType::Falling
            | MovementType::TurningAroundOnGround
            | MovementType::CrouchingTransition
            | MovementType::Moonwalking
            | MovementType::TurningAroundJumping
            | MovementType::TurningAroundFalling
    )
}

fn require(condition: bool, reason: &'static str) -> Result<(), Defect> {
    condition.then_some(()).ok_or(Defect(reason))
}

pub(super) fn ordinary_inputs(wram: &[u8]) -> Result<(), Defect> {
    let w = |at| word(wram, at);
    for (at, expected) in [
        (0x0a42, 0xe695),
        (0x0a58, 0xa337),
        (0x0a5a, 0xe0e6),
        (0x0d32, 0xc4f0),
    ] {
        require(w(at)? == expected, "C1 unsupported ordinary callback")?;
    }
    require(
        w(0x0a60)? == 0xe913 || (w(0x0a60)? == 0xe926 && w(0x0af4)? <= 8 && w(0x09b4)? == 0x8000),
        "C1 unsupported ordinary input callback",
    )?;
    require(
        w(0x0b42)? < 2 && w(0x0b3c)? <= 1 && w(0x0dbc)? <= 12,
        "C1 ordinary momentum exceeds envelope",
    )?;
    require(
        matches!(w(0x0a44)?, 0xe725 | 0xe8aa),
        "C1 unsupported POST callback",
    )?;
    require(
        w(0x0a1e)?.to_le_bytes()[1] < 28,
        "C1 unsupported movement dispatch",
    )?;
    require(
        w(0x0aae)? & 0x8000 != 0 || matches!(w(0x0aae)?, 0 | 2),
        "C1 invalid echo destination",
    )?;
    require(
        matches!(w(0x07df)?, 0 | 0xe524),
        "C1 unsupported room callback",
    )?;
    require(
        w(0x178c)? == 0x804c && wram.get(0x178e) == Some(&0xa0),
        "C1 unsupported graphics callback",
    )?;
    for at in [
        0x0a66, 0x0a78, 0x0a76, 0x09a2, 0x18aa, 0x0a52, 0x0a54, 0x0b3e, 0x0b56, 0x0b58, 0x0b5a,
        0x0b5c, 0x0b4c, 0x0b34,
    ] {
        require(w(at)? == 0, "C1 unsupported normal collision input")?;
    }
    require(
        w(0x195e)? & 0x8000 != 0 && w(0x1962)? & 0x8000 != 0,
        "C1 unsupported liquid physics",
    )?;
    for at in [0x0b46, 0x0b2e] {
        require(w(at)? <= 8, "C1 collision magnitude exceeds envelope")?;
    }
    for at in [0x0b4a, 0x0b36] {
        require(w(at)? <= 2, "C1 invalid movement direction")?;
    }
    require(w(0x0b32)? <= 0x8000, "C1 acceleration exceeds envelope")?;
    for (base, count) in [
        (0x1e7d_usize, 8_usize),
        (0x1c37, 40),
        (0x1ef5, 6),
        (0xef78, 32),
        (0x1997, 18),
    ] {
        for index in 0..count {
            require(
                w(base.saturating_add(index.saturating_mul(2)))? == 0,
                "C1 unsupported callback pool occupant",
            )?;
        }
    }
    for offset in (0..20).step_by(2) {
        if w(0x0c40_usize.saturating_add(offset))? != 0 {
            require(
                offset < 10 && matches!(w(0x0c18_usize.saturating_add(offset))?, 0x8000 | 0x8700),
                "C1 unsupported later player body",
            )?;
        }
    }
    memory_destinations(wram)
}

fn memory_destinations(wram: &[u8]) -> Result<(), Defect> {
    let w = |at| word(wram, at);
    require(
        w(0x0590)? <= 0x01fc && w(0x0590)?.is_multiple_of(4),
        "C1 invalid OAM destination",
    )?;
    require(
        w(0x0330)? == 0 && w(0x0334)? == 0,
        "C1 unsupported graphics queue destination",
    )?;
    require(
        wram.get(0x0360) == Some(&0),
        "C1 unsupported reverse DMA destination",
    )?;
    for offset in (0..12).step_by(2) {
        if w(0x18b4_usize.saturating_add(offset))?.to_le_bytes()[0] != 0 {
            let channel = *wram
                .get(0x18c0_usize.saturating_add(offset))
                .ok_or(Defect("C1 short HDMA channel"))?;
            require(
                channel <= 0x70 && channel.is_multiple_of(16),
                "C1 invalid HDMA hardware destination",
            )?;
        }
    }
    let width = usize::from(w(0x07a5)?);
    let height = usize::from(w(0x07a7)?);
    require(
        (1..=128).contains(&width) && height > 0,
        "C1 invalid room dimensions",
    )?;
    let map_x = usize::from(w(0x07a1)?);
    let map_y = usize::from(w(0x07a3)?);
    require(
        map_x.saturating_add(width.div_ceil(16)) <= 64
            && map_y.saturating_add(height.div_ceil(16)) < 32,
        "C1 invalid explored-map destination",
    )?;
    collision_rectangle(wram)
}

fn collision_rectangle(wram: &[u8]) -> Result<(), Defect> {
    let x = usize::from(word(wram, 0x0af6)?);
    let y = usize::from(word(wram, 0x0afa)?);
    require(
        (32..=4063).contains(&x) && (128..=3967).contains(&y),
        "C1 collision coordinate outside envelope",
    )?;
    let width = usize::from(word(wram, 0x07a5)?);
    require((1..=128).contains(&width), "C1 invalid collision width")?;
    let col_min = x.saturating_sub(32) / 16;
    let col_max = x.saturating_add(32) / 16;
    let row_min = y.saturating_sub(128) / 16;
    let row_max = y.saturating_add(128) / 16;
    require(
        row_max <= 255
            && col_max <= 255
            && row_max.saturating_mul(width).saturating_add(col_max) < 8192,
        "C1 collision lookup destination outside bank",
    )?;
    for row in row_min..=row_max {
        for column in col_min..=col_max {
            let index = row.saturating_mul(width).saturating_add(column);
            let raw = word(wram, 0x10002_usize.saturating_add(index.saturating_mul(2)))?;
            require(
                matches!(raw >> 12, 0 | 1 | 8 | 9),
                "C1 unsupported collision producer class",
            )?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn ordinary_fixture() -> Vec<u8> {
        let mut wram = vec![0; 0x20000];
        for (at, value) in [
            (0x0a42_usize, 0xe695_u16),
            (0x0a44, 0xe725),
            (0x0a58, 0xa337),
            (0x0a5a, 0xe0e6),
            (0x0a60, 0xe913),
            (0x09b4, 0x8000),
            (0x0d32, 0xc4f0),
            (0x178c, 0x804c),
            (0x195e, 0xffff),
            (0x1962, 0xffff),
            (0x07a5, 16),
            (0x07a7, 16),
            (0x0af6, 44),
            (0x0afa, 139),
        ] {
            wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
        }
        wram[0x178e] = 0xa0;
        wram
    }

    #[test]
    fn actor_capability_rejects_unknown_callback_and_content_cursor_on_quiet_inputs() {
        let Some((_, rom)) = crate::harness::tests::environment() else {
            return;
        };
        let content = Content::load(&std::fs::read(rom).unwrap()).unwrap();
        let cursor = *content.doors[0].first().unwrap();
        let mut wram = vec![0; 0x20000];
        let put = |wram: &mut [u8], at: usize, value: u16| {
            wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
        };
        for (at, value) in [
            (0x0f78_usize, 0xe23f),
            (0x0fa6, 0xa6),
            (0x0fa8, 0xf76b),
            (0x0f92, cursor),
        ] {
            put(&mut wram, at, value);
        }
        assert!(content.actors(&wram, &[0], false).is_ok());
        for (at, value) in [
            (0x0f78_usize, 0xe13f),
            (0x0f8a, 1),
            (0x0fa8, 0),
            (0x0f92, 1),
            (0x0fb4, 2),
        ] {
            let mut bad = wram.clone();
            put(&mut bad, at, value);
            assert!(content.actors(&bad, &[0], false).is_err());
        }
    }

    #[test]
    fn quiet_receipt_requires_known_callbacks_and_valid_destinations() {
        let good = ordinary_fixture();
        assert!(ordinary_inputs(&good).is_ok());
        for at in [
            0x0a42, 0x0a44, 0x07df, 0x178c, 0x0330, 0x0334, 0x0360, 0x0590, 0x1c37,
        ] {
            let mut bad = good.clone();
            bad[at] = 1;
            assert!(ordinary_inputs(&bad).is_err());
        }
        let mut bad_channel = good;
        bad_channel[0x18b4] = 1;
        bad_channel[0x18c0] = 1;
        assert!(ordinary_inputs(&bad_channel).is_err());
    }

    #[test]
    fn collision_rectangle_checks_raw_cells_beyond_loaded_room() {
        let mut wram = vec![0; 0x20000];
        let put = |wram: &mut [u8], at: usize, value: u16| {
            wram[at..at.saturating_add(2)].copy_from_slice(&value.to_le_bytes());
        };
        put(&mut wram, 0x0af6, 44);
        put(&mut wram, 0x0afa, 139);
        put(&mut wram, 0x07a5, 16);
        assert!(collision_rectangle(&wram).is_ok());
        // This raw lookup lies beyond a one-row room and still executes in the original.
        put(&mut wram, 0x10002_usize.saturating_add(16 * 16 * 2), 0xc000);
        assert!(collision_rectangle(&wram).is_err());
        put(&mut wram, 0x0afa, 0);
        assert!(collision_rectangle(&wram).is_err());
    }
}
