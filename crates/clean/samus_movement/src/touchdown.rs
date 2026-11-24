//! Spec: `TouchdownRequests`. Acceptance offers ordered sounds and independent graphics
//! permission; transport and appearance make their own admission decisions later.

use crate::tick::Frame;
use types::{
    Action, FloorResponse, FloorResponseKind, Parts, QueueAllowance, SoundIdentity, SoundLibrary,
    SoundRequest, Sprite,
};

/// Spec: `TouchdownRequests`. Landing allowance is six, independent of normal drawing.
const LANDING_ALLOWANCE: QueueAllowance = QueueAllowance(6);
/// Spec: `TouchdownRequests`. Whole downward speed at least five admits the hard cue.
const HARD_SPEED: u16 = 5;
/// Spec: `TouchdownRequests`. Touchdown and wall-kick cues use library 3.
const LIBRARY_3: SoundLibrary = SoundLibrary::Library3;

/// Spec: `TouchdownRequests`. Library is part of each opportunity, not inferred by transport.
const fn request(identity: SoundIdentity) -> SoundRequest {
    let library = match identity {
        SoundIdentity::SpinEnded | SoundIdentity::ScrewAttackEnded => SoundLibrary::Samus,
        SoundIdentity::SamusLandedHard | SoundIdentity::SamusLandedOrWallJumped => LIBRARY_3,
    };
    SoundRequest {
        identity,
        library,
        queue_allowance: LANDING_ALLOWANCE,
    }
}

/// Spec: `TouchdownRequests`. Wall kicks bypass cinematic suppression and offer no graphics.
/// Real touchdowns first end a prior spin, then offer the speed cue. Whole-only soft speeds
/// offer neither cue nor graphics; cinematic removes sounds only, not graphics permission.
/// Prior figure is independent of current items; wall jumps always end with the plain cue.
pub fn opportunities(
    response: Option<FloorResponse>,
    wall_kick: bool,
    prior_action: Action,
    frame: &Frame,
) -> (Vec<SoundRequest>, bool) {
    if wall_kick {
        return (vec![request(SoundIdentity::SamusLandedOrWallJumped)], false);
    }
    let Some(response) = response.filter(|r| {
        matches!(
            r.kind,
            FloorResponseKind::UprightTouchdown
                | FloorResponseKind::BallSettles
                | FloorResponseKind::SpringHop
                | FloorResponseKind::Rebound(_)
        )
    }) else {
        return (Vec::new(), false);
    };
    let speed = response.speed.0;
    let cue = if speed.pixel().0 >= HARD_SPEED {
        Some(SoundIdentity::SamusLandedHard)
    } else if speed.sub() != 0 {
        Some(SoundIdentity::SamusLandedOrWallJumped)
    } else {
        None
    };
    let graphics = cue.is_some();
    if frame.cinematic_active {
        return (Vec::new(), graphics);
    }
    let mut requests = Vec::new();
    if prior_action.spins() {
        let identity = if prior_action == Action::Spinning
            && frame.sprite_before_movement == Sprite::ScrewAttack
        {
            SoundIdentity::ScrewAttackEnded
        } else {
            SoundIdentity::SpinEnded
        };
        requests.push(request(identity));
    }
    requests.extend(cue.map(request));
    (requests, graphics)
}
