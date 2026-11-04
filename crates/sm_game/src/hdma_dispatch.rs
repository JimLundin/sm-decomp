//! The HDMA objects' indirect calls and pre-instructions, as data: each bank
//! address the original dispatch matched maps to a named target, and an
//! unknown address keeps the original unreachable failure.
use crate::explosion_window::{self, Phase};
use crate::hdma_objects::{self, HdmaEffect};
use crate::hdma_program::HdmaLease as _;
use crate::menus::host::Host;
use crate::player_transitions::xray::BeamPhase;
use crate::visual_state::{EyeBeamPhase, HazePhase, HdmaWave, TourianStatuePhase, Weather};
use crate::{liquid_effects, room_visuals, scrolling_sky, xray_background};

/// An HDMA instruction's indirect call target.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HdmaCall {
    InitializeRainbowBeam,
    BeginPreFlash,
    BeginPrescaledPreFlash,
    BeginExplosion,
    BeginPrescaledExplosion,
    FinishPowerBomb,
    BeginCrystalFlash,
    FinishCrystalFlash,
    Nothing,
    FreezeAndBackup,
    CaptureBg1Right,
    CaptureBg1Left,
    BuildOverlay,
    CaptureBg2Right,
    UploadLeft,
    UploadRight,
    SetBackdrop,
    InitializeSuitPickup,
}

/// Each bank address the original dispatch matched, in its arm order.
pub const HDMA_CALLS: [(u32, HdmaCall); 21] = [
    (0x88_e767, HdmaCall::InitializeRainbowBeam),
    (0x88_8b14, HdmaCall::BeginPreFlash),
    (0x88_8b32, HdmaCall::BeginPrescaledPreFlash),
    (0x88_8b39, HdmaCall::BeginExplosion),
    (0x88_8b47, HdmaCall::BeginPrescaledExplosion),
    (0x88_8b4e, HdmaCall::FinishPowerBomb),
    (0x88_a2e4, HdmaCall::BeginCrystalFlash),
    (0x88_a309, HdmaCall::BeginExplosion),
    (0x88_a317, HdmaCall::FinishCrystalFlash),
    (0x88_df91, HdmaCall::Nothing),
    (0x88_df92, HdmaCall::Nothing),
    (0x88_e726, HdmaCall::Nothing),
    (0x91_caf9, HdmaCall::FreezeAndBackup),
    (0x91_cb1c, HdmaCall::CaptureBg1Right),
    (0x91_cb57, HdmaCall::CaptureBg1Left),
    (0x91_cb8e, HdmaCall::BuildOverlay),
    (0x91_d0d3, HdmaCall::CaptureBg2Right),
    (0x91_d173, HdmaCall::UploadLeft),
    (0x91_d1a0, HdmaCall::UploadRight),
    (0x91_d2bc, HdmaCall::SetBackdrop),
    (0x91_d692, HdmaCall::InitializeSuitPickup),
];

/// An HDMA object's per-frame pre-instruction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HdmaPreAction {
    Nothing,
    SetupXrayBlending,
    AdvanceXrayBackground,
    XrayBeamInitial,
    XrayBeamWidening,
    XrayBeamFull,
    RestoreLeft,
    RestoreRight,
    FinishRestore,
    PowerBombBlending,
    PowerBombAfterglow,
    PowerBombScaled,
    PowerBombPrescaled,
    PreFlashScaled,
    PreFlashPrescaled,
    CrystalFlashBlending,
    CrystalFlashAfterglow,
    CrystalFlashScaled,
    SkyVerticalScroll,
    SkyHorizontalScroll,
    SkyParallax,
    AdvanceSkyParallax,
    FirefleaLighting,
    LavaSurface,
    LavaBackground,
    WaterSurface,
    WaterBackground,
    WaterWave,
    WaterFlat,
    Rain,
    Spores,
    Fog,
    StatueWaiting,
    StatueShaking,
    StatueSinking,
    StatueFinished,
    StatueBlending,
    BeginBrightHaze,
    BeginDimHaze,
    HazeFadingIn,
    HazeFadedIn,
    HazeFadingOut,
    DraygonLayer,
    SuitPickup,
    GravitySuitPickup,
    PhantoonLayer,
    PhantoonWave,
    RainbowBeam,
    EyeBeamWidening,
    EyeBeamFull,
    EyeBeamFading,
    TitleGradientBackdrop,
    TitleGradient,
    IntroCrossfade,
    IntroWave,
}

