//! Strict adapter for supplied vectors. `update` compares complete state/events/contacts;
//! other operations intentionally observe only the named isolated cut and expected fields.
//! Invented content is valid even when a stage does not read all its clips.
use super::*;
use enum_map::{Enum, EnumMap};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use toml::Value;
macro_rules! object {
    ($($key:literal : $v:expr),* $(,)?) => { Value::Table([$(($key.to_owned(), value($v))),*].into_iter().collect()) };
}
const TEST_RADIUS: [u8; 2] = [3, 2];
const TEST_POSITION: [u32; 2] = [6_553_600, 13_107_200];
const TEST_DAMAGE: u16 = 12;
const TEST_ROOM: [u16; 2] = [4, 4];
const TEST_SLOTS: usize = 5;
const SUPPLIED_CASES: usize = 81;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Suite {
    vector: Vec<Case>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Case {
    id: String,
    source: String,
    entries: Vec<String>,
    operation: String,
    before: Value,
    after: Value,
    rules: Option<Overrides>,
    scenario: Option<String>,
    step: Option<u16>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Overrides {
    acceleration: Option<i16>,
    trail_interval: Option<u16>,
}
struct Fields {
    values: toml::Table,
}
impl Fields {
    fn new(value: Value) -> Result<Self, String> {
        if let Value::Table(values) = value {
            Ok(Self { values })
        } else {
            Err("expected table".into())
        }
    }
    fn take<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>, String> {
        self.values
            .remove(key)
            .map(|value| {
                value
                    .try_into()
                    .map_err(|e: toml::de::Error| format!("{key}: {e}"))
            })
            .transpose()
    }
    fn required<T: serde::de::DeserializeOwned>(&mut self, key: &str) -> Result<T, String> {
        self.take(key)?.ok_or_else(|| format!("missing {key}"))
    }
    fn or<T: serde::de::DeserializeOwned>(&mut self, key: &str, fallback: T) -> Result<T, String> {
        Ok(self.take(key)?.unwrap_or(fallback))
    }
    fn finish(self) -> Result<(), String> {
        if self.values.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "unknown keys {:?}",
                self.values.keys().collect::<Vec<_>>()
            ))
        }
    }
}
fn value<T: Serialize>(v: T) -> Value {
    Value::try_from(v).expect("test observations must serialize")
}
fn axes<T>([x, y]: [T; 2]) -> Axes<T> {
    Axes { x, y }
}
fn pair<T>(a: Axes<T>) -> [T; 2] {
    [a.x, a.y]
}
fn slot(index: usize) -> Result<ShotSlot, String> {
    [
        ShotSlot::Slot0,
        ShotSlot::Slot1,
        ShotSlot::Slot2,
        ShotSlot::Slot3,
        ShotSlot::Slot4,
    ]
    .get(index)
    .copied()
    .ok_or_else(|| "invalid slot".into())
}
fn heading(name: &str) -> Result<Heading, String> {
    value(name)
        .try_into()
        .map_err(|e: toml::de::Error| e.to_string())
}
fn aim(name: &str) -> Result<MuzzleAim, String> {
    match name {
        "Forbidden" => Ok(MuzzleAim::Forbidden),
        "PreviousAim" => Ok(MuzzleAim::PreviousAim),
        _ => Ok(MuzzleAim::Aimed(heading(name)?)),
    }
}
fn heading_name(h: Heading) -> String {
    format!("{h:?}")
}
fn optional_picture(name: String) -> Option<PictureId> {
    if name == "None" {
        None
    } else {
        Some(PictureId(name))
    }
}
fn optional_appearance(name: String) -> Option<TrailAppearanceId> {
    if name == "None" || name == "Inactive" {
        None
    } else {
        Some(TrailAppearanceId(name))
    }
}
fn radius(v: [u8; 2]) -> Axes<Radius> {
    axes(v.map(Radius))
}
fn offset([x, y]: [i64; 2]) -> Offset {
    Offset { x, y }
}
fn offsets(f: &mut Fields, key: &str, flat: Offset) -> Result<EnumMap<Heading, Offset>, String> {
    let Some(map) = f.take::<BTreeMap<String, [i64; 2]>>(key)? else {
        return Ok(EnumMap::from_fn(|_| flat));
    };
    let mut output = EnumMap::from_fn(|_| flat);
    let mut seen = EnumMap::<Heading, bool>::default();
    for (name, v) in map {
        let h = heading(&name)?;
        output[h] = offset(v);
        seen[h] = true;
    }
    if seen.values().all(|v| *v) {
        Ok(output)
    } else {
        Err("incomplete offset map".into())
    }
}
fn cannon(f: &mut Fields) -> Result<CannonInput, String> {
    let current = aim(&f.or("aim", "Right".to_owned())?)?;
    let previous_muzzle = aim(&f.or("previous", "Forbidden".to_owned())?)?;
    let pending_aim = f
        .take::<String>("pending")?
        .map(|s| heading(&s))
        .transpose()?;
    let flat = offset(f.or("offset", [0, 0])?);
    let moving_flat = offset(f.or("moving_offset", [0, 0])?);
    Ok(CannonInput {
        shoot: f.or("shoot", false)?,
        new_press: f.or("new", false)?,
        previous_new_press: f.or("previous_new", false)?,
        muzzle: Muzzle {
            aim: current,
            lift: f.or("lift", 0)?,
            stationary: offsets(f, "offsets", flat)?,
            moving: offsets(f, "moving_offsets", moving_flat)?,
        },
        previous_muzzle,
        pending_aim,
        moving_muzzle: f.or("moving", false)?,
        centre: SamusCentre(axes(f.or("centre", [0, 0])?)),
    })
}
fn inheritance(f: &mut Fields) -> Result<Inheritance, String> {
    let [left, right, up, down]: [u16; 4] = f.or("whole", [0, 0, 0, 0])?;
    let [lf, rf, uf, df]: [u16; 4] = f.or("fraction", [0, 0, 0, 0])?;
    Ok(Inheritance {
        left: MovementDistance {
            whole: left,
            fraction: PixelFraction(lf),
        },
        right: MovementDistance {
            whole: right,
            fraction: PixelFraction(rf),
        },
        up: MovementDistance {
            whole: up,
            fraction: PixelFraction(uf),
        },
        down: MovementDistance {
            whole: down,
            fraction: PixelFraction(df),
        },
        camera_vertical_fraction: PixelFraction(f.or("camera_fraction", 0)?),
    })
}
fn frame(v: Value) -> Result<Frame, String> {
    let mut f = Fields::new(v)?;
    let result = Frame {
        hold: FrameHold::try_from(f.required::<u16>("hold")?).map_err(str::to_owned)?,
        picture: PictureId(f.required("picture")?),
        radius: radius(f.required("radius")?),
        trail_appearance: optional_appearance(f.required("appearance")?),
    };
    f.finish()?;
    Ok(result)
}
fn clip(f: &mut Fields) -> Result<Clip, String> {
    let frames = f
        .required::<Vec<Value>>("frames")?
        .into_iter()
        .map(frame)
        .collect::<Result<Vec<_>, _>>()?;
    if frames.is_empty() {
        return Err("empty clip".into());
    }
    let ending = f.required::<String>("ending")?;
    let repeat = f.or("repeat", 0usize)?;
    let ending = match ending.as_str() {
        "Remove" => ClipEnding::Remove,
        "RepeatFrom" => ClipEnding::RepeatFrom(
            ClipFrame::new(repeat, frames.len().saturating_sub(1)).ok_or("invalid repeat")?,
        ),
        _ => return Err("unknown clip ending".into()),
    };
    Ok(Clip { frames, ending })
}
fn content(v: Value) -> Result<PowerBeam, String> {
    let mut f = Fields::new(v)?;
    let damage = Damage::try_from(f.required::<u16>("damage")?).map_err(str::to_owned)?;
    let mut flying_clips = EnumMap::from_fn(|_| ClipId("Power".into()));
    if let Some(map) = f.take::<BTreeMap<String, String>>("heading_clips")? {
        let mut seen = EnumMap::<Heading, bool>::default();
        for (h, c) in map {
            let h = heading(&h)?;
            seen[h] = true;
            flying_clips[h] = ClipId(c);
        }
        if !seen.values().all(|v| *v) {
            return Err("incomplete clip map".into());
        }
    }
    let mut clips = BTreeMap::new();
    for (id, v) in f.required::<BTreeMap<String, Value>>("clips")? {
        let mut fields = Fields::new(v)?;
        clips.insert(ClipId(id), clip(&mut fields)?);
        fields.finish()?;
    }
    f.finish()?;
    let result = PowerBeam {
        damage,
        flying_clips,
        impact_clip: ClipId("Impact".into()),
        clips,
    };
    validate_content(&result)?;
    Ok(result)
}
// Test importer validation only; no production validation interface is introduced.
fn validate_content(content: &PowerBeam) -> Result<(), String> {
    for id in content
        .flying_clips
        .values()
        .chain(std::iter::once(&content.impact_clip))
    {
        if !content.clips.contains_key(id) {
            return Err(format!("missing clip {}", id.0));
        }
    }
    for clip in content.clips.values() {
        if clip.frames.is_empty() {
            return Err("empty clip".into());
        }
        if let ClipEnding::RepeatFrom(index) = clip.ending
            && index.index() >= clip.frames.len()
        {
            return Err("invalid repeat".into());
        }
    }
    Ok(())
}
fn invented_content(r: Axes<Radius>) -> PowerBeam {
    let frame = Frame {
        hold: FrameHold(1),
        picture: PictureId("Spark".into()),
        radius: r,
        trail_appearance: None,
    };
    let clip = Clip {
        frames: vec![frame],
        ending: ClipEnding::Remove,
    };
    PowerBeam {
        damage: Damage(TEST_DAMAGE),
        flying_clips: EnumMap::from_fn(|_| ClipId("Power".into())),
        impact_clip: ClipId("Impact".into()),
        clips: [
            (ClipId("Power".into()), clip.clone()),
            (ClipId("Impact".into()), clip),
        ]
        .into_iter()
        .collect(),
    }
}
fn body(f: &mut Fields) -> Result<ShotBody, String> {
    let mut body = ShotBody {
        position: axes(
            f.or::<[u32; 2]>("position", TEST_POSITION)?
                .map(FixedPosition),
        ),
        heading: heading(&f.or("heading", "Right".to_owned())?)?,
        speed: axes(
            f.or::<[u16; 2]>("speed", [0, 0])?
                .map(|v| BeamSpeed(v.cast_signed())),
        ),
        radius: radius(f.or("radius", TEST_RADIUS)?),
        damage: Damage::try_from(f.or::<u16>("damage", TEST_DAMAGE)?).map_err(str::to_owned)?,
        trail: FrameWord(f.or("trail", 0)?),
        clip: ClipId(f.or("clip", "Power".to_owned())?),
        next_frame: ClipFrame::at(f.or("next", 0)?),
        remaining_hold: FrameWord(f.or("remaining", 1)?),
        picture: optional_picture(f.or("picture", "None".to_owned())?),
        trail_appearance: optional_appearance(f.or("appearance", "None".to_owned())?),
    };
    if let Some(centre) = f.take::<[u16; 2]>("centre")? {
        body.position = axes(centre.map(|v| FixedPosition::from_parts(v, PixelFraction(0))));
    }
    Ok(body)
}
fn shot(v: Value) -> Result<ShotState, String> {
    let mut f = Fields::new(v)?;
    let phase = f.required::<String>("phase")?;
    let result = if phase == "Free" {
        ShotState::Free {
            trail: FrameWord(f.or("trail", 0)?),
            retained_fraction: axes(
                f.or::<[u16; 2]>("retained_fraction", [0, 0])?
                    .map(PixelFraction),
            ),
        }
    } else {
        let body = body(&mut f)?;
        match phase.as_str() {
            "Flying" => ShotState::Flying {
                body,
                consumed: f.or("consumed", false)?,
            },
            "Impact" => ShotState::Impact { body },
            _ => return Err("unknown phase".into()),
        }
    };
    f.finish()?;
    Ok(result)
}
fn state(v: Value) -> Result<State, String> {
    let mut f = Fields::new(v)?;
    let mut s = empty_state();
    s.cooldown = FrameWord(f.required("cooldown")?);
    s.count = ShotCount(f.required("count")?);
    let slots = f.required::<Vec<Value>>("slots")?;
    if slots.len() != TEST_SLOTS {
        return Err("expected five slots".into());
    }
    for (i, v) in slots.into_iter().enumerate() {
        s.slots[slot(i)?] = shot(v)?;
    }
    f.finish()?;
    Ok(s)
}
fn empty_state() -> State {
    State::reset(EnumMap::from_fn(|_| {
        axes([PixelFraction(0), PixelFraction(0)])
    }))
}
fn validate_state(s: &State, content: &PowerBeam) -> Result<(), String> {
    for shot in s.slots.values() {
        if let Some(body) = shot.body() {
            let clip = content.clips.get(&body.clip).ok_or("missing body clip")?;
            if body.next_frame.index() > clip.frames.len() {
                return Err("invalid restored next frame".into());
            }
        }
    }
    Ok(())
}
fn slot_number(s: ShotSlot) -> usize {
    s.into_usize()
}
fn phase(s: &ShotState) -> &'static str {
    match s {
        ShotState::Free { .. } => "Free",
        ShotState::Flying { .. } => "Flying",
        ShotState::Impact { .. } => "Impact",
    }
}
fn body_value(b: &ShotBody) -> toml::Table {
    let Value::Table(v) = object!("heading":heading_name(b.heading),"position":pair(b.position).map(|v|v.0),"speed":pair(b.speed).map(|v|v.0.cast_unsigned()),"radius":pair(b.radius).map(|v|v.0),"damage":b.damage.value(),"trail":b.trail.0,"clip":&b.clip.0,"remaining":b.remaining_hold.0,"next":b.next_frame.index(),"picture":b.picture.as_ref().map_or("None",|v|v.0.as_str()),"appearance":b.trail_appearance.as_ref().map_or("None",|v|v.0.as_str()))
    else {
        panic!("object macro must create tables");
    };
    v
}
fn shot_value(s: &ShotState) -> Value {
    let mut v = s.body().map_or_else(toml::Table::new, body_value);
    v.insert("phase".into(), value(phase(s)));
    match s {
        ShotState::Free {
            trail,
            retained_fraction,
        } => {
            v.insert("trail".into(), value(trail.0));
            v.insert(
                "retained_fraction".into(),
                value(pair(*retained_fraction).map(|v| v.0)),
            );
        }
        ShotState::Flying { consumed, .. } => {
            v.insert("consumed".into(), value(consumed));
        }
        ShotState::Impact { .. } => {}
    }
    Value::Table(v)
}
fn state_value(s: &State) -> Value {
    object!("cooldown":s.cooldown.0,"count":s.count.0,"slots":s.slots.values().map(shot_value).collect::<Vec<_>>())
}
fn event_value(e: &Event) -> Value {
    match e {
        Event::FireSound(s) => object!("kind":"FireSound","slot":slot_number(*s)),
        Event::Protection(n) => object!("kind":"Protection","updates":n.0),
        Event::PendingAimConsumed => object!("kind":"PendingAimConsumed"),
        Event::ImpactSound(s) => object!("kind":"ImpactSound","slot":slot_number(*s)),
        Event::Trail {
            slot,
            position,
            picture,
            appearance,
        } => {
            object!("kind":"Trail","slot":slot_number(*slot),"position":pair(*position).map(|v|v.0),"picture":picture.as_ref().map_or("None",|v|v.0.as_str()),"appearance":appearance.as_ref().map_or("None",|v|v.0.as_str()))
        }
    }
}
struct ScriptedWorld {
    responses: Vec<Vec<bool>>,
    visits: Vec<(Contact, Vec<u16>)>,
}
impl ScriptedWorld {
    fn new(responses: Vec<Vec<bool>>) -> Self {
        Self {
            responses,
            visits: vec![],
        }
    }
}
impl World for ScriptedWorld {
    fn contact_cell(&mut self, contact: &Contact, cell: u16) -> ContactResponse {
        if cell == 0 {
            self.visits.push((contact.clone(), vec![]));
        }
        let index = self.visits.len().saturating_sub(1);
        if let Some((_, visited)) = self.visits.last_mut() {
            visited.push(cell);
        }
        if self
            .responses
            .get(index)
            .and_then(|r| r.get(usize::from(cell)))
            .copied()
            .unwrap_or(false)
        {
            ContactResponse::Blocks
        } else {
            ContactResponse::Passes
        }
    }
}
fn contact_value((c, visited): &(Contact, Vec<u16>)) -> Value {
    object!("slot":slot_number(c.slot),"heading":heading_name(c.heading),"axis":format!("{:?}",c.axis),"centre":pair(c.centre),"radius":pair(c.radius).map(|v|v.0),"leading":c.leading_boundary,"start":c.perpendicular_start,"count":c.cell_count,"visited":visited)
}
fn input(f: &mut Fields) -> Result<UpdateInput, String> {
    let mode = match f.or("mode", "CannonAndShots".to_owned())?.as_str() {
        "CannonAndShots" => UpdateMode::CannonAndShots,
        "ShotsOnly" => UpdateMode::ShotsOnly,
        _ => return Err("unknown mode".into()),
    };
    Ok(UpdateInput {
        mode,
        time_stopped: f.or("frozen", false)?,
        cannon: cannon(f)?,
        inheritance: inheritance(f)?,
        camera: ScreenOrigin(axes(f.or("camera", [0, 0])?)),
        room: RoomScreens(axes(f.or("room_screens", TEST_ROOM)?)),
    })
}
fn complete(f: &mut Fields, rules: Rules) -> Result<Value, String> {
    let mut s = state(f.required("state")?)?;
    let mut fields = Fields::new(f.required("input")?)?;
    let consumed = fields.or::<Vec<usize>>("consumed_slots", vec![])?;
    let i = input(&mut fields)?;
    fields.finish()?;
    let c = content(f.required("content")?)?;
    validate_state(&s, &c)?;
    let mut world = ScriptedWorld::new(f.or("world", vec![])?);
    let events = update(&mut s, &i, &c, rules, &mut world);
    for index in consumed {
        s.consume(slot(index)?);
    }
    let restored: State = value(&s)
        .try_into()
        .map_err(|e: toml::de::Error| e.to_string())?;
    assert_eq!(s, restored, "complete State serde roundtrip");
    Ok(
        object!("state":state_value(&s),"events":events.iter().map(event_value).collect::<Vec<_>>(),"contacts":world.visits.iter().map(contact_value).collect::<Vec<_>>()),
    )
}
fn spawning(f: &mut Fields) -> Result<Value, String> {
    let cinput = cannon(f)?;
    let mut state = empty_state();
    state.cooldown = FrameWord(f.required("cooldown")?);
    state.count = ShotCount(f.required("count")?);
    let damages = f.required::<[u16; 5]>("damages")?;
    let mut content = invented_content(radius(f.required("radius")?));
    content.damage = Damage::try_from(f.required::<u16>("damage")?).map_err(str::to_owned)?;
    for (i, d) in damages.into_iter().enumerate() {
        if d != 0 {
            let mut fields = Fields::new(object!("damage":d))?;
            state.slots[slot(i)?] = ShotState::Flying {
                body: body(&mut fields)?,
                consumed: false,
            };
            fields.finish()?;
        }
    }
    update_spacing(&mut state.cooldown, UpdateMode::CannonAndShots, false);
    let mut events = vec![];
    let spawned = prepare_shot(&mut state, &cinput, &content, &mut events);
    let mut out = Fields::new(
        object!("spawned":spawned.is_some(),"cooldown":state.cooldown.0,"count":state.count.0),
    )?
    .values;
    if let Some(s) = spawned
        && let Some(b) = state.slots[s].body()
    {
        let protection = events
            .iter()
            .find_map(|e| {
                if let Event::Protection(n) = e {
                    Some(n.0)
                } else {
                    None
                }
            })
            .ok_or("missing protection request")?;
        out.extend(Fields::new(object!("slot":slot_number(s),"heading":heading_name(b.heading),"centre":[b.position.x.integer(),b.position.y.integer()],"pending_used":events.contains(&Event::PendingAimConsumed),"trail":b.trail.0,"damage":b.damage.value(),"remaining":b.remaining_hold.0,"next":b.next_frame.index(),"radius":pair(b.radius).map(|v|v.0),"fire_sound":events.contains(&Event::FireSound(s)),"invincibility":protection))?.values);
    }
    Ok(Value::Table(out))
}
fn origin(f: &mut Fields) -> Result<Value, String> {
    let input = cannon(f)?;
    // This isolated cut reports aim resolution, not a successful firing event.
    Ok(cannon_origin(&input).map_or_else(||object!("valid":false,"pending_used":false),|(h,c)|object!("valid":true,"pending_used":input.pending_aim.is_some(),"heading":heading_name(h),"centre":pair(c.0))))
}
fn drift(f: &mut Fields) -> Result<Value, String> {
    let h = heading(&f.required::<String>("heading")?)?;
    let speed = inherited_motion(h, inheritance(f)?);
    Ok(object!("speed":pair(speed).map(|v|v.0.cast_unsigned())))
}
fn flight(f: &mut Fields, rules: Rules) -> Result<Value, String> {
    let mut b = body(f)?;
    accelerate(&mut b, rules);
    let aim = b.heading.axes();
    if aim.x != 0 {
        b.position.x.move_by(b.speed.x);
    }
    if aim.y != 0 {
        b.position.y.move_by(b.speed.y);
    }
    Ok(
        object!("position":pair(b.position).map(|v|v.0),"speed":pair(b.speed).map(|v|v.0.cast_unsigned())),
    )
}
fn probe(f: &mut Fields) -> Result<Value, String> {
    let blocked = f.or("blocked", false)?;
    let mut b = body(f)?;
    b.radius = radius([0, 0]);
    let mut shot = ShotState::Flying {
        body: b,
        consumed: false,
    };
    let mut fields = Fields::new(object!())?;
    let input = input(&mut fields)?;
    fields.finish()?;
    let mut world = ScriptedWorld::new(vec![vec![blocked; 16], vec![blocked; 16]]);
    let content = invented_content(radius([0, 0]));
    muzzle_contact(
        &mut shot,
        ShotSlot::Slot0,
        &input,
        &content,
        &mut world,
        &mut vec![],
    );
    let b = shot.body().ok_or("probe lost body")?;
    Ok(
        object!("position":pair(b.position).map(|v|v.0),"speed":pair(b.speed).map(|v|v.0.cast_unsigned()),"contact_axes":world.visits.iter().map(|(c,_)|format!("{:?}",c.axis)).collect::<Vec<_>>()),
    )
}
fn contact(f: &mut Fields) -> Result<Value, String> {
    let axis = match f.required::<String>("axis")?.as_str() {
        "Horizontal" => Axis::Horizontal,
        "Vertical" => Axis::Vertical,
        _ => return Err("unknown axis".into()),
    };
    let along = f.required::<u16>("centre")?;
    let perpendicular = f.required("perpendicular")?;
    let r = Radius(f.required("radius")?);
    let across = Radius(f.required("perpendicular_radius")?);
    let speed = BeamSpeed(f.required::<u16>("speed")?.cast_signed());
    let screens = f.required("screens")?;
    let responses = f.or::<Vec<bool>>("responses", vec![])?;
    let (centre, radius) = match axis {
        Axis::Horizontal => (
            Axes {
                x: along,
                y: perpendicular,
            },
            Axes { x: r, y: across },
        ),
        Axis::Vertical => (
            Axes {
                x: perpendicular,
                y: along,
            },
            Axes { x: across, y: r },
        ),
    };
    let edge = edge_geometry(centre, radius, speed, axis, screens);
    let mut fields = Fields::new(object!())?;
    let mut b = body(&mut fields)?;
    fields.finish()?;
    b.position = Axes {
        x: FixedPosition::from_parts(centre.x, PixelFraction(0)),
        y: FixedPosition::from_parts(centre.y, PixelFraction(0)),
    };
    b.radius = radius;
    b.speed = Axes { x: speed, y: speed };
    let mut world = ScriptedWorld::new(vec![responses]);
    let blocked = contact_edge(
        &b,
        ShotSlot::Slot0,
        axis,
        RoomScreens(axes([screens, screens])),
        &mut world,
    );
    let visited = world
        .visits
        .first()
        .map_or_else(Vec::new, |(_, v)| v.clone());
    Ok(
        object!("leading":edge.leading,"start":edge.start,"span":edge.span_minus_one,"visited":visited,"blocked":blocked),
    )
}
fn bursting(f: &mut Fields) -> Result<Value, String> {
    let mut shot = ShotState::Flying {
        body: body(f)?,
        consumed: false,
    };
    let mut events = vec![];
    burst(
        &mut shot,
        ShotSlot::Slot0,
        &ClipId("Impact".into()),
        &mut events,
    );
    let mut out = body_value(shot.body().ok_or("burst lost body")?);
    out.insert("phase".into(), value(phase(&shot)));
    out.insert(
        "impact_sound".into(),
        value(events.contains(&Event::ImpactSound(ShotSlot::Slot0))),
    );
    Ok(Value::Table(out))
}
fn diagonal(f: &mut Fields, rules: Rules) -> Result<Value, String> {
    let blocked = f.required("stopped_horizontally")?;
    let mut b = body(f)?;
    accelerate(&mut b, rules);
    let mut shot = ShotState::Flying {
        body: b,
        consumed: false,
    };
    let mut world = ScriptedWorld::new(vec![vec![blocked; 16], vec![false; 16]]);
    let c = invented_content(radius(TEST_RADIUS));
    let mut events = vec![];
    motion_axis(
        &mut shot,
        ShotSlot::Slot0,
        Axis::Horizontal,
        &c,
        RoomScreens(axes(TEST_ROOM)),
        &mut world,
        &mut events,
    );
    motion_axis(
        &mut shot,
        ShotSlot::Slot0,
        Axis::Vertical,
        &c,
        RoomScreens(axes(TEST_ROOM)),
        &mut world,
        &mut events,
    );
    let mut out = body_value(shot.body().ok_or("diagonal lost body")?);
    out.insert("phase".into(), value(phase(&shot)));
    out.insert(
        "impact_sound".into(),
        value(events.contains(&Event::ImpactSound(ShotSlot::Slot0))),
    );
    out.insert(
        "vertical_checked".into(),
        value(world.visits.iter().any(|(c, _)| c.axis == Axis::Vertical)),
    );
    Ok(Value::Table(out))
}
fn trail(f: &mut Fields, rules: Rules) -> Result<Value, String> {
    let mut b = body(f)?;
    let mut events = vec![];
    update_trail(&mut b, ShotSlot::Slot0, rules, &mut events);
    Ok(object!("trail":b.trail.0,"emitted":!events.is_empty()))
}
fn ending(f: &mut Fields, consume: bool) -> Result<Value, String> {
    let p = f.required::<String>("phase")?;
    let spent = f.or("consumed", false)?;
    let count = f.or("count", 1)?;
    let trail = f.or("trail", 0)?;
    let mut fields = Fields::new(object!("phase":&p,"consumed":spent,"trail":trail))?;
    if p != "Flying" {
        fields.values.remove("consumed");
    }
    let mut s = empty_state();
    s.count = ShotCount(count);
    s.slots[ShotSlot::Slot0] = shot(Value::Table(fields.values))?;
    if consume {
        s.consume(ShotSlot::Slot0);
    } else {
        let mut fields = Fields::new(object!("mode":"ShotsOnly"))?;
        let input = input(&mut fields)?;
        fields.finish()?;
        let _events = update(
            &mut s,
            &input,
            &invented_content(radius(TEST_RADIUS)),
            Rules::REFERENCE,
            &mut ScriptedWorld::new(vec![]),
        );
    }
    let current = &s.slots[ShotSlot::Slot0];
    let spent = matches!(current, ShotState::Flying { consumed: true, .. });
    let actual_trail = match current {
        ShotState::Free { trail, .. } => trail.0,
        ShotState::Flying { body, .. } | ShotState::Impact { body } => body.trail.0,
    };
    Ok(object!("phase":phase(current),"consumed":spent,"count":s.count.0,"trail":actual_trail))
}
// Isolated clip cut: the fixture's `removed` means update_clip requested removal, not that
// State has been freed. Complete updates assert the resulting Free slot and count as well.
fn animation(f: &mut Fields) -> Result<Value, String> {
    let mut b = body(f)?;
    let c = clip(f)?;
    if b.next_frame.index() > c.frames.len() {
        return Err("invalid next frame".into());
    }
    let mut content = invented_content(b.radius);
    content.clips.insert(b.clip.clone(), c);
    validate_content(&content)?;
    let removed = update_clip(&mut b, &content);
    let mut out = body_value(&b);
    out.insert("removed".into(), value(removed));
    Ok(Value::Table(out))
}
// Intentionally partial mode observation, measured by an ordinary update of a live shot.
// The fixture checks movement occurred and no firing request occurred, not full state.
fn mode_observation(f: &mut Fields, rules: Rules) -> Result<Value, String> {
    let mut s = empty_state();
    s.cooldown = FrameWord(f.required("cooldown")?);
    s.count = ShotCount(1);
    let mut fields = Fields::new(object!("remaining":2,"trail":2,"speed":[1024,0]))?;
    s.slots[ShotSlot::Slot0] = ShotState::Flying {
        body: body(&mut fields)?,
        consumed: false,
    };
    fields.finish()?;
    let before = s.slots[ShotSlot::Slot0].clone();
    let mut fields = Fields::new(
        object!("mode":"ShotsOnly","shoot":true,"frozen":f.required::<bool>("frozen")?),
    )?;
    let i = input(&mut fields)?;
    fields.finish()?;
    let mut world = ScriptedWorld::new(vec![]);
    let events = update(
        &mut s,
        &i,
        &invented_content(radius(TEST_RADIUS)),
        rules,
        &mut world,
    );
    Ok(
        object!("cooldown":s.cooldown.0,"shots_updated":s.slots[ShotSlot::Slot0]!=before,"shoot":events.iter().any(|e|matches!(e,Event::FireSound(_)))),
    )
}
fn observe(case: &Case, rules: Rules) -> Result<Value, String> {
    let mut f = Fields::new(case.before.clone())?;
    let result = match case.operation.as_str() {
        "spacing" => {
            let cooldown = f.required::<u16>("cooldown")?;
            let frozen = f.required::<bool>("frozen")?;
            let mut s = empty_state();
            s.cooldown = FrameWord(cooldown);
            let mut fields = Fields::new(object!("frozen":frozen))?;
            let i = input(&mut fields)?;
            fields.finish()?;
            let _events = update(
                &mut s,
                &i,
                &invented_content(radius(TEST_RADIUS)),
                rules,
                &mut ScriptedWorld::new(vec![]),
            );
            object!("cooldown":s.cooldown.0)
        }
        "mode" => mode_observation(&mut f, rules)?,
        "spawn" => spawning(&mut f)?,
        "origin" => origin(&mut f)?,
        "drift" => drift(&mut f)?,
        "flight" => flight(&mut f, rules)?,
        "probe" => probe(&mut f)?,
        "burst" => bursting(&mut f)?,
        "contact" => contact(&mut f)?,
        "diagonal" => diagonal(&mut f, rules)?,
        "trail" => trail(&mut f, rules)?,
        "view" => {
            let centre = axes(f.required("centre")?);
            let camera = ScreenOrigin(axes(f.required("camera")?));
            object!("visible":within_lifetime_view(centre,camera))
        }
        "end" => ending(&mut f, false)?,
        "consume" => ending(&mut f, true)?,
        "clip" => animation(&mut f)?,
        "update" => complete(&mut f, rules)?,
        "draw" => draw(&mut f)?,
        _ => return Err(format!("unknown operation {}", case.operation)),
    };
    f.finish()?;
    Ok(result)
}
// Partial comparison is only for explicitly isolated observations in the supplied fixtures.
fn compare_partial(expected: &Value, actual: &Value, path: &str) -> Result<(), String> {
    match (expected, actual) {
        (Value::Table(e), Value::Table(a)) => {
            for (k, v) in e {
                let actual = a
                    .get(k)
                    .ok_or_else(|| format!("unknown expected key {path}.{k}"))?;
                compare_partial(v, actual, &format!("{path}.{k}"))?;
            }
            Ok(())
        }
        (Value::Array(e), Value::Array(a)) => {
            if e.len() != a.len() {
                return Err(format!(
                    "{path} expected {} items actual {}",
                    e.len(),
                    a.len()
                ));
            }
            for (i, (e, a)) in e.iter().zip(a).enumerate() {
                compare_partial(e, a, &format!("{path}[{i}]"))?;
            }
            Ok(())
        }
        _ => {
            if expected == actual {
                Ok(())
            } else {
                Err(format!("{path} expected {expected} actual {actual}"))
            }
        }
    }
}
fn compare_complete(expected: &Value, actual: &Value, path: &str) -> Result<(), String> {
    compare_partial(expected, actual, path)?;
    compare_partial(actual, expected, path)
}
#[test]
fn supplied_vectors_and_catalogue() -> Result<(), String> {
    let suite: Suite = toml::from_str(include_str!(
        "../../../../scenarios/vectors/weapons-projectiles.toml"
    ))
    .map_err(|e| e.to_string())?;
    assert_eq!(suite.vector.len(), SUPPLIED_CASES);
    let catalogue = catalogue();
    let mut seen = EnumMap::<BehaviourId, bool>::default();
    let mut failures = vec![];
    let mut scenarios = BTreeMap::<String, (u16, Value)>::new();
    for case in &suite.vector {
        if case.source != "constructed" {
            return Err(format!("unknown source {}", case.source));
        }
        for name in &case.entries {
            let id = catalogue
                .iter()
                .find(|(_, b)| format!("{:?}", b.id) == *name)
                .map(|(id, _)| id)
                .ok_or_else(|| format!("unknown entry {name}"))?;
            seen[id] = true;
        }
        let mut rules = Rules::REFERENCE;
        if let Some(o) = &case.rules {
            if let Some(a) = o.acceleration {
                rules.acceleration = BeamSpeed(a);
            }
            if let Some(t) = o.trail_interval {
                rules.trail_interval = FrameWord(t);
            }
        }
        if let (Some(scenario), Some(step)) = (&case.scenario, case.step)
            && let Some((last_step, last_state)) = scenarios.get(scenario)
        {
            assert_eq!(step, last_step.wrapping_add(1));
            let before = case.before.get("state").ok_or("missing linked state")?;
            compare_complete(before, last_state, &case.id)?;
        }
        match observe(case, rules) {
            Ok(actual) => {
                let comparison = if case.operation == "update" || case.operation == "draw" {
                    compare_complete(&case.after, &actual, &case.id)
                } else {
                    compare_partial(&case.after, &actual, &case.id)
                };
                if let Err(e) = comparison {
                    failures.push(format!("{e}; entries {:?}", case.entries));
                }
                if let (Some(scenario), Some(step), Some(s)) =
                    (&case.scenario, case.step, actual.get("state"))
                {
                    scenarios.insert(scenario.clone(), (step, s.clone()));
                }
            }
            Err(e) => failures.push(format!("{} adapter {e}", case.id)),
        }
    }
    assert!(
        seen.values().all(|v| *v),
        "every catalogue entry has vector evidence"
    );
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n"))
    }
}
fn draw(f: &mut Fields) -> Result<Value, String> {
    let flicker = match f.required::<String>("flicker")?.as_str() {
        "Even" => FlickerPhase::Even,
        "Odd" => FlickerPhase::Odd,
        _ => return Err("unknown phase".into()),
    };
    let origin = ScreenOrigin(axes(f.required("view_origin")?));
    let view_name = f.or("view", "Normal".to_owned())?;
    let mut s = empty_state();
    for v in f.required::<Vec<Value>>("slots")? {
        let mut fields = Fields::new(v)?;
        let slot = slot(fields.required("slot")?)?;
        let phase = fields.required::<String>("phase")?;
        let b = body(&mut fields)?;
        let consumed = fields.or("consumed", false)?;
        // Legacy draw observations describe a Free candidate with body/picture fields;
        // those are parsed as known fixture keys, but no such body is installed in State.
        // Impact likewise has no consumed flag; its fixture flag describes an ignored hit.
        s.slots[slot] = match phase.as_str() {
            "Free" => ShotState::Free {
                trail: FrameWord(0),
                retained_fraction: axes([PixelFraction(0), PixelFraction(0)]),
            },
            "Flying" => ShotState::Flying { body: b, consumed },
            "Impact" => ShotState::Impact { body: b },
            _ => return Err("unknown phase".into()),
        };
        fields.finish()?;
    }
    let view = match view_name.as_str() {
        "Normal" => DrawView::Normal(origin),
        "Rotating" => {
            let mut centres = EnumMap::from_fn(|_| ScreenCentre(axes([0, 0])));
            let mut seen = EnumMap::<ShotSlot, bool>::default();
            for (k, v) in f.required::<BTreeMap<String, [u16; 2]>>("screen_centres")? {
                let slot = slot(k.parse::<usize>().map_err(|e| e.to_string())?)?;
                centres[slot] = ScreenCentre(axes(v));
                seen[slot] = true;
            }
            if s.slots
                .iter()
                .any(|(slot, shot)| shot.body().is_some() && !seen[slot])
            {
                return Err("missing transformed centre".into());
            }
            DrawView::Rotating(centres)
        }
        _ => return Err("unknown view".into()),
    };
    let pictures = pictures(&s, flicker, &view);
    let intent = |p: PictureIntent| object!("slot":slot_number(p.slot),"picture":p.picture.0,"screen":pair(p.screen.0));
    Ok(
        object!("flying":pictures.flying.into_iter().map(intent).collect::<Vec<_>>(),"impact":pictures.impact.into_iter().map(intent).collect::<Vec<_>>()),
    )
}
#[test]
fn fresh_muzzle_impact_outside_view() -> Result<(), String> {
    const FAR_CAMERA: [u16; 2] = [500, 500];
    const TEST_CENTRE: [u16; 2] = [100, 200];
    let mut fields =
        Fields::new(object!("shoot": true, "centre": TEST_CENTRE, "camera": FAR_CAMERA))?;
    let input = input(&mut fields)?;
    fields.finish()?;
    let content = invented_content(radius(TEST_RADIUS));
    let mut state = empty_state();
    let mut world = ScriptedWorld::new(vec![vec![true]]);
    let events = update(&mut state, &input, &content, Rules::REFERENCE, &mut world);
    let mut expected = empty_state();
    expected.cooldown = FrameWord(HELD_SPACING);
    expected.slots[ShotSlot::Slot0] = ShotState::Free {
        trail: FrameWord(INITIAL_TRAIL),
        retained_fraction: axes([PixelFraction(0), PixelFraction(0)]),
    };
    assert_eq!(state, expected);
    assert_eq!(
        events,
        vec![
            Event::FireSound(ShotSlot::Slot0),
            Event::Protection(FrameWord(PROTECTION)),
            Event::ImpactSound(ShotSlot::Slot0)
        ]
    );
    let contact = Contact {
        slot: ShotSlot::Slot0,
        heading: Heading::Right,
        axis: Axis::Horizontal,
        centre: axes(TEST_CENTRE),
        radius: radius(TEST_RADIUS),
        leading_boundary: 102,
        perpendicular_start: 198,
        cell_count: 1,
    };
    assert_eq!(world.visits, vec![(contact, vec![0])]);
    Ok(())
}
#[test]
fn unusual_restored_state_and_content() -> Result<(), String> {
    const RESTORED_COUNT: u16 = 32773;
    const REMOVAL_COUNT: u16 = 32768;
    const CLAMP_BOUNDARY: u16 = 32767;
    const WIDE_OFFSET: [i64; 2] = [65543, -65531];
    const WIDE_LIFT: u64 = 65539;
    const EXPECTED_ORIGIN: [u16; 2] = [107, 202];
    const SAMUS_CENTRE: [u16; 2] = [100, 200];
    let mut fields = Fields::new(
        object!("centre": SAMUS_CENTRE, "offset": WIDE_OFFSET, "lift": WIDE_LIFT, "shoot": true),
    )?;
    let cannon = cannon(&mut fields)?;
    fields.finish()?;
    assert_eq!(
        cannon_origin(&cannon).map(|(_, centre)| pair(centre.0)),
        Some(EXPECTED_ORIGIN)
    );
    let mut state = empty_state();
    state.count = ShotCount(RESTORED_COUNT);
    let mut content = invented_content(radius(TEST_RADIUS));
    content.damage = Damage(0);
    content
        .clips
        .get_mut(&ClipId("Power".into()))
        .ok_or("missing invented clip")?
        .frames[0]
        .hold = FrameHold(0);
    let mut events = vec![];
    let first = prepare_shot(&mut state, &cannon, &content, &mut events);
    assert_eq!(first, Some(ShotSlot::Slot0));
    // Intentionally isolated preparation cuts: the occupied zero-damage slot is replaced.
    state.cooldown = FrameWord(0);
    let second = prepare_shot(&mut state, &cannon, &content, &mut events);
    assert_eq!(second, first);
    assert_eq!(
        state.count.0,
        RESTORED_COUNT.wrapping_add(1).wrapping_add(1)
    );
    if let Some(body) = state.slots[ShotSlot::Slot0].body_mut() {
        assert!(!update_clip(body, &content));
        assert_eq!(body.remaining_hold, FrameWord(0));
        assert!(!update_clip(body, &content));
        assert_eq!(body.remaining_hold, FrameWord(u16::MAX));
    }
    let restored: State = value(&state)
        .try_into()
        .map_err(|e: toml::de::Error| e.to_string())?;
    assert_eq!(state, restored);
    state.count = ShotCount(REMOVAL_COUNT);
    remove_shot(&mut state.slots[ShotSlot::Slot0], &mut state.count);
    assert_eq!(state.count, ShotCount(CLAMP_BOUNDARY));
    assert!(
        Fields::new(object!("unknown_input": true))?
            .finish()
            .is_err()
    );
    assert!(
        compare_partial(
            &object!("unknown_output": true),
            &object!("visible": true),
            "strict"
        )
        .is_err()
    );
    assert!(
        compare_complete(
            &object!("state":true),
            &object!("state":true,"events":true),
            "complete"
        )
        .is_err()
    );
    assert!(
        clip(&mut Fields::new(
            object!("frames":Vec::<Value>::new(),"ending":"Remove")
        )?)
        .is_err()
    );
    validate_content(&invented_content(radius(TEST_RADIUS)))?;
    Ok(())
}

