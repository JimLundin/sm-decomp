//! Read the Ceres shaft's fixed overlay uploads and ordered rotation track.

use data::assets::shaft::{OVERLAY_BYTES, ROTATION_SAMPLES, RotationSample, Shaft};
use data::rom::{AddressError, Lorom, RomAddress};
use data::spritemap::Spritemap;

const OVERLAY: RomAddress = RomAddress::new(0xb0, 0xc400);
const FIGURE: RomAddress = RomAddress::new(0xa6, 0xf921);
const PLATFORM: RomAddress = RomAddress::new(0xa6, 0xf918);
const ROTATION: RomAddress = RomAddress::new(0x89, 0xad5f);

/// Read the overlay strip, both platform looks and the signed rotation samples.
///
/// # Errors
/// An incomplete asset returns an address error.
pub fn import(rom: Lorom<'_>) -> Result<Shaft, AddressError> {
    let overlay = rom
        .read(OVERLAY, OVERLAY_BYTES)?
        .try_into()
        .map_err(|_| AddressError::PastEnd(OVERLAY))?;
    let platform = *rom
        .read(PLATFORM, 8)?
        .as_chunks::<4>()
        .0
        .first_chunk::<2>()
        .ok_or(AddressError::PastEnd(PLATFORM))?;

    let mut rotation = [RotationSample::default(); ROTATION_SAMPLES];
    let (samples, _) = rom.read(ROTATION, ROTATION_SAMPLES * 6)?.as_chunks::<6>();
    for (
        sample,
        &[
            hold_lo,
            hold_hi,
            cross_lo,
            cross_hi,
            diagonal_lo,
            diagonal_hi,
        ],
    ) in rotation.iter_mut().zip(samples)
    {
        *sample = RotationSample {
            hold: u16::from_le_bytes([hold_lo, hold_hi]),
            cross: i16::from_le_bytes([cross_lo, cross_hi]),
            diagonal: i16::from_le_bytes([diagonal_lo, diagonal_hi]),
        };
    }

    Ok(Shaft {
        overlay,
        platform,
        rotation,
    })
}

/// Read the stationary figure shown during the overlay's rumble.
///
/// # Errors
/// An incomplete spritemap returns an address error.
pub fn overlay_figure(rom: Lorom<'_>) -> Result<Spritemap, AddressError> {
    Spritemap::load(rom, FIGURE)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn station_assets_preserve_order_and_signed_coefficients() {
        let mut image = vec![0; data::rom::Rom::SIZE];
        let empty = Lorom::new(&image);
        let overlay_at = empty.file_offset(OVERLAY).unwrap();
        let platform_at = empty.file_offset(PLATFORM).unwrap();
        let rotation_at = empty.file_offset(ROTATION).unwrap();
        image.truncate(overlay_at + OVERLAY_BYTES);
        image[rotation_at..rotation_at + 6].copy_from_slice(&[2, 1, 254, 255, 0, 1]);
        image[platform_at..platform_at + 8].copy_from_slice(&[1, 2, 3, 4, 8, 7, 6, 5]);
        image[overlay_at] = 0x42;

        let shaft = import(Lorom::new(&image)).unwrap();
        assert_eq!(
            shaft.rotation[0],
            RotationSample {
                hold: 258,
                cross: -2,
                diagonal: 256
            }
        );
        assert_eq!(shaft.platform, [[1, 2, 3, 4], [8, 7, 6, 5]]);
        assert_eq!(shaft.overlay[0], 0x42);
        assert_eq!(
            import(Lorom::new(&image[..image.len() - 1])),
            Err(AddressError::PastEnd(OVERLAY))
        );
    }
}