/// Each bank address the original dispatch matched, in its arm order.
pub const HDMA_PRE_ACTIONS: [(u32, HdmaPreAction); 59] = [
    (0x81_92db, HdmaPreAction::Nothing),
    (0x88_84b8, HdmaPreAction::Nothing),
    (0xa4_943c, HdmaPreAction::Nothing),
    (0x91_d27f, HdmaPreAction::SetupXrayBlending),
    (0x88_86ef, HdmaPreAction::AdvanceXrayBackground),
    (0x88_8732, HdmaPreAction::XrayBeamInitial),
    (0x88_8754, HdmaPreAction::XrayBeamWidening),
    (0x88_87ab, HdmaPreAction::XrayBeamFull),
    (0x88_8934, HdmaPreAction::RestoreLeft),
    (0x88_89ba, HdmaPreAction::RestoreRight),
    (0x88_8a08, HdmaPreAction::FinishRestore),
    (0x88_8b8f, HdmaPreAction::PowerBombBlending),
    (0x88_8b98, HdmaPreAction::PowerBombAfterglow),
    (0x88_8de9, HdmaPreAction::PowerBombScaled),
    (0x88_8eb2, HdmaPreAction::PowerBombPrescaled),
    (0x88_90df, HdmaPreAction::PreFlashScaled),
    (0x88_91a8, HdmaPreAction::PreFlashPrescaled),
    (0x88_a339, HdmaPreAction::CrystalFlashBlending),
    (0x88_a35d, HdmaPreAction::CrystalFlashAfterglow),
    (0x88_a552, HdmaPreAction::CrystalFlashScaled),
    (0x88_a643, HdmaPreAction::SkyVerticalScroll),
    (0x88_a673, HdmaPreAction::SkyHorizontalScroll),
    (0x88_adb2, HdmaPreAction::SkyParallax),
    (0x88_adba, HdmaPreAction::SkyParallax),
    (0x88_adc2, HdmaPreAction::AdvanceSkyParallax),
    (0x88_b0bc, HdmaPreAction::FirefleaLighting),
    (0x88_b3b0, HdmaPreAction::LavaSurface),
    (0x88_b4d5, HdmaPreAction::LavaBackground),
    (0x88_c48e, HdmaPreAction::WaterSurface),
    (0x88_c589, HdmaPreAction::WaterBackground),
    (0x88_c5e4, HdmaPreAction::WaterWave),
    (0x88_c636, HdmaPreAction::WaterFlat),
    (0x88_d9a1, HdmaPreAction::Rain),
    (0x88_da47, HdmaPreAction::Spores),
    (0x88_db36, HdmaPreAction::Fog),
    (0x88_dbd7, HdmaPreAction::StatueWaiting),
    (0x88_dc23, HdmaPreAction::StatueShaking),
    (0x88_dc69, HdmaPreAction::StatueSinking),
    (0x88_dcba, HdmaPreAction::StatueFinished),
    (0x88_dd43, HdmaPreAction::StatueBlending),
    (0x88_de10, HdmaPreAction::BeginBrightHaze),
    (0x88_de15, HdmaPreAction::BeginDimHaze),
    (0x88_de2d, HdmaPreAction::HazeFadingIn),
    (0x88_de74, HdmaPreAction::HazeFadedIn),
    (0x88_de96, HdmaPreAction::HazeFadingOut),
    (0x88_df94, HdmaPreAction::DraygonLayer),
    (0x88_e026, HdmaPreAction::SuitPickup),
    (0x88_e05c, HdmaPreAction::GravitySuitPickup),
    (0x88_e449, HdmaPreAction::PhantoonLayer),
    (0x88_e567, HdmaPreAction::PhantoonWave),
    (0x88_e7bc, HdmaPreAction::RainbowBeam),
    (0x88_e9e6, HdmaPreAction::EyeBeamWidening),
    (0x88_ea3c, HdmaPreAction::EyeBeamFull),
    (0x88_eacb, HdmaPreAction::EyeBeamFading),
    (0x88_ebb0, HdmaPreAction::TitleGradientBackdrop),
    (0x88_ebd2, HdmaPreAction::TitleGradient),
    (0x88_ec1d, HdmaPreAction::IntroCrossfade),
    (0x88_e726, HdmaPreAction::Nothing),
    (0x88_ecb6, HdmaPreAction::IntroWave),
];

