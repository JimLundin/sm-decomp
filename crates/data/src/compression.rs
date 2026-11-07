//! The original's compression format, used for level data, tiles and tilemaps.
//!
//! The stream is a list of commands, each a header byte and its operands, ended by the byte
//! 0xFF:
//!
//! - `cccLLLLL`: command `ccc`, length `LLLLL + 1`;
//! - `111cccLL LLLLLLLL`: the extended header, command `ccc`, a 10-bit length plus one.
//!
//! Copies read the output byte by byte, so an overlapping copy repeats the bytes it has just
//! written.

/// The most bytes a stream may produce: one bank of work RAM, the largest destination.
pub const OUTPUT_LIMIT: usize = 0x1_0000;

/// The stream's terminator.
const END: u8 = 0xff;

/// The eight commands, by their 3-bit code.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Command {
    /// 0: the next `length` bytes of the stream.
    DirectCopy,
    /// 1: one operand byte, `length` times.
    ByteFill,
    /// 2: two operand bytes, alternating, `length` bytes in all.
    WordFill,
    /// 3: an operand byte, incremented after each output byte.
    SigmaFill,
    /// 4: a copy from a 16-bit offset into the output.
    DictionaryCopy,
    /// 5: the same, each byte inverted.
    InvertedDictionaryCopy,
    /// 6: a copy from a byte's distance back from the output's end.
    SlidingCopy,
    /// 7: the same, each byte inverted.
    InvertedSlidingCopy,
}

impl Command {
    /// The command with code `code`'s low three bits.
    const fn from_code(code: u8) -> Self {
        match code & 7 {
            0 => Self::DirectCopy,
            1 => Self::ByteFill,
            2 => Self::WordFill,
            3 => Self::SigmaFill,
            4 => Self::DictionaryCopy,
            5 => Self::InvertedDictionaryCopy,
            6 => Self::SlidingCopy,
            7..=u8::MAX => Self::InvertedSlidingCopy,
        }
    }
}

/// Why a stream doesn't decode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecompressError {
    /// The source ended before the terminator, `consumed` bytes in.
    Truncated {
        /// Source bytes read.
        consumed: usize,
    },
    /// A copy read output that hasn't been written: the original would read stale memory.
    UnwrittenReference {
        /// The output position of the byte being written.
        position: usize,
        /// The output position the copy reads, or `None` if before the output's start.
        source: Option<usize>,
    },
    /// The output passed [`OUTPUT_LIMIT`].
    TooLong,
}

/// A decoded stream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decompressed {
    /// The output.
    pub bytes: Vec<u8>,
    /// Source bytes read, the terminator included.
    pub consumed: usize,
}

/// The source cursor.
struct Source<'a> {
    bytes: core::slice::Iter<'a, u8>,
    length: usize,
}

impl Source<'_> {
    fn consumed(&self) -> usize {
        self.length.wrapping_sub(self.bytes.len())
    }

    fn byte(&mut self) -> Result<u8, DecompressError> {
        let consumed = self.consumed();
        self.bytes
            .next()
            .copied()
            .ok_or(DecompressError::Truncated { consumed })
    }
}

/// Decode the stream at the start of `source`, which may run on past the terminator.
///
/// # Errors
///
/// [`DecompressError`] if the source ends early, a copy reads unwritten output, or the
/// output is too long.
pub fn decompress(source: &[u8]) -> Result<Decompressed, DecompressError> {
    let mut input = Source {
        bytes: source.iter(),
        length: source.len(),
    };
    let mut output = Vec::new();
    loop {
        let header = input.byte()?;
        if header == END {
            return Ok(Decompressed {
                bytes: output,
                consumed: input.consumed(),
            });
        }
        let (command, length) = if header >> 5 == 7 {
            let low = input.byte()?;
            (
                Command::from_code(header >> 2),
                usize::from(u16::from_be_bytes([header & 3, low])).wrapping_add(1),
            )
        } else {
            (
                Command::from_code(header >> 5),
                usize::from(header & 0x1f).wrapping_add(1),
            )
        };
        if output.len().saturating_add(length) > OUTPUT_LIMIT {
            return Err(DecompressError::TooLong);
        }
        run(command, length, &mut input, &mut output)?;
    }
}

/// One command: read its operands, then write its `length` bytes.
fn run(
    command: Command,
    length: usize,
    input: &mut Source<'_>,
    output: &mut Vec<u8>,
) -> Result<(), DecompressError> {
    match command {
        Command::DirectCopy => {
            for _ in 0..length {
                output.push(input.byte()?);
            }
        }
        Command::ByteFill => {
            let value = input.byte()?;
            output.extend(core::iter::repeat_n(value, length));
        }
        Command::WordFill => {
            let pair = [input.byte()?, input.byte()?];
            output.extend(pair.iter().cycle().take(length));
        }
        Command::SigmaFill => {
            let first = input.byte()?;
            output.extend(
                core::iter::successors(Some(first), |value| Some(value.wrapping_add(1)))
                    .take(length),
            );
        }
        Command::DictionaryCopy | Command::InvertedDictionaryCopy => {
            let offset = usize::from(u16::from_le_bytes([input.byte()?, input.byte()?]));
            copy(command, Some(offset), length, output)?;
        }
        Command::SlidingCopy | Command::InvertedSlidingCopy => {
            let distance = usize::from(input.byte()?);
            copy(command, output.len().checked_sub(distance), length, output)?;
        }
    }
    Ok(())
}

/// A copy of `length` bytes from output position `from`, one byte at a time.
fn copy(
    command: Command,
    from: Option<usize>,
    length: usize,
    output: &mut Vec<u8>,
) -> Result<(), DecompressError> {
    let mask = match command {
        Command::InvertedDictionaryCopy | Command::InvertedSlidingCopy => 0xff,
        Command::DirectCopy
        | Command::ByteFill
        | Command::WordFill
        | Command::SigmaFill
        | Command::DictionaryCopy
        | Command::SlidingCopy => 0,
    };
    for step in 0..length {
        let source = from.map(|from| from.wrapping_add(step));
        let value = source.and_then(|source| output.get(source)).ok_or(
            DecompressError::UnwrittenReference {
                position: output.len(),
                source,
            },
        )?;
        output.push(value ^ mask);
    }
    Ok(())
}

#[cfg(test)]
mod tests;

// ROM sources for the data above, kept out of the public docs (the original's bank and
// RAM addresses, as the decoders use them):
// Decoded as bank `$80`'s decompression routines do (`$80:B0FF` with a 24-bit source,
// `$80:B119` to memory; `$80:B271` writes the same stream to VRAM). The stream is a list
