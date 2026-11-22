//! Cannon drawing order selected by the current fitted pose.

use super::PoseId;
use crate::rom::{AddressError, Lorom, RomAddress};

const POSE_MODE_POINTERS: RomAddress = RomAddress::new(0x90, 0xc7df);

impl PoseId {
    /// Read whether the arm cannon participates in the pose's drawing order.
    ///
    /// # Errors
    /// A pose pointer or mode address outside the checked ROM is invalid.
    pub fn has_cannon_draw(self, rom: Lorom<'_>) -> Result<bool, AddressError> {
        let index = usize::from(self.0).saturating_mul(2);
        let address = POSE_MODE_POINTERS
            .advance(index)
            .ok_or(AddressError::PastEnd(POSE_MODE_POINTERS))?;
        let [low, high] = rom
            .read(address, 2)?
            .try_into()
            .map_err(|_| AddressError::PastEnd(address))?;
        let entry = RomAddress::new(address.bank, u16::from_le_bytes([low, high]));
        let mode = entry.advance(1).ok_or(AddressError::PastEnd(entry))?;
        let [byte] = rom
            .read(mode, 1)?
            .try_into()
            .map_err(|_| AddressError::PastEnd(mode))?;
        Ok(byte & 0x0f != 0)
    }
}
