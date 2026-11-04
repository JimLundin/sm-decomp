//! Physical FX projection and deliberate auxiliary payload order.
#![forbid(unsafe_code)]
use crate::fx_state::{Field, FxState};

pub fn address(field: Field) -> usize {
    match field {
        Field::Blending => 0x1986,
        Field::SkyWavePhase => 0x596,
        Field::SkyScreenRow => 0x598,
        Field::SkyVisibleHeight => 0x59a,
        Field::SkyCompatibilityWord => 0x59c,
        Field::EarthquakeSoundIndex => 0x607,
        Field::EarthquakeSoundTimer => 0x609,
        Field::HudHorizontal => 0xcad8,
        Field::HudVertical => 0xcada,
        Field::EffectHorizontal => 0xcadc,
        Field::EffectVertical => 0xcade,
    }
}
pub fn decode(bytes: &impl Fn(usize) -> u8) -> FxState {
    FxState::supplied(Field::ALL.map(|field| {
        let at = address(field);
        u16::from_le_bytes([bytes(at), bytes(at + 1)])
    }))
}
pub fn lighting_address(field: crate::fx_lighting::Field) -> usize {
    match field {
        crate::fx_lighting::Field::Timer => 0x1778,
        crate::fx_lighting::Field::FlashIndex => 0x177a,
        crate::fx_lighting::Field::CompatibilityWord => 0x177c,
        crate::fx_lighting::Field::Darkness => 0x177e,
        crate::fx_lighting::Field::CompatibilityLimit => 0x1780,
        crate::fx_lighting::Field::CompatibilityShade => 0x1782,
    }
}
pub fn decode_lighting(bytes: &impl Fn(usize) -> u8) -> crate::visual_state::FirefleaLighting {
    let mut value = crate::visual_state::FirefleaLighting::default();
    for field in crate::fx_lighting::Field::ALL {
        let at = lighting_address(field);
        *field.get_mut(&mut value) = u16::from_le_bytes([bytes(at), bytes(at + 1)]);
    }
    value
}
pub fn projected_lighting_byte(
    value: &crate::visual_state::FirefleaLighting,
    at: usize,
) -> Option<u8> {
    crate::fx_lighting::Field::ALL
        .into_iter()
        .find_map(|field| {
            let first = lighting_address(field);
            (first..first + 2)
                .contains(&at)
                .then(|| field.get(value).to_le_bytes()[at - first])
        })
}
pub fn write_lighting_byte(
    value: &mut crate::visual_state::FirefleaLighting,
    at: usize,
    byte: u8,
) -> bool {
    let Some(field) = crate::fx_lighting::Field::ALL.into_iter().find(|&field| {
        let first = lighting_address(field);
        (first..first + 2).contains(&at)
    }) else {
        return false;
    };
    let word = field.get_mut(value);
    let mut bytes = word.to_le_bytes();
    bytes[at - lighting_address(field)] = byte;
    *word = u16::from_le_bytes(bytes);
    true
}
pub fn projected_byte(state: &FxState, at: usize) -> Option<u8> {
    Field::ALL.into_iter().find_map(|field| {
        let first = address(field);
        (first..first + 2)
            .contains(&at)
            .then(|| field.get(state).to_le_bytes()[at - first])
    })
}
pub fn write_byte(state: &mut FxState, at: usize, byte: u8) -> bool {
    let Some(field) = Field::ALL.into_iter().find(|&field| {
        let first = address(field);
        (first..first + 2).contains(&at)
    }) else {
        return false;
    };
    let word = field.get_mut(state);
    let mut bytes = word.to_le_bytes();
    bytes[at - address(field)] = byte;
    *word = u16::from_le_bytes(bytes);
    true
}
pub fn encode(state: &FxState, mut span: impl FnMut(usize, &[u8])) {
    for field in Field::ALL {
        span(address(field), &field.get(state).to_le_bytes());
    }
}
pub fn checkpoint(bytes: &mut Vec<u8>, state: &FxState) {
    bytes.push(u8::from(state.is_initialized()));
    for field in Field::ALL {
        bytes.extend_from_slice(&field.get(state).to_le_bytes());
    }
}
pub fn read_checkpoint(
    read: &mut crate::actors::snapshot::CheckpointReader<'_>,
) -> Result<FxState, String> {
    if read.byte()? != 1 {
        return Err("FX checkpoint is not initialized".into());
    }
    let mut words = [0; 11];
    for word in &mut words {
        *word = read.word()?;
    }
    Ok(FxState::supplied(words))
}

