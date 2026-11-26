//! Independent missile and exhaust observations from the locked battle record.

use super::{
    data_failure,
    observe::{CLOCK, EXHAUST, OAM, SLOTS, STOCK, expected_events, field, pieces},
    word,
};
use crate::replay::{readers::ExpectedCut, result::Outcome};
use data::rom::{Lorom, RomAddress};
use data::spritemap::{SpriteSize, Spritemap};
use oracle_record::FrameRecord;

const FLIGHT: u16 = 0x9f03;
const IMPACT: u16 = 0xa039;
const EXHAUST_START: u16 = 0xb5a1;
const SPRITES: u8 = 0x93;

pub(in crate::replay) struct Expected {
    shot_pictures: Vec<u16>,
    shot_maps: Vec<Spritemap>,
    puff_pictures: Vec<u16>,
    missile_oam_attributes: Vec<u16>,
    cuts: Vec<ExpectedCut>,
}

impl Expected {
    pub(super) fn load(rom: Lorom<'_>) -> Result<Self, Outcome> {
        let flight = rom_word(rom, SPRITES, FLIGHT.wrapping_add(2))?;
        let mut shot_pictures = vec![flight];
        for number in 0..6_u16 {
            let address = IMPACT.wrapping_add(number.wrapping_mul(8)).wrapping_add(2);
            let picture = rom_word(rom, SPRITES, address)?;
            if !shot_pictures.contains(&picture) {
                shot_pictures.push(picture);
            }
        }
        let shot_maps = shot_pictures
            .iter()
            .map(|&pointer| {
                Spritemap::load(rom, RomAddress::new(SPRITES, pointer))
                    .map_err(|_| data_failure(None, "independent impact picture source"))
            })
            .collect::<Result<_, _>>()?;
        let puff_pictures = (0..4_u16)
            .map(|frame| {
                rom_word(
                    rom,
                    0x90,
                    EXHAUST_START
                        .wrapping_add(frame.wrapping_mul(4))
                        .wrapping_add(2),
                )
            })
            .collect::<Result<Vec<_>, _>>()?;
        let pieces = rom_word(rom, SPRITES, flight)?;
        let mut missile_oam_attributes = puff_pictures.clone();
        for piece in 0..pieces {
            missile_oam_attributes.push(rom_word(
                rom,
                SPRITES,
                flight.wrapping_add(piece.wrapping_mul(5)).wrapping_add(5),
            )?);
        }
        Ok(Self {
            shot_pictures,
            shot_maps,
            puff_pictures,
            missile_oam_attributes,
            cuts: Vec::new(),
        })
    }

    pub(super) fn read(
        &mut self,
        prior: Option<&FrameRecord>,
        frame: &FrameRecord,
        timing: &super::inputs::Timing,
    ) -> Result<(), Outcome> {
        let semantics = self.state(frame)?;
        let mut fields = semantics.clone();
        fields.extend(expected_events(prior, frame, timing.turn)?);
        let sprites = if timing.publish {
            self.oam(frame)?
        } else {
            Vec::new()
        };
        fields.push(field(OAM, sprites.into_iter().flatten()));
        self.cuts.push(ExpectedCut {
            semantic_count: semantics.len(),
            output_count: fields.len().saturating_sub(semantics.len()),
            replies: 0,
            fields,
            semantics,
        });
        Ok(())
    }

    pub(in crate::replay) fn at(&self, cut: u64) -> Result<ExpectedCut, Outcome> {
        usize::try_from(cut)
            .ok()
            .and_then(|cut| self.cuts.get(cut))
            .cloned()
            .ok_or_else(|| data_failure(Some(cut), "missile expected cut"))
    }

    fn state(
        &self,
        frame: &FrameRecord,
    ) -> Result<Vec<crate::replay::compare::Observation>, Outcome> {
        let w = |at| word(frame, at);
        let mut slots = Vec::new();
        for offset in (0..10_usize).step_by(2) {
            slots.extend(self.slot(frame, offset)?);
        }
        let mut exhaust = Vec::new();
        for offset in (0..36_usize).step_by(2) {
            exhaust.extend(self.puff(frame, offset)?);
        }
        Ok(vec![
            field(
                STOCK,
                [
                    w(0x09c6)?,
                    u16::from(w(0x09d2)? == 1),
                    u16::from(w(0x0a04)? != 0),
                    w(0x18ac)?,
                ],
            ),
            field(CLOCK, [w(0x0ccc)?, w(0x0cce)?]),
            field(SLOTS, slots),
            field(EXHAUST, exhaust),
        ])
    }

