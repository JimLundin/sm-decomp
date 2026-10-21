use crate::c_library::{free, malloc, realloc};

struct BitReader<'a> {
    input: &'a [u8],
    position: usize,
    buffer: u32,
    count: i32,
}

impl BitReader<'_> {
    fn byte(&mut self) -> u8 {
        let Some(&byte) = self.input.get(self.position) else {
            return 0;
        };
        self.position += 1;
        byte
    }

    fn fill(&mut self) {
        loop {
            if self.buffer >= 1u32 << self.count {
                self.position = self.input.len();
                return;
            }
            self.buffer |= u32::from(self.byte()) << self.count;
            self.count += 8;
            if self.count > 24 {
                return;
            }
        }
    }

    fn bits(&mut self, count: i32) -> u32 {
        if self.count < count {
            self.fill();
        }
        let value = self.buffer & ((1u32 << count) - 1);
        self.buffer >>= count;
        self.count -= count;
        value
    }

    fn extra_bits(&mut self, count: i32) -> u32 {
        if count == 0 {
            0
        } else {
            self.bits(count)
        }
    }

    fn symbol(&mut self, codes: &Huffman) -> Option<usize> {
        if self.count < 16 {
            if self.position == self.input.len() {
                return None;
            }
            self.fill();
        }
        let fast = codes.fast[(self.buffer & 511) as usize];
        if fast != 0 {
            let size = i32::from(fast >> 9);
            self.buffer >>= size;
            self.count -= size;
            return Some(usize::from(fast & 511));
        }
        let reversed = i32::from((self.buffer as u16).reverse_bits());
        let mut size = 10;
        while reversed >= codes.max_code[size] {
            size += 1;
        }
        if size >= 16 {
            return None;
        }
        let index = (reversed >> (16 - size)) - i32::from(codes.first_code[size])
            + i32::from(codes.first_symbol[size]);
        if index >= 288 || codes.size[index as usize] as usize != size {
            return None;
        }
        self.buffer >>= size;
        self.count -= size as i32;
        Some(usize::from(codes.value[index as usize]))
    }
}

struct Huffman {
    fast: [u16; 512],
    first_code: [u16; 16],
    max_code: [i32; 17],
    first_symbol: [u16; 16],
    size: [u8; 288],
    value: [u16; 288],
}

impl Huffman {
    const EMPTY: Self = Self {
        fast: [0; 512],
        first_code: [0; 16],
        max_code: [0; 17],
        first_symbol: [0; 16],
        size: [0; 288],
        value: [0; 288],
    };

    fn build(&mut self, lengths: &[u8]) -> Option<()> {
        let mut counts = [0i32; 17];
        let mut next_code = [0i32; 16];
        self.fast.fill(0);
        for &length in lengths {
            counts[length as usize] += 1;
        }
        counts[0] = 0;
        let mut code = 0;
        let mut symbol = 0;
        for size in 1..16 {
            if counts[size] > 1 << size {
                return None;
            }
            next_code[size] = code;
            self.first_code[size] = code as u16;
            self.first_symbol[size] = symbol as u16;
            code += counts[size];
            if counts[size] != 0 && code - 1 >= 1 << size {
                return None;
            }
            self.max_code[size] = code << (16 - size);
            code <<= 1;
            symbol += counts[size];
        }
        self.max_code[16] = 0x10000;
        for (symbol, &size) in lengths.iter().enumerate() {
            if size == 0 {
                continue;
            }
            let size = usize::from(size);
            let index = next_code[size] - i32::from(self.first_code[size])
                + i32::from(self.first_symbol[size]);
            self.size[index as usize] = size as u8;
            self.value[index as usize] = symbol as u16;
            if size <= 9 {
                let reversed = (next_code[size] as u16).reverse_bits() >> (16 - size);
                for index in (usize::from(reversed)..512).step_by(1 << size) {
                    self.fast[index] = ((size << 9) | symbol) as u16;
                }
            }
            next_code[size] += 1;
        }
        Some(())
    }
}

struct Output {
    data: *mut u8,
    length: usize,
    capacity: u32,
}

impl Output {
    unsafe fn reserve(&mut self, extra: u32) -> Option<()> {
        let length = self.length as u32;
        if u32::MAX - length < extra {
            return None;
        }
        let mut capacity = self.capacity;
        while length + extra > capacity {
            if capacity > u32::MAX / 2 {
                return None;
            }
            capacity *= 2;
        }
        let data = realloc(self.data.cast(), capacity as usize).cast::<u8>();
        if data.is_null() {
            return None;
        }
        self.data = data;
        self.capacity = capacity;
        Some(())
    }

    unsafe fn literal(&mut self, byte: u8) -> Option<()> {
        if self.length >= self.capacity as usize {
            self.reserve(1)?;
        }
        *self.data.add(self.length) = byte;
        self.length += 1;
        Some(())
    }

    unsafe fn repeat(&mut self, distance: usize, length: usize) -> Option<()> {
        if (self.length as core::ffi::c_long) < distance as core::ffi::c_long {
            return None;
        }
        if self.length + length > self.capacity as usize {
            self.reserve(length as u32)?;
        }
        let from = self.data.add(self.length - distance);
        if distance == 1 {
            let byte = *from;
            for offset in 0..length {
                *self.data.add(self.length + offset) = byte;
            }
        } else {
            for offset in 0..length {
                *self.data.add(self.length + offset) = *from.add(offset);
            }
        }
        self.length += length;
        Some(())
    }
}

