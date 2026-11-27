//! Independent shaft records and the comparable production publications.
use super::{ExpectedCut, Outcome, data_failure, digits, word};
use crate::replay::compare::Observation;
use game::production::{
    Production,
    blackout::{Output, Request, SoundChannel},
};
use game::state::video::Oam;
use oracle_record::{Cut, FrameRecord, NmiKind};
use types::ceres_blackout::{BlackoutLight, BlackoutPhase, ShaftStage, TimerActivity, WorkPass};

fn field(name: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: format!("blackout.{name}"),
        bytes,
    }
}
fn words(values: impl IntoIterator<Item = u16>) -> Vec<u8> {
    values.into_iter().flat_map(u16::to_le_bytes).collect()
}
fn light(value: BlackoutLight) -> u16 {
    match value {
        BlackoutLight::ForcedBlank => 128,
        BlackoutLight::Lit(l) => u16::from(l.get()),
    }
}

fn phase_and_wait(phase: BlackoutPhase) -> (u16, u16) {
    match phase {
        BlackoutPhase::Shaft => (0, 0),
        BlackoutPhase::Hold(left) => (1, u16::from(left.get())),
        BlackoutPhase::Fade => (2, 0),
        BlackoutPhase::AwaitBlank => (3, 0),
        BlackoutPhase::Saving(_) => (4, 0),
        BlackoutPhase::Explosion(_) => (5, 0),
        BlackoutPhase::Dispatched(_) => (6, 0),
    }
}

pub(in crate::replay) fn live_scene(
    r: &FrameRecord,
    scene: &types::ceres_blackout::BlackoutState,
    video: &game::state::video::Video,
    (expected, actual): (&mut Vec<Observation>, &mut Vec<Observation>),
) -> Result<(), Outcome> {
    let lamp = |raw| {
        if raw & 128 != 0 {
            128
        } else {
            u16::from(raw & 15)
        }
    };
    let display_mirror = super::byte(&r.wram, 0x51)?;
    let prepared_light = lamp(display_mirror);
    let phase = match word(&r.wram, 0x998)? {
        8 => 0,
        32 => 1,
        33 if prepared_light < 128 => 2,
        33 => 3,
        _ => return Err(data_failure(Some(r.index), "live blackout phase")),
    };
    let (carried_phase, wait) = phase_and_wait(scene.phase);
    let display = |registers: &game::state::video::Registers| {
        u16::from(registers.brightness.0) | if registers.forced_blank { 128 } else { 0 }
    };
    for (name, observed, carried) in [
        ("phase", phase, carried_phase),
        ("wait", word(&r.wram, 0xaa0)?, wait),
        ("prepared-light", prepared_light, light(scene.light)),
        (
            "published-light",
            lamp(r.ppu.inidisp),
            light(scene.published.light),
        ),
        ("finished", word(&r.wram, 0x5b6)?, scene.finished.0),
        (
            "prepared-pass",
            u16::from(matches!(r.cut, Cut::NmiReturn(nmi) if nmi.kind == NmiKind::Overrun)),
            u16::from(matches!(scene.pass, WorkPass::Prepared(_))),
        ),
        (
            "video-prepared-light",
            u16::from(display_mirror & 0x8f),
            display(&video.batch.registers),
        ),
        (
            "video-published-light",
            u16::from(r.ppu.inidisp & 0x8f),
            display(&video.committed),
        ),
    ] {
        expected.push(field(name, words([observed])));
        actual.push(field(name, words([carried])));
    }
    expected.push(field("frozen-clock", digits(&r.wram)?.to_le_bytes().into()));
    actual.push(field(
        "frozen-clock",
        scene.timer.time.0.to_le_bytes().into(),
    ));
    Ok(())
}
pub(in crate::replay) fn oam_bytes(o: &Oam) -> Vec<u8> {
    o.low
        .iter()
        .flat_map(|e| [e.x, e.y, e.tile, e.attributes])
        .chain(o.high.iter().copied())
        .collect()
}
fn ids(low: &[u8]) -> Vec<u8> {
    low.as_chunks::<4>()
        .0
        .iter()
        .take(17)
        .flat_map(|e| [e[2], e[3]])
        .collect()
}
fn timer_oam(raw: &[u8]) -> Result<Vec<u8>, Outcome> {
    let bad = || data_failure(None, "timer OAM projection");
    let mut bytes = raw.get(..68).ok_or_else(bad)?.to_vec();
    bytes.extend_from_slice(raw.get(512..516).ok_or_else(bad)?);
    bytes.push(raw.get(516).ok_or_else(bad)? & 3);
    Ok(bytes)
}
fn memory(vram: &[u16], cgram: impl IntoIterator<Item = u16>, oam: &[u8]) -> Vec<u8> {
    let mut data = words(vram.iter().copied().chain(cgram));
    data.extend_from_slice(oam);
    data
}