/// Run the HDMA call at `address` for object `index`, as `run_pre_action`.
pub(crate) fn run_call<I>(address: u32, index: usize, it: &mut I)
where
    I: crate::hdma_program::Interpreter,
    I::Game: DispatchHost,
{
    let slot = 2 * index as u16;
    let Some(&(_, target)) = HDMA_CALLS.iter().find(|(at, _)| *at == address) else {
        it.unreachable();
        return;
    };
    match target {
        // The X-ray background's capture, overlay and uploads run on the lease's
        // Session and queues.
        HdmaCall::Nothing => it.on_lease(|_| {}),
        HdmaCall::CaptureBg1Right => it.on_lease(xray_background::capture_bg1_right),
        HdmaCall::CaptureBg1Left => it.on_lease(xray_background::capture_bg1_left),
        HdmaCall::BuildOverlay => it.on_lease(xray_background::build_overlay),
        HdmaCall::CaptureBg2Right => it.on_lease(xray_background::capture_bg2_right),
        HdmaCall::UploadLeft => it.on_lease(xray_background::upload_left),
        HdmaCall::UploadRight => it.on_lease(xray_background::upload_right),
        HdmaCall::SetBackdrop => it.on_lease(xray_background::set_backdrop),
        HdmaCall::FreezeAndBackup => it.on_object(index, xray_background::freeze_and_backup),
        HdmaCall::FinishPowerBomb => it.on_object(index, |index, game| {
            explosion_window::finish_power_bomb(index, slot, game)
        }),
        HdmaCall::FinishCrystalFlash => it.on_object(index, |index, game| {
            explosion_window::finish_crystal_flash(index, slot, game)
        }),
        HdmaCall::InitializeRainbowBeam => it.effect(HdmaEffect::InitializeRainbowBeam, slot),
        HdmaCall::InitializeSuitPickup => it.effect(HdmaEffect::InitializeSuitPickup, slot),
        // The explosion starts store the explosion words and window sources the
        // lease owns.
        HdmaCall::BeginPreFlash => it.on_lease(explosion_window::begin_pre_flash),
        HdmaCall::BeginPrescaledPreFlash => {
            it.on_lease(explosion_window::begin_prescaled_pre_flash)
        }
        HdmaCall::BeginExplosion => it.on_lease(explosion_window::begin_explosion),
        HdmaCall::BeginPrescaledExplosion => {
            it.on_lease(explosion_window::begin_prescaled_explosion)
        }
        HdmaCall::BeginCrystalFlash => it.on_lease(explosion_window::begin_crystal_flash),
    }
}

/// The hosts every pre-instruction target runs on.
pub trait DispatchHost: Host {}
impl<T> DispatchHost for T where T: Host + ?Sized {}

/// Run the pre-instruction at `address` for HDMA object `index`. Targets
/// that own their object's words run on it as typed fields; the rest run on
/// the compatibility host.
pub(crate) fn run_pre_action<I>(address: u32, index: usize, it: &mut I, handler_carry: bool)
where
    I: crate::hdma_program::Interpreter,
    I::Game: DispatchHost,
{
    let slot = 2 * index as u16;
    let Some(&(_, target)) = HDMA_PRE_ACTIONS.iter().find(|(at, _)| *at == address) else {
        it.unreachable();
        return;
    };
    if let Some(effect) = pre_action_effect(target) {
        it.effect(effect, slot);
        return;
    }
    use HdmaPreAction::*;
    match target {
        Nothing => {}
        LavaSurface => {
            it.on_lease(|lease| liquid_effects::lava_surface(index, handler_carry, lease))
        }
        LavaBackground => it.on_lease(|lease| {
            liquid_effects::lava_background(index, lease, |lease, command| {
                lease.foreign(|game| {
                    crate::display::video::Video::apply(game, "lava background", command)
                })
            })
        }),
        WaterSurface => {
            it.on_lease(|lease| liquid_effects::water_surface(index, handler_carry, lease))
        }
        WaterBackground => it.on_lease(|lease| liquid_effects::water_background(index, lease)),
        WaterWave => it.on_lease(|lease| liquid_effects::water_wave(index, lease)),
        WaterFlat => it.on_lease(liquid_effects::water_flat),
        SetupXrayBlending => it.on_lease(|lease| {
            crate::player_transitions::xray::apply_blending(
                lease,
                crate::player_transitions::xray::BlendPhase::Setup,
            )
        }),
        PowerBombBlending => it.on_lease(explosion_window::set_power_bomb_blending),
        CrystalFlashBlending => it.on_lease(explosion_window::set_crystal_flash_blending),
        SkyParallax => it.on_lease(scrolling_sky::update_parallax),
        AdvanceSkyParallax => it.on_lease(scrolling_sky::advance_parallax),
        FirefleaLighting => it.on_lease(room_visuals::advance_fireflea_lighting),
        TitleGradient => it.on_lease(hdma_objects::advance_title_gradient),
        IntroCrossfade => it.on_lease(hdma_objects::advance_intro_crossfade),
        Rain | Spores | Fog => {
            let weather = match target {
                Rain => Weather::Rain,
                Spores => Weather::Spores,
                _ => Weather::Fog,
            };
            it.on_lease(|lease| room_visuals::scroll_weather(slot, weather, lease))
        }
        StatueBlending => it.on_lease(|lease| {
            let mut fx = crate::fx_domain::Controls::fx_controls(lease);
            fx.blending = 44;
            crate::fx_domain::Controls::publish_fx_controls(
                lease,
                fx,
                &[crate::fx_state::Field::Blending],
            );
        }),
        RestoreLeft => it.on_object(index, xray_background::restore_left),
        RestoreRight => it.on_lease(xray_background::restore_right),
        FinishRestore => it.on_object(index, xray_background::finish_restore),
        SkyHorizontalScroll => it.on_object(index, scrolling_sky::update_horizontal_scroll),
        SkyVerticalScroll => it.on_lease(scrolling_sky::update_vertical_scroll),
        TitleGradientBackdrop => it.on_lease(hdma_objects::advance_title_gradient_backdrop),
        PowerBombAfterglow => it.on_object(index, explosion_window::power_bomb_afterglow),
        CrystalFlashAfterglow => it.on_object(index, explosion_window::crystal_flash_afterglow),
        PowerBombScaled | PreFlashScaled | CrystalFlashScaled => {
            let phase = match target {
                PowerBombScaled => Phase::PowerBomb,
                PreFlashScaled => Phase::PreFlash,
                _ => Phase::CrystalFlash,
            };
            it.on_object(index, |index, game| {
                explosion_window::update_scaled(index, slot, phase, game)
            })
        }
        PowerBombPrescaled | PreFlashPrescaled => {
            let phase = match target {
                PowerBombPrescaled => Phase::PowerBomb,
                _ => Phase::PreFlash,
            };
            it.on_object(index, |index, game| {
                explosion_window::update_prescaled(index, slot, phase, game)
            })
        }
        StatueWaiting | StatueShaking | StatueSinking | StatueFinished => {
            let phase = match target {
                StatueWaiting => TourianStatuePhase::Waiting,
                StatueShaking => TourianStatuePhase::Shaking,
                StatueSinking => TourianStatuePhase::Sinking,
                _ => TourianStatuePhase::Finished,
            };
            it.on_object(index, |index, game| {
                room_visuals::advance_tourian_statue(index, phase, handler_carry, game)
            })
        }
        AdvanceXrayBackground
        | XrayBeamInitial
        | XrayBeamWidening
        | XrayBeamFull
        | BeginBrightHaze
        | BeginDimHaze
        | HazeFadingIn
        | HazeFadedIn
        | HazeFadingOut
        | DraygonLayer
        | SuitPickup
        | GravitySuitPickup
        | PhantoonLayer
        | PhantoonWave
        | RainbowBeam
        | EyeBeamWidening
        | EyeBeamFull
        | EyeBeamFading
        | IntroWave => {
            unreachable!("pre_action_effect runs {target:?}")
        }
    }
}

