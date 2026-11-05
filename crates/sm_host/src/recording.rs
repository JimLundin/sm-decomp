//! Owned recording buffers and the input/patch log codec.
#![forbid(unsafe_code)]
use std::collections::TryReserveError;
pub type BufferResult = Result<(), TryReserveError>;
pub(crate) const BASE_SRAM_SIZE: usize = 8192;

// Shared wire constants are used by both the recorder and retained replay decoder.
macro_rules! log_format {
    ($($name:ident : $ty:ty = $value:expr),* $(,)?) => {
        pub(crate) mod log_format {
            $(pub(crate) const $name: $ty = $value;)*
            #[cfg(any(feature = "frontend", sm_verify_dll))]
            pub(crate) fn serialized_layout(layout: &mut String) {
                layout.push_str("recorder-log{repeat{command:u8;delay:command-low-bits-plus-extension;variant{input:bit-toggle;patch:length-prefix-plus-extension,address-high-u8,address-low-u8,bytes:length*u8;}}extension:sum-u8-until-less-than-escape;");
                $(layout.push_str(concat!(stringify!($name), ":", stringify!($ty), "="));
                    layout.push_str(&$name.to_string()); layout.push(';');)*
                layout.push('}');
            }
        }
    };
}
log_format! {
    INPUT_BITS: i32 = 12,
    INPUT_LIMIT: i32 = 0xc0,
    PATCH_LIMIT: i32 = 0xd0,
    INPUT_SHIFT: i32 = 4,
    INPUT_DELAY_MASK: i32 = 15,
    PATCH_DELAY_MASK: i32 = 1,
    EXTENSION: u32 = 255,
    PATCH_LENGTH_SHIFT: i32 = 2,
    PATCH_LENGTH_MASK: i32 = 3,
    PATCH_LENGTH_BASE: i32 = 1,
    PATCH_LENGTH_ESCAPE: i32 = 4,
    ADDRESS_BANK_SHIFT: i32 = 1,
    ADDRESS_BANK_MASK: i32 = 1,
    ADDRESS_BANK_BIT: i32 = 16,
    ADDRESS_HIGH_SHIFT: i32 = 8,
    RAM_MASK: u32 = 0x1ffff,
}

/// Logical truncation retains initialized bytes for later reuse, as the recorder did.
#[derive(Default)]
pub struct RecordingBytes {
    storage: Vec<u8>,
    pub size: usize,
}
impl RecordingBytes {
    pub fn resize(&mut self, size: usize) -> BufferResult {
        self.size = size;
        if size > self.storage.len() {
            let old = self.storage.len();
            let capacity = size.max(old.wrapping_add(old >> 1).wrapping_add(8));
            self.storage.try_reserve_exact(capacity - old)?;
            self.storage.resize(capacity, 0);
        }
        Ok(())
    }
    pub fn as_slice(&self) -> &[u8] {
        &self.storage[..self.size]
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.storage[..self.size]
    }
    // Replay historically also reads retained bytes after logical truncation.
    pub fn byte(&self, index: usize) -> u8 {
        self.storage[index]
    }
    pub fn as_ptr(&self) -> *const u8 {
        self.storage.as_ptr()
    }
    pub fn as_mut_ptr(&mut self) -> *mut u8 {
        self.storage.as_mut_ptr()
    }
    pub fn push(&mut self, value: u8) -> BufferResult {
        let end = self.size;
        self.resize(end + 1)?;
        self.storage[end] = value;
        Ok(())
    }
    pub fn extend(&mut self, bytes: &[u8]) -> BufferResult {
        let start = self.size;
        self.resize(start + bytes.len())?;
        self.storage[start..self.size].copy_from_slice(bytes);
        Ok(())
    }
    fn variable_length(&mut self, mut value: u32) -> BufferResult {
        while value >= log_format::EXTENSION {
            self.push(log_format::EXTENSION as u8)?;
            value = value.wrapping_sub(log_format::EXTENSION);
        }
        self.push(value as u8)
    }
}