const LENGTH_BASE: [u32; 31] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258, 0, 0,
];
const LENGTH_EXTRA: [i32; 31] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0, 0, 0,
];
const DISTANCE_BASE: [u32; 32] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577, 0, 0,
];
const DISTANCE_EXTRA: [i32; 32] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13, 0, 0,
];

struct Inflater<'a> {
    reader: BitReader<'a>,
    output: Output,
    literal_codes: Huffman,
    distance_codes: Huffman,
}

impl Inflater<'_> {
    unsafe fn compressed_block(&mut self) -> Option<()> {
        loop {
            let symbol = self.reader.symbol(&self.literal_codes)?;
            match symbol {
                0..=255 => self.output.literal(symbol as u8)?,
                256 => return Some(()),
                _ => {
                    let index = symbol - 257;
                    let length = LENGTH_BASE[index] + self.reader.extra_bits(LENGTH_EXTRA[index]);
                    let index = self.reader.symbol(&self.distance_codes)?;
                    let distance =
                        DISTANCE_BASE[index] + self.reader.extra_bits(DISTANCE_EXTRA[index]);
                    self.output.repeat(distance as usize, length as usize)?;
                }
            }
        }
    }

    fn dynamic_codes(&mut self) -> Option<()> {
        const ORDER: [usize; 19] = [
            16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
        ];
        let literal_count = self.reader.bits(5) as usize + 257;
        let distance_count = self.reader.bits(5) as usize + 1;
        let code_count = self.reader.bits(4) as usize + 4;
        let total = literal_count + distance_count;
        let mut sizes = [0u8; 19];
        for &index in &ORDER[..code_count] {
            sizes[index] = self.reader.bits(3) as u8;
        }
        let mut codes = Huffman::EMPTY;
        codes.build(&sizes)?;
        let mut lengths = [0u8; 455];
        let mut position = 0;
        while position < total {
            let symbol = self.reader.symbol(&codes)?;
            if symbol < 16 {
                lengths[position] = symbol as u8;
                position += 1;
                continue;
            }
            let (count, value) = match symbol {
                16 => {
                    let count = self.reader.bits(2) as usize + 3;
                    if position == 0 {
                        return None;
                    }
                    (count, lengths[position - 1])
                }
                17 => (self.reader.bits(3) as usize + 3, 0),
                18 => (self.reader.bits(7) as usize + 11, 0),
                _ => return None,
            };
            if total - position < count {
                return None;
            }
            lengths[position..position + count].fill(value);
            position += count;
        }
        self.literal_codes.build(&lengths[..literal_count])?;
        self.distance_codes.build(&lengths[literal_count..total])
    }

    unsafe fn stored_block(&mut self) -> Option<()> {
        if self.reader.count & 7 != 0 {
            self.reader.bits(self.reader.count & 7);
        }
        let mut header = [0u8; 4];
        let mut position = 0;
        while self.reader.count > 0 {
            header[position] = self.reader.buffer as u8;
            position += 1;
            self.reader.buffer >>= 8;
            self.reader.count -= 8;
        }
        if self.reader.count < 0 {
            return None;
        }
        for byte in &mut header[position..] {
            *byte = self.reader.byte();
        }
        let length = u16::from_le_bytes([header[0], header[1]]);
        if u16::from_le_bytes([header[2], header[3]]) != length ^ 0xffff {
            return None;
        }
        let length = usize::from(length);
        let bytes = self
            .reader
            .input
            .get(self.reader.position..self.reader.position + length)?;
        if self.output.length + length > self.output.capacity as usize {
            self.output.reserve(length as u32)?;
        }
        core::ptr::copy_nonoverlapping(
            bytes.as_ptr(),
            self.output.data.add(self.output.length),
            length,
        );
        self.reader.position += length;
        self.output.length += length;
        Some(())
    }

    unsafe fn decode(&mut self, header: bool) -> Option<()> {
        if header {
            let cmf = u32::from(self.reader.byte());
            let flags = u32::from(self.reader.byte());
            if self.reader.position == self.reader.input.len()
                || (cmf * 256 + flags) % 31 != 0
                || flags & 32 != 0
                || cmf & 15 != 8
            {
                return None;
            }
        }
        loop {
            let last = self.reader.bits(1) != 0;
            match self.reader.bits(2) {
                0 => self.stored_block()?,
                3 => return None,
                kind => {
                    if kind == 1 {
                        let mut lengths = [8u8; 288];
                        lengths[144..256].fill(9);
                        lengths[256..280].fill(7);
                        self.literal_codes.build(&lengths)?;
                        self.distance_codes.build(&[5; 32])?;
                    } else {
                        self.dynamic_codes()?;
                    }
                    self.compressed_block()?;
                }
            }
            if last {
                return Some(());
            }
        }
    }
}

pub(super) fn inflate(input: &[u8], initial_size: usize, header: bool) -> Option<Vec<u8>> {
    let _: i32 = input.len().try_into().ok()?;
    let initial_size: i32 = initial_size.try_into().ok()?;
    unsafe {
        let data = malloc(initial_size as usize).cast::<u8>();
        if data.is_null() {
            return None;
        }
        let mut inflater = Inflater {
            reader: BitReader {
                input,
                position: 0,
                buffer: 0,
                count: 0,
            },
            output: Output {
                data,
                length: 0,
                capacity: initial_size as u32,
            },
            literal_codes: Huffman::EMPTY,
            distance_codes: Huffman::EMPTY,
        };
        let result = inflater.decode(header).map(|()| {
            let output = &inflater.output;
            std::slice::from_raw_parts(output.data, output.length as i32 as usize).to_vec()
        });
        free(inflater.output.data.cast());
        result
    }
}