/// The pre-instruction targets that are HDMA effects, which run on the lease.
fn pre_action_effect(target: HdmaPreAction) -> Option<HdmaEffect> {
    use HdmaPreAction::*;
    Some(match target {
        AdvanceXrayBackground => HdmaEffect::AdvanceXrayBackground,
        XrayBeamInitial => HdmaEffect::AdvanceXrayBeam(BeamPhase::Initial),
        XrayBeamWidening => HdmaEffect::AdvanceXrayBeam(BeamPhase::Widening),
        XrayBeamFull => HdmaEffect::AdvanceXrayBeam(BeamPhase::Full),
        BeginBrightHaze => HdmaEffect::BeginHaze { color: 0x80 },
        BeginDimHaze => HdmaEffect::BeginHaze { color: 0x20 },
        HazeFadingIn => HdmaEffect::AdvanceHaze(HazePhase::FadingIn),
        HazeFadedIn => HdmaEffect::AdvanceHaze(HazePhase::FadedIn),
        HazeFadingOut => HdmaEffect::AdvanceHaze(HazePhase::FadingOut),
        DraygonLayer => HdmaEffect::SelectDraygonLayer,
        SuitPickup => HdmaEffect::AdvanceSuitPickup { gravity: false },
        GravitySuitPickup => HdmaEffect::AdvanceSuitPickup { gravity: true },
        PhantoonLayer => HdmaEffect::SelectPhantoonLayer,
        PhantoonWave => HdmaEffect::AdvanceWave(HdmaWave::Phantoon),
        RainbowBeam => HdmaEffect::AdvanceRainbowBeam,
        EyeBeamWidening => HdmaEffect::AdvanceEyeBeam(EyeBeamPhase::Widening),
        EyeBeamFull => HdmaEffect::AdvanceEyeBeam(EyeBeamPhase::Full),
        EyeBeamFading => HdmaEffect::AdvanceEyeBeam(EyeBeamPhase::Fading),
        IntroWave => HdmaEffect::AdvanceWave(HdmaWave::Intro),
        _ => return None,
    })
}