#[test]
fn ordinary_late_consumption_and_drawing() -> Result<(), String> {
    let mut fields = Fields::new(
        object!("trail":1,"remaining":2,"next":1,"picture":"Spark","appearance":"Dots"),
    )?;
    let initial = body(&mut fields)?;
    fields.finish()?;
    let mut s = empty_state();
    s.count = ShotCount(1);
    s.slots[ShotSlot::Slot0] = ShotState::Flying {
        body: initial.clone(),
        consumed: false,
    };
    let mut fields = Fields::new(object!("mode":"ShotsOnly","room_screens":[0,0]))?;
    let i = input(&mut fields)?;
    fields.finish()?;
    let c = invented_content(radius(TEST_RADIUS));
    let rules = Rules {
        acceleration: BeamSpeed(0),
        trail_interval: Rules::REFERENCE.trail_interval,
    };
    let mut world = ScriptedWorld::new(vec![]);
    let mut expected = s.clone();
    let events = update(&mut s, &i, &c, rules, &mut world);
    // Expected retained quantities are constructed from the input, not from the result.
    let mut expected_body = initial.clone();
    expected_body.trail = FrameWord(INITIAL_TRAIL);
    expected_body.remaining_hold = FrameWord(1);
    expected.slots[ShotSlot::Slot0] = ShotState::Flying {
        body: expected_body.clone(),
        consumed: false,
    };
    assert_eq!(s, expected);
    assert_eq!(
        events,
        vec![Event::Trail {
            slot: ShotSlot::Slot0,
            position: initial.position,
            picture: initial.picture,
            appearance: initial.trail_appearance
        }]
    );
    assert_eq!(world.visits, vec![]);
    s.consume(ShotSlot::Slot0);
    expected.slots[ShotSlot::Slot0] = ShotState::Flying {
        body: expected_body,
        consumed: true,
    };
    assert_eq!(s, expected);
    assert_eq!(
        pictures(&s, FlickerPhase::Odd, &DrawView::Normal(i.camera)),
        PictureIntents {
            flying: vec![PictureIntent {
                slot: ShotSlot::Slot0,
                picture: PictureId("Spark".into()),
                screen: ScreenCentre(axes([100, 200]))
            }],
            impact: vec![],
        }
    );
    let events = update(&mut s, &i, &c, rules, &mut world);
    expected.count = ShotCount(0);
    expected.slots[ShotSlot::Slot0] = ShotState::Free {
        trail: FrameWord(INITIAL_TRAIL),
        retained_fraction: axes([PixelFraction(0), PixelFraction(0)]),
    };
    assert_eq!(s, expected);
    assert_eq!(events, vec![]);
    assert_eq!(world.visits, vec![]);
    assert_eq!(
        pictures(&s, FlickerPhase::Odd, &DrawView::Normal(i.camera)),
        PictureIntents {
            flying: vec![],
            impact: vec![]
        }
    );
    Ok(())
}

