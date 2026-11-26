//! Compare the scene, its published drawing and the receiver's carry in player terms.

use super::super::{compare::Observation, data_failure, result::Outcome};
use game::production::Production;
use game::production::played::explosion::{Flow, Output, Pages};
use game::state::owned::OwnedState;
use screens_explosion::{BurstFrame, MapRegion, MusicRequest, SceneEvent};
use types::explosion::{BackdropFrame, BurstGroup, ExplosionStage, FadeWait, Light};

pub(super) fn bytes(field: &str, bytes: Vec<u8>) -> Observation {
    Observation {
        field: format!("explosion.{field}"),
        bytes,
    }
}
pub(super) fn words(field: &str, values: impl IntoIterator<Item = u16>) -> Observation {
    bytes(
        field,
        values.into_iter().flat_map(u16::to_le_bytes).collect(),
    )
}
pub(super) fn quantity(field: &str, value: impl Into<i64>) -> Observation {
    Observation::quantity(format!("explosion.{field}"), value.into())
}

pub(super) fn visible(entries: impl IntoIterator<Item = [u8; 4]>, high: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for (index, [x, y, tile, attributes]) in entries.into_iter().enumerate() {
        if y == game::exact::oam::HIDDEN_Y {
            continue;
        }
        let bits =
            high.get(index / 4).copied().unwrap_or_default() >> ((index % 4).saturating_mul(2));
        out.extend([
            u8::try_from(index).unwrap_or_default(),
            x,
            y,
            tile,
            attributes,
            bits & 3,
        ]);
    }
    out
}

pub(super) fn fixed(out: &mut Vec<u16>, value: i32) {
    let [a, b, c, d] = value.to_le_bytes();
    out.extend([u16::from_le_bytes([c, d]), u16::from_le_bytes([a, b])]);
}

const fn group_id(group: BurstGroup) -> u16 {
    match group {
        BurstGroup::First => 1,
        BurstGroup::Middle => 2,
        BurstGroup::Last => 3,
        BurstGroup::Final => 4,
    }
}

pub(in crate::replay) fn actual(
    production: &Production,
    output: Option<&Output>,
) -> Result<Vec<Observation>, Outcome> {
    let OwnedState::Explosion(state) = production.state() else {
        return Err(data_failure(None, "station actual scope"));
    };
    let scene = &state.scene.0;
    let (phase, hold) = match scene.stage {
        ExplosionStage::Pending => (0, 0),
        ExplosionStage::WaitingForMusic => (1, 0),
        ExplosionStage::Revealing => (2, 0),
        ExplosionStage::Station => (3, 0),
        ExplosionStage::Departing => (4, 0),
        ExplosionStage::Holding(hold) => (5, hold.0.get()),
        ExplosionStage::Dimming => (6, 0),
        ExplosionStage::Complete => (7, 0),
    };
    let mut fields = vec![
        quantity("stage", phase),
        quantity(
            "light",
            match scene.light {
                Light::Blanked => 128,
                Light::Shown(light) => light.get(),
            },
        ),
        words(
            "context",
            [
                state.finished,
                u16::from(
                    state.destination
                        == game::production::played::explosion::Destination::LandingSite,
                ),
                state.maximum,
                u16::from(state.japanese_subtitles),
                u16::from(state.flow == Flow::ZebesFlight),
            ],
        ),
        bytes("health", canonical::to_bytes(&state.health)),
    ];
    if let Some(output) = output {
        fields.extend(page_writes(output));
        if scene.stage != ExplosionStage::Pending {
            fields.extend(drawn(state, output, hold)?);
        }
        fields.push(ordered_requests(&output.events)?);
    } else {
        fields.push(words("requests", []));
    }
    Ok(fields)
}

