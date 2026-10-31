//! Ordered object staging shared by owned storage and the native compatibility host.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Clip {
    None,
    Inside,
    Outside,
    /// `Inside` and `Outside` for enemy spritemaps ($81:8B22 and $81:8B96). Both
    /// lack a CLC before the Y add, so it takes the carry of the 16-bit X add:
    /// an entry whose X wraps (an enemy straddling the left screen edge) is a
    /// pixel lower.
    EnemyInside,
    EnemyOutside,
    Bounded,
    BoundedOutside,
}

pub trait Geometry {
    type Error;
    fn count(&mut self) -> Result<u16, Self::Error>;
    fn x(&mut self, record: usize) -> Result<u16, Self::Error>;
    fn y(&mut self, record: usize) -> Result<u8, Self::Error>;
    fn tile(&mut self, record: usize) -> Result<u16, Self::Error>;
}

/// Offsets are bytes relative to object staging, including full native aliases.
/// Slice stores preflight the complete word; byte stores retain low/high prefixes.
pub trait Host {
    type Error;
    fn read_native_word(&mut self, offset: usize) -> Result<u16, Self::Error>;
    fn read_little_word(&mut self, offset: usize) -> Result<u16, Self::Error>;
    fn write_byte(&mut self, offset: usize, value: u8) -> Result<(), Self::Error>;
    fn write_native_slice(&mut self, offset: usize, value: u16) -> Result<(), Self::Error>;
    fn write_native_bytes(&mut self, offset: usize, value: u16) -> Result<(), Self::Error>;
    fn started(&mut self, clip: Clip) -> Result<(), Self::Error>;
}

pub fn draw<H: Host>(
    host: &mut H,
    map: &mut impl Geometry<Error = H::Error>,
    position: (u16, u16),
    clip: Clip,
    tile_word: impl FnMut(u16) -> Result<u16, H::Error>,
) -> Result<(), H::Error> {
    draw_with_publication(host, map, position, clip, tile_word, |_, _, _, _, _, _| {
        Ok(())
    })
}

/// Publish authored identity only after all original bytes of one record succeed.
/// Native drawing uses the same body with an infallible empty publication.
pub fn draw_with_publication<H: Host>(
    host: &mut H,
    map: &mut impl Geometry<Error = H::Error>,
    position: (u16, u16),
    clip: Clip,
    mut tile_word: impl FnMut(u16) -> Result<u16, H::Error>,
    mut published: impl FnMut(&mut H, usize, usize, u16, u16, u16) -> Result<(), H::Error>,
) -> Result<(), H::Error> {
    let mut index = usize::from(host.read_native_word(544)?);
    let count = map.count()?;
    let bounded = matches!(clip, Clip::Bounded | Clip::BoundedOutside);
    if count != 0 && (!bounded || index < 512) {
        host.started(clip)?;
    }
    for record in 0..usize::from(count) {
        if bounded && index >= 512 {
            break;
        }
        let mut x = position.0.wrapping_add(map.x(record)?);
        if bounded {
            let mut y = u16::from(position.1 as u8) + u16::from(map.y(record)?);
            let hidden = if map.y(record)? & 0x80 == 0 {
                y >= 0xe0
            } else if y & 0x100 != 0 {
                y as u8 >= 0xe0
            } else {
                (y as u8) < 0xe0
            };
            if hidden != matches!(clip, Clip::BoundedOutside) {
                x = 0x180;
                y = 0xe0;
            }
            host.write_byte(index, x as u8)?;
            host.write_byte(index + 1, y as u8)?;
            let tile = tile_word(map.tile(record)?)?;
            host.write_native_slice(index + 2, tile)?;
            extension(host, map, record, index, x)?;
            published(host, record, index, x, y, tile)?;
            index += 4;
        } else {
            host.write_byte(index, x as u8)?;
            extension(host, map, record, index, x)?;
            let x_carry = match clip {
                Clip::EnemyInside | Clip::EnemyOutside => {
                    position.0.overflowing_add(map.x(record)?).1
                }
                _ => false,
            };
            let y = u16::from(position.1 as u8) + u16::from(map.y(record)?) + u16::from(x_carry);
            let inside = (y & 0x100 == 0) == (map.y(record)? & 0x80 == 0);
            let output_y = if matches!(clip, Clip::None)
                || inside == matches!(clip, Clip::Inside | Clip::EnemyInside)
            {
                y as u8
            } else {
                0xf0
            };
            host.write_byte(index + 1, output_y)?;
            let tile = tile_word(map.tile(record)?)?;
            host.write_native_slice(index + 2, tile)?;
            published(host, record, index, x, u16::from(output_y), tile)?;
            index = (index + 4) & 0x1ff;
        }
    }
    host.write_native_slice(544, index as u16)
}

