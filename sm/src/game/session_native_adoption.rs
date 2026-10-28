//! Native compatibility edges carry complete supplied semantic snapshots.
#![forbid(unsafe_code)]

use crate::game_state::{Game, persistent_session::SessionSnapshotPayload};

// The legacy machine/actor codec has no complete semantic Session payload.
// Rejection precedes all machine/actor transfer; no metadata is inferred.
pub(crate) fn require_legacy_binary(game: &Game) -> Result<(), String> {
    if game
        .capture_session_snapshot()
        .map_err(|error| format!("{error:?}"))?
        .is_owned()
    {
        return Err("persistent Session requires a complete semantic checkpoint codec".into());
    }
    Ok(())
}

// Capture only an actually committed counterpart. Raw native bytes never create
// the evolved semantic Display, audio, content identities or input/history.
pub(crate) fn capture(game: &mut Game) -> Result<SessionSnapshotPayload, String> {
    let result = game.capture_session_snapshot();
    if result.is_err() {
        game.revoke_session_authority();
    }
    result.map_err(|error| format!("{error:?}"))
}

// Only the caller's selected full payload may reestablish ownership. The core
// compares its projection against actual final normalized bytes before moving it.
pub(crate) fn adopt(game: &mut Game, payload: &SessionSnapshotPayload) -> Result<(), String> {
    let result = game
        .prepare_session_adoption(payload)
        .and_then(|prepared| game.commit_session_adoption(prepared));
    if result.is_err() {
        game.revoke_session_authority();
    }
    result.map_err(|error| format!("{error:?}"))
}

pub(crate) fn reconcile_current(game: &mut Game) -> Result<(), String> {
    let actual = capture(game)?;
    adopt(game, &actual)
}