fn ordered_requests(source: &[SceneEvent]) -> Result<Observation, Outcome> {
    let mut events = Vec::new();
    for event in source {
        match event {
            SceneEvent::InstallArt(_) => events.push(1),
            SceneEvent::ReplaceSpacePalette(_) => events.push(2),
            SceneEvent::RequestMusic { request, delay } => events.extend([
                3,
                match request {
                    MusicRequest::Stop => 0,
                    MusicRequest::CeresMusicData => 1,
                    MusicRequest::ZebesFlightSong => 2,
                },
                u16::from(delay.0),
            ]),
            SceneEvent::StartStationFlash { center, .. } => events.extend([
                4,
                u16::try_from(center.x.0 >> 16)
                    .map_err(|_| data_failure(None, "station flash x"))?,
                u16::try_from(center.y.0 >> 16)
                    .map_err(|_| data_failure(None, "station flash y"))?,
            ]),
            SceneEvent::InstallMapDrawing { region, .. } => events.push(match region {
                MapRegion::Upper => 5,
                MapRegion::Lower => 6,
            }),
            SceneEvent::BeginZebesFlight => events.push(7),
        }
    }
    Ok(words("requests", events))
}

fn page_writes(output: &Output) -> Vec<Observation> {
    let mut fields = Vec::new();
    for page in &output.pages {
        match page {
            Pages::Station {
                map,
                graphics,
                sprites,
            } => {
                fields.extend([
                    bytes("map", map.clone()),
                    bytes("plane-tiles", graphics.clone()),
                    bytes("sprite-tiles", sprites.clone()),
                ]);
            }
            Pages::Ship(region, map) => fields.push(bytes(
                match region {
                    MapRegion::Upper => "ship-front",
                    MapRegion::Lower => "empty-lower",
                },
                map.clone(),
            )),
        }
    }
    fields
}

fn drawn(
    state: &game::production::played::explosion::State,
    output: &Output,
    hold: u16,
) -> Result<Vec<Observation>, Outcome> {
    let mut fields = Vec::new();
    let scene = &state.scene.0;
    let mut pose = vec![u16::from(scene.pose.angle.0), scene.pose.zoom.0];
    fixed(&mut pose, scene.pose.scroll.x.0);
    fixed(&mut pose, scene.pose.scroll.y.0);
    fields.push(words("pose", pose));
    let Some((m, scroll, pivot)) = output.plane else {
        return Err(data_failure(None, "station plane missing"));
    };
    fields.push(words(
        "transform",
        [m.a, m.b, m.c, m.d, scroll.x, scroll.y, pivot.x, pivot.y].map(i16::cast_unsigned),
    ));
    fields.push(words(
        "timers",
        [
            u16::try_from(scene.age.0).map_err(|_| data_failure(None, "station picture age"))?,
            u16::from(
                matches!(
                    scene.stage,
                    ExplosionStage::WaitingForMusic
                        | ExplosionStage::Revealing
                        | ExplosionStage::Dimming
                ) && scene.fade == FadeWait::OneFrame,
            ),
            hold,
        ],
    ));
    let mut sky = Vec::new();
    for point in [
        output.picture.large_asteroids,
        output.picture.small_asteroids,
    ] {
        fixed(&mut sky, point.x.0);
        fixed(&mut sky, point.y.0);
    }
    sky.push(u16::from(output.picture.backdrop == BackdropFrame::Second));
    fields.push(words("sky", sky));
    let mut bursts = Vec::new();
    for (owner, shown) in scene.bursts.iter().zip(&output.picture.bursts) {
        let sample = match shown.frame {
            BurstFrame::Hidden => 0,
            BurstFrame::Showing(index) => u16::try_from(index.saturating_add(1))
                .map_err(|_| data_failure(None, "station burst sample"))?,
        };
        bursts.extend([
            group_id(shown.id.group),
            u16::from(shown.id.occurrence.get()),
            owner.age.0.cast_unsigned(),
            sample,
        ]);
        fixed(&mut bursts, shown.position.x.0);
        fixed(&mut bursts, shown.position.y.0);
    }
    fields.push(words("bursts", bursts));
    fields.push(bytes(
        "oam",
        visible(
            state
                .oam
                .low
                .iter()
                .map(|e| [e.x, e.y, e.tile, e.attributes]),
            &state.oam.high,
        ),
    ));
    fields.push(words("palette", state.palette.iter().map(|s| s.0)));
    Ok(fields)
}
