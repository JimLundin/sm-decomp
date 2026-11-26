//! The exact format boundary: what the model asks of the cartridge's encoding.

use types::save_files::{BatteryImage, Cartridge, CompletionMark, EncodedRun, RunSnapshot};

/// The cartridge's exact encoding, supplied at integration (and invented by the tests).
///
/// The model works on typed values and asks the format only for its marks, and to encode or
/// decode a run. The image mapping is the adapter's, at its input and output edges.
pub trait SaveFormat {
    /// The mark of a completed game. It differs from the incomplete mark.
    fn completed_mark(&self) -> CompletionMark;

    /// The mark of a game not completed.
    fn incomplete_mark(&self) -> CompletionMark;

    /// All 814 words of a run. Every word the checks cover comes from here: route, location
    /// and opaque values included.
    fn encode(&self, run: &RunSnapshot) -> EncodedRun;

    /// The typed run these words hold. Every payload decodes: a field the format doesn't
    /// recognize stays an opaque value, and the explored cells of Ceres stay empty.
    fn decode(&self, payload: &EncodedRun) -> RunSnapshot;

    /// The cartridge an image holds, every byte accounted for.
    fn cartridge(&self, image: &BatteryImage) -> Cartridge;

    /// The image of a cartridge: `cartridge`'s inverse, byte for byte.
    fn image(&self, cartridge: &Cartridge) -> BatteryImage;
}