use crate::scene::hdma_sources::{
    HdmaLiveDescription, HdmaLiveRegion, HdmaOwnedBuffer, HdmaRegionSource, LiveOwner,
};
/// Physical mapping is confined to this compatibility projection.
pub fn source_address(owner: LiveOwner, offset: usize) -> Option<usize> {
    let first: usize = match owner {
        LiveOwner::ExplosionLeft => 0xc406,
        LiveOwner::ExplosionRight => 0xc506,
        LiveOwner::ExplosionOffscreen => 0xc606,
        LiveOwner::HdmaTable1 => 0x9800,
        LiveOwner::HdmaTable2 => 0x9d00,
        LiveOwner::LiquidSurface => 0x9c00,
        LiveOwner::LiquidBackground => 0x9c46,
        LiveOwner::WindowLeft => 0x9e00,
        LiveOwner::WindowRight => 0x9e10,
        LiveOwner::SkyHorizontal => 0x9e80,
        LiveOwner::SkyIndirect => 0x9f00,
        LiveOwner::SkyBg2 => 0x9f80,
        _ => return None,
    };
    first.checked_add(offset)
}
pub fn source_coordinate(at: usize) -> Option<(LiveOwner, usize)> {
    match at {
        0xc406..=0xc607 => Some((LiveOwner::ExplosionLeft, at - 0xc406)),
        0x9800..=0x9bff => Some((LiveOwner::HdmaTable1, at - 0x9800)),
        0x9d00..=0x9dff => Some((LiveOwner::HdmaTable2, at - 0x9d00)),
        0x9c00..=0x9cff => Some((LiveOwner::LiquidSurface, at - 0x9c00)),
        0x9e00..=0x9fff => Some((LiveOwner::WindowLeft, at - 0x9e00)),
        _ => None,
    }
}
pub fn decode_sources(
    bytes: &impl Fn(usize) -> u8,
    window: &crate::menu_window_state::WindowState,
) -> HdmaLiveDescription {
    assert!(
        window.is_initialized(),
        "FX source window is not initialized"
    );
    let buffers = vec![
        HdmaOwnedBuffer {
            owner: LiveOwner::ExplosionLeft,
            bytes: (0xc406..0xc608).map(bytes).collect(),
        },
        HdmaOwnedBuffer {
            owner: LiveOwner::HdmaTable1,
            bytes: (0x9800..0x9c00).map(bytes).collect(),
        },
        HdmaOwnedBuffer {
            owner: LiveOwner::HdmaTable2,
            bytes: (0x9d00..0x9e00).map(bytes).collect(),
        },
        HdmaOwnedBuffer {
            owner: LiveOwner::LiquidSurface,
            bytes: (0x9c00..0x9d00).map(bytes).collect(),
        },
        HdmaOwnedBuffer {
            owner: LiveOwner::WindowLeft,
            bytes: (0x9e00..0xa000).map(bytes).collect(),
        },
    ];
    let mut regions: Vec<_> = [
        (LiveOwner::ExplosionLeft, LiveOwner::ExplosionLeft, 0, 514),
        (
            LiveOwner::ExplosionRight,
            LiveOwner::ExplosionLeft,
            256,
            258,
        ),
        (
            LiveOwner::ExplosionOffscreen,
            LiveOwner::ExplosionLeft,
            512,
            2,
        ),
        (LiveOwner::HdmaTable1, LiveOwner::HdmaTable1, 0, 1024),
        (LiveOwner::HdmaTable2, LiveOwner::HdmaTable2, 0, 256),
        (LiveOwner::LiquidSurface, LiveOwner::LiquidSurface, 0, 256),
        (
            LiveOwner::LiquidBackground,
            LiveOwner::LiquidSurface,
            0x46,
            256 - 0x46,
        ),
        (LiveOwner::WindowLeft, LiveOwner::WindowLeft, 0, 512),
        (
            LiveOwner::WindowRight,
            LiveOwner::WindowLeft,
            0x10,
            512 - 0x10,
        ),
        (
            LiveOwner::SkyHorizontal,
            LiveOwner::WindowLeft,
            0x80,
            512 - 0x80,
        ),
        (LiveOwner::SkyIndirect, LiveOwner::WindowLeft, 0x100, 256),
        (LiveOwner::SkyBg2, LiveOwner::WindowLeft, 0x180, 128),
    ]
    .into_iter()
    .map(|(owner, storage, offset, length)| HdmaLiveRegion {
        owner,
        source: HdmaRegionSource::Buffer { storage, offset },
        length,
    })
    .collect();
    // Native source topology names the existing screen allocation; it never imports screen copies.
    regions.extend([
        HdmaLiveRegion {
            owner: LiveOwner::MessageDescriptor,
            source: HdmaRegionSource::Workspace {
                point: crate::presentation_workspace::Point::new(
                    crate::presentation_workspace::Role::MessageDescriptor,
                    0,
                ),
            },
            length: 7,
        },
        HdmaLiveRegion {
            owner: LiveOwner::MessageScanlines,
            source: HdmaRegionSource::Workspace {
                point: crate::presentation_workspace::Point::new(
                    crate::presentation_workspace::Role::MessageScanlines,
                    0,
                ),
            },
            length: 480,
        },
    ]);
    let mut live = HdmaLiveDescription::new(buffers, regions).expect("FX source topology");
    let lanes = core::array::from_fn(|index| {
        Some(
            crate::menu_window_encoding::address(crate::menu_window_state::Field::ALL[index / 2])
                - 0x9e00
                + index % 2,
        )
    });
    live.bind_window_controls(
        crate::fx_sources::WindowControlLayout {
            storage: LiveOwner::WindowLeft,
            lanes,
        },
        Some(window),
    )
    .expect("FX window source topology");
    live
}
pub fn physical_source_coordinate(
    session: &crate::session::Session,
    at: usize,
) -> Result<Option<(LiveOwner, usize)>, crate::scene::state::DisplayError> {
    if session.fx_sources.is_some() {
        return Ok(source_coordinate(at));
    }
    display_source_coordinate(&session.presentation.owner()?.state, at)
}
pub fn display_source_coordinate(
    display: &crate::scene::state::DisplayState,
    at: usize,
) -> Result<Option<(LiveOwner, usize)>, crate::scene::state::DisplayError> {
    if at >= 0x20000 {
        return Ok(None);
    }
    let bank =
        0x7e + u8::try_from(at / 65_536).map_err(|_| crate::scene::state::DisplayError::Source)?;
    let address =
        u16::try_from(at % 65_536).map_err(|_| crate::scene::state::DisplayError::Source)?;
    display.hdma_live.physical_coordinate_with_workspace(
        &display.hdma,
        bank,
        address,
        &display.workspace,
    )
}
/// An optional source projection claims no bytes before either source owner exists.
pub fn projected_source_byte(session: &crate::session::Session, at: usize) -> Option<u8> {
    ControlReader::new(session).projected_source_byte(at)
}
pub fn write_source_byte(session: &mut crate::session::Session, at: usize, byte: u8) -> bool {
    if session.fx_sources.is_none() && !session.presentation.is_owned() {
        return false;
    }
    let Some((owner, offset)) =
        physical_source_coordinate(session, at).expect("prepared FX source resolution")
    else {
        return false;
    };
    store_session_source_byte(session, owner, offset, byte)
        .expect("prepared FX source publication");
    true
}
pub fn encode_sources(session: &crate::session::Session, mut span: impl FnMut(usize, &[u8])) {
    if session.fx_sources.is_some() {
        let reader = ControlReader::new(session);
        for at in (0x9800..0xa000).chain(0xc406..0xc608) {
            if crate::menu_window_encoding::lane(at).is_some() {
                continue;
            }
            if let Some(byte) = reader.projected_source_byte(at) {
                span(at, &[byte]);
            }
        }
        return;
    }
    let Ok(display) = session.presentation.owner() else {
        return;
    };
    encode_display_sources(&display.state, session, span);
}
/// Encode real prepared source provenance for export and normalized-image checks.
pub fn encode_display_sources(
    display: &crate::scene::state::DisplayState,
    numeric: &crate::session::Session,
    mut span: impl FnMut(usize, &[u8]),
) {
    let graph = display_control_graph(display).expect("source export topology");
    for binding in display
        .hdma
        .channels
        .iter()
        .flat_map(|channel| [&channel.table, &channel.source])
        .chain(
            display
                .hdma
                .objects
                .iter()
                .map(|object| &object.instruction),
        )
    {
        let crate::scene::hdma_sources::SourceBinding::LiveOwner { owner, windows } = binding
        else {
            continue;
        };
        for window in windows.iter() {
            for displacement in 0..window.length {
                let address = u16::try_from(usize::from(window.address) + displacement).unwrap();
                let physical = physical_work_ram(window.bank, address);
                let selected = selected_source_owner(
                    &display.hdma_live,
                    &display.workspace,
                    &graph,
                    *owner,
                    window.offset + displacement,
                )
                .expect("source export provider");
                let byte = match selected {
                    ControlOwner::Workspace(point) => display
                        .workspace
                        .read_source_point(point)
                        .expect("Workspace export provider"),
                    ControlOwner::Source(owner, offset) => display
                        .hdma_live
                        .read_buffer(owner, offset)
                        .expect("source export buffer"),
                    ControlOwner::Scalar(lane) => {
                        let at = control_address(lane);
                        if (0x18b0..0x195c).contains(&at) {
                            common_scalar_word(&display.hdma.common, at & !1).to_le_bytes()[at % 2]
                        } else if let Some(byte) =
                            projected_lighting_byte(&display.visual.fireflea, at)
                        {
                            byte
                        } else {
                            scalar_control_byte(numeric, at).expect("source export scalar")
                        }
                    }
                };
                if let Some(at) = physical {
                    span(at, &[byte]);
                }
                if let ControlOwner::Workspace(point) = selected {
                    display
                        .workspace
                        .visit_source_bus_addresses(point, |bank, address| {
                            if let Some(at) = physical_work_ram(bank, address) {
                                span(at, &[byte]);
                            }
                        })
                        .expect("actual Workspace source export");
                }
            }
        }
    }
}

/// Resolve a producer role through its actual live topology, independently of
/// the compatibility address associated with that role in legacy rooms.
pub fn session_source_byte(
    session: &crate::session::Session,
    owner: LiveOwner,
    offset: usize,
) -> Result<u8, crate::scene::state::DisplayError> {
    ControlReader::new(session).source_byte(owner, offset)
}
pub fn store_session_source_byte(
    session: &mut crate::session::Session,
    owner: LiveOwner,
    offset: usize,
    byte: u8,
) -> Result<(), crate::scene::state::DisplayError> {
    if session.presentation.is_owned() && session.fx_sources.is_none() {
        let display = &mut session.presentation.owner_mut()?.state;
        let selected = current_source_owner(
            &display.hdma_live,
            &display.workspace,
            &display.hdma,
            owner,
            offset,
        )?;
        // The actual scalar store uses the Session port after ending this borrow.
        store_owner_byte(session, selected, byte)?;
        let display = &mut session.presentation.owner_mut()?.state;
        crate::fx_sources::forget_current_origins(
            &mut display.hdma_live,
            &display.workspace,
            &display.hdma,
            selected,
        );
        Ok(())
    } else if !session.presentation.is_owned() {
        // The identity graph: the source's own owner, and no origins to forget.
        let selected = source_owner(
            session_live(session)?,
            session.presentation.workspace(),
            owner,
            offset,
        )?;
        store_owner_byte(session, selected, byte)
    } else {
        let graph = control_graph(session)?;
        let selected = selected_source_owner(
            session_live(session)?,
            session.presentation.workspace(),
            &graph,
            owner,
            offset,
        )?;
        store_owner_byte(session, selected, byte)?;
        forget_owner_origins(session, &graph, selected)
    }
}