#[test]
fn ordinary_failed_preparation_and_existing_impact() -> Result<(), String> {
    const RESTORED_COOLDOWN: u16 = 257;
    let mut fields = Fields::new(object!("shoot":true,"aim":"Forbidden","camera":[500,500]))?;
    let i = input(&mut fields)?;
    fields.finish()?;
    let mut s = empty_state();
    s.cooldown = FrameWord(RESTORED_COOLDOWN);
    s.count = ShotCount(1);
    let mut fields = Fields::new(
        object!("clip":"Impact","remaining":2,"next":1,"picture":"Old","appearance":"Dots","trail":1),
    )?;
    let initial = body(&mut fields)?;
    fields.finish()?;
    s.slots[ShotSlot::Slot0] = ShotState::Impact {
        body: initial.clone(),
    };
    let mut expected = s.clone();
    expected.cooldown = FrameWord(1);
    let mut retained = initial;
    retained.remaining_hold = FrameWord(1);
    expected.slots[ShotSlot::Slot0] = ShotState::Impact { body: retained };
    let mut world = ScriptedWorld::new(vec![]);
    let events = update(
        &mut s,
        &i,
        &invented_content(radius(TEST_RADIUS)),
        Rules::REFERENCE,
        &mut world,
    );
    // Spacing 257 to 256 permits the low-byte availability test; invalid aim leaves 1.
    // Existing impact outside lifetime view still advances its hold, with no trail/motion.
    assert_eq!(s, expected);
    assert_eq!(events, vec![]);
    assert_eq!(world.visits, vec![]);
    Ok(())
}

