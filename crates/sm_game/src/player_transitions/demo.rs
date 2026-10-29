//! Demo input playback words held by the Session.

/// Mode-dependent words also projected through the retained X-ray/suit HDMA path.
/// The timer and angle reuse their existing palette/transition owners.
#[derive(Clone, Default, Debug, PartialEq)]
pub struct DemoInput {
    pub pre_action: u16,
    pub delay: u16,
    pub program: u16,
    pub held: u16,
    pub pressed: u16,
    pub enabled: u16,
    pub previous_held: u16,
    pub previous_pressed: u16,
}
