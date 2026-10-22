//! Owned recording buffers and the input/patch log codec.
#![forbid(unsafe_code)]
use std::collections::TryReserveError;
pub type BufferResult = Result<(), TryReserveError>;

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
        while value >= 255 {
            self.push(255)?;
            value = value.wrapping_sub(255);
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
        let mask = if command < 0xc0 { 15 } else { 1 };
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
            for bit in 0..12 {
                if changed >> bit & 1 != 0 {
                    self.command((bit << 4) as u8)?;
                }
            }
        }
        self.frames_since_last = self.frames_since_last.wrapping_add(1);
        self.total_frames = self.total_frames.wrapping_add(1);
        Ok(())
    }
    pub fn record_patch(&mut self, address: u32, bytes: &[u8]) -> BufferResult {
        assert!(address < 0x20000);
        let length = i32::try_from(bytes.len()).expect("recorded patch length overflow");
        let prefix = (length - 1).min(3);
        self.command((0xc0 | (if address & 0x10000 != 0 { 2 } else { 0 }) | prefix << 2) as u8)?;
        if prefix == 3 {
            self.log.variable_length((length - 4) as u32)?;
        }
        self.log.push((address >> 8) as u8)?;
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
            for bit in 0..12 {
                if self.last_inputs >> bit & 1 != 0 {
                    self.command((bit << 4) as u8)?;
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