/// Actual WRAM allocation index for the native bus, retaining the original
/// BankWindow separately. Other bus windows keep their nonphysical provenance.
pub fn physical_work_ram(bank: u8, address: u16) -> Option<usize> {
    match bank {
        0x7e | 0x7f => Some((usize::from(bank & 1) << 16) | usize::from(address)),
        0x00..=0x3f | 0x80..=0xbf if address < 0x2000 => Some(usize::from(address)),
        _ => None,
    }
}
pub fn physical_source_spans(
    display: &crate::scene::state::DisplayState,
) -> Vec<std::ops::Range<usize>> {
    let mut spans: Vec<_> = (0..crate::fx_state::ControlOwnership::LANES)
        .map(|lane| {
            let at = control_address(lane);
            at..at + 1
        })
        .collect();
    for binding in display
        .hdma
        .channels
        .iter()
        .flat_map(|channel| [&channel.table, &channel.source])
        .chain(
            display
                .hdma
                .objects
                .iter()
                .map(|object| &object.instruction),
        )
    {
        let crate::scene::hdma_sources::SourceBinding::LiveOwner { owner, windows } = binding
        else {
            continue;
        };
        for window in windows.iter() {
            let Some(first) = physical_work_ram(window.bank, window.address) else {
                continue;
            };
            let length = if matches!(window.bank, 0x7e | 0x7f) {
                window.length
            } else {
                window.length.min(0x2000 - usize::from(window.address))
            };
            spans.push(first..first + length);
            for offset in window.offset..window.offset + window.length {
                if let Ok(Some(point)) =
                    display
                        .hdma_live
                        .workspace_coordinate(&display.workspace, *owner, offset)
                {
                    display
                        .workspace
                        .visit_source_bus_addresses(point, |bank, address| {
                            if let Some(at) = physical_work_ram(bank, address) {
                                spans.push(at..at + 1);
                            }
                        })
                        .expect("actual Workspace publication spans");
                }
                if let Ok(coordinate) = display.hdma_live.canonical_coordinate(*owner, offset)
                    && let Ok(Some(at)) = source_counterpart(display, coordinate.0, coordinate.1)
                {
                    spans.push(at..at + 1);
                }
            }
        }
    }
    spans
}
/// Native bytes adopt in the actual last-publication order. Snapshot restoration
/// supplies no new ordinals and follows the selected checkpoint's saved origin.
/// With no physical origin, the supplied canonical owner remains authoritative
/// when disjoint physical windows alone cannot identify an original writer.
pub fn adopt_display_sources(
    display: &mut crate::scene::state::DisplayState,
    window_state: &mut crate::menu_window_state::WindowState,
    bytes: &impl Fn(usize) -> u8,
    order: &impl Fn(usize) -> Option<u64>,
) -> Result<(), crate::scene::state::DisplayError> {
    adopt_workspace_sources(display, bytes, order)?;
    let mut candidates = std::collections::BTreeMap::new();
    for binding in display
        .hdma
        .channels
        .iter()
        .flat_map(|channel| [&channel.table, &channel.source])
        .chain(
            display
                .hdma
                .objects
                .iter()
                .map(|object| &object.instruction),
        )
    {
        let crate::scene::hdma_sources::SourceBinding::LiveOwner { owner, windows } = binding
        else {
            continue;
        };
        for window in windows.iter() {
            for displacement in 0..window.length {
                let address = u16::try_from(usize::from(window.address) + displacement).unwrap();
                let Some(at) = physical_work_ram(window.bank, address) else {
                    continue;
                };
                let offset = window.offset + displacement;
                if display.hdma_live.workspace_point(*owner, offset)?.is_some() {
                    continue;
                }
                let coordinate = display.hdma_live.canonical_coordinate(*owner, offset)?;
                let byte = bytes(at);
                let ordinal = order(at);
                let entry = candidates.entry(coordinate).or_insert((
                    byte,
                    true,
                    ordinal.map(|value| (value, at)),
                ));
                entry.1 &= entry.0 == byte;
                if let Some(ordinal) = ordinal
                    && entry.2.is_none_or(|(previous, _)| ordinal > previous)
                {
                    entry.2 = Some((ordinal, at));
                }
            }
        }
    }
    for (&(storage, offset), entry) in &mut candidates {
        if let Some(at) = source_counterpart(display, storage, offset)? {
            let byte = bytes(at);
            entry.1 &= entry.0 == byte;
            if let Some(ordinal) = order(at)
                && entry.2.is_none_or(|(previous, _)| ordinal > previous)
            {
                entry.2 = Some((ordinal, at));
            }
        }
    }
    for ((storage, offset), (byte, equal, latest)) in candidates {
        let origin = latest
            .map(|(_, at)| crate::fx_domain::SourceOrigin {
                address: at,
                normalized: false,
            })
            .or_else(|| {
                display
                    .hdma_live
                    .native_origins()
                    .get(&(storage, offset))
                    .copied()
            });
        let value = if let Some(origin) = origin {
            bytes(origin.address)
        } else if equal {
            byte
        } else if let Some((field, lane)) =
            display.hdma_live.window_control_lane(storage, offset)?
        {
            field.get(window_state).to_le_bytes()[lane]
        } else {
            display.hdma_live.read(&display.hdma, storage, offset)?
        };
        if let Some((field, lane)) = display.hdma_live.window_control_lane(storage, offset)? {
            let word = field.get_mut(window_state);
            let mut lanes = word.to_le_bytes();
            lanes[lane] = value;
            *word = u16::from_le_bytes(lanes);
        } else {
            display
                .hdma_live
                .write(&mut display.hdma, storage, offset, value)?;
        }
        if let Some((_, at)) = latest {
            display.hdma_live.note_native_origin(storage, offset, at)?;
        }
    }
    Ok(())
}

/// Group screen roles by actual allocation coordinates at the selected native boundary.
/// Only write provenance is retained here; all screen bytes stay in Workspace.
fn adopt_workspace_sources(
    display: &mut crate::scene::state::DisplayState,
    bytes: &impl Fn(usize) -> u8,
    order: &impl Fn(usize) -> Option<u64>,
) -> Result<(), crate::scene::state::DisplayError> {
    use crate::scene::state::DisplayError;
    struct Candidate {
        point: crate::presentation_workspace::Point,
        source: crate::presentation_workspace::SourcePoint,
        aliases: Vec<(LiveOwner, usize)>,
        physical: Vec<usize>,
        first: Option<u8>,
        equal: bool,
        latest: Option<(u64, usize)>,
        saved: Option<crate::fx_domain::SourceOrigin>,
    }
    let mut candidates = std::collections::BTreeMap::new();
    for binding in display
        .hdma
        .channels
        .iter()
        .flat_map(|channel| [&channel.table, &channel.source])
        .chain(
            display
                .hdma
                .objects
                .iter()
                .map(|object| &object.instruction),
        )
    {
        let crate::scene::hdma_sources::SourceBinding::LiveOwner { owner, windows } = binding
        else {
            continue;
        };
        for window in windows.iter() {
            for displacement in 0..window.length {
                let offset = window
                    .offset
                    .checked_add(displacement)
                    .ok_or(DisplayError::Source)?;
                let Some(point) = display.hdma_live.workspace_point(*owner, offset)? else {
                    continue;
                };
                let source = display.workspace.source_coordinate(point)?;
                let coordinate = (source.origin, source.offset);
                let address = u16::try_from(usize::from(window.address) + displacement)
                    .map_err(|_| DisplayError::Source)?;
                let physical = physical_work_ram(window.bank, address);

                let entry = candidates
                    .entry((
                        match coordinate.0 {
                            crate::presentation_workspace::Origin::Shared => None,
                            crate::presentation_workspace::Origin::Allocation(id) => Some(id),
                        },
                        coordinate.1,
                    ))
                    .or_insert_with(|| Candidate {
                        point,
                        source,
                        aliases: Vec::new(),
                        physical: Vec::new(),
                        first: None,
                        equal: true,
                        latest: None,
                        saved: None,
                    });
                if !entry.aliases.contains(&(*owner, offset)) {
                    entry.aliases.push((*owner, offset));
                }
                let mut admit = |at| {
                    if !entry.physical.contains(&at) {
                        entry.physical.push(at);
                    }
                    let byte = bytes(at);
                    if let Some(first) = entry.first {
                        entry.equal &= first == byte;
                    } else {
                        entry.first = Some(byte);
                    }
                    if let Some(ordinal) = order(at)
                        && entry.latest.is_none_or(|(previous, _)| ordinal > previous)
                    {
                        entry.latest = Some((ordinal, at));
                    }
                };
                if let Some(at) = physical {
                    admit(at);
                }
                display
                    .workspace
                    .visit_source_bus_addresses(source, |bank, address| {
                        if let Some(at) = physical_work_ram(bank, address) {
                            admit(at);
                        }
                    })?;
                if let Some(saved) = display
                    .hdma_live
                    .native_origins()
                    .get(&(*owner, offset))
                    .copied()
                {
                    if entry.saved.is_some_and(|prior| prior != saved) && entry.latest.is_none() {
                        return Err(DisplayError::Identity);
                    }
                    entry.saved = Some(saved);
                }
            }
        }
    }
    for entry in candidates.into_values() {
        if entry
            .saved
            .is_some_and(|saved| !entry.physical.contains(&saved.address))
        {
            return Err(DisplayError::Source);
        }
        let origin = entry
            .latest
            .map(|(_, address)| crate::fx_domain::SourceOrigin {
                address,
                normalized: false,
            })
            .or(entry.saved);
        let value = match entry.first {
            // No physical candidate exists: preserve the actual supplied allocation.
            None => None,
            Some(first) => Some(if let Some(origin) = origin {
                bytes(origin.address)
            } else if entry.equal {
                first
            } else {
                return Err(DisplayError::Source);
            }),
        };
        if let Some(value) = value {
            display.workspace.write_source_point(entry.source, value)?;
        }
        if let Some((_, at)) = entry.latest {
            for (owner, offset) in display
                .hdma_live
                .workspace_aliases(&display.workspace, entry.point)?
            {
                display.hdma_live.note_native_origin(owner, offset, at)?;
            }
        }
    }
    Ok(())
}

