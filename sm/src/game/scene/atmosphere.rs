//! Complete atmospheric tile output with live queries and ordered staging effects.
#![forbid(unsafe_code)]

pub(crate) trait Host {
    type Error;
    fn frame_and_kind(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn cursor(&mut self) -> Result<u16, Self::Error>;
    fn x(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn camera_x(&mut self) -> Result<u16, Self::Error>;
    fn y(&mut self, slot: usize) -> Result<u16, Self::Error>;
    fn camera_y(&mut self) -> Result<u16, Self::Error>;
    /// Entry is the captured full cursor, in bytes relative to staging.
    fn write_x(&mut self, entry: usize, value: u8) -> Result<(), Self::Error>;
    fn write_y(&mut self, entry: usize, value: u8) -> Result<(), Self::Error>;
    /// Capture the loaded table before computing its native kind-word offset.
    fn tile_pointer(&mut self, kind: u16) -> Result<u16, Self::Error>;
    /// Capture the native record lazily, retaining the original diagnostic.
    fn read_tile(&mut self, address: u16) -> Result<u16, Self::Error>;
    /// Preflight the whole native word before either staging byte changes.
    fn write_tile(&mut self, entry: usize, value: u16) -> Result<(), Self::Error>;
    /// Publish little-endian low then high bytes, retaining completed prefixes.
    fn publish_cursor(&mut self, value: u16) -> Result<(), Self::Error>;
    fn written(&mut self) -> Result<(), Self::Error>;
}

pub(crate) fn draw_tile<H: Host>(host: &mut H, slot: usize, kind: u16) -> Result<(), H::Error> {
    let frame_offset = 2 * u16::from(host.frame_and_kind(slot)? as u8);
    let saved = host.cursor()?;
    let x = host.x(slot)?.wrapping_sub(host.camera_x()?).wrapping_sub(4);
    if (x as i16) < 0
        || host
            .x(slot)?
            .wrapping_sub(host.camera_x()?)
            .wrapping_sub(260)
            & 0x8000
            == 0
    {
        return Ok(());
    }
    let entry = usize::from(host.cursor()?);
    host.write_x(entry, x as u8)?;
    let y = host.y(slot)?.wrapping_sub(host.camera_y()?).wrapping_sub(4);
    if (y as i16) < 0
        || host
            .y(slot)?
            .wrapping_sub(host.camera_y()?)
            .wrapping_sub(260)
            & 0x8000
            == 0
    {
        return Ok(());
    }
    host.write_y(entry, y as u8)?;
    let pointer = host.tile_pointer(kind)?;
    let tile = host.read_tile(pointer.wrapping_add(frame_offset))?;
    host.write_tile(entry, tile)?;
    host.publish_cursor(saved.wrapping_add(4))?;
    host.written()
}