fn extension<H: Host>(
    host: &mut H,
    map: &mut impl Geometry<Error = H::Error>,
    record: usize,
    index: usize,
    x: u16,
) -> Result<(), H::Error> {
    let offset = 512 + 2 * (index >> 5);
    let old = host.read_native_word(offset)?;
    let bits = (((x & 0x100) >> 8) | (u16::from((map.x(record)? as i16) < 0) * 2))
        << (2 * (index >> 2 & 7));
    host.write_native_slice(offset, old | bits)
}

/// Which entries a room-coordinate spritemap routine skips, and the carry its
/// X add takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RoomClip {
    /// Mother Brain's ($A9:93EE): entries above the screen; each X takes the
    /// carry of its Y subtract.
    Above,
    /// Ridley's ($A6:DC13): entries outside the screen's 224 rows; the X add
    /// takes the clear carry of the CMP #$E0 that let it through.
    Rows,
}

/// Add a spritemap at room position `origin` to OAM, as the bosses' own
/// routines do: `camera` is the layer 1 position, `clip` says which entries
/// are skipped, and `palette` is ORed into each tile word. There is no other
/// off-screen handling, and a zero count draws 65536 entries, as the
/// original's DEC/BNE loop does. The cursor and high table are little-endian
/// words.
pub fn draw_room<H: Host>(
    host: &mut H,
    map: &mut impl Geometry<Error = H::Error>,
    origin: (u16, u16),
    camera: (u16, u16),
    palette: u16,
    clip: RoomClip,
) -> Result<(), H::Error> {
    let mut index = usize::from(host.read_little_word(544)?);
    let count = map.count()?;
    let records = if count == 0 {
        0x10000
    } else {
        usize::from(count)
    };
    for record in 0..records {
        let row = origin.1.wrapping_add(map.y(record)? as i8 as u16);
        let y = row.wrapping_sub(camera.1);
        if y & 0x8000 != 0 || clip == RoomClip::Rows && y.wrapping_sub(0xe0) & 0x8000 == 0 {
            continue;
        }
        let carry = clip == RoomClip::Above && row >= camera.1;
        let entry_x = map.x(record)?;
        let x = origin
            .0
            .wrapping_add(entry_x)
            .wrapping_add(u16::from(carry))
            .wrapping_sub(camera.0);
        host.write_byte(index, x as u8)?;
        host.write_byte(index + 1, y as u8)?;
        let tile = palette | map.tile(record)?;
        host.write_native_bytes(index + 2, tile)?;
        let offset = 512 + 2 * (index >> 5);
        let bits =
            (((x & 0x100) >> 8) | (u16::from((entry_x as i16) < 0) * 2)) << (2 * (index >> 2 & 7));
        let [low, high] = (host.read_little_word(offset)? | bits).to_le_bytes();
        host.write_byte(offset, low)?;
        host.write_byte(offset + 1, high)?;
        index = (index + 4) & 0x1ff;
    }
    let [low, high] = (index as u16).to_le_bytes();
    host.write_byte(544, low)?;
    host.write_byte(545, high)
}

/// Point the object cursor ($0590) back at the first entry.
pub fn reset_cursor<H: Host>(host: &mut H) -> Result<(), H::Error> {
    host.write_byte(544, 0)?;
    host.write_byte(545, 0)
}

/// Hide the entries after the cursor as large sprites at X 80h, Y 0, as the
/// ending's sprite pass does: their high-table pairs become 01 (large), then
/// each entry's X/Y word is 0080h. Nothing is done once the cursor has
/// reached the end of OAM.
pub fn hide_unused_large<H: Host>(host: &mut H) -> Result<(), H::Error> {
    let cursor = host.read_little_word(544)?;
    if cursor >= 0x200 {
        return Ok(());
    }
    let (first, group) = (usize::from(cursor >> 2), usize::from(cursor >> 5));
    let [low, high] =
        (host.read_little_word(512 + 2 * group)? | (0x5555u16 << (2 * (first & 7)))).to_le_bytes();
    host.write_byte(512 + 2 * group, low)?;
    host.write_byte(513 + 2 * group, high)?;
    for group in group + 1..16 {
        host.write_byte(512 + 2 * group, 0x55)?;
        host.write_byte(513 + 2 * group, 0x55)?;
    }
    // The extension writes precede rereading the first unused entry.
    let first = usize::from(host.read_little_word(544)? >> 2);
    for entry in first..128 {
        host.write_native_bytes(4 * entry, 0x80)?;
    }
    Ok(())
}

pub fn finish_frame<H: Host>(host: &mut H) -> Result<(), H::Error> {
    let first = host.read_little_word(544)? >> 2;
    for sprite in first..128 {
        host.write_byte(1 + usize::from(sprite) * 4, 0xf0)?;
    }
    host.write_native_bytes(544, 0)
}

pub fn clear_high_table<H: Host>(host: &mut H) -> Result<(), H::Error> {
    for word in 0..16 {
        host.write_native_bytes(512 + 2 * word, 0)?;
    }
    Ok(())
}