fn source_counterpart(
    display: &crate::scene::state::DisplayState,
    owner: LiveOwner,
    offset: usize,
) -> Result<Option<usize>, crate::scene::state::DisplayError> {
    if let Some(point) = display.hdma_live.workspace_point(owner, offset)? {
        return display.workspace.alias_address(point);
    }
    if let Some((field, lane)) = display.hdma_live.window_control_lane(owner, offset)? {
        return Ok(Some(crate::menu_window_encoding::address(field) + lane));
    }
    if let LiveOwner::ObjectVariable { object, variable } = owner {
        use crate::scene::hdma_sources::HdmaVariable;
        let field = match variable {
            HdmaVariable::A => 0,
            HdmaVariable::B => 1,
            HdmaVariable::C => 2,
            HdmaVariable::D => 3,
            HdmaVariable::E => 4,
            HdmaVariable::F => 5,
        };
        return Ok(Some(0x1914 + 12 * field + 2 * usize::from(object) + offset));
    }
    Ok(None)
}

/// Carry an explicitly normalized physical selection into the same source's
/// checkpoint metadata. Never borrow an unrelated branch's native write order.
pub fn note_source_normalization(
    session: &mut crate::session::Session,
    at: usize,
) -> Result<(), crate::scene::state::DisplayError> {
    let Some((owner, offset)) = physical_source_coordinate(session, at)? else {
        return Ok(());
    };
    let graph = control_graph(session)?;
    let live = session_live(session)?;
    let selected = selected_source_owner(
        live,
        session.presentation.workspace(),
        &graph,
        owner,
        offset,
    )?;
    let mut aliases = std::collections::BTreeSet::new();
    for region in live.checkpoint_parts().1.values() {
        for offset in 0..region.length {
            if selected_source_owner(
                live,
                session.presentation.workspace(),
                &graph,
                region.owner,
                offset,
            )? == selected
            {
                aliases.insert(live.canonical_coordinate(region.owner, offset)?);
            }
        }
    }
    let live = match &mut session.fx_sources {
        Some(live) => live,
        None => &mut session.presentation.owner_mut()?.state.hdma_live,
    };
    for (owner, offset) in aliases {
        live.note_source_normalization(owner, offset, at)?;
    }
    Ok(())
}

/// Select the current physical authority of one actual canonical source byte.
/// None means no physical publication exists and its supplied owner stays live.
pub fn suspended_source_address(
    display: &crate::scene::state::DisplayState,
    owner: LiveOwner,
    offset: usize,
    bytes: &impl Fn(usize) -> u8,
    order: &impl Fn(usize) -> Option<u64>,
) -> Result<Option<usize>, crate::scene::state::DisplayError> {
    let graph = display_control_graph(display)?;
    let selected = selected_source_owner(
        &display.hdma_live,
        &display.workspace,
        &graph,
        owner,
        offset,
    )?;
    Ok(
        display_owner_address(display, &graph, selected, bytes, order)?.or_else(|| {
            if let ControlOwner::Scalar(lane) = selected {
                Some(control_address(lane))
            } else {
                None
            }
        }),
    )
}
pub fn suspended_source_byte(
    display: &crate::scene::state::DisplayState,
    window: &crate::menu_window_state::WindowState,
    owner: LiveOwner,
    offset: usize,
    bytes: &impl Fn(usize) -> u8,
    order: &impl Fn(usize) -> Option<u64>,
) -> Result<u8, crate::scene::state::DisplayError> {
    let graph = display_control_graph(display)?;
    let selected = selected_source_owner(
        &display.hdma_live,
        &display.workspace,
        &graph,
        owner,
        offset,
    )?;
    display_owner_byte(display, window, &graph, selected, bytes, order)
}

/// Compatibility identities for FX38, Window38 and common HDMA172 bytes.
pub fn control_address(lane: usize) -> usize {
    assert!(lane < crate::fx_state::ControlOwnership::LANES);
    match lane {
        0..=3 => 0x1982 + lane,
        4..=25 => address(Field::ALL[(lane - 4) / 2]) + ((lane - 4) % 2),
        26..=37 => {
            lighting_address(crate::fx_lighting::Field::ALL[(lane - 26) / 2]) + ((lane - 26) % 2)
        }
        38..=75 => {
            crate::menu_window_encoding::address(
                crate::menu_window_state::Field::ALL[(lane - 38) / 2],
            ) + ((lane - 38) % 2)
        }
        _ => 0x18b0 + lane - 76,
    }
}
pub fn control_lane(at: usize) -> Option<usize> {
    if (0x18b0..0x195c).contains(&at) {
        return Some(76 + at - 0x18b0);
    }
    // Every projected byte asks this, so answer from a table of the fixed lanes.
    static LANES: std::sync::OnceLock<Box<[u8; 0x20000]>> = std::sync::OnceLock::new();
    let lanes = LANES.get_or_init(|| {
        let mut lanes = Box::new([u8::MAX; 0x20000]);
        for lane in (0..76).rev() {
            lanes[control_address(lane)] = lane as u8;
        }
        lanes
    });
    match lanes.get(at) {
        Some(&lane) if lane != u8::MAX => Some(usize::from(lane)),
        _ => None,
    }
}

/// One terminal provider after resolving the actual topology. Scalar cycles
/// have one representative; reads and stores never recurse through providers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ControlOwner {
    Scalar(usize),
    Workspace(crate::presentation_workspace::SourcePoint),
    Source(LiveOwner, usize),
}
pub(crate) type ControlGraph = [ControlOwner; crate::fx_state::ControlOwnership::LANES];

fn session_live(
    session: &crate::session::Session,
) -> Result<&HdmaLiveDescription, crate::scene::state::DisplayError> {
    match &session.fx_sources {
        Some(live) => Ok(live),
        None => Ok(&session.presentation.owner()?.state.hdma_live),
    }
}
fn source_owner(
    live: &HdmaLiveDescription,
    workspace: &crate::presentation_workspace::Workspace,
    owner: LiveOwner,
    offset: usize,
) -> Result<ControlOwner, crate::scene::state::DisplayError> {
    if let Some(point) = live.workspace_coordinate(workspace, owner, offset)? {
        return Ok(ControlOwner::Workspace(point));
    }
    let (owner, offset) = live.canonical_coordinate(owner, offset)?;
    if let Some((field, byte)) = live.window_control_lane(owner, offset)? {
        return Ok(ControlOwner::Scalar(
            control_lane(crate::menu_window_encoding::address(field) + byte).unwrap(),
        ));
    }
    if let LiveOwner::ObjectVariable { object, variable } = owner {
        use crate::scene::hdma_sources::HdmaVariable;
        let column = match variable {
            HdmaVariable::A => 0,
            HdmaVariable::B => 1,
            HdmaVariable::C => 2,
            HdmaVariable::D => 3,
            HdmaVariable::E => 4,
            HdmaVariable::F => 5,
        };
        return Ok(ControlOwner::Scalar(
            76 + (0x1914 - 0x18b0) + 12 * column + 2 * usize::from(object) + offset,
        ));
    }
    Ok(ControlOwner::Source(owner, offset))
}
pub(crate) fn control_graph(
    session: &crate::session::Session,
) -> Result<ControlGraph, crate::scene::state::DisplayError> {
    let edges = core::array::from_fn(ControlOwner::Scalar);
    if !session.presentation.is_owned() {
        return Ok(edges);
    }
    let live = session_live(session)?;
    let edges = control_edges(live, session.presentation.workspace(), &|at| {
        physical_source_coordinate(session, at)
    })?;
    Ok(resolve_control_edges(edges))
}
fn control_edges(
    live: &HdmaLiveDescription,
    workspace: &crate::presentation_workspace::Workspace,
    coordinate: &impl Fn(usize) -> Result<Option<(LiveOwner, usize)>, crate::scene::state::DisplayError>,
) -> Result<ControlGraph, crate::scene::state::DisplayError> {
    let mut edges = core::array::from_fn(ControlOwner::Scalar);
    for (lane, edge) in edges.iter_mut().enumerate() {
        // An explicit transferred Window lane takes precedence over nominal
        // address inference. A relocated former buffer lane is real source data.
        if (38..76).contains(&lane)
            && live
                .window_control_layout()
                .is_some_and(|layout| layout.lanes[lane - 38].is_some())
        {
            continue;
        }
        if let Some((owner, offset)) = coordinate(control_address(lane))? {
            *edge = source_owner(live, workspace, owner, offset)?;
        }
    }
    Ok(edges)
}
fn resolve_control_edges(edges: ControlGraph) -> ControlGraph {
    let mut graph = edges;
    for (start, terminal) in graph.iter_mut().enumerate() {
        let mut path = [0; crate::fx_state::ControlOwnership::LANES];
        let mut count = 0;
        let mut selected = ControlOwner::Scalar(start);
        while let ControlOwner::Scalar(lane) = selected {
            if let Some(cycle) = path[..count].iter().position(|&prior| prior == lane) {
                selected = ControlOwner::Scalar(*path[cycle..count].iter().min().unwrap());
                break;
            }
            path[count] = lane;
            count += 1;
            selected = edges[lane];
        }
        *terminal = selected;
    }
    graph
}