    fn slot(&self, frame: &FrameRecord, offset: usize) -> Result<Vec<u16>, Outcome> {
        let w = |column: usize| word(frame, column.saturating_add(offset));
        let trail = w(0x0c90)?;
        let fraction_x = w(0x0b8c)?;
        let fraction_y = w(0x0ba0)?;
        let at = w(0x0c40)?;
        if at == 0 {
            return Ok(vec![0, trail, fraction_x, fraction_y]);
        }
        let bad = || data_failure(Some(frame.index), "cinematic missile body record");
        let (phase, base) = match w(0x0c18)? {
            0x8100 => (1, FLIGHT),
            0x8800 => (2, IMPACT),
            _ => return Err(bad()),
        };
        let cursor = at
            .checked_sub(base)
            .filter(|distance| distance.is_multiple_of(8))
            .ok_or_else(bad)?
            / 8;
        let art = slot_index(&self.shot_pictures, w(0x0cb8)?).ok_or_else(bad)?;
        if w(0x0c04)? != 7 || w(0x0c7c)? != 0x0100 {
            return Err(bad());
        }
        Ok(vec![
            phase,
            trail,
            fraction_x,
            fraction_y,
            w(0x0b64)?,
            w(0x0b78)?,
            w(0x0bdc)?,
            w(0x0bf0)?,
            u16::MAX,
            0,
            w(0x0bb4)?,
            w(0x0bc8)?,
            w(0x0c2c)?,
            w(0x0c54)?,
            cursor,
            art,
            2,
            0,
            1,
        ])
    }

    fn puff(&self, frame: &FrameRecord, offset: usize) -> Result<Vec<u16>, Outcome> {
        let w = |column: usize| word(frame, column.saturating_add(offset));
        let hold = w(0xd658)?;
        let bad = || data_failure(Some(frame.index), "cinematic exhaust record");
        if w(0xd67c)? != 0 {
            return Err(bad());
        }
        if hold == 0 {
            return Ok(vec![0]);
        }
        let x = w(0xd730)?;
        let y = w(0xd778)?;
        let art = slot_index(&self.puff_pictures, w(0xd6e8)?).ok_or_else(bad)?;
        let next = art.wrapping_add(1);
        if w(0xd754)? != x
            || w(0xd79c)? != y
            || w(0xd6a0)? != EXHAUST_START.wrapping_add(next.wrapping_mul(4))
        {
            return Err(bad());
        }
        Ok(vec![1, x, y, next, hold, art])
    }

    fn oam(&self, frame: &FrameRecord) -> Result<Vec<[u16; 4]>, Outcome> {
        let (low, high) = frame
            .oam
            .split_at_checked(512)
            .ok_or_else(|| data_failure(Some(frame.index), "cinematic missile OAM"))?;
        let visible = pieces(low.as_chunks::<4>().0.iter().copied(), high);
        self.check_impact_omission(frame, &visible)?;
        Ok(visible
            .into_iter()
            .filter(|&[_, _, attrs, _]| self.missile_oam_attributes.contains(&attrs))
            .collect())
    }

    fn check_impact_omission(
        &self,
        frame: &FrameRecord,
        visible: &[[u16; 4]],
    ) -> Result<(), Outcome> {
        let camera_x = word(frame, 0x0911)?;
        let camera_y = word(frame, 0x0915)?;
        for offset in (0..10_usize).step_by(2) {
            let w = |column: usize| word(frame, column.saturating_add(offset));
            if w(0x0c18)? != 0x8800 {
                continue;
            }
            let picture = w(0x0cb8)?;
            let map = slot_index(&self.shot_pictures, picture)
                .and_then(|index| self.shot_maps.get(usize::from(index)))
                .ok_or_else(|| data_failure(Some(frame.index), "live missile impact picture"))?;
            let x = w(0x0b64)?.wrapping_sub(camera_x);
            let y = w(0x0b78)?.wrapping_sub(camera_y);
            if map.entries.iter().any(|piece| {
                visible.contains(&[
                    x.wrapping_add(piece.x.cast_unsigned()) & 511,
                    y.wrapping_add(u16::from(piece.y)) & 255,
                    piece.attributes.word(),
                    u16::from(piece.size == SpriteSize::Large),
                ])
            }) {
                return Err(data_failure(
                    Some(frame.index),
                    "cinematic impact appears in OAM",
                ));
            }
        }
        Ok(())
    }
}

fn slot_index(values: &[u16], value: u16) -> Option<u16> {
    values
        .iter()
        .position(|&existing| existing == value)
        .and_then(|index| u16::try_from(index).ok())
}

fn rom_word(rom: Lorom<'_>, bank: u8, address: u16) -> Result<u16, Outcome> {
    let bad = || data_failure(None, "independent missile picture source");
    let bytes = rom
        .read(RomAddress::new(bank, address), 2)
        .map_err(|_| bad())?;
    Ok(u16::from_le_bytes(
        *bytes.first_chunk::<2>().ok_or_else(bad)?,
    ))
}