#[test]
fn ordinary_countdown_boundaries_and_modes() -> Result<(), String> {
    const SIGNED_NONPOSITIVE: u16 = 32768;
    for mode in [UpdateMode::CannonAndShots, UpdateMode::ShotsOnly] {
        for frozen in [false, true] {
            let mut fields = Fields::new(
                object!("remaining":0,"next":1,"trail":0,"picture":"Spark","appearance":"Dots"),
            )?;
            let body = body(&mut fields)?;
            fields.finish()?;
            let mut s = empty_state();
            s.cooldown = FrameWord(SIGNED_NONPOSITIVE);
            s.count = ShotCount(1);
            s.slots[ShotSlot::Slot0] = ShotState::Flying {
                body,
                consumed: false,
            };
            let mut fields = Fields::new(object!("room_screens":[0,0]))?;
            let mut i = input(&mut fields)?;
            fields.finish()?;
            i.mode = mode;
            i.time_stopped = frozen;
            let mut expected = s.clone();
            let active = mode == UpdateMode::ShotsOnly || !frozen;
            if mode == UpdateMode::CannonAndShots {
                expected.cooldown = FrameWord(if frozen { FROZEN_HOLD } else { 0 });
            }
            if active && let Some(b) = expected.slots[ShotSlot::Slot0].body_mut() {
                b.trail = FrameWord(u16::MAX);
                b.remaining_hold = FrameWord(u16::MAX);
            }
            let mut world = ScriptedWorld::new(vec![]);
            let events = update(
                &mut s,
                &i,
                &invented_content(radius(TEST_RADIUS)),
                Rules {
                    acceleration: BeamSpeed(0),
                    trail_interval: FrameWord(0),
                },
                &mut world,
            );
            assert_eq!(s, expected, "{mode:?} frozen {frozen}");
            assert_eq!(events, vec![]);
            assert_eq!(world.visits, vec![]);
        }
    }
    Ok(())
}