pub struct StateRecorder {
    pub last_inputs: u16,
    pub frames_since_last: u32,
    pub total_frames: u32,
    pub replay_pos: u32,
    pub replay_pos_last_complete: u32,
    pub replay_frame_counter: u32,
    pub replay_next_cmd_at: u32,
    pub snapshot_flags: u32,
    pub replay_cmd: u8,
    pub replay_mode: bool,
    pub log: RecordingBytes,
    pub base_snapshot: RecordingBytes,
    pub has_base_audio_queue: bool,
    pub base_audio_queue: [u8; 71],
}
impl Default for StateRecorder {
    fn default() -> Self {
        Self {
            last_inputs: 0,
            frames_since_last: 0,
            total_frames: 0,
            replay_pos: 0,
            replay_pos_last_complete: 0,
            replay_frame_counter: 0,
            replay_next_cmd_at: 0,
            snapshot_flags: 0,
            replay_cmd: 0,
            replay_mode: false,
            log: RecordingBytes::default(),
            base_snapshot: RecordingBytes::default(),
            has_base_audio_queue: false,
            base_audio_queue: [0; 71],
        }
    }
}
impl StateRecorder {
    fn command(&mut self, command: u8) -> BufferResult {
        let frames = self.frames_since_last as i32;
        self.frames_since_last = 0;
        let mask = if i32::from(command) < log_format::INPUT_LIMIT {
            log_format::INPUT_DELAY_MASK
        } else {
            log_format::PATCH_DELAY_MASK
        };
        self.log
            .push((i32::from(command) | frames.min(mask)) as u8)?;
        if frames >= mask {
            self.log.variable_length((frames - mask) as u32)?;
        }
        Ok(())
    }
    pub fn record(&mut self, inputs: u16) -> BufferResult {
        let changed = inputs ^ self.last_inputs;
        if changed != 0 {
            self.last_inputs = inputs;
            for bit in 0..log_format::INPUT_BITS {
                if changed >> bit & 1 != 0 {
                    self.command((bit << log_format::INPUT_SHIFT) as u8)?;
                }
            }
        }
        self.frames_since_last = self.frames_since_last.wrapping_add(1);
        self.total_frames = self.total_frames.wrapping_add(1);
        Ok(())
    }
    pub fn record_patch(&mut self, address: u32, bytes: &[u8]) -> BufferResult {
        assert!(address <= log_format::RAM_MASK);
        let length = i32::try_from(bytes.len()).expect("recorded patch length overflow");
        let prefix = (length - log_format::PATCH_LENGTH_BASE).min(log_format::PATCH_LENGTH_MASK);
        self.command(
            (log_format::INPUT_LIMIT
                | (if address & (1 << log_format::ADDRESS_BANK_BIT) != 0 {
                    log_format::ADDRESS_BANK_MASK << log_format::ADDRESS_BANK_SHIFT
                } else {
                    0
                })
                | prefix << log_format::PATCH_LENGTH_SHIFT) as u8,
        )?;
        if prefix == log_format::PATCH_LENGTH_MASK {
            self.log
                .variable_length((length - log_format::PATCH_LENGTH_ESCAPE) as u32)?;
        }
        self.log
            .push((address >> log_format::ADDRESS_HIGH_SHIFT) as u8)?;
        self.log.push(address as u8)?;
        for &byte in bytes {
            self.log.push(byte)?;
        }
        Ok(())
    }
    pub fn rebase(&mut self) -> BufferResult {
        let old_log = std::mem::take(&mut self.log);
        let old_delay = self.frames_since_last;
        self.frames_since_last = 0;
        if self.last_inputs != 0 {
            for bit in 0..log_format::INPUT_BITS {
                if self.last_inputs >> bit & 1 != 0 {
                    self.command((bit << log_format::INPUT_SHIFT) as u8)?;
                }
            }
        }
        if self.replay_mode {
            if self.replay_next_cmd_at != u32::MAX {
                self.replay_next_cmd_at = self.replay_next_cmd_at.wrapping_sub(old_delay);
                self.frames_since_last = self.replay_next_cmd_at;
                self.replay_pos_last_complete = self.log.size as u32;
                self.command(self.replay_cmd)?;
                let old_position = self.replay_pos as i32;
                self.replay_pos = self.log.size as u32;
                self.log
                    .extend(&old_log.as_slice()[old_position as usize..])?;
            }
            self.total_frames = self.total_frames.wrapping_sub(self.replay_frame_counter);
            self.replay_frame_counter = 0;
        } else {
            self.total_frames = 0;
        }
        self.frames_since_last = 0;
        Ok(())
    }
    pub fn stop(&mut self) {
        if self.replay_mode {
            self.replay_mode = false;
            self.total_frames = self.replay_frame_counter;
            self.log.size = self.replay_pos_last_complete as usize;
        }
    }
}
