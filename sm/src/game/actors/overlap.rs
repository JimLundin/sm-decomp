//! Shared strict wrapping-axis overlap rule.
#![forbid(unsafe_code)]
pub(in crate::game) fn axis_overlaps(
    actor: u16,
    player: u16,
    actor_radius: u16,
    player_radius: u16,
) -> bool {
    let distance = (actor.wrapping_sub(player) as i16).wrapping_abs() as u16;
    distance < actor_radius || distance.wrapping_sub(actor_radius) < player_radius
}
