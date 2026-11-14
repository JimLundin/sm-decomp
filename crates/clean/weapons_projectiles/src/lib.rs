//! Uncharged power beam model. Spec: HoldingShoot through BeamVisibility.
//! The caller owns State, imports content values, supplies World, and consumes requests.
//!
//! Ordinary frame protocol: call [`update`] once, dispatch its [`Event`] requests in returned
//! order, let actors report late hits with [`State::consume`], then call [`pictures`] with the
//! presentation phase and view. Consumed Flying bodies remain drawable at this cut and are
//! removed on the next active weapons update. World reactions happen immediately *inside*
//! update, not when dispatching the returned sound, protection and trail requests.
//!
//! Public stage functions expose isolated observations for integrations that need cuts. They
//! do not replace the ordinary entry: spacing, preparation, muzzle probing, descending slot
//! updates, late consumption and drawing must still occur in the specified order. Stage calls
//! do not gate stopped time themselves; [`update`] owns that update-mode decision.
#![expect(
    clippy::doc_markdown,
    reason = "Plain spec entry ids in Spec citations are intentional, not Rust item links"
)]
#![expect(
    clippy::similar_names,
    reason = "Shot state and its stable slot are distinct spec concepts consistently named shot and slot"
)]
mod catalogue;
mod domain;
pub use catalogue::*;
pub use domain::*;

const BYTE_BITS: u32 = 8;
const PIXEL_BITS: u32 = 16;
const SPEED_TO_POSITION: i32 = 256;
const LOW_BYTE: u16 = 255;
const CELL_MASK: u16 = 65520;
const CELL_BITS: u32 = 4;
const MAX_SPAN: u16 = 16;
const SLOT_COUNT: u16 = 5;
const FROZEN_HOLD: u16 = 32;
const PRESSED_SPACING: u16 = 15;
const HELD_SPACING: u16 = 25;
const INITIAL_TRAIL: u16 = 4;
const PROTECTION: u16 = 10;
const CARDINAL_BASE: u16 = 1024;
const DIAGONAL_BASE: u16 = 683;
const UP_SHIFT: u32 = 2;
const UP_EXTENSION: u16 = 49152;
const IMPACT_DAMAGE: Damage = Damage(8);
const VIEW_LOW: i16 = -64;
const VIEW_HIGH: i16 = 320;
const SCREEN_HEIGHT: u16 = 256;
const IMPACT_LOW: i16 = -48;
const IMPACT_HIGH: i16 = 304;

