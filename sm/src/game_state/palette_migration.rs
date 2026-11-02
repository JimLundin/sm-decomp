//! Native palette ownership lasts only for the complete transfer's exclusive borrow.
//! The old Ready box is unavailable until the original physical prefix is complete.
use super::{Game, WorkRam, persistent_session};
use sm_game::palette_migration::{Buffer, Host, Image, Publication, Source, Transfer};

fn address(buffer: Buffer, offset: usize) -> usize {
    (match buffer {
        Buffer::Current => 0xc000usize,
        Buffer::Target => 0xc200usize,
    })
    .checked_add(offset)
    .expect("palette address")
}
fn selected_address(span: sm_game::palette_migration::Span, index: usize) -> usize {
    let first = address(span.buffer, span.first);
    let distance = index.checked_mul(2).expect("palette alias extent");
    if span.descending {
        first.checked_sub(distance)
    } else {
        first.checked_add(distance)
    }
    .expect("palette alias position")
}
fn owned_extent(span: sm_game::palette_migration::Span, count: usize) -> bool {
    if count == 0 {
        return true;
    }
    let Some(first) = (match span.buffer {
        Buffer::Current => 0xc000usize,
        Buffer::Target => 0xc200usize,
    })
    .checked_add(span.first) else {
        return false;
    };
    let Some(distance) = (count - 1).checked_mul(2) else {
        return false;
    };
    let last = if span.descending {
        first.checked_sub(distance)
    } else {
        first.checked_add(distance)
    };
    last.is_some_and(|last| (0xc000..0xc3ff).contains(&first) && (0xc000..0xc3ff).contains(&last))
}
/// Explicit retained full-allocation alias boundary. It does not admit a palette
/// owner or prevalidate the later original source/store failure prefixes.
fn foreign_alias(transfer: Transfer, game: &mut Game) {
    for index in 0..transfer.destination.words {
        let value = match &transfer.source {
            Source::Fill(value) => *value,
            Source::Colors(source) => {
                let at = selected_address(*source, index);
                source
                    .order
                    .decode([game.compatibility_byte(at), game.compatibility_byte(at + 1)])
            }
            Source::Rom { image, first } => {
                let at = first
                    .checked_add(index.checked_mul(2).expect("palette source extent"))
                    .expect("palette source offset");
                u16::from_ne_bytes([image.read(at), image.read(at + 1)])
            }
        };
        let at = selected_address(transfer.destination, index);
        let bytes = transfer.destination.order.encode(value);
        game.compatibility_write_byte(at, bytes[0]);
        game.compatibility_write_byte(at + 1, bytes[1]);
    }
}

/// No Game/CPU/backend callback is admitted while this owner borrows Game.
/// Each byte publication is an encoding operation, retaining its original epoch
/// and source-publication order. On unwind the old box has its old epoch, so no
/// completed native prefix can be hidden by a stale semantic palette.
struct Owned<'a> {
    game: &'a mut Game,
    previous: Option<persistent_session::AggregateSlot>,
}
impl<'a> Owned<'a> {
    fn acquire(game: &'a mut Game) -> Self {
        let previous = if matches!(
            game.persistent_session.aggregate,
            persistent_session::AggregateSlot::Ready(_)
        ) {
            Some(std::mem::replace(
                &mut game.persistent_session.aggregate,
                persistent_session::AggregateSlot::Leased,
            ))
        } else {
            None
        };
        Self { game, previous }
    }
    fn image(&self) -> Image {
        let words = |buffer| {
            core::array::from_fn(|index| {
                let at = address(buffer, 2 * index);
                u16::from_le_bytes([self.game.ram.read_byte(at), self.game.ram.read_byte(at + 1)])
            })
        };
        Image {
            colors: words(Buffer::Current),
            targets: words(Buffer::Target),
        }
    }
}
impl Publication for Owned<'_> {
    fn byte(&mut self, buffer: Buffer, offset: usize, byte: u8) {
        WorkRam::write_byte(&self.game.ram, address(buffer, offset), byte);
    }
}
impl Drop for Owned<'_> {
    fn drop(&mut self) {
        if let Some(previous) = self.previous.take() {
            // Do not refresh persistent_ram_epoch, preflight or admit a Runtime.
            // Restoring the old box leaves it suspended by the actual writes.
            self.game.persistent_session.aggregate = previous;
        }
    }
}
impl Host for Game {
    fn palette_transfer(&mut self, transfer: Transfer) {
        if transfer.destination.words == 0 {
            return;
        }
        if !owned_extent(transfer.destination, transfer.destination.words)
            || matches!(&transfer.source, Source::Colors(source)
                if !owned_extent(*source, transfer.destination.words))
        {
            foreign_alias(transfer, self);
            return;
        }
        // Preserve both byte queries of the first RAM source while its original
        // Ready/physical supplier is still active. ROM remains immutable and lazy.
        let first = match &transfer.source {
            Source::Fill(value) => *value,
            Source::Colors(source) => {
                let at = selected_address(*source, 0);
                source
                    .order
                    .decode([self.compatibility_byte(at), self.compatibility_byte(at + 1)])
            }
            Source::Rom { image, first } => {
                u16::from_ne_bytes([image.read(*first), image.read(*first + 1)])
            }
        };
        let mut owner = Owned::acquire(self);
        let mut image = owner.image();
        sm_game::palette_migration::run(transfer, first, &mut image, &mut owner);
    }
}