const fn register_bytes(r: &game::state::video::Registers) -> [u8; 7] {
    [
        r.mode.0,
        r.main_screen.bits.to_le_bytes()[0],
        r.sub_screen.bits.to_le_bytes()[0],
        r.main_window_mask.bits.to_le_bytes()[0],
        r.sub_window_mask.bits.to_le_bytes()[0],
        r.colour_math.select.0,
        r.colour_math.layers.0,
    ]
}

fn recorded_setup(phase: u16, r: &FrameRecord) -> Result<Vec<u8>, Outcome> {
    let mut bytes = Vec::new();
    if phase >= 4 {
        for at in [0x55, 0x69, 0x6b, 0x6d, 0x6e, 0x70, 0x71] {
            bytes.push(super::byte(&r.wram, at)?);
        }
    }
    if phase >= 5 {
        bytes.push(r.ppu.bgmode);
        bytes.extend(r.ppu.screens);
        bytes.extend([r.ppu.cgwsel, r.ppu.cgadsub]);
    }
    Ok(bytes)
}

pub(in crate::replay) fn actual(
    prod: &Production,
    out: &Output,
    late: bool,
) -> Result<Vec<Observation>, Outcome> {
    let state = prod
        .blackout()
        .ok_or_else(|| data_failure(None, "blackout projection"))?;
    let o = &state.owners.0;
    let scene = &o.scene;
    let (phase, wait) = phase_and_wait(scene.phase);
    let regs = &state.video.committed;
    let prepared = matches!(scene.pass, WorkPass::Prepared(_));
    let mut shape = words([
        phase,
        wait,
        light(scene.light),
        light(scene.published.light),
        if regs.forced_blank {
            128
        } else {
            u16::from(regs.brightness.0)
        },
        scene.finished.0,
        u16::from(prepared),
        u16::from(scene.shaft == ShaftStage::Rotating),
        u16::from(scene.timer.activity == TimerActivity::Active),
    ]);
    shape.extend(scene.timer.time.0.to_le_bytes());
    if phase >= 4 {
        shape.extend(register_bytes(&state.video.batch.registers));
    }
    if phase >= 5 {
        shape.extend(register_bytes(regs));
    }
    let clock = o.run.profile.play_time;
    let room = words([
        u16::from(clock.frames.get()),
        u16::from(clock.seconds.get()),
        u16::from(clock.minutes.get()),
        u16::from(clock.hours.get()),
        o.random.number.0,
        o.health.energy.whole.get(),
        o.run.profile.energy.maximum,
    ]);
    let raw = oam_bytes(&state.video.memory.oam);
    let mut drawing = timer_oam(&raw)?;
    drawing.extend(words(scene.published.timer_sprites.iter().map(|p| p.0)));
    if let WorkPass::Prepared(p) = &scene.pass {
        drawing.extend(words(p.timer_sprites.iter().map(|p| p.0)));
    }
    let mut fields = vec![
        field("scene", shape),
        field("room", room),
        field("timer-drawing", drawing),
        field(
            "battery",
            prod.battery()
                .ok_or_else(|| data_failure(None, "blackout cartridge"))?
                .into(),
        ),
        field("requests", requests(out)),
    ];
    if late {
        let mut bytes = memory(
            &state.video.memory.vram,
            state.video.memory.cgram.iter().map(|c| c.0),
            &raw,
        );
        bytes.extend(words([
            u16::try_from(state.video.batch.vram.len())
                .map_err(|_| data_failure(None, "regular room transfer count"))?,
            u16::from(state.platform.is_some()),
        ]));
        if let Some(dispatch) = &out.dispatch {
            bytes.extend(memory(
                &[],
                dispatch.palette.iter().map(|c| c.0),
                &oam_bytes(&dispatch.oam),
            ));
        }
        fields.push(field("memory", bytes));
    }
    Ok(fields)
}

fn requests(out: &Output) -> Vec<u8> {
    let mut req = Vec::new();
    for r in &out.requests {
        match *r {
            Request::StandAndLock => req.push(1),
            Request::ContinueRoom => req.push(2),
            Request::DisableShaftInterrupts => req.push(3),
            Request::PrepareSpriteOnly => req.push(4),
            Request::CaptureEscapeSave(_) => req.push(5),
            Request::StopMusic(delay) => req.extend([6, delay.0]),
            Request::Silence(channel) => req.push(match channel {
                SoundChannel::One => 7,
                SoundChannel::Two => 8,
                SoundChannel::Three => 9,
            }),
            Request::PrepareStationExplosion(_) => req.push(10),
            Request::DispatchExplosion(_) => req.push(11),
        }
    }
    if let Some(dispatch) = &out.dispatch {
        req.extend(words([
            dispatch.finished,
            dispatch.health.energy.whole.get(),
            dispatch.maximum,
            u16::from(dispatch.japanese_subtitles),
        ]));
    }
    req
}

