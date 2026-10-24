//! Initial entry, door continuations and the scroll-completion interrupt.
#![forbid(unsafe_code)]
use super::{
    EntryPoll, EntrySignal, EntryWait, background,
    content::{AssetSource, LoadMode, SharedAsset, VramDestination},
    effects::{
        AssetRead, AssetSlot, BufferClear, CameraEffect as C, CameraField as F, DeviceWord,
        EntryHost, ExternalField as E, InstallBuffer, LeafEffect as L, SfxQueue, StagingSource,
        VramSlot,
    },
    installation,
};
use crate::game::{animated_tiles, enemy_projectiles, palette_effects, player_projectiles};
use std::num::NonZeroU8;

fn waiting(cause: EntryWait, value: u8) -> Option<EntryPoll> {
    NonZeroU8::new(value).map(|returned| EntryPoll::Waiting { cause, returned })
}
fn actors(h: &mut impl EntryHost) {
    h.call(L::SelectEnemies);
    h.call(L::RunEnemies);
    h.call(L::DrawScene);
    h.call(L::EnsureSamusDrawn);
}
fn next_mode(h: &mut impl EntryHost) {
    let value = h.read_external(E::GameMode).wrapping_add(1);
    h.write_external(E::GameMode, value);
}
fn room_irq(h: &mut impl EntryHost) {
    let irq = h.session().room.entry.irq_policy;
    h.write_external(E::NextIrqHandler, if irq == 0 { 4 } else { irq });
}
pub(crate) fn load_destination(h: &mut impl EntryHost) {
    installation::load_destination_cre(h);
    installation::load_door(h);
    installation::load_header(h);
    installation::load_state(h);
    installation::load_tiles_and_palette(h);
}
fn initial(h: &mut impl EntryHost) -> EntryPoll {
    crate::verify_probe::hit("coroutine.start");
    let resume = h.session().continuations.secondary;
    if resume == 0 {
        h.device_word(DeviceWord::DmaEnable, 0);
        h.session_mut().room.entry.scrolling_hook = 0;
        h.write_external(E::MusicDataIndex, 0);
        h.write_external(E::MusicTrackIndex, 0);
        h.write_external(E::TimerStatus, 0);
        h.call(L::ResetSoundQueues);
        h.session_mut().feedback.sound_disabled = 0xffff;
        h.call(L::DisableNmi);
        h.call(L::DisableIrq);
        load_destination(h);
    }
    if resume <= 1 {
        if let Some(poll) = waiting(EntryWait::Music, h.wait(EntryWait::Music)) {
            h.session_mut().continuations.secondary = 1;
            return poll;
        }
        animated_tiles::execution::clear(h.session_mut());
        h.call(L::WaitVblankClearHdma);
        h.call(L::InitializeRoomSpecialEffects);
        crate::game::room_mechanisms::clear_session(h.session_mut());
        enemy_projectiles::lifecycle::clear(h);
        palette_effects::execution::clear(h.session_mut());
        h.call(L::UpdateBeamTilesAndPalette);
        h.call(L::LoadSpritePalettes);
        h.call(L::LoadRoomAssets);
        h.call(L::LoadRoomMusic);
    }
    if resume <= 2 {
        if let Some(poll) = waiting(EntryWait::Music, h.wait(EntryWait::Music)) {
            h.session_mut().continuations.secondary = 2;
            return poll;
        }
        h.call(L::UpdateMusicTrack);
        h.clear_buffer(BufferClear::InitialBg2);
        installation::load_level(h, LoadMode::Initial);
        h.call(L::LoadFxHeader);
        background::load(h, LoadMode::Initial);
        h.camera_call(C::CalculateLayer2X);
        h.camera_call(C::CalculateLayer2Y);
        let value = h.read_camera(F::Layer2X);
        h.write_camera(F::Bg2XScroll, value);
        let value = h.read_camera(F::Layer2Y);
        h.write_camera(F::Bg2YScroll, value);
        h.camera_call(C::CalculateBgScrolls);
        h.camera_call(C::DisplayVisibleRoom);
        h.call(L::EnableNmi);
        room_irq(h);
        h.call(L::EnableIrq);
    }
    if resume <= 3 {
        if let Some(poll) = waiting(EntryWait::Music, h.wait(EntryWait::Music)) {
            h.session_mut().continuations.secondary = 3;
            return poll;
        }
        h.spawn_initial_hardcoded_object();
        h.session_mut().room.entry.transition_selector = 0xe737;
    }
    h.session_mut().continuations.secondary = 0;
    EntryPoll::Complete
}
fn state10(h: &mut impl EntryHost) -> EntryPoll {
    h.session_mut().room.entry.enemy_interlock = 1;
    h.session_mut().room.entry.elevator_interlock = 1;
    h.write_external(E::DebugMinimap, 0);
    h.session_mut().mechanisms.save_locked = 0;
    actors(h);
    installation::load_destination_cre(h);
    for index in (0..128).rev() {
        h.session_mut().visual.targets[index] = 0;
        h.session_mut().visual.targets[index + 128] = 0;
    }
    for index in [9, 10, 13, 14, 17, 18, 19, 29] {
        h.session_mut().visual.targets[index] = h.session().visual.colors[index];
    }
    if (h.session().room.entry.previous_cre | h.session().room.entry.cre) & 1 == 0 {
        for index in [20, 21, 22, 23, 28] {
            h.session_mut().visual.targets[index] = h.session().visual.colors[index];
        }
        if h.read_external(E::TimerStatus) != 0 {
            for index in [209, 210, 212, 221] {
                h.session_mut().visual.targets[index] = h.session().visual.colors[index];
            }
            h.call(L::DrawTimer);
        }
    }
    h.call(L::ClearTransitionSounds);
    h.call(L::QueueDoorSound);
    h.session_mut().feedback.sound_disabled = 0xffff;
    h.session_mut().room.entry.transition_selector = 0xe29e;
    next_mode(h);
    EntryPoll::Returned(0)
}
fn wait48(h: &mut impl EntryHost) -> EntryPoll {
    let delay = h.session().room.entry.elevator_delay.wrapping_sub(1);
    h.session_mut().room.entry.elevator_delay = delay;
    if delay & 0x8000 != 0 {
        next_mode(h);
        state10(h)
    } else {
        actors(h);
        EntryPoll::Returned(0)
    }
}
fn place_and_load(h: &mut impl EntryHost) {
    let camera = h.read_camera(F::Layer1X);
    h.session_mut().player.x = camera.wrapping_add(h.session().player.x & 0xff);
    h.write_external(E::PreviousPlayerX, h.session().player.x);
    let camera = h.read_camera(F::Layer1Y);
    h.session_mut().player.y = camera.wrapping_add(h.session().player.y & 0xff);
    h.write_external(E::PreviousPlayerY, h.session().player.y);
    h.session_mut().room.entry.completion = 0;
    let irq = if h.session().room.entry.direction & 3 == 2 {
        16
    } else {
        22
    };
    h.write_external(E::NextIrqHandler, irq);
    h.call(L::WaitVblankEnableIrq);
    if h.session().room.entry.cre & 2 != 0
        && !h
            .session()
            .world
            .skips_cre_upload(&h.session().room.entry.exit)
    {
        h.decompress(
            AssetSource::Shared(SharedAsset::CreTiles),
            InstallBuffer::CreTiles,
        );
    }
    let source = h.live_asset(AssetSlot::Tiles, AssetRead::ThreeLittleEndian);
    h.decompress(source, InstallBuffer::Tiles);
    let source = h.live_asset(AssetSlot::Palette, AssetRead::ThreeLittleEndian);
    h.decompress(source, InstallBuffer::Palette);
    h.copy_vram(
        VramDestination::Slot(VramSlot::MainTiles),
        AssetSource::Staging(StagingSource::TilesFirst),
        0x2000,
    );
    h.copy_vram(
        VramDestination::Slot(VramSlot::TilesSecond),
        AssetSource::Staging(StagingSource::TilesSecond),
        0x2000,
    );
    h.copy_vram(
        VramDestination::Slot(VramSlot::TilesThird),
        AssetSource::Staging(StagingSource::TilesThird),
        0x1000,
    );
    if h.session().room.entry.cre & 6 != 0
        && !h
            .session()
            .world
            .skips_cre_upload(&h.session().room.entry.exit)
    {
        h.copy_vram(
            VramDestination::Slot(VramSlot::CreFirst),
            AssetSource::Staging(StagingSource::CreFirst),
            0x1000,
        );
        h.copy_vram(
            VramDestination::Slot(VramSlot::CreSecond),
            AssetSource::Staging(StagingSource::CreSecond),
            0x2000,
        );
        h.copy_vram(
            VramDestination::Slot(VramSlot::BackgroundFirst),
            AssetSource::Shared(SharedAsset::TransitionBackground),
            0x1000,
        );
    }
    if h.session().room.entry.direction & 3 == 3 {
        h.write_external(E::NextIrqHandler, 16);
    }
    h.session_mut().room.entry.transition_selector = 0xe4a9;
}
fn more(h: &mut impl EntryHost) -> EntryPoll {
    crate::verify_probe::hit("coroutine.door");
    let resume = h.session().continuations.primary;
    if resume == 0 {
        h.call(L::LoadEnemyGraphics);
        h.call(L::LoadRoomMusic);
        enemy_projectiles::lifecycle::clear(h);
        animated_tiles::execution::clear(h.session_mut());
        palette_effects::execution::clear(h.session_mut());
        crate::game::room_mechanisms::clear_session(h.session_mut());
        installation::initialize_door_objects(h);
        h.call(L::LoadFxHeader);
        h.close_door();
        h.call(L::UpdateBeamTilesAndPalette);
        h.call(L::LoadSpritePalettes);
        h.call(L::LoadRoomAssets);
        h.call(L::InitializeEnemies);
        player_projectiles::motion::reset(h);
        h.call(L::LoadSuitTargetPalette);
        background::load(h, LoadMode::Door);
    }
    if resume <= 1 {
        let mut wait = resume == 1;
        loop {
            if wait {
                if let Some(poll) = waiting(EntryWait::Nmi, h.wait(EntryWait::Nmi)) {
                    h.session_mut().continuations.primary = 1;
                    return poll;
                }
            }
            if h.session().room.entry.completion & 0x8000 != 0 {
                break;
            }
            wait = true;
        }
        h.session_mut().visual.colors[196] = 15328;
        h.call(L::SpawnBg3ScrollHdma);
        let enabled = h.read_external(E::HdmaEnabled) | 0x8000;
        h.write_external(E::HdmaEnabled, enabled);
    }
    if resume <= 2 {
        if let Some(poll) = waiting(EntryWait::Mechanisms, h.wait(EntryWait::Mechanisms)) {
            h.session_mut().continuations.primary = 2;
            return poll;
        }
        if h.session().room.entry.direction & 2 == 0 {
            if h.session().room.entry.direction & 3 != 0 {
                h.session_mut().player.x &= 0xfff8;
            } else {
                h.session_mut().player.x |= 7;
            }
        }
        h.session_mut().room.entry.transition_selector = 0xe659;
    }
    h.session_mut().continuations.primary = 0;
    EntryPoll::Returned(0)
}
fn finish_transition(h: &mut impl EntryHost) {
    let x = h.session().player.x;
    match x & 0xf0 {
        0x10 => h.session_mut().player.x = (x | 0xf).wrapping_add(8),
        0xe0 => h.session_mut().player.x = (x & 0xfff0).wrapping_sub(8),
        _ => {}
    }
    let y = h.session().player.y;
    if y & 0xf0 == 0x10 {
        h.session_mut().player.y = (y | 0xf).wrapping_add(8);
    }
    h.clear_buffer(BufferClear::TransitionXray);
    room_irq(h);
    if h.session().room.entry.elevator_flags != 0 {
        let command = if h.session().room.entry.elevator_direction & 0x8000 == 0 {
            7
        } else {
            0
        };
        h.samus_command(command);
    }
    h.call(L::SetLiquidPhysics);
    h.session_mut().room.entry.transition_selector = 0xe737;
    let display = h.read_external(E::DisplayAndObjectSelect) | 0x1f;
    h.write_external(E::DisplayAndObjectSelect, display);
}
/// Full numeric address is retained at the compatibility dispatch boundary.
pub(crate) fn dispatch(h: &mut impl EntryHost, address: u32) -> EntryPoll {
    match address {
        0x82e17d => {
            if h.session().room.entry.elevator_flags == 0 {
                next_mode(h);
                return state10(h);
            }
            h.samus_command(0);
            if h.session().room.entry.elevator_direction & 0x8000 != 0 {
                next_mode(h);
                return state10(h);
            }
            h.session_mut().room.entry.elevator_delay = 48;
            h.session_mut().room.entry.transition_selector = 0xe19f;
            return wait48(h);
        }
        0x82e19f => return wait48(h),
        0x82e29e => {
            actors(h);
            if h.sfx_empty(SfxQueue::One)
                && h.sfx_empty(SfxQueue::Two)
                && h.sfx_empty(SfxQueue::Three)
            {
                h.session_mut().room.entry.transition_selector = 0xe2db;
            }
        }
        0x82e2db => {
            if h.palette_fade() != 0 {
                h.session_mut().room.entry.transition_selector = 0xe2f7;
            } else {
                actors(h);
            }
        }
        0x82e2f7 => {
            installation::load_door(h);
            h.call(L::StopHdmaEffects);
            let enabled = h.read_external(E::HdmaEnabled) & 0x7fff;
            h.write_external(E::HdmaEnabled, enabled);
            h.write_external(E::NextIrqHandler, 8);
            h.session_mut().room.entry.transition_selector = 0xe310;
        }
        0x82e310 => {
            let axis = if h.session().room.entry.direction & 2 != 0 {
                F::Layer1X
            } else {
                F::Layer1Y
            };
            let position = h.read_camera(axis);
            let aligned = position as u8 == 0;
            if !aligned {
                let value = if position & 0x80 != 0 {
                    position.wrapping_add(1)
                } else {
                    position.wrapping_sub(1)
                };
                h.write_camera(axis, value);
            }
            h.camera_call(C::CalculateLayer2AndScrolls);
            if aligned {
                h.session_mut().room.entry.transition_selector = 0xe353;
            }
        }
        0x82e353 => {
            if h.session().room.entry.direction & 3 == 3 {
                h.camera_call(C::FixDoorMovingUp);
            }
            h.session_mut().room.entry.transition_selector = 0xe36e;
        }
        0x82e36e => {
            installation::save_departing_map(h);
            installation::load_header(h);
            installation::load_state(h);
            installation::load_destination_map(h);
            h.call(L::InitializeRoomSpecialEffects);
            installation::load_level(h, LoadMode::Door);
            h.session_mut().room.entry.transition_selector = 0xe38e;
        }
        0x82e38e => {
            h.write_camera(F::Bg2Hofs, 0);
            h.write_camera(F::Bg2Vofs, 0);
            h.session_mut().room.entry.scrolling_hook = 0;
            if h.session().room.entry.direction & 3 == 2 {
                let value = h.read_camera(F::Bg1Vofs).wrapping_add(1);
                h.write_camera(F::Bg1Vofs, value);
            }
            if h.session().room.entry.direction & 3 != 3 {
                h.session_mut().room.entry.frame = 0;
            }
            h.camera_call(C::SetupDoorScroll);
            h.session_mut().room.entry.transition_selector = 0xe3c0;
        }
        0x82e3c0 => place_and_load(h),
        0x82e4a9 => return more(h),
        0x82e659 => {
            animated_tiles::execution::advance(h);
            h.session_mut().room.entry.transition_selector = 0xe664;
        }
        0x82e664 => {
            if h.queued_music() == 0 {
                h.session_mut().room.entry.transition_selector = 0xe6a2;
                h.call(L::LoadChangedMusic);
            }
        }
        0x82e6a2 => finish_transition(h),
        0x82e737 => {
            animated_tiles::execution::advance(h);
            h.call(L::SelectEnemies);
            h.call(L::RunEnemies);
            enemy_projectiles::lifecycle::advance(h);
            h.call(L::DrawScene);
            h.call(L::EnsureSamusDrawn);
            h.call(L::QueueEnemyTiles);
            if h.palette_fade() != 0 {
                h.session_mut().feedback.sound_disabled = 0;
                h.call(L::SpinJumpSound);
                h.session_mut().room.entry.elevator_interlock = 0;
                h.session_mut().room.entry.enemy_interlock = 0;
                h.write_external(E::GameMode, 8);
                return EntryPoll::Complete;
            }
        }
        _ => return EntryPoll::legacy_unknown(h.invalid_stage(address)),
    }
    EntryPoll::Returned(0)
}
fn irq(h: &mut impl EntryHost) -> EntryPoll {
    let effect = match h.session().room.entry.direction & 3 {
        0 => C::StepDoorScrollRight,
        1 => C::StepDoorScrollLeft,
        2 => C::StepDoorScrollDown,
        _ => C::StepDoorScrollUp,
    };
    if h.camera_call(effect) != 0 {
        h.write_camera(F::Layer1X, h.session().room.entry.arrival_x);
        h.write_camera(F::Layer1Y, h.session().room.entry.arrival_y);
        h.session_mut().room.entry.completion |= 0x8000;
        EntryPoll::Complete
    } else {
        EntryPoll::Returned(0)
    }
}
pub(crate) fn resume(h: &mut impl EntryHost, signal: EntrySignal) -> EntryPoll {
    match signal {
        EntrySignal::Initial => initial(h),
        EntrySignal::DoorState10 => state10(h),
        EntrySignal::ScrollIrq => irq(h),
        EntrySignal::DoorState9 | EntrySignal::DoorState11 => {
            let address = 0x820000 | u32::from(h.session().room.entry.transition_selector);
            let poll = dispatch(h, address);
            if poll.returned_byte() == 0
                && signal == EntrySignal::DoorState11
                && h.read_external(E::TimerStatus) != 0
            {
                h.call(L::DrawTimer);
            }
            poll
        }
    }
}