pub(crate) fn common_scalar_word(common: &crate::hdma::HdmaObjects, at: usize) -> u16 {
    match at {
        0x18b0 => common.enabled,
        0x18b2 => common.selected,
        _ => {
            let word = (at - 0x18b4) / 2;
            let object = &common.objects[word % 6];
            match word / 6 {
                0 => object.channels,
                1 => object.bank_slot,
                2 => object.instruction_list,
                3 => object.table,
                4 => object.instruction_timer,
                5 => object.pre_instruction,
                6 => object.pre_instruction_bank,
                7 => object.timer,
                8 => object.a,
                9 => object.b,
                10 => object.c,
                11 => object.d,
                12 => object.e,
                13 => object.f,
                _ => unreachable!(),
            }
        }
    }
}
pub(crate) fn common_scalar_word_mut(common: &mut crate::hdma::HdmaObjects, at: usize) -> &mut u16 {
    match at {
        0x18b0 => &mut common.enabled,
        0x18b2 => &mut common.selected,
        _ => {
            let word = (at - 0x18b4) / 2;
            let object = &mut common.objects[word % 6];
            match word / 6 {
                0 => &mut object.channels,
                1 => &mut object.bank_slot,
                2 => &mut object.instruction_list,
                3 => &mut object.table,
                4 => &mut object.instruction_timer,
                5 => &mut object.pre_instruction,
                6 => &mut object.pre_instruction_bank,
                7 => &mut object.timer,
                8 => &mut object.a,
                9 => &mut object.b,
                10 => &mut object.c,
                11 => &mut object.d,
                12 => &mut object.e,
                13 => &mut object.f,
                _ => unreachable!(),
            }
        }
    }
}
fn scalar_control_byte(session: &crate::session::Session, at: usize) -> Option<u8> {
    match at {
        0x1982..=0x1983 => Some(session.room.fx_blending.to_le_bytes()[at - 0x1982]),
        0x1984..=0x1985 => Some(session.room.fx_layer3_blending.to_le_bytes()[at - 0x1984]),
        0x18b0..=0x195b => {
            Some(common_scalar_word(session.hdma_scalar_objects(), at & !1).to_le_bytes()[at % 2])
        }
        _ => crate::menu_window_encoding::projected_byte(&session.menu_window, at)
            .or_else(|| projected_byte(&session.fx, at))
            .or_else(|| projected_lighting_byte(&session.presentation.visual().fireflea, at)),
    }
}
fn set_scalar_control_byte(session: &mut crate::session::Session, at: usize, byte: u8) {
    let (word, first) = match at {
        0x1982..=0x1983 => (&mut session.room.fx_blending, 0x1982),
        0x1984..=0x1985 => (&mut session.room.fx_layer3_blending, 0x1984),
        0x18b0..=0x195b => (
            common_scalar_word_mut(session.hdma_scalar_objects_mut(), at & !1),
            at & !1,
        ),
        _ => {
            if !crate::menu_window_encoding::write_byte(&mut session.menu_window, at, byte)
                && !write_byte(&mut session.fx, at, byte)
            {
                assert!(write_lighting_byte(
                    &mut session.presentation.visual_mut().fireflea,
                    at,
                    byte
                ));
            }
            return;
        }
    };
    let mut bytes = word.to_le_bytes();
    bytes[at - first] = byte;
    *word = u16::from_le_bytes(bytes);
}
fn ownership_for(
    session: &crate::session::Session,
    graph: &ControlGraph,
) -> crate::fx_state::ControlOwnership {
    let mut ownership = crate::fx_state::ControlOwnership {
        initialized: session.presentation.is_owned(),
        ..Default::default()
    };
    for (lane, &owner) in graph.iter().enumerate() {
        if owner != ControlOwner::Scalar(lane) {
            ownership.select_source(lane);
        }
    }
    ownership
}
pub fn validate_control_ownership(
    session: &crate::session::Session,
) -> Result<(), crate::scene::state::DisplayError> {
    let graph = control_graph(session)?;
    if session.fx_control_ownership != ownership_for(session, &graph) {
        return Err(crate::scene::state::DisplayError::Source);
    }
    for (lane, &owner) in graph.iter().enumerate() {
        if owner != ControlOwner::Scalar(lane)
            && scalar_control_byte(session, control_address(lane)) != Some(0)
        {
            return Err(crate::scene::state::DisplayError::Source);
        }
    }
    if session.presentation.is_owned() && session.fx_sources.is_none() {
        let display = &session.presentation.owner()?.state;
        for (&(owner, offset), origin) in display.hdma_live.native_origins() {
            let selected = selected_source_owner(
                &display.hdma_live,
                &display.workspace,
                &graph,
                owner,
                offset,
            )?;
            let mut matches = false;
            visit_owner_addresses(display, &graph, selected, |at| {
                matches |= at == origin.address
            })?;
            if !matches {
                return Err(crate::scene::state::DisplayError::Source);
            }
        }
    }
    Ok(())
}
/// Resolve the completed topology before mutating it. Terminal buffers own their
/// real bytes; scalar aliases have one storage representative and inactive zero.
/// Conflicting scalar cycles require actual saved publication selection evidence.
fn scalar_supplied(session: &crate::session::Session, lane: usize) -> bool {
    match lane {
        4..=25 => session.fx.is_initialized(),
        38..=75 => session.menu_window.is_initialized(),
        _ => true,
    }
}
pub fn adopt_control_ownership(
    session: &mut crate::session::Session,
) -> Result<(), crate::scene::state::DisplayError> {
    let graph = control_graph(session)?;
    let existing = session.fx_control_ownership.initialized;
    let edges = if session.presentation.is_owned() {
        control_edges(
            session_live(session)?,
            session.presentation.workspace(),
            &|at| physical_source_coordinate(session, at),
        )?
    } else {
        core::array::from_fn(ControlOwner::Scalar)
    };
    for (lane, &owner) in graph.iter().enumerate() {
        if owner == ControlOwner::Scalar(lane) && !existing {
            let mut cycle = Vec::new();
            let mut next = lane;
            loop {
                if cycle.contains(&next) {
                    break;
                }
                cycle.push(next);
                let ControlOwner::Scalar(selected) = edges[next] else {
                    break;
                };
                next = selected;
            }
            if next == lane && cycle.len() > 1 {
                let mut first = None;
                let mut equal = true;
                let mut selected = None;
                let mut conflicting_selection = false;
                for &candidate in &cycle {
                    if !scalar_supplied(session, candidate) {
                        continue;
                    }
                    let value = scalar_control_byte(session, control_address(candidate)).unwrap();
                    if let Some(previous) = first {
                        equal &= previous == value;
                    } else {
                        first = Some(value);
                    }
                    if let Some((source, offset)) =
                        physical_source_coordinate(session, control_address(candidate))?
                    {
                        let live = session_live(session)?;
                        let coordinate = live.canonical_coordinate(source, offset)?;
                        if let Some(origin) = live.native_origins().get(&coordinate) {
                            // Saved native origins identify actual selected source
                            // bytes. They are evidence, never fabricated ordinals.
                            // The origin names the supplied physical candidate,
                            // not the destination selected by graph routing.
                            // Outside the scalar class no raw candidate evidence
                            // was supplied; native import must settle that case.
                            let origin_lane = control_lane(origin.address)
                                .filter(|&lane| scalar_supplied(session, lane));
                            let origin_lane = origin_lane
                                .filter(|lane| cycle.contains(lane))
                                .ok_or(crate::scene::state::DisplayError::Source)?;
                            let value =
                                scalar_control_byte(session, control_address(origin_lane)).unwrap();
                            conflicting_selection |=
                                selected.is_some_and(|previous| previous != value);
                            selected = Some(value);
                        }
                    }
                }
                let value = if equal {
                    first.ok_or(crate::scene::state::DisplayError::Incomplete)?
                } else if !conflicting_selection {
                    selected.ok_or(crate::scene::state::DisplayError::Source)?
                } else {
                    return Err(crate::scene::state::DisplayError::Source);
                };
                set_scalar_control_byte(session, control_address(lane), value);
            }
        }
        if owner != ControlOwner::Scalar(lane) {
            set_scalar_control_byte(session, control_address(lane), 0);
        }
    }
    session.fx_control_ownership = ownership_for(session, &graph);
    Ok(())
}
fn owner_byte(
    session: &crate::session::Session,
    owner: ControlOwner,
) -> Result<u8, crate::scene::state::DisplayError> {
    match owner {
        ControlOwner::Scalar(lane) => {
            if (38..76).contains(&lane) && !session.menu_window.is_initialized() {
                return Err(crate::scene::state::DisplayError::Incomplete);
            }
            Ok(scalar_control_byte(session, control_address(lane)).unwrap())
        }
        ControlOwner::Workspace(point) => session.presentation.workspace().read_source_point(point),
        ControlOwner::Source(owner, offset) => session_live(session)?.read_buffer(owner, offset),
    }
}
fn store_owner_byte(
    session: &mut crate::session::Session,
    owner: ControlOwner,
    byte: u8,
) -> Result<(), crate::scene::state::DisplayError> {
    match owner {
        ControlOwner::Scalar(lane) => {
            if (38..76).contains(&lane) && !session.menu_window.is_initialized() {
                return Err(crate::scene::state::DisplayError::Incomplete);
            }
            set_scalar_control_byte(session, control_address(lane), byte);
            Ok(())
        }
        ControlOwner::Workspace(point) => session
            .presentation
            .workspace_mut()
            .write_source_point(point, byte),
        ControlOwner::Source(owner, offset) => {
            let live = match &mut session.fx_sources {
                Some(live) => live,
                None => &mut session.presentation.owner_mut()?.state.hdma_live,
            };
            live.write_buffer(owner, offset, byte)
        }
    }
}
fn forget_owner_origins(
    session: &mut crate::session::Session,
    graph: &ControlGraph,
    selected: ControlOwner,
) -> Result<(), crate::scene::state::DisplayError> {
    if !session.presentation.is_owned() {
        return Ok(());
    }
    let live = session_live(session)?;
    let mut origins = Vec::new();
    for &(owner, offset) in live.native_origins().keys() {
        if selected_source_owner(live, session.presentation.workspace(), graph, owner, offset)?
            == selected
        {
            origins.push((owner, offset));
        }
    }
    let live = match &mut session.fx_sources {
        Some(live) => live,
        None => &mut session.presentation.owner_mut()?.state.hdma_live,
    };
    for (owner, offset) in origins {
        live.forget_native_origin(owner, offset)?;
    }
    Ok(())
}
pub fn control_byte(session: &crate::session::Session, at: usize) -> u8 {
    ControlReader::new(session).control_byte(at)
}
pub fn control_word(session: &crate::session::Session, at: usize) -> u16 {
    ControlReader::new(session).control_word(at)
}

