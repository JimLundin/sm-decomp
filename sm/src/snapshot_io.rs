//! Checked, ordered byte transfers for the existing save-state format.
#![forbid(unsafe_code)]
use crate::recording::RecordingBytes;
use std::collections::TryReserveError;

pub enum SnapshotIo<'a> {
    Save(&'a mut RecordingBytes),
    Load(&'a [u8]),
}

pub(crate) enum Block<'a> {
    Save(&'a mut [u8]),
    Load(&'a [u8]),
}

#[derive(Debug)]
pub enum TransferError {
    Truncated,
    Allocation(TryReserveError),
}

impl SnapshotIo<'_> {
    pub fn remaining(&self) -> usize {
        match self {
            Self::Load(bytes) => bytes.len(),
            Self::Save(_) => 0,
        }
    }

    pub fn transfer(&mut self, state: &mut [u8]) -> Result<(), TransferError> {
        self.block(state.len(), |block| match block {
            Block::Save(output) => output.copy_from_slice(state),
            Block::Load(input) => state.copy_from_slice(input),
        })
    }

    pub(crate) fn block(
        &mut self,
        size: usize,
        transfer: impl FnOnce(Block<'_>),
    ) -> Result<(), TransferError> {
        match self {
            Self::Save(output) => {
                let start = output.size;
                output
                    .resize(start + size)
                    .map_err(TransferError::Allocation)?;
                transfer(Block::Save(&mut output.as_mut_slice()[start..]));
            }
            Self::Load(input) => {
                let (head, tail) = input
                    .split_at_checked(size)
                    .ok_or(TransferError::Truncated)?;
                transfer(Block::Load(head));
                *input = tail;
            }
        }
        Ok(())
    }
}