pub(in crate::replay) fn expected(
    prior: &FrameRecord,
    current: &FrameRecord,
    late: bool,
) -> Result<ExpectedCut, Outcome> {
    let last = &prior.wram;
    let now = &current.wram;
    let previous_flow = word(last, 0x998)?;
    let flow = word(now, 0x998)?;
    let station = word(now, 0x1f51)? != 0;
    let dispatched = station
        && word(last, 0x1f51)? != 0
        && word(now, 0x5b6)? == word(last, 0x5b6)?.wrapping_add(1);
    // Observe the input to the station's first turn before that consumer changes OAM and counts.
    let boundary = if dispatched { prior } else { current };
    let ram = &boundary.wram;
    let phase = match (flow, station, dispatched) {
        (8, _, _) => 0,
        (32, _, _) => 1,
        (33, _, _) if super::byte(now, 0x51)? & 128 == 0 => 2,
        (33, _, _) => 3,
        (34, false, _) => 4,
        (34, true, false) => 5,
        (34, true, true) => 6,
        _ => return Err(data_failure(Some(current.index), "escape phase")),
    };
    let prepared = matches!(current.cut,Cut::NmiReturn(nmi) if nmi.kind==NmiKind::Overrun);
    let lamp = |l: u8| if l & 128 != 0 { 128 } else { u16::from(l & 15) };
    let mut shape = words([
        phase,
        word(ram, 0xaa0)?,
        lamp(super::byte(ram, 0x51)?),
        lamp(boundary.ppu.inidisp),
        lamp(boundary.ppu.inidisp),
        word(ram, 0x5b6)?,
        u16::from(prepared),
        u16::from(word(ram, 0x93f)? & 0x8000 != 0),
        u16::from(word(ram, 0x943)? != 0),
    ]);
    shape.extend(digits(ram)?.to_le_bytes());
    shape.extend(recorded_setup(phase, boundary)?);
    let room = words([
        word(ram, 0x9da)?,
        word(ram, 0x9dc)?,
        word(ram, 0x9de)?,
        word(ram, 0x9e0)?,
        word(ram, 0x5e5)?,
        word(ram, 0x9c2)?,
        word(ram, 0x9c4)?,
    ]);
    let mut drawing = timer_oam(&boundary.oam)?;
    drawing.extend(ids(&boundary.oam));
    if prepared {
        drawing.extend(ids(crate::source::OAM_LOW_TABLE
            .read(ram)
            .map_err(|_| data_failure(None, "prepared timer entries"))?));
    }
    let req = if dispatched {
        let mut r = vec![11];
        r.extend(words([
            word(last, 0x5b6)?,
            word(last, 0x9c2)?,
            word(last, 0x9c4)?,
            u16::from(word(last, 0x9e2)? != 0),
        ]));
        r
    } else if station {
        audio(last, now, current.index)?
    } else if flow == 34 && previous_flow == 33 {
        vec![3, 4, 5]
    } else if previous_flow == 8 && flow == 32 {
        vec![1]
    } else if matches!(previous_flow, 32 | 33) {
        vec![2]
    } else {
        Vec::new()
    };
    let mut fields = vec![
        field("scene", shape),
        field("room", room),
        field("timer-drawing", drawing),
        field("battery", boundary.sram.to_vec()),
        field("requests", req),
    ];
    if late {
        let mut bytes = memory(
            &boundary.vram,
            boundary.cgram.iter().copied(),
            &boundary.oam,
        );
        bytes.extend(words([word(ram, 0x330)? / 7, word(ram, 0x334)? / 9]));
        if dispatched {
            bytes.extend(memory(&[], boundary.cgram.iter().copied(), &boundary.oam));
        }
        fields.push(field("memory", bytes));
    }
    Ok(ExpectedCut {
        semantics: fields.iter().take(2).cloned().collect(),
        semantic_count: 2,
        output_count: fields.len().saturating_sub(2),
        replies: 0,
        fields,
    })
}

fn audio(last: &[u8], now: &[u8], cut: u64) -> Result<Vec<u8>, Outcome> {
    let music = usize::from(word(last, 0x639)?);
    let mut r = vec![6];
    if super::byte(now, 0x619_usize.saturating_add(music))? != 0
        || word(now, 0x639)? != (word(last, 0x639)?.wrapping_add(2) & 0x0e)
    {
        return Err(data_failure(Some(cut), "station music queue"));
    }
    r.push(super::byte(now, 0x629_usize.saturating_add(music))?);
    for (cursor, base, cue, intent) in [
        (0x646, 0x656_usize, 2, 7),
        (0x647, 0x666, 113, 8),
        (0x648, 0x676, 1, 9),
    ] {
        if super::byte(now, cursor)? != (super::byte(last, cursor)?.wrapping_add(1) & 15)
            || super::byte(
                now,
                base.saturating_add(usize::from(super::byte(last, cursor)?)),
            )? != cue
        {
            return Err(data_failure(Some(cut), "station silence queue"));
        }
        r.push(intent);
    }
    r.push(10);
    Ok(r)
}
