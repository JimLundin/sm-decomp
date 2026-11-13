//! The content fingerprint (docs/architecture/assets.md §2, §6): the digest of a record set's
//! canonical encoding, by which a Checkpoint or a cache knows the content it was made with.

use core::hash::Hasher;

use canonical::{Canonical, to_bytes};

use crate::digest::Digest;

/// The digest of a record set's canonical encoding and of its shape.
///
/// Two sets with the same fingerprint hold the same records; a set whose format changed has
/// another one even where its bytes happen to agree, since the shape's schema is mixed in.
#[derive(Canonical, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ContentFingerprint(pub u64);

impl ContentFingerprint {
    /// The fingerprint of `content`.
    #[must_use]
    pub fn of<T: Canonical>(content: &T) -> Self {
        Self::of_encoding::<T>(&to_bytes(content))
    }

    /// The fingerprint of the content whose canonical encoding as a `T` is `encoding`.
    #[must_use]
    pub fn of_encoding<T: Canonical>(encoding: &[u8]) -> Self {
        let mut digest = Digest::default();
        digest.write_u64(T::SCHEMA);
        digest.write(encoding);
        Self(digest.finish())
    }
}

#[cfg(test)]
mod tests {
    use super::ContentFingerprint;
    use crate::assets::art::{Colour, Palette, PaletteSet, Tile, TileSheet};

    #[test]
    fn equal_content_has_one_fingerprint_and_a_changed_record_another() {
        let mut palettes = PaletteSet(vec![Palette::default(); 2]);
        let before = ContentFingerprint::of(&palettes);
        assert_eq!(before, ContentFingerprint::of(&palettes.clone()));
        palettes.0[1].0[3] = Colour(0x7fff);
        assert_ne!(before, ContentFingerprint::of(&palettes));
    }

    #[test]
    fn the_fingerprint_of_a_value_is_that_of_its_encoding() {
        let sheet = TileSheet(vec![Tile([[5; 8]; 8])]);
        let encoding = canonical::to_bytes(&sheet);
        assert_eq!(
            ContentFingerprint::of(&sheet),
            ContentFingerprint::of_encoding::<TileSheet>(&encoding)
        );
    }

    #[test]
    fn two_shapes_with_the_same_bytes_have_different_fingerprints() {
        // Both encode as a zero length.
        let none = canonical::to_bytes(&PaletteSet::default());
        assert_eq!(none, canonical::to_bytes(&TileSheet::default()));
        assert_ne!(
            ContentFingerprint::of_encoding::<PaletteSet>(&none),
            ContentFingerprint::of_encoding::<TileSheet>(&none)
        );
    }
}
