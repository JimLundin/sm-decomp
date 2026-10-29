//! Real pure Scene dependency closure for detached Session presentation owners.
#![forbid(unsafe_code)]
#[path = "../../game/scene/atmosphere.rs"]
pub(crate) mod atmosphere;
#[path = "../../game/scene/atmosphere_protocol.rs"]
pub(crate) mod atmosphere_protocol;
#[path = "../../game/scene/checkpoint_codec.rs"]
pub(crate) mod checkpoint_codec;
#[path = "../../game/scene/commit.rs"]
pub(crate) mod commit;
#[path = "../../game/scene/component.rs"]
pub(crate) mod component;
#[path = "../../game/scene/delivery.rs"]
pub(crate) mod delivery;
#[path = "../../game/scene/error.rs"]
pub(crate) mod error;
#[path = "../../game/scene/hdma.rs"]
pub(crate) mod hdma;
#[path = "../../game/scene/hdma_sources.rs"]
pub(crate) mod hdma_sources;
#[path = "../../game/scene/irq.rs"]
pub(crate) mod irq;
#[path = "../../game/scene/objects.rs"]
pub(crate) mod objects;
#[path = "../../game/scene/owned_hdma.rs"]
pub(crate) mod owned_hdma;
#[path = "../../game/scene/records.rs"]
pub(crate) mod records;
#[path = "../../game/scene/resident_media.rs"]
pub(crate) mod resident_media;
#[path = "../../game/scene/samus.rs"]
pub(crate) mod samus;
#[path = "../../game/scene/samus_body.rs"]
pub(crate) mod samus_body;
#[path = "../../game/scene/samus_content.rs"]
pub(crate) mod samus_content;
#[path = "../../game/scene/samus_position_tiles.rs"]
pub(crate) mod samus_position_tiles;
#[path = "../../game/scene/samus_tile_upload.rs"]
pub(crate) mod samus_tile_upload;
#[path = "../../game/scene/settle.rs"]
pub(crate) mod settle;
#[path = "../../game/scene/start.rs"]
pub(crate) mod start;
#[allow(
    unused_imports,
    reason = "Detached Scene carrier retains the state facade without external slot reexport consumers."
)]
#[path = "../../game/scene/state.rs"]
pub(crate) mod state;
