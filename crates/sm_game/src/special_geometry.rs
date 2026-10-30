//! Checked even-offset orbit arithmetic with explicit sample observations.
#![forbid(unsafe_code)]
#[derive(Debug)]
pub struct InvalidSample {
    pub index: usize,
    pub length: usize,
}
pub fn point(
    angle: u16,
    radius: u16,
    length: usize,
    mut sample: impl FnMut(usize) -> Option<u16>,
) -> Result<(u16, u16), InvalidSample> {
    fn axis(
        angle: u16,
        radius: u16,
        length: usize,
        sample: &mut impl FnMut(usize) -> Option<u16>,
    ) -> Result<u16, InvalidSample> {
        let negative = angle.wrapping_sub(128) & 0x8000 == 0;
        let phase = if negative {
            u16::from((angle as u8).wrapping_add(128))
        } else {
            angle
        };
        let index = 64 + usize::from(phase.wrapping_mul(2)) / 2;
        let value = sample(index).ok_or(InvalidSample { index, length })?;
        let scaled = ((u32::from(value) * u32::from(radius as u8)) >> 8) as u16;
        Ok(if negative {
            scaled.wrapping_neg()
        } else {
            scaled
        })
    }
    Ok((
        axis(angle, radius, length, &mut sample)?,
        axis(
            u16::from((angle as u8).wrapping_sub(64)),
            radius,
            length,
            &mut sample,
        )?,
    ))
}