#[test]
fn ordinary_zero_content_hold_and_trail_interval() -> Result<(), String> {
    let mut fields = Fields::new(object!("trail":1,"picture":"Old","appearance":"Dots"))?;
    let initial = body(&mut fields)?;
    fields.finish()?;
    let mut s = empty_state();
    s.count = ShotCount(1);
    s.slots[ShotSlot::Slot0] = ShotState::Flying {
        body: initial.clone(),
        consumed: false,
    };
    let mut c = invented_content(radius(TEST_RADIUS));
    c.clips
        .get_mut(&initial.clip)
        .ok_or("missing invented clip")?
        .frames[0]
        .hold = FrameHold(0);
    let mut fields = Fields::new(object!("mode":"ShotsOnly","frozen":true,"room_screens":[0,0]))?;
    let i = input(&mut fields)?;
    fields.finish()?;
    let rules = Rules {
        acceleration: BeamSpeed(0),
        trail_interval: FrameWord(0),
    };
    let mut expected = s.clone();
    let b = expected.slots[ShotSlot::Slot0]
        .body_mut()
        .ok_or("missing expected body")?;
    b.trail = FrameWord(0);
    b.remaining_hold = FrameWord(0);
    b.next_frame = ClipFrame::at(1);
    b.picture = Some(PictureId("Spark".into()));
    // Inactive frame retains Dots, rather than clearing the body appearance.
    let mut world = ScriptedWorld::new(vec![]);
    let events = update(&mut s, &i, &c, rules, &mut world);
    assert_eq!(s, expected);
    assert_eq!(
        events,
        vec![Event::Trail {
            slot: ShotSlot::Slot0,
            position: initial.position,
            picture: initial.picture,
            appearance: initial.trail_appearance
        }]
    );
    assert_eq!(world.visits, vec![]);
    let b = expected.slots[ShotSlot::Slot0]
        .body_mut()
        .ok_or("missing expected body")?;
    b.trail = FrameWord(u16::MAX);
    b.remaining_hold = FrameWord(u16::MAX);
    let events = update(&mut s, &i, &c, rules, &mut world);
    assert_eq!(s, expected);
    assert_eq!(events, vec![]);
    assert_eq!(world.visits, vec![]);
    Ok(())
}