/// Spec: FlyingBeam, BeamTrail. Adjustable parameters; other named behaviours are fixed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// Signed wrapping acceleration on each aimed axis, in speed units.
    pub acceleration: BeamSpeed,
    /// Countdown restored when a trail fires.
    pub trail_interval: FrameWord,
}
impl Rules {
    /// Reference profile given by the spec.
    pub const REFERENCE: Self = Self {
        acceleration: BeamSpeed(16),
        trail_interval: FrameWord(4),
    };
}
impl FixedPosition {
    /// Spec: FlyingBeam. High word is the integer centre.
    #[must_use]
    pub const fn integer(self) -> u16 {
        let [a, b, _, _] = self.0.to_be_bytes();
        u16::from_be_bytes([a, b])
    }
    /// Spec: MuzzleContact. Low word is the retained fraction.
    #[must_use]
    pub const fn fraction(self) -> PixelFraction {
        let [_, _, a, b] = self.0.to_be_bytes();
        PixelFraction(u16::from_be_bytes([a, b]))
    }
    /// Construct an integer centre with its fraction.
    #[must_use]
    pub fn from_parts(integer: u16, fraction: PixelFraction) -> Self {
        Self(u32::from(integer).wrapping_shl(PIXEL_BITS) | u32::from(fraction.0))
    }
    /// Spec: FlyingBeam. Add a signed speed with wrapping carry into the integer centre.
    pub fn move_by(&mut self, speed: BeamSpeed) {
        let distance = i32::from(speed.0)
            .wrapping_mul(SPEED_TO_POSITION)
            .cast_unsigned();
        self.0 = self.0.wrapping_add(distance);
    }
    fn shift_integer(&mut self, amount: i16) {
        *self = Self::from_parts(self.integer().wrapping_add_signed(amount), self.fraction());
    }
}
/// Spec: ShotSpacing. Cooldown updates before availability, including frozen cannon updates.
///
/// Running CannonAndShots decrements only a signed-positive whole word, otherwise sets zero.
/// This is not a wrapping countdown like holds/trails nor the low-byte availability test.
/// Isolated stage: follow with preparation only when the cannon is active and time running;
/// ShotsOnly must still update shots even with stopped time. Prefer update for that protocol.
pub fn update_spacing(cooldown: &mut FrameWord, mode: UpdateMode, time_stopped: bool) {
    if mode == UpdateMode::ShotsOnly {
        return;
    }
    cooldown.0 = if time_stopped {
        FROZEN_HOLD
    } else if cooldown.0.cast_signed() > 0 {
        cooldown.0.wrapping_sub(1)
    } else {
        0
    };
}
/// Spec: CannonOrigin. Resolve pending/previous aim and use the current descriptor's offsets.
#[must_use]
pub fn cannon_origin(input: &CannonInput) -> Option<(Heading, SamusCentre)> {
    let heading = match (input.pending_aim, input.muzzle.aim, input.previous_muzzle) {
        (Some(heading), _, _)
        | (None, MuzzleAim::Aimed(heading), _)
        | (None, MuzzleAim::PreviousAim, MuzzleAim::Aimed(heading)) => heading,
        (None, MuzzleAim::Forbidden | MuzzleAim::PreviousAim, _) => return None,
    };
    let offset = if input.moving_muzzle {
        input.muzzle.moving[heading]
    } else {
        input.muzzle.stationary[heading]
    };
    Some((
        heading,
        SamusCentre(Axes {
            x: input
                .centre
                .0
                .x
                .wrapping_add(origin_word(offset.x.cast_unsigned())),
            y: input
                .centre
                .0
                .y
                .wrapping_add(origin_word(offset.y.cast_unsigned()))
                .wrapping_sub(origin_word(input.muzzle.lift)),
        }),
    ))
}
/// Spec: CannonOrigin. Offset and lift storage widths do not bound the wrapping origin.
const fn origin_word(value: u64) -> u16 {
    let [_, _, _, _, _, _, high, low] = value.to_be_bytes();
    u16::from_be_bytes([high, low])
}
/// Spec: InheritedMotion. Reproduce the asymmetric byte sources and upward extension quirk.
#[must_use]
pub fn inherited_motion(heading: Heading, movement: Inheritance) -> Axes<BeamSpeed> {
    let axes = heading.axes();
    let base = if axes.x != 0 && axes.y != 0 {
        DIAGONAL_BASE
    } else {
        CARDINAL_BASE
    };
    let x = match axes.x.cmp(&0) {
        std::cmp::Ordering::Greater => base
            .wrapping_add(low_whole(movement.right))
            .wrapping_add(high_fraction(movement.left.fraction)),
        std::cmp::Ordering::Less => base
            .wrapping_neg()
            .wrapping_add(low_whole(movement.left))
            .wrapping_add(high_fraction(movement.camera_vertical_fraction)),
        std::cmp::Ordering::Equal => 0,
    };
    let y = inherited_vertical(axes.y, base, movement);
    Axes {
        x: BeamSpeed(x.cast_signed()),
        y: BeamSpeed(y.cast_signed()),
    }
}
const fn low_whole(distance: MovementDistance) -> u16 {
    (distance.whole & LOW_BYTE).wrapping_shl(BYTE_BITS)
}
const fn high_fraction(fraction: PixelFraction) -> u16 {
    fraction.0.wrapping_shr(BYTE_BITS)
}
const fn inherited_vertical(axis: i16, base: u16, movement: Inheritance) -> u16 {
    if axis > 0 {
        return base
            .wrapping_add(low_whole(movement.down))
            .wrapping_add(high_fraction(movement.up.fraction));
    }
    if axis == 0 {
        return 0;
    }
    if movement.up.whole & LOW_BYTE == 0 {
        return base.wrapping_neg();
    }
    let inherited = low_whole(movement.up)
        .wrapping_add(high_fraction(movement.right.fraction))
        .wrapping_shr(UP_SHIFT)
        | UP_EXTENSION;
    base.wrapping_neg().wrapping_add(inherited)
}
/// Spec: HoldingShoot, CannonOrigin. Prepare a body, without muzzle contact or flight.
///
/// Isolated stage prerequisite: spacing has run in a running CannonAndShots update. An
/// unavailable attempt changes nothing. An available attempt reserves count and cooldown 1;
/// invalid aim rolls count back wrapping, leaving cooldown 1, slots and events unchanged.
/// Success replaces the reserved slot, preserves its fractions and appearance channels,
/// assigns pressed/held spacing and appends fire, protection, then pending-aim requests.
/// The caller must next run muzzle_contact, apply fresh-impact lifetime eligibility, and
/// update every slot in descending order including this shot. Ordinary callers use update.
pub fn prepare_shot(
    state: &mut State,
    input: &CannonInput,
    content: &PowerBeam,
    events: &mut Vec<Event>,
) -> Option<ShotSlot> {
    let slot = reserve_shot(state, input.shoot)?;
    let Some((heading, centre)) = cannon_origin(input) else {
        state.count.0 = state.count.0.wrapping_sub(1);
        return None;
    };
    let body = prepared_body(&state.slots[slot], heading, centre, content);
    state.slots[slot] = ShotState::Flying {
        body,
        consumed: false,
    };
    state.cooldown = FrameWord(if input.new_press || input.previous_new_press {
        PRESSED_SPACING
    } else {
        HELD_SPACING
    });
    events.push(Event::FireSound(slot));
    events.push(Event::Protection(FrameWord(PROTECTION)));
    if input.pending_aim.is_some() {
        events.push(Event::PendingAimConsumed);
    }
    Some(slot)
}
/// Spec: HoldingShoot, MuzzleContact. Build the replacement body without changing the slot.
/// Only fractions and appearance channels come from the old slot; all other body quantities
/// are prepared anew. Zero speeds are probe inputs, not inherited flight speeds.
fn prepared_body(
    old: &ShotState,
    heading: Heading,
    centre: SamusCentre,
    content: &PowerBeam,
) -> ShotBody {
    let fractions = match old {
        ShotState::Free {
            retained_fraction, ..
        } => *retained_fraction,
        ShotState::Flying { body, .. } | ShotState::Impact { body } => Axes {
            x: body.position.x.fraction(),
            y: body.position.y.fraction(),
        },
    };
    let clip = content.flying_clips[heading].clone();
    // Valid imported clips always have a first frame. This total fallback is not validation
    // and does not turn radius zero (valid content) into a missing-content sentinel.
    let radius = content
        .clips
        .get(&clip)
        .and_then(|clip| clip.frames.first())
        .map_or(
            Axes {
                x: Radius(0),
                y: Radius(0),
            },
            |frame| frame.radius,
        );
    ShotBody {
        position: Axes {
            x: FixedPosition::from_parts(centre.0.x, fractions.x),
            y: FixedPosition::from_parts(centre.0.y, fractions.y),
        },
        heading,
        speed: Axes {
            x: BeamSpeed(0),
            y: BeamSpeed(0),
        },
        radius,
        damage: content.damage,
        trail: FrameWord(INITIAL_TRAIL),
        clip,
        next_frame: ClipFrame::at(0),
        remaining_hold: FrameWord(1),
        picture: old.body().and_then(|body| body.picture.clone()),
        trail_appearance: old.body().and_then(|body| body.trail_appearance.clone()),
    }
}
/// Spec: HoldingShoot. Availability reservation uses independent count, not occupancy.
/// Signed wrapping count-minus-five must be negative and only the cooldown's low byte must
/// be zero (unlike spacing's signed whole-word test). Reserve count wrapping and cooldown 1.
/// Free counts as zero damage; an occupied zero-damage body is also replaceable without a
/// removal decrement. Search ascending, falling back to the final slot if every damage is
/// nonzero. These rules intentionally support independently restored count and occupancy.
fn reserve_shot(state: &mut State, shoot: bool) -> Option<ShotSlot> {
    if !shoot
        || state.count.0.wrapping_sub(SLOT_COUNT).cast_signed() >= 0
        || state.cooldown.0 & LOW_BYTE != 0
    {
        return None;
    }
    state.count.0 = state.count.0.wrapping_add(1);
    state.cooldown = FrameWord(1);
    Some(
        state
            .slots
            .iter()
            .find(|(_, shot)| shot.body().is_none_or(|body| body.damage.value() == 0))
            .map_or(ShotSlot::Slot4, |(slot, _)| slot),
    )
}
/// Spec: FlyingBeam. Accelerate all aimed components before moving either axis.
///
/// Isolated stage: only unconsumed Flying is eligible; follow with horizontal then vertical
/// motion/contact (vertical only if still Flying), lifetime check, and clip update.
pub const fn accelerate(body: &mut ShotBody, rules: Rules) {
    let axes = body.heading.axes();
    body.speed.x.0 = body
        .speed
        .x
        .0
        .wrapping_add(rules.acceleration.0.wrapping_mul(axes.x));
    body.speed.y.0 = body
        .speed
        .y
        .0
        .wrapping_add(rules.acceleration.0.wrapping_mul(axes.y));
}
/// Spec: ShotContact. Computed edge geometry, including edges skipped by the span/room gates.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EdgeGeometry {
    /// Leading integer pixel.
    pub leading: u16,
    /// Perpendicular first pixel.
    pub start: u16,
    /// Cell span minus one.
    pub span_minus_one: u16,
    /// Whether the span is checked against World.
    pub requested: bool,
}
/// Spec: ShotContact. Leading edge uses the sign of speed, including zero's positive edge.
#[must_use]
pub fn edge_geometry(
    centre: Axes<u16>,
    radius: Axes<Radius>,
    speed: BeamSpeed,
    axis: Axis,
    screens: u16,
) -> EdgeGeometry {
    let (along, perpendicular, r, across) = match axis {
        Axis::Horizontal => (centre.x, centre.y, radius.x, radius.y),
        Axis::Vertical => (centre.y, centre.x, radius.y, radius.x),
    };
    let leading = if speed.0 < 0 {
        along.wrapping_sub(u16::from(r.0))
    } else {
        along.wrapping_add(u16::from(r.0)).wrapping_sub(1)
    };
    let start = perpendicular.wrapping_sub(u16::from(across.0));
    let span_minus_one = perpendicular
        .wrapping_add(u16::from(across.0))
        .wrapping_sub(1)
        .wrapping_sub(start & CELL_MASK)
        .wrapping_shr(CELL_BITS);
    let requested = span_minus_one < MAX_SPAN
        && leading
            .wrapping_shr(BYTE_BITS)
            .wrapping_sub(screens)
            .cast_signed()
            < 0;
    EdgeGeometry {
        leading,
        start,
        span_minus_one,
        requested,
    }
}
/// Spec: ShotContact. Visit and react to every cell immediately; all must block to burst.
///
/// Isolated stage prerequisite: this axis's movement (or muzzle probe) is already applied,
/// using the radius before the clip change. True requires burst; after horizontal contact,
/// process vertical only if still Flying. Follow contact with lifetime and clip decisions.
pub fn contact_edge(
    body: &ShotBody,
    slot: ShotSlot,
    axis: Axis,
    room: RoomScreens,
    world: &mut impl World,
) -> bool {
    let centre = Axes {
        x: body.position.x.integer(),
        y: body.position.y.integer(),
    };
    let (speed, screens) = match axis {
        Axis::Horizontal => (body.speed.x, room.0.x),
        Axis::Vertical => (body.speed.y, room.0.y),
    };
    let edge = edge_geometry(centre, body.radius, speed, axis, screens);
    if !edge.requested {
        return false;
    }
    let contact = Contact {
        slot,
        heading: body.heading,
        axis,
        centre,
        radius: body.radius,
        leading_boundary: edge.leading,
        perpendicular_start: edge.start,
        cell_count: edge.span_minus_one.wrapping_add(1),
    };
    let mut blocked = true;
    for cell in 0..contact.cell_count {
        let response = world.contact_cell(&contact, cell);
        blocked &= response == ContactResponse::Blocks;
    }
    blocked
}
/// Spec: ShotContact. Burst shifts integer centres along every aimed axis, preserving fractions,
/// speeds, radius and picture until the subsequent clip step.
///
/// Isolated stage prerequisite: an all-blocking edge on Flying. Other phases are unchanged.
/// Follow with the fresh-impact lifetime check before the impact clip's first frame.
pub fn burst(shot: &mut ShotState, slot: ShotSlot, impact_clip: &ClipId, events: &mut Vec<Event>) {
    let ShotState::Flying { body, .. } = shot else {
        return;
    };
    let axes = body.heading.axes();
    body.position
        .x
        .shift_integer(i16::from(body.radius.x.0).wrapping_mul(axes.x));
    body.position
        .y
        .shift_integer(i16::from(body.radius.y.0).wrapping_mul(axes.y));
    body.damage = IMPACT_DAMAGE;
    body.clip = impact_clip.clone();
    body.next_frame = ClipFrame::at(0);
    body.remaining_hold = FrameWord(1);
    *shot = ShotState::Impact { body: body.clone() };
    events.push(Event::ImpactSound(slot));
}
fn motion_axis(
    shot: &mut ShotState,
    slot: ShotSlot,
    axis: Axis,
    content: &PowerBeam,
    room: RoomScreens,
    world: &mut impl World,
    events: &mut Vec<Event>,
) {
    let ShotState::Flying { body, .. } = shot else {
        return;
    };
    let aims = body.heading.axes();
    match axis {
        Axis::Horizontal if aims.x != 0 => body.position.x.move_by(body.speed.x),
        Axis::Vertical if aims.y != 0 => body.position.y.move_by(body.speed.y),
        Axis::Horizontal | Axis::Vertical => return,
    }
    if contact_edge(body, slot, axis, room, world) {
        burst(shot, slot, &content.impact_clip, events);
    }
}
/// Spec: MuzzleContact. Probe aimed axes at zero speed, except pure Left at -1. Only surviving
/// Flying bodies have fractions zeroed and receive inherited speed after this probe.
///
/// Isolated stage prerequisite: successful prepare_shot, before any clip change. An impact
/// retains probe fractions and speeds; the caller must apply its fresh-impact lifetime check.
/// Then perform descending slot updates, including this shot's first flight/clip update in
/// the same frame. This stage does not perform that subsequent work or gate stopped time.
pub fn muzzle_contact(
    shot: &mut ShotState,
    slot: ShotSlot,
    input: &UpdateInput,
    content: &PowerBeam,
    world: &mut impl World,
    events: &mut Vec<Event>,
) {
    if let ShotState::Flying { body, .. } = shot {
        body.speed = Axes {
            x: BeamSpeed(if body.heading == Heading::Left { -1 } else { 0 }),
            y: BeamSpeed(0),
        };
    }
    motion_axis(
        shot,
        slot,
        Axis::Horizontal,
        content,
        input.room,
        world,
        events,
    );
    motion_axis(
        shot,
        slot,
        Axis::Vertical,
        content,
        input.room,
        world,
        events,
    );
    if let ShotState::Flying { body, .. } = shot {
        body.position.x = FixedPosition::from_parts(body.position.x.integer(), PixelFraction(0));
        body.position.y = FixedPosition::from_parts(body.position.y.integer(), PixelFraction(0));
        body.speed = inherited_motion(body.heading, input.inheritance);
    }
}
/// Spec: BeamTrail. Decrement wrapping; emit the pre-motion position and retained channels.
///
/// Zero decrements to 65535, without emission. An interval of zero is valid: on emission it
/// restores zero and the next update wraps. Isolated stage applies only to unconsumed Flying
/// before acceleration/contact; update owns time-stop eligibility.
pub fn update_trail(body: &mut ShotBody, slot: ShotSlot, rules: Rules, events: &mut Vec<Event>) {
    body.trail.0 = body.trail.0.wrapping_sub(1);
    if body.trail.0 != 0 {
        return;
    }
    body.trail = rules.trail_interval;
    events.push(Event::Trail {
        slot,
        position: body.position,
        picture: body.picture.clone(),
        appearance: body.trail_appearance.clone(),
    });
}
/// Spec: LeavingView. Lifetime eligibility, not drawing eligibility: signed wrapping distance
/// is inside the lower-inclusive/upper-exclusive box.
///
/// Observe after flight/contact and before clip changes; fresh impacts share it, existing
/// impacts do not. A false isolated result requires remove_shot. BeamVisibility instead uses phase and screen-coordinate cutoffs.
#[must_use]
pub fn within_lifetime_view(centre: Axes<u16>, camera: ScreenOrigin) -> bool {
    let x = centre.x.wrapping_sub(camera.0.x).cast_signed();
    let y = centre.y.wrapping_sub(camera.0.y).cast_signed();
    (VIEW_LOW..VIEW_HIGH).contains(&x) && (VIEW_LOW..VIEW_HIGH).contains(&y)
}
/// Spec: LeavingView. Compatibility spelling of within_lifetime_view; not draw eligibility.
#[must_use]
pub fn inside_view(centre: Axes<u16>, camera: ScreenOrigin) -> bool {
    within_lifetime_view(centre, camera)
}
/// Spec: EndingShot. Clear the body and fractions, keep trail, decrement count then signed clamp.
pub fn remove_shot(shot: &mut ShotState, count: &mut ShotCount) {
    let Some(body) = shot.body() else {
        return;
    };
    *shot = ShotState::Free {
        trail: body.trail,
        retained_fraction: Axes {
            x: PixelFraction(0),
            y: PixelFraction(0),
        },
    };
    let decremented = count.0.wrapping_sub(1);
    count.0 = if decremented.cast_signed() < 0 {
        0
    } else {
        decremented
    };
}
/// Spec: BeamPictures. Wrap the hold; on zero display next/repeated frame or request removal.
///
/// Inactive trail frames preserve the retained appearance. A zero hold wraps to 65535 on
/// the next eligible update, rather than immediately advancing. Isolated stage follows motion,
/// contact and any eligible lifetime removal; true requires the caller to remove_shot so count
/// and free-slot trail are updated. Existing impacts only run this stage, not motion/view.
#[must_use]
pub fn update_clip(body: &mut ShotBody, content: &PowerBeam) -> bool {
    body.remaining_hold.0 = body.remaining_hold.0.wrapping_sub(1);
    if body.remaining_hold.0 != 0 {
        return false;
    }
    let Some(clip) = content.clips.get(&body.clip) else {
        return false;
    };
    let index = if body.next_frame.index() < clip.frames.len() {
        body.next_frame
    } else {
        match clip.ending {
            ClipEnding::Remove => return true,
            ClipEnding::RepeatFrom(frame) => frame,
        }
    };
    let Some(frame) = clip.frames.get(index.index()) else {
        return false;
    };
    body.picture = Some(frame.picture.clone());
    body.radius = frame.radius;
    if let Some(appearance) = &frame.trail_appearance {
        body.trail_appearance = Some(appearance.clone());
    }
    body.remaining_hold = FrameWord(frame.hold.value());
    body.next_frame = ClipFrame::at(index.index().wrapping_add(1));
    false
}
/// Spec: ShotContact. One update's ordered reactions and requests, shared across slot work.
struct UpdatePass<'a, W> {
    input: &'a UpdateInput,
    content: &'a PowerBeam,
    rules: Rules,
    world: &'a mut W,
    events: Vec<Event>,
}
impl<W: World> UpdatePass<'_, W> {
    /// Spec: EndingShot, FlyingBeam, BeamPictures. Complete one slot before visiting the next.
    fn update_slot(&mut self, shot: &mut ShotState, slot: ShotSlot, count: &mut ShotCount) {
        let Self {
            input,
            content,
            rules,
            world,
            events,
        } = self;
        if matches!(shot, ShotState::Flying { consumed: true, .. }) {
            remove_shot(shot, count);
            return;
        }
        if let ShotState::Flying { body, .. } = shot {
            update_trail(body, slot, *rules, events);
            accelerate(body, *rules);
            motion_axis(
                shot,
                slot,
                Axis::Horizontal,
                content,
                input.room,
                *world,
                events,
            );
            motion_axis(
                shot,
                slot,
                Axis::Vertical,
                content,
                input.room,
                *world,
                events,
            );
            if remove_outside_lifetime_view(shot, count, input.camera) {
                return;
            }
        }
        if shot
            .body_mut()
            .is_some_and(|body| update_clip(body, content))
        {
            remove_shot(shot, count);
        }
    }
}
/// Spec: LeavingView. Flying and newly formed impacts share this lifetime decision, before
/// their clip changes. Existing impacts do not use it; drawing has separate eligibility.
fn remove_outside_lifetime_view(
    shot: &mut ShotState,
    count: &mut ShotCount,
    camera: ScreenOrigin,
) -> bool {
    let outside = shot.body().is_some_and(|body| {
        !within_lifetime_view(
            Axes {
                x: body.position.x.integer(),
                y: body.position.y.integer(),
            },
            camera,
        )
    });
    if outside {
        remove_shot(shot, count);
    }
    outside
}
/// Spec: ShotSpacing through BeamPictures. Ordinary complete weapons update.
///
/// Dispatch returned requests in order, then apply late actor hits via State::consume and
/// draw via pictures. World has already applied ordered contact reactions during this call.
/// CannonAndShots updates spacing first; stopped time sets cooldown 32 and suspends firing,
/// motion, removal and clips. ShotsOnly ignores stopped time, leaves cooldown unchanged and
/// never fires. Running cannon preparation and muzzle contact precede descending slot work.
#[must_use]
pub fn update(
    state: &mut State,
    input: &UpdateInput,
    content: &PowerBeam,
    rules: Rules,
    world: &mut impl World,
) -> Vec<Event> {
    let mut events = Vec::new();
    update_spacing(&mut state.cooldown, input.mode, input.time_stopped);
    if input.mode == UpdateMode::CannonAndShots && input.time_stopped {
        return events;
    }
    if input.mode == UpdateMode::CannonAndShots
        && let Some(slot) = prepare_shot(state, &input.cannon, content, &mut events)
    {
        muzzle_contact(
            &mut state.slots[slot],
            slot,
            input,
            content,
            world,
            &mut events,
        );
        // Spec: LeavingView. A fresh muzzle impact, like a flight impact, is checked
        // after its contact and before its first impact picture.
        if matches!(state.slots[slot], ShotState::Impact { .. }) {
            remove_outside_lifetime_view(&mut state.slots[slot], &mut state.count, input.camera);
        }
    }
    let mut pass = UpdatePass {
        input,
        content,
        rules,
        world,
        events,
    };
    for slot in ShotSlot::DESCENDING {
        pass.update_slot(&mut state.slots[slot], slot, &mut state.count);
    }
    pass.events
}
/// Spec: BeamVisibility. Separate descending producers use caller presentation phase and view;
/// consumed Flying remains eligible, rotating views bypass impact's normal horizontal cutoff.
#[must_use]
pub fn pictures(state: &State, phase: FlickerPhase, view: &DrawView) -> PictureIntents {
    let mut intents = PictureIntents {
        flying: Vec::new(),
        impact: Vec::new(),
    };
    for slot in ShotSlot::DESCENDING {
        if let Some((intent, impact)) = picture_for(&state.slots[slot], slot, phase, view) {
            if impact {
                intents.impact.push(intent);
            } else {
                intents.flying.push(intent);
            }
        }
    }
    intents
}
fn picture_for(
    shot: &ShotState,
    slot: ShotSlot,
    phase: FlickerPhase,
    view: &DrawView,
) -> Option<(PictureIntent, bool)> {
    let body = shot.body()?;
    let picture = body.picture.as_ref()?;
    let impact = matches!(shot, ShotState::Impact { .. });
    if !impact && slot.even() != (phase == FlickerPhase::Odd) {
        return None;
    }
    let screen = match view {
        DrawView::Normal(origin) => ScreenCentre(Axes {
            x: body.position.x.integer().wrapping_sub(origin.0.x),
            y: body.position.y.integer().wrapping_sub(origin.0.y),
        }),
        DrawView::Rotating(centres) => centres[slot],
    };
    if screen.0.y >= SCREEN_HEIGHT {
        return None;
    }
    if impact
        && matches!(view, DrawView::Normal(_))
        && !(IMPACT_LOW..IMPACT_HIGH).contains(&screen.0.x.cast_signed())
    {
        return None;
    }
    Some((
        PictureIntent {
            slot,
            picture: picture.clone(),
            screen,
        },
        impact,
    ))
}

#[cfg(test)]
mod tests;