/// Reads one unchanging Session's control and source bytes. A Session without
/// an owned Display has the identity control graph, so none is built; an owned
/// one builds its graph once, on first use, for every later read.
pub struct ControlReader<'s> {
    session: &'s crate::session::Session,
    graph: std::cell::OnceCell<ControlGraph>,
}
impl<'s> ControlReader<'s> {
    pub fn new(session: &'s crate::session::Session) -> Self {
        Self {
            session,
            graph: std::cell::OnceCell::new(),
        }
    }
    /// `control_graph`, or `None` for the identity graph of an unowned Display.
    fn graph(&self) -> Result<Option<&ControlGraph>, crate::scene::state::DisplayError> {
        if !self.session.presentation.is_owned() {
            return Ok(None);
        }
        if let Some(graph) = self.graph.get() {
            return Ok(Some(graph));
        }
        let graph = control_graph(self.session)?;
        Ok(Some(self.graph.get_or_init(|| graph)))
    }
    pub fn control_byte(&self, at: usize) -> u8 {
        let session = self.session;
        let lane = control_lane(at).expect("migrated control byte");
        let selected = if session.presentation.is_owned() && session.fx_sources.is_none() {
            let display = &session
                .presentation
                .owner()
                .expect("current control display")
                .state;
            current_control_owner(&display.hdma_live, &display.workspace, &display.hdma, lane)
                .expect("current control topology")
        } else {
            self.graph()
                .expect("control topology")
                .map_or(ControlOwner::Scalar(lane), |graph| graph[lane])
        };
        owner_byte(session, selected).expect("control owner")
    }
    pub fn control_word(&self, at: usize) -> u16 {
        let first = self.control_byte(at);
        let second = self.control_byte(at + 1);
        u16::from_le_bytes([first, second])
    }
    /// `projected_source_byte` through this reader's graph.
    pub fn projected_source_byte(&self, at: usize) -> Option<u8> {
        let session = self.session;
        if session.fx_sources.is_none() && !session.presentation.is_owned() {
            return None;
        }
        let (owner, offset) =
            physical_source_coordinate(session, at).expect("prepared FX source resolution")?;
        Some(
            self.source_byte(owner, offset)
                .expect("prepared FX source projection"),
        )
    }
    /// `session_source_byte` through this reader's graph.
    pub fn source_byte(
        &self,
        owner: LiveOwner,
        offset: usize,
    ) -> Result<u8, crate::scene::state::DisplayError> {
        let session = self.session;
        let selected = if session.presentation.is_owned() && session.fx_sources.is_none() {
            let display = &session.presentation.owner()?.state;
            current_source_owner(
                &display.hdma_live,
                &display.workspace,
                &display.hdma,
                owner,
                offset,
            )?
        } else {
            let live = session_live(session)?;
            let graph = self.graph()?;
            match source_owner(live, session.presentation.workspace(), owner, offset)? {
                ControlOwner::Scalar(lane) => {
                    graph.map_or(ControlOwner::Scalar(lane), |graph| graph[lane])
                }
                // source_owner returns a Source only once its window lane is ruled out.
                ControlOwner::Source(owner, offset) => {
                    return live.read_unbound_buffer(owner, offset);
                }
                selected => selected,
            }
        };
        owner_byte(session, selected)
    }
}
pub fn write_control_byte(session: &mut crate::session::Session, at: usize, byte: u8) {
    let lane = control_lane(at).expect("migrated control byte");
    if session.presentation.is_owned() && session.fx_sources.is_none() {
        let display = &session
            .presentation
            .owner()
            .expect("current control display")
            .state;
        let selected =
            current_control_owner(&display.hdma_live, &display.workspace, &display.hdma, lane)
                .expect("current control topology");
        store_owner_byte(session, selected, byte).expect("control publication");
        let display = &mut session
            .presentation
            .owner_mut()
            .expect("current control display")
            .state;
        crate::fx_sources::forget_current_origins(
            &mut display.hdma_live,
            &display.workspace,
            &display.hdma,
            selected,
        );
    } else if !session.presentation.is_owned() {
        // The identity graph: the lane's own scalar, and no origins to forget.
        store_owner_byte(session, ControlOwner::Scalar(lane), byte).expect("control publication");
    } else {
        let graph = control_graph(session).expect("control topology");
        store_owner_byte(session, graph[lane], byte).expect("control publication");
        forget_owner_origins(session, &graph, graph[lane]).expect("control selection publication");
    }
}
pub fn write_control_word(session: &mut crate::session::Session, at: usize, value: u16) {
    let [first, second] = value.to_le_bytes();
    write_control_byte(session, at, first);
    write_control_byte(session, at + 1, second);
}
/// Native scoped adoption selects current providers but retains publication
/// provenance; it does not manufacture a new typed producer ordinal.
pub fn import_window(
    session: &mut crate::session::Session,
    window: crate::menu_window_state::WindowState,
) {
    if !session.fx_control_ownership.initialized {
        session.menu_window = window;
        return;
    }
    let graph = control_graph(session).expect("Window adoption topology");
    for field in crate::menu_window_state::Field::ALL {
        let at = crate::menu_window_encoding::address(field);
        for (byte_lane, byte) in field.get(&window).to_le_bytes().into_iter().enumerate() {
            let lane = control_lane(at + byte_lane).unwrap();
            store_owner_byte(session, graph[lane], byte).expect("Window adoption provider");
        }
    }
}
pub fn materialize_controls(
    session: &mut crate::session::Session,
) -> Result<(), crate::scene::state::DisplayError> {
    validate_control_ownership(session)?;
    let graph = control_graph(session)?;
    let mut values = [0; crate::fx_state::ControlOwnership::LANES];
    for (lane, byte) in values.iter_mut().enumerate() {
        *byte = owner_byte(session, graph[lane])?;
    }
    session.fx_control_ownership = Default::default();
    for (lane, value) in values.into_iter().enumerate() {
        set_scalar_control_byte(session, control_address(lane), value);
    }
    Ok(())
}
pub fn session_controls(session: &crate::session::Session) -> FxState {
    let reader = ControlReader::new(session);
    decode(&|at| reader.control_byte(at))
}
pub fn session_lighting(
    session: &crate::session::Session,
) -> crate::visual_state::FirefleaLighting {
    let reader = ControlReader::new(session);
    decode_lighting(&|at| reader.control_byte(at))
}
pub fn session_room_blending(session: &crate::session::Session) -> crate::fx_domain::RoomBlending {
    let reader = ControlReader::new(session);
    crate::fx_domain::RoomBlending {
        primary: reader.control_word(0x1982),
        liquid: reader.control_word(0x1984),
    }
}
pub fn session_window(session: &crate::session::Session) -> crate::menu_window_state::WindowState {
    let reader = ControlReader::new(session);
    crate::menu_window_encoding::decode(&|at| reader.control_byte(at))
}
pub fn session_common(session: &crate::session::Session) -> crate::hdma::HdmaObjects {
    let reader = ControlReader::new(session);
    let mut common = crate::hdma::HdmaObjects::default();
    for at in (0x18b0..0x195c).step_by(2) {
        *common_scalar_word_mut(&mut common, at) = reader.control_word(at);
    }
    common
}

