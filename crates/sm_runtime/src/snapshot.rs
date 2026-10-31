//! Checked, ordered byte transfers for the existing save-state format.
use std::collections::TryReserveError;

/// Growable bytes a snapshot is saved into.
pub trait SnapshotOutput {
    /// Appends `size` bytes, which the caller then overwrites, and returns them.
    fn append(&mut self, size: usize) -> Result<&mut [u8], TryReserveError>;
}

pub enum SnapshotIo<'a> {
    Save(&'a mut dyn SnapshotOutput),
    Load(&'a [u8]),
}

pub enum Block<'a> {
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

    pub fn block(
        &mut self,
        size: usize,
        transfer: impl FnOnce(Block<'_>),
    ) -> Result<(), TransferError> {
        match self {
            Self::Save(output) => {
                let bytes = output.append(size).map_err(TransferError::Allocation)?;
                transfer(Block::Save(bytes));
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

pub struct Fields<'a> {
    block: Block<'a>,
    position: usize,
}

impl Fields<'_> {
    fn bytes(&mut self, bytes: &mut [u8]) {
        let end = self.position + bytes.len();
        match &mut self.block {
            Block::Save(output) => output[self.position..end].copy_from_slice(bytes),
            Block::Load(input) => bytes.copy_from_slice(&input[self.position..end]),
        }
        self.position = end;
    }
}

pub trait State {
    const SIZE: usize;
    fn layout(layout: &mut String);
    fn transfer(&mut self, fields: &mut Fields<'_>);
}

/// Static projections infer declared field types without constructing hardware values.
pub fn state_layout<T: State>(layout: &mut String, name: &str) {
    layout.push_str(name);
    layout.push(':');
    T::layout(layout);
}
pub fn field_layout<S, T: State>(layout: &mut String, name: &str, _field: fn(&S) -> &T) {
    state_layout::<T>(layout, name);
}

impl SnapshotIo<'_> {
    pub fn state<T: State>(&mut self, state: &mut T) -> Result<(), TransferError> {
        self.block(T::SIZE, |block| {
            let mut fields = Fields { block, position: 0 };
            state.transfer(&mut fields);
            assert_eq!(fields.position, T::SIZE);
        })
    }
}

macro_rules! scalar_state {
    ($($ty:ty),* $(,)?) => {$(
        impl State for $ty {
            const SIZE: usize = core::mem::size_of::<Self>();
                    fn layout(layout: &mut String) {
                layout.push_str(concat!(stringify!($ty), "-ne-"));
                layout.push_str(if cfg!(target_endian = "little") { "le;" } else { "be;" });
            }
            fn transfer(&mut self, fields: &mut Fields<'_>) {
                let mut bytes = self.to_ne_bytes();
                fields.bytes(&mut bytes);
                *self = Self::from_ne_bytes(bytes);
            }
        }
    )*};
}
scalar_state!(u8, i8, u16, i16, u32, i32, f64);

impl State for bool {
    const SIZE: usize = 1;
    fn layout(layout: &mut String) {
        layout.push_str("bool-u8-nonzero;");
    }
    fn transfer(&mut self, fields: &mut Fields<'_>) {
        let mut byte = u8::from(*self);
        byte.transfer(fields);
        *self = byte != 0;
    }
}

impl<T: State, const N: usize> State for [T; N] {
    const SIZE: usize = T::SIZE * N;
    fn layout(layout: &mut String) {
        layout.push_str(&format!("array[{N}]{{"));
        T::layout(layout);
        layout.push('}');
    }
    fn transfer(&mut self, fields: &mut Fields<'_>) {
        for value in self {
            value.transfer(fields);
        }
    }
}

/// Serializes the named fields of a record in order, leaving out `skip`.
#[macro_export]
macro_rules! state_record {
    ($ty:ty, $size:literal, { $($field:ident),* } skip { $($skip:ident),* }) => {
        impl $crate::snapshot::State for $ty {
            const SIZE: usize = $size;
                    fn layout(layout: &mut String) {
                layout.push_str(concat!(stringify!($ty), "{"));
                $($crate::snapshot::field_layout::<Self, _>(layout, stringify!($field), |state| &state.$field);)*
                layout.push('}');
            }
            fn transfer(&mut self, fields: &mut $crate::snapshot::Fields<'_>) {
                let Self { $($field,)* $($skip: _,)* } = self;
                $($crate::snapshot::State::transfer($field, fields);)*
            }
        }
    };
}