/// Resolve suspended typed consumers using the same actual alias classes and
/// real WRAM publication ranks as source consumers. This is a narrow native
/// counterpart adapter; ordinary raw RAM reads remain physical bus reads.
pub(crate) fn display_control_edges(
    display: &crate::scene::state::DisplayState,
) -> Result<ControlGraph, crate::scene::state::DisplayError> {
    control_edges(&display.hdma_live, &display.workspace, &|at| {
        display_source_coordinate(display, at)
    })
}
pub(crate) fn display_control_graph(
    display: &crate::scene::state::DisplayState,
) -> Result<ControlGraph, crate::scene::state::DisplayError> {
    Ok(resolve_control_edges(display_control_edges(display)?))
}
/// Follow only the requested current provider path. Unrelated source topology
/// cannot reject an earlier original byte query.
pub(crate) fn current_source_owner(
    live: &HdmaLiveDescription,
    workspace: &crate::presentation_workspace::Workspace,
    state: &crate::scene::state::HdmaState,
    owner: LiveOwner,
    offset: usize,
) -> Result<ControlOwner, crate::scene::state::DisplayError> {
    resolve_current_owner(
        live,
        workspace,
        state,
        source_owner(live, workspace, owner, offset)?,
    )
}
pub(crate) fn current_control_owner(
    live: &HdmaLiveDescription,
    workspace: &crate::presentation_workspace::Workspace,
    state: &crate::scene::state::HdmaState,
    lane: usize,
) -> Result<ControlOwner, crate::scene::state::DisplayError> {
    resolve_current_owner(live, workspace, state, ControlOwner::Scalar(lane))
}
fn resolve_current_owner(
    live: &HdmaLiveDescription,
    workspace: &crate::presentation_workspace::Workspace,
    state: &crate::scene::state::HdmaState,
    mut selected: ControlOwner,
) -> Result<ControlOwner, crate::scene::state::DisplayError> {
    let mut path = [0; crate::fx_state::ControlOwnership::LANES];
    let mut count = 0;
    loop {
        let ControlOwner::Scalar(lane) = selected else {
            return Ok(selected);
        };
        if let Some(cycle) = path[..count].iter().position(|&prior| prior == lane) {
            return Ok(ControlOwner::Scalar(
                *path[cycle..count].iter().min().unwrap(),
            ));
        }
        path[count] = lane;
        count += 1;
        if (38..76).contains(&lane)
            && live
                .window_control_layout()
                .is_some_and(|layout| layout.lanes[lane - 38].is_some())
        {
            return Ok(selected);
        }
        let at = control_address(lane);
        let bank = 0x7e
            + u8::try_from(at / 65_536).map_err(|_| crate::scene::state::DisplayError::Source)?;
        let address =
            u16::try_from(at % 65_536).map_err(|_| crate::scene::state::DisplayError::Source)?;
        let Some((owner, offset)) =
            live.physical_coordinate_with_workspace(state, bank, address, workspace)?
        else {
            return Ok(selected);
        };
        selected = source_owner(live, workspace, owner, offset)?;
    }
}

pub(crate) fn selected_source_owner(
    live: &HdmaLiveDescription,
    workspace: &crate::presentation_workspace::Workspace,
    graph: &ControlGraph,
    owner: LiveOwner,
    offset: usize,
) -> Result<ControlOwner, crate::scene::state::DisplayError> {
    Ok(match source_owner(live, workspace, owner, offset)? {
        ControlOwner::Scalar(lane) => graph[lane],
        owner => owner,
    })
}
pub(crate) fn visit_owner_addresses(
    display: &crate::scene::state::DisplayState,
    graph: &ControlGraph,
    selected: ControlOwner,
    mut visit: impl FnMut(usize),
) -> Result<(), crate::scene::state::DisplayError> {
    for binding in display
        .hdma
        .channels
        .iter()
        .flat_map(|channel| [&channel.table, &channel.source])
        .chain(
            display
                .hdma
                .objects
                .iter()
                .map(|object| &object.instruction),
        )
    {
        let crate::scene::hdma_sources::SourceBinding::LiveOwner { owner, windows } = binding
        else {
            continue;
        };
        for window in windows.iter() {
            for displacement in 0..window.length {
                if selected_source_owner(
                    &display.hdma_live,
                    &display.workspace,
                    graph,
                    *owner,
                    window.offset + displacement,
                )? != selected
                {
                    continue;
                }
                let address = u16::try_from(usize::from(window.address) + displacement)
                    .map_err(|_| crate::scene::state::DisplayError::Source)?;
                if let Some(at) = physical_work_ram(window.bank, address) {
                    visit(at);
                }
            }
        }
    }
    for (lane, &owner) in graph.iter().enumerate() {
        if owner == selected {
            visit(control_address(lane));
        }
    }
    if let ControlOwner::Workspace(point) = selected {
        display
            .workspace
            .visit_source_bus_addresses(point, |bank, address| {
                if let Some(at) = physical_work_ram(bank, address) {
                    visit(at);
                }
            })?;
    }
    Ok(())
}
fn display_owner_address(
    display: &crate::scene::state::DisplayState,
    graph: &ControlGraph,
    selected: ControlOwner,
    bytes: &impl Fn(usize) -> u8,
    order: &impl Fn(usize) -> Option<u64>,
) -> Result<Option<usize>, crate::scene::state::DisplayError> {
    let mut first = None;
    let mut equal = true;
    let mut latest = None;
    let mut saved = None;
    let mut saved_conflict = false;
    let mut admit = |at| {
        if let Some(previous) = first {
            equal &= bytes(previous) == bytes(at);
        } else {
            first = Some(at);
        }
        if let Some(rank) = order(at)
            && latest.is_none_or(|(previous, _)| rank > previous)
        {
            latest = Some((rank, at));
        }
    };
    for binding in display
        .hdma
        .channels
        .iter()
        .flat_map(|channel| [&channel.table, &channel.source])
        .chain(
            display
                .hdma
                .objects
                .iter()
                .map(|object| &object.instruction),
        )
    {
        let crate::scene::hdma_sources::SourceBinding::LiveOwner { owner, windows } = binding
        else {
            continue;
        };
        for window in windows.iter() {
            for displacement in 0..window.length {
                let offset = window.offset + displacement;
                if selected_source_owner(
                    &display.hdma_live,
                    &display.workspace,
                    graph,
                    *owner,
                    offset,
                )? != selected
                {
                    continue;
                }
                let address = u16::try_from(usize::from(window.address) + displacement)
                    .map_err(|_| crate::scene::state::DisplayError::Source)?;
                if let Some(at) = physical_work_ram(window.bank, address) {
                    admit(at);
                }
                let coordinate = display.hdma_live.canonical_coordinate(*owner, offset)?;
                if let Some(origin) = display.hdma_live.native_origins().get(&coordinate) {
                    if saved.is_some_and(|prior| bytes(prior) != bytes(origin.address)) {
                        // Genuine live publication still settles competing saved origins.
                        saved_conflict = true;
                    }
                    saved = Some(origin.address);
                }
            }
        }
    }
    // Every typed member is a real native counterpart, including Window named
    // publishers whose actual canonical layout has been relocated.
    for (lane, &owner) in graph.iter().enumerate() {
        if owner == selected {
            admit(control_address(lane));
        }
    }
    if let ControlOwner::Workspace(point) = selected {
        display
            .workspace
            .visit_source_bus_addresses(point, |bank, address| {
                if let Some(at) = physical_work_ram(bank, address) {
                    admit(at);
                }
            })?;
    }
    if latest.is_none() && (saved_conflict || saved.is_none() && !equal) {
        return Err(crate::scene::state::DisplayError::Source);
    }
    Ok(latest
        .map(|(_, at)| at)
        .or(saved)
        .or_else(|| equal.then_some(first).flatten()))
}
pub fn suspended_domain_byte(
    display: &crate::scene::state::DisplayState,
    window: &crate::menu_window_state::WindowState,
    at: usize,
    bytes: &impl Fn(usize) -> u8,
    order: &impl Fn(usize) -> Option<u64>,
) -> Result<u8, crate::scene::state::DisplayError> {
    let graph = display_control_graph(display)?;
    let selected = graph[control_lane(at).ok_or(crate::scene::state::DisplayError::Source)?];
    display_owner_byte(display, window, &graph, selected, bytes, order)
}
fn display_owner_byte(
    display: &crate::scene::state::DisplayState,
    window: &crate::menu_window_state::WindowState,
    graph: &ControlGraph,
    selected: ControlOwner,
    bytes: &impl Fn(usize) -> u8,
    order: &impl Fn(usize) -> Option<u64>,
) -> Result<u8, crate::scene::state::DisplayError> {
    if let Some(at) = display_owner_address(display, graph, selected, bytes, order)? {
        return Ok(bytes(at));
    }
    match selected {
        ControlOwner::Workspace(point) => display.workspace.read_source_point(point),
        ControlOwner::Source(owner, offset) => display.hdma_live.read_buffer(owner, offset),
        ControlOwner::Scalar(lane) => {
            let at = control_address(lane);
            if let Some(byte) = crate::menu_window_encoding::projected_byte(window, at) {
                return Ok(byte);
            }
            if (0x18b0..0x195c).contains(&at) {
                return Ok(common_scalar_word(&display.hdma.common, at & !1).to_le_bytes()[at % 2]);
            }
            Ok(bytes(at))
        }
    }
}
pub fn store_display_source(
    display: &mut crate::scene::state::DisplayState,
    owner: LiveOwner,
    offset: usize,
    byte: u8,
) -> Result<(), crate::scene::state::DisplayError> {
    let graph = display_control_graph(display)?;
    let selected = selected_source_owner(
        &display.hdma_live,
        &display.workspace,
        &graph,
        owner,
        offset,
    )?;
    match selected {
        ControlOwner::Workspace(point) => display.workspace.write_source_point(point, byte)?,
        ControlOwner::Source(owner, offset) => {
            display.hdma_live.write_buffer(owner, offset, byte)?
        }
        ControlOwner::Scalar(lane) => {
            let at = control_address(lane);
            if (0x18b0..0x195c).contains(&at) {
                let word = common_scalar_word_mut(&mut display.hdma.common, at & !1);
                let mut bytes = word.to_le_bytes();
                bytes[at % 2] = byte;
                *word = u16::from_le_bytes(bytes);
            } else if !write_lighting_byte(&mut display.visual.fireflea, at, byte) {
                return Err(crate::scene::state::DisplayError::Incomplete);
            }
        }
    }
    let mut origins = Vec::new();
    for &(owner, offset) in display.hdma_live.native_origins().keys() {
        if selected_source_owner(
            &display.hdma_live,
            &display.workspace,
            &graph,
            owner,
            offset,
        )? == selected
        {
            origins.push((owner, offset));
        }
    }
    for (owner, offset) in origins {
        display.hdma_live.forget_native_origin(owner, offset)?;
    }
    Ok(())
}
/// A separately prepared Display retains the same inactive common lanes as its
/// attached Session. No typed source value is copied into a second common slot.
pub fn normalize_display_common(
    display: &mut crate::scene::state::DisplayState,
) -> Result<(), crate::scene::state::DisplayError> {
    let graph = display_control_graph(display)?;
    for (lane, &owner) in graph.iter().enumerate().skip(76) {
        if owner != ControlOwner::Scalar(lane) {
            let at = control_address(lane);
            let word = common_scalar_word_mut(&mut display.hdma.common, at & !1);
            let mut bytes = word.to_le_bytes();
            bytes[at % 2] = 0;
            *word = u16::from_le_bytes(bytes);
        }
    }
    Ok(())
}

/// Import completed native source publications into their current terminal
/// providers, without reactivating nonrepresentative common or Window storage.
pub fn adopt_session_sources(
    session: &mut crate::session::Session,
    bytes: &impl Fn(usize) -> u8,
    order: &impl Fn(usize) -> Option<u64>,
) -> Result<(), crate::scene::state::DisplayError> {
    let graph = control_graph(session)?;
    let display = &session.presentation.owner()?.state;
    let mut coordinates = std::collections::BTreeSet::new();
    for binding in display
        .hdma
        .channels
        .iter()
        .flat_map(|channel| [&channel.table, &channel.source])
        .chain(
            display
                .hdma
                .objects
                .iter()
                .map(|object| &object.instruction),
        )
    {
        let crate::scene::hdma_sources::SourceBinding::LiveOwner { owner, windows } = binding
        else {
            continue;
        };
        for window in windows.iter() {
            for offset in window.offset..window.offset + window.length {
                coordinates.insert(display.hdma_live.canonical_coordinate(*owner, offset)?);
            }
        }
    }
    for (owner, offset) in coordinates {
        let (selected, value, origin) = {
            let display = &session.presentation.owner()?.state;
            let selected = selected_source_owner(
                &display.hdma_live,
                &display.workspace,
                &graph,
                owner,
                offset,
            )?;
            let origin = display_owner_address(display, &graph, selected, bytes, order)?;
            let value = display_owner_byte(
                display,
                &session.menu_window,
                &graph,
                selected,
                bytes,
                order,
            )?;
            (selected, value, origin)
        };
        store_owner_byte(session, selected, value)?;
        if !session.fx_control_ownership.initialized {
            // A genuine selected native origin settles cold scalar cycles.
            // Supply that selected value to each actual candidate before the
            // ordered ownership transfer deactivates its redundant storage.
            for (lane, &owner) in graph.iter().enumerate() {
                if owner == selected {
                    set_scalar_control_byte(session, control_address(lane), value);
                }
            }
        }
        if let Some(at) = origin {
            session
                .presentation
                .owner_mut()?
                .state
                .hdma_live
                .note_native_origin(owner, offset, at)?;
        }
    }
    adopt_control_ownership(session)
}

pub(crate) type ControlOrigins = std::collections::BTreeMap<(LiveOwner, usize), ControlOwner>;
pub(crate) fn capture_control_origins(
    display: &crate::scene::state::DisplayState,
    graph: &ControlGraph,
) -> Result<ControlOrigins, crate::scene::state::DisplayError> {
    display
        .hdma_live
        .native_origins()
        .keys()
        .map(|&(owner, offset)| {
            selected_source_owner(&display.hdma_live, &display.workspace, graph, owner, offset)
                .map(|selected| ((owner, offset), selected))
        })
        .collect()
}
/// At an explicit ownership transition, retire provenance whose selected class
/// changed or whose physical candidate is no longer supplied by the new owner.
pub(crate) fn reconcile_control_origins(
    display: &mut crate::scene::state::DisplayState,
    previous: &ControlOrigins,
) -> Result<(), crate::scene::state::DisplayError> {
    let graph = display_control_graph(display)?;
    let mut obsolete = Vec::new();
    for (&coordinate, origin) in display.hdma_live.native_origins() {
        let selected = selected_source_owner(
            &display.hdma_live,
            &display.workspace,
            &graph,
            coordinate.0,
            coordinate.1,
        );
        let valid = match selected {
            Ok(selected) if previous.get(&coordinate).is_none_or(|&old| old == selected) => {
                let mut matches = false;
                visit_owner_addresses(display, &graph, selected, |at| {
                    matches |= at == origin.address
                })?;
                matches
            }
            _ => false,
        };
        if !valid {
            obsolete.push(coordinate);
        }
    }
    for coordinate in obsolete {
        display.hdma_live.forget_origin_coordinate(coordinate);
    }
    Ok(())
}

#[cfg(test)]
mod control_lane_table {
    use super::*;

    #[test]
    fn the_table_answers_as_the_lane_search_does() {
        for at in (0..0x18b0).chain(0x195c..0x20000) {
            let searched = (0..76).find(|&lane| control_address(lane) == at);
            assert_eq!(control_lane(at), searched, "address {at:#x}");
        }
    }

    /// `ControlReader` builds no graph without an owned Display: it is the identity.
    #[test]
    fn an_unowned_display_has_the_identity_control_graph() {
        let session = crate::session::Session::with_world(
            crate::room_entry::world::WorldBinding::canonical_legacy(),
        );
        assert!(!session.presentation.is_owned());
        let graph = control_graph(&session).unwrap();
        for (lane, &owner) in graph.iter().enumerate() {
            assert_eq!(owner, ControlOwner::Scalar(lane));
        }
    }
}
