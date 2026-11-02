//! Major boss fields on the existing retained Actor/extended/protocol owners.
//! Full and odd selectors preserve independently queried bytes and aliases.
#![forbid(unsafe_code)]
use crate::session_access::SessionAccess;
pub trait State: SessionAccess {
    fn exceptional_byte(&self, at: usize) -> u8;
    fn set_exceptional_byte(&mut self, at: usize, value: u8);
}
#[derive(Clone, Copy)]
pub(crate) enum Field {
    CrocomireAttackCrocomireDeathSequenceIndex,
    CrocomireAttackCrocomireFightFlags,
    CrocomireAttackCrocomireFightFunction,
    CrocomireAttackCrocomireStepCounter,
    CrocomireAttackCrocomireYVelocity,
    CrocomireAttackCrocomireProjectileCounter,
    CrocomireAttackCrocomireUnread7800,
    CrocomireAttackCrocomireUnread7802,
    CrocomireAttackCrocomireMeltingHdmaObject,
    CrocomireAttackCrocomireAcidDamageSoundTimer,
    BaseEnemyPtr,
    BaseXPos,
    BaseXSubpos,
    BaseYPos,
    BaseYSubpos,
    BaseXWidth,
    BaseYHeight,
    BaseProperties,
    BaseExtraProperties,
    BaseAiHandlerBits,
    BaseHealth,
    BaseSpritemapPointer,
    BaseTimer,
    BaseCurrentInstruction,
    BaseInstructionTimer,
    BasePaletteIndex,
    BaseVramTilesIndex,
    BaseFlashTimer,
    BaseInvincibilityTimer,
    BaseShakeTimer,
    BaseFrameCounter,
    KraidBodyFunction,
    KraidBodyScript,
    KraidBodyScriptDelay,
    KraidBodyMouthHitReady,
    KraidBodyTimer,
    KraidBodyNext,
    KraidBodyUnread7802,
    KraidBodyUnread7804,
    KraidBodyThinkingTimer,
    KraidBodyMinEjectY,
    KraidBodyMouthFlags,
    KraidBodyHealthEighths,
    KraidBodyTargetX,
    KraidBodyHealthQuarters,
    KraidBodyHurtFrame,
    KraidBodyHurtFrameTimer,
    KraidBodyUnread782e,
    KraidPartFunction,
    KraidPartProgress,
    KraidPartXOffset,
    KraidPartTimer,
    KraidPartNext,
    KraidFingernailFunction,
    KraidFingernailXSubspeed,
    KraidFingernailXSpeed,
    KraidFingernailYSubspeed,
    KraidFingernailYSpeed,
    KraidFingernailLaunched,
    KraidFingernailNext,
    KraidFingernailsFingernailXSubvelocity,
    KraidFingernailsFingernailFunctionTimer,
    KraidFingernailsFingernailNextFunction,
    MotherBrainMotherBrainBodyFunction,
    MotherBrainMotherBrainBodyBrainPaletteCounter,
    MotherBrainMotherBrainBodyBrainPaletteReload,
    MotherBrainMotherBrainBodyFunctionTimer,
    MotherBrainMotherBrainBodyCooldown,
    MotherBrainMotherBrainBodyExplosionTimer,
    MotherBrainMotherBrainBodyForm,
    MotherBrainMotherBrainBodyPose,
    MotherBrainMotherBrainBodyHitboxes,
    MotherBrainMotherBrainBodyAttacksDisabled,
    MotherBrainMotherBrainBodyWalkCounter,
    MotherBrainMotherBrainBodyHdmaObject,
    MotherBrainMotherBrainBodyNeckPalette,
    MotherBrainMotherBrainBodyBrainPalette,
    MotherBrainMotherBrainBodyRoomPaletteList,
    MotherBrainMotherBrainBodyRoomPaletteTimer,
    MotherBrainMotherBrainBodyRingVolleys,
    MotherBrainMotherBrainBodyBabyMetroidCry,
    MotherBrainMotherBrainBodyBeamSoundsLeft,
    MotherBrainMotherBrainBodyBeamSoundPlaying,
    MotherBrainMotherBrainBodyHandBeamPhase,
    MotherBrainMotherBrainBodyAttackPhase,
    MotherBrainMotherBrainBodyRingTargetAngle,
    MotherBrainMotherBrainBodyDeleteTurrets,
    MotherBrainMotherBrainBodyCorpseState,
    MotherBrainMotherBrainBodyBrainInstructionTimer,
    MotherBrainMotherBrainBodyBrainInstructions,
    MotherBrainMotherBrainBodyBeamAngle,
    MotherBrainMotherBrainBodyBeamWidth,
    MotherBrainMotherBrainBodyGreyTransition,
    MotherBrainMotherBrainBodyFakeDeathExplosionTimer,
    MotherBrainMotherBrainBodyFakeDeathExplosionIndex,
    MotherBrainMotherBrainBodyBeamRightEdge,
    MotherBrainMotherBrainBodyBeamLeftEdge,
    MotherBrainMotherBrainBodyBeamLeftOriginX,
    MotherBrainMotherBrainBodyBeamLeftOriginY,
    MotherBrainMotherBrainBodyBeamRightOriginX,
    MotherBrainMotherBrainBodyBeamRightOriginY,
    MotherBrainMotherBrainBrainFunction,
    MotherBrainMotherBrainBrainSubfunction,
    MotherBrainMotherBrainBrainSubfunctionTimer,
    MotherBrainMotherBrainBrainShakeTimer,
    MotherBrainMotherBrainBrainBeamPaletteIndex,
    MotherBrainMotherBrainBrainUnpauseHook,
    MotherBrainMotherBrainBrainBombs,
    MotherBrainMotherBrainBrainPainfulWalkStage,
    MotherBrainMotherBrainBrainPainfulWalkDelay,
    MotherBrainMotherBrainBrainPainfulWalkFunction,
    MotherBrainMotherBrainBrainPainfulWalkTimer,
    MotherBrainMotherBrainBrainBabyMetroid,
    MotherBrainMotherBrainBrainBabyMetroidReleased,
    MotherBrainMotherBrainBrainBrainPaletteEnabled,
    MotherBrainMotherBrainBrainHealthPaletteEnabled,
    MotherBrainMotherBrainBrainDroolEnabled,
    MotherBrainMotherBrainBrainDroolParameter,
    MotherBrainMotherBrainBrainBreathEnabled,
    MotherBrainMotherBrainBrainBreathActive,
    MotherBrainMotherBrainBrainSamusRainbowFunction,
    MotherBrainMotherBrainBrainNeckFunction,
    MotherBrainMotherBrainBrainWalkFunction,
    MotherBrainMotherBrainBrainTargetX,
    MotherBrainMotherBrainBrainSamusRainbowCounter,
    MotherBrainMotherBrainBrainSegment1X,
    MotherBrainMotherBrainBrainSegment1Y,
    MotherBrainMotherBrainBrainSegment2X,
    MotherBrainMotherBrainBrainSegment2Y,
    MotherBrainMotherBrainBrainSegment3X,
    MotherBrainMotherBrainBrainSegment3Y,
    MotherBrainMotherBrainBrainSegment4X,
    MotherBrainMotherBrainBrainSegment4Y,
    MotherBrainMotherBrainBrainSegment5X,
    MotherBrainMotherBrainBrainSegment5Y,
    MotherBrainMotherBrainBrainNeckMoving,
    MotherBrainMotherBrainBrainLowerMotion,
    MotherBrainMotherBrainBrainUpperMotion,
    MotherBrainMotherBrainBrainNeckSpeed,
    MotherBrainEffectsBrainExplosionTimer,
    MotherBrainEffectsBrainExplosionIndex,
    MotherBrainEffectsBrainRainbowPaletteIndex,
    BabyMetroidBabyMetroidFunction,
    BabyMetroidBabyMetroidXVelocity,
    BabyMetroidBabyMetroidYVelocity,
    BabyMetroidBabyMetroidPaletteAnimation,
    BabyMetroidBabyMetroidPaletteHandlerDelay,
    BabyMetroidBabyMetroidFunctionTimer,
    BabyMetroidBabyMetroidContactEnabled,
    BabyMetroidBabyMetroidAggroTimer,
    BabyMetroidBabyMetroidProximityCounter,
    BabyMetroidBabyMetroidCryEnabled,
    BabyMetroidBabyMetroidCryTimer,
    BabyMetroidCutsceneBabyMetroidInCutsceneFunction,
    BabyMetroidCutsceneBabyMetroidInCutsceneXVelocity,
    BabyMetroidCutsceneBabyMetroidInCutsceneYVelocity,
    BabyMetroidCutsceneBabyMetroidInCutscenePaletteAnimation,
    BabyMetroidCutsceneBabyMetroidInCutscenePaletteHandlerDelay,
    BabyMetroidCutsceneBabyMetroidInCutsceneFunctionTimer,
    BabyMetroidCutsceneBabyMetroidInCutsceneCryEnabled,
    BabyMetroidCutsceneBabyMetroidInCutsceneFlashTimer,
    BabyMetroidCutsceneBabyMetroidInCutsceneDeathExplosionIndex,
    BabyMetroidCutsceneBabyMetroidInCutsceneDeathExplosionTimer,
    BabyMetroidCutsceneBabyMetroidInCutsceneHealthPaletteEnabled,
    BabyMetroidCutsceneBabyMetroidInCutsceneAngle,
    BabyMetroidCutsceneBabyMetroidInCutsceneSpeed,
    BabyMetroidCutsceneBabyMetroidInCutsceneFadeTimer,
    BabyMetroidCutsceneBabyMetroidInCutsceneFadeIndex,
    BabyMetroidCutsceneBabyMetroidInCutsceneMovement,
    BabyMetroidCutsceneBabyMetroidInCutscenePaletteFunction,
    BabyMetroidCutsceneBabyMetroidInCutsceneShakeOriginX,
    BabyMetroidCutsceneBabyMetroidInCutsceneShakeOriginY,
    CorpsesCorpseDrainedPaletteIndex,
    CorpsesCorpseVramTransfers,
    MotherBrainEnemyDataAiVarB,
    MotherBrainEnemyDataAiVarC,
}
impl Field {
    pub(crate) fn offset(self) -> isize {
        match self {
            Self::CrocomireAttackCrocomireDeathSequenceIndex => 48,
            Self::CrocomireAttackCrocomireFightFlags => 50,
            Self::CrocomireAttackCrocomireFightFunction => 52,
            Self::CrocomireAttackCrocomireStepCounter => 54,
            Self::CrocomireAttackCrocomireYVelocity => 56,
            Self::CrocomireAttackCrocomireProjectileCounter => 58,
            Self::CrocomireAttackCrocomireUnread7800 => 0x7800 - 0xf78,
            Self::CrocomireAttackCrocomireUnread7802 => 0x7802 - 0xf78,
            Self::CrocomireAttackCrocomireMeltingHdmaObject => 0x783e - 0xf78,
            Self::CrocomireAttackCrocomireAcidDamageSoundTimer => 0x8000 - 0xf78,
            Self::BaseEnemyPtr => 0,
            Self::BaseXPos => 2,
            Self::BaseXSubpos => 4,
            Self::BaseYPos => 6,
            Self::BaseYSubpos => 8,
            Self::BaseXWidth => 10,
            Self::BaseYHeight => 12,
            Self::BaseProperties => 14,
            Self::BaseExtraProperties => 16,
            Self::BaseAiHandlerBits => 18,
            Self::BaseHealth => 20,
            Self::BaseSpritemapPointer => 22,
            Self::BaseTimer => 24,
            Self::BaseCurrentInstruction => 26,
            Self::BaseInstructionTimer => 28,
            Self::BasePaletteIndex => 30,
            Self::BaseVramTilesIndex => 32,
            Self::BaseFlashTimer => 36,
            Self::BaseInvincibilityTimer => 40,
            Self::BaseShakeTimer => 42,
            Self::BaseFrameCounter => 44,
            Self::KraidBodyFunction => 48,
            Self::KraidBodyScript => 50,
            Self::KraidBodyScriptDelay => 52,
            Self::KraidBodyMouthHitReady => 56,
            Self::KraidBodyTimer => 58,
            Self::KraidBodyNext => 26760,
            Self::KraidBodyUnread7802 => 26762,
            Self::KraidBodyUnread7804 => 26764,
            Self::KraidBodyThinkingTimer => 26766,
            Self::KraidBodyMinEjectY => 26768,
            Self::KraidBodyMouthFlags => 26770,
            Self::KraidBodyHealthEighths => 26772,
            Self::KraidBodyTargetX => 26790,
            Self::KraidBodyHealthQuarters => 26792,
            Self::KraidBodyHurtFrame => 26802,
            Self::KraidBodyHurtFrameTimer => 26804,
            Self::KraidBodyUnread782e => 26806,
            Self::KraidPartFunction => 48,
            Self::KraidPartProgress => 50,
            Self::KraidPartXOffset => 52,
            Self::KraidPartTimer => 58,
            Self::KraidPartNext => 26760,
            Self::KraidFingernailFunction => 48,
            Self::KraidFingernailXSubspeed => 50,
            Self::KraidFingernailXSpeed => 52,
            Self::KraidFingernailYSubspeed => 54,
            Self::KraidFingernailYSpeed => 56,
            Self::KraidFingernailLaunched => 60,
            Self::KraidFingernailNext => 26760,
            Self::KraidFingernailsFingernailXSubvelocity => 50,
            Self::KraidFingernailsFingernailFunctionTimer => 58,
            Self::KraidFingernailsFingernailNextFunction => 26760,
            Self::MotherBrainMotherBrainBodyFunction => 48,
            Self::MotherBrainMotherBrainBodyBrainPaletteCounter => 54,
            Self::MotherBrainMotherBrainBodyBrainPaletteReload => 56,
            Self::MotherBrainMotherBrainBodyFunctionTimer => 58,
            Self::MotherBrainMotherBrainBodyCooldown => 60,
            Self::MotherBrainMotherBrainBodyExplosionTimer => 62,
            Self::MotherBrainMotherBrainBodyForm => 26760,
            Self::MotherBrainMotherBrainBodyPose => 26764,
            Self::MotherBrainMotherBrainBodyHitboxes => 26768,
            Self::MotherBrainMotherBrainBodyAttacksDisabled => 26772,
            Self::MotherBrainMotherBrainBodyWalkCounter => 26774,
            Self::MotherBrainMotherBrainBodyHdmaObject => 26778,
            Self::MotherBrainMotherBrainBodyNeckPalette => 26784,
            Self::MotherBrainMotherBrainBodyBrainPalette => 26786,
            Self::MotherBrainMotherBrainBodyRoomPaletteList => 26788,
            Self::MotherBrainMotherBrainBodyRoomPaletteTimer => 26790,
            Self::MotherBrainMotherBrainBodyRingVolleys => 26798,
            Self::MotherBrainMotherBrainBodyBabyMetroidCry => 26800,
            Self::MotherBrainMotherBrainBodyBeamSoundsLeft => 26802,
            Self::MotherBrainMotherBrainBodyBeamSoundPlaying => 26804,
            Self::MotherBrainMotherBrainBodyHandBeamPhase => 26806,
            Self::MotherBrainMotherBrainBodyAttackPhase => 26808,
            Self::MotherBrainMotherBrainBodyRingTargetAngle => 26812,
            Self::MotherBrainMotherBrainBodyDeleteTurrets => 26818,
            Self::MotherBrainMotherBrainBodyCorpseState => 26822,
            Self::MotherBrainMotherBrainBodyBrainInstructionTimer => 28808,
            Self::MotherBrainMotherBrainBodyBrainInstructions => 28810,
            Self::MotherBrainMotherBrainBodyBeamAngle => 28842,
            Self::MotherBrainMotherBrainBodyBeamWidth => 28846,
            Self::MotherBrainMotherBrainBodyGreyTransition => 28854,
            Self::MotherBrainMotherBrainBodyFakeDeathExplosionTimer => 28856,
            Self::MotherBrainMotherBrainBodyFakeDeathExplosionIndex => 28858,
            Self::MotherBrainMotherBrainBodyBeamRightEdge => 28860,
            Self::MotherBrainMotherBrainBodyBeamLeftEdge => 28862,
            Self::MotherBrainMotherBrainBodyBeamLeftOriginX => 28864,
            Self::MotherBrainMotherBrainBodyBeamLeftOriginY => 28866,
            Self::MotherBrainMotherBrainBodyBeamRightOriginX => 28868,
            Self::MotherBrainMotherBrainBodyBeamRightOriginY => 28870,
            Self::MotherBrainMotherBrainBrainFunction => 48,
            Self::MotherBrainMotherBrainBrainSubfunction => 56,
            Self::MotherBrainMotherBrainBrainSubfunctionTimer => 58,
            Self::MotherBrainMotherBrainBrainShakeTimer => 26760,
            Self::MotherBrainMotherBrainBrainBeamPaletteIndex => 26762,
            Self::MotherBrainMotherBrainBrainUnpauseHook => 26764,
            Self::MotherBrainMotherBrainBrainBombs => 26770,
            Self::MotherBrainMotherBrainBrainPainfulWalkStage => 26772,
            Self::MotherBrainMotherBrainBrainPainfulWalkDelay => 26774,
            Self::MotherBrainMotherBrainBrainPainfulWalkFunction => 26776,
            Self::MotherBrainMotherBrainBrainPainfulWalkTimer => 26778,
            Self::MotherBrainMotherBrainBrainBabyMetroid => 26780,
            Self::MotherBrainMotherBrainBrainBabyMetroidReleased => 26782,
            Self::MotherBrainMotherBrainBrainBrainPaletteEnabled => 26792,
            Self::MotherBrainMotherBrainBrainHealthPaletteEnabled => 26794,
            Self::MotherBrainMotherBrainBrainDroolEnabled => 26796,
            Self::MotherBrainMotherBrainBrainDroolParameter => 26798,
            Self::MotherBrainMotherBrainBrainBreathEnabled => 26800,
            Self::MotherBrainMotherBrainBrainBreathActive => 26802,
            Self::MotherBrainMotherBrainBrainSamusRainbowFunction => 26804,
            Self::MotherBrainMotherBrainBrainNeckFunction => 26808,
            Self::MotherBrainMotherBrainBrainWalkFunction => 26812,
            Self::MotherBrainMotherBrainBrainTargetX => 26814,
            Self::MotherBrainMotherBrainBrainSamusRainbowCounter => 26818,
            Self::MotherBrainMotherBrainBrainSegment1X => 28812,
            Self::MotherBrainMotherBrainBrainSegment1Y => 28814,
            Self::MotherBrainMotherBrainBrainSegment2X => 28818,
            Self::MotherBrainMotherBrainBrainSegment2Y => 28820,
            Self::MotherBrainMotherBrainBrainSegment3X => 28824,
            Self::MotherBrainMotherBrainBrainSegment3Y => 28826,
            Self::MotherBrainMotherBrainBrainSegment4X => 28830,
            Self::MotherBrainMotherBrainBrainSegment4Y => 28832,
            Self::MotherBrainMotherBrainBrainSegment5X => 28836,
            Self::MotherBrainMotherBrainBrainSegment5Y => 28838,
            Self::MotherBrainMotherBrainBrainNeckMoving => 28842,
            Self::MotherBrainMotherBrainBrainLowerMotion => 28844,
            Self::MotherBrainMotherBrainBrainUpperMotion => 28846,
            Self::MotherBrainMotherBrainBrainNeckSpeed => 28848,
            Self::MotherBrainEffectsBrainExplosionTimer => 56,
            Self::MotherBrainEffectsBrainExplosionIndex => 58,
            Self::MotherBrainEffectsBrainRainbowPaletteIndex => 26762,
            Self::BabyMetroidBabyMetroidFunction => 48,
            Self::BabyMetroidBabyMetroidXVelocity => 50,
            Self::BabyMetroidBabyMetroidYVelocity => 52,
            Self::BabyMetroidBabyMetroidPaletteAnimation => 54,
            Self::BabyMetroidBabyMetroidPaletteHandlerDelay => 56,
            Self::BabyMetroidBabyMetroidFunctionTimer => 58,
            Self::BabyMetroidBabyMetroidContactEnabled => 62,
            Self::BabyMetroidBabyMetroidAggroTimer => 26762,
            Self::BabyMetroidBabyMetroidProximityCounter => 26764,
            Self::BabyMetroidBabyMetroidCryEnabled => 26768,
            Self::BabyMetroidBabyMetroidCryTimer => 26770,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneFunction => 48,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneXVelocity => 50,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneYVelocity => 52,
            Self::BabyMetroidCutsceneBabyMetroidInCutscenePaletteAnimation => 54,
            Self::BabyMetroidCutsceneBabyMetroidInCutscenePaletteHandlerDelay => 56,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneFunctionTimer => 58,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneCryEnabled => 26768,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneFlashTimer => 26772,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneDeathExplosionIndex => 26774,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneDeathExplosionTimer => 26776,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneHealthPaletteEnabled => 26778,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneAngle => 26780,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneSpeed => 26782,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneFadeTimer => 26784,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneFadeIndex => 26786,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneMovement => 26788,
            Self::BabyMetroidCutsceneBabyMetroidInCutscenePaletteFunction => 26790,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneShakeOriginX => 26792,
            Self::BabyMetroidCutsceneBabyMetroidInCutsceneShakeOriginY => 26794,
            Self::CorpsesCorpseDrainedPaletteIndex => 26776,
            Self::CorpsesCorpseVramTransfers => 30894,
            Self::MotherBrainEnemyDataAiVarB => 50,
            Self::MotherBrainEnemyDataAiVarC => 52,
        }
    }
}
pub(crate) fn field_address(slot: i32, field: Field) -> usize {
    usize::try_from(isize::try_from(slot).expect("field selector") + 0xf78 + field.offset())
        .expect("actor field outside allocation")
}
pub(crate) fn read_owned_at<S: State + ?Sized>(s: &S, at: usize) -> Option<u16> {
    if at % 2 != 0 {
        return None;
    }
    let s = s.session();
    if (0xf78..0x1778).contains(&at) {
        let a = &s.actors.slots[(at - 0xf78) / 64];
        return Some(match (at - 0xf78) % 64 {
            0 => a.definition,
            2 => a.x.whole,
            4 => a.x.fraction,
            6 => a.y.whole,
            8 => a.y.fraction,
            10 => a.radius.x,
            12 => a.radius.y,
            14 => a.properties,
            16 => a.extra_properties,
            18 => a.ai_handlers,
            20 => a.health,
            22 => a.spritemap,
            24 => a.timer,
            26 => a.instruction,
            28 => a.instruction_delay,
            30 => a.palette,
            32 => a.tile,
            34 => a.layer,
            36 => a.flash,
            38 => a.frozen,
            40 => a.invincibility,
            42 => a.shake,
            44 => a.frame,
            46 => u16::from_le_bytes([a.bank, a.hurt_time]),
            at @ 48..=56 => a.variables[(at - 48) / 2],
            58 => a.pre_instruction,
            60 => a.parameters[0],
            62 => a.parameters[1],
            _ => unreachable!(),
        });
    }
    match at {
        0x801e => Some(s.actors.extended.botwoon_head[0].pending_hit),
        0x8020 => Some(s.actors.extended.botwoon_head[0].active),
        0x803c => Some(s.actors.extended.botwoon_head[0].tail_ready),
        0x8818 => Some(s.actors.extended.botwoon_path[0].previous_health),
        0x883e => Some(s.actors.extended.botwoon_path[0].body_landed),
        0x7800..0x97dc => Some(
            *s.actors
                .extended
                .word((at - 0x7800) / 0x800, ((at - 0x7800) % 0x800) / 2),
        ),
        0x2000..0x3000 => Some(
            *s.actors
                .extended
                .word(4 + (at - 0x2000) / 0x800, ((at - 0x2000) % 0x800) / 2),
        ),
        _ => None,
    }
}
pub(crate) fn store_owned_at<S: State + ?Sized>(s: &mut S, at: usize, value: u16) -> bool {
    if at % 2 != 0 {
        return false;
    }
    let s = s.session_mut();
    if (0xf78..0x1778).contains(&at) {
        let a = &mut s.actors.slots[(at - 0xf78) / 64];
        match (at - 0xf78) % 64 {
            0 => a.definition = value,
            2 => a.x.whole = value,
            4 => a.x.fraction = value,
            6 => a.y.whole = value,
            8 => a.y.fraction = value,
            10 => a.radius.x = value,
            12 => a.radius.y = value,
            14 => a.properties = value,
            16 => a.extra_properties = value,
            18 => a.ai_handlers = value,
            20 => a.health = value,
            22 => a.spritemap = value,
            24 => a.timer = value,
            26 => a.instruction = value,
            28 => a.instruction_delay = value,
            30 => a.palette = value,
            32 => a.tile = value,
            34 => a.layer = value,
            36 => a.flash = value,
            38 => a.frozen = value,
            40 => a.invincibility = value,
            42 => a.shake = value,
            44 => a.frame = value,
            46 => {
                let [bank, hurt] = value.to_le_bytes();
                a.bank = bank;
                a.hurt_time = hurt;
            }
            at @ 48..=56 => a.variables[(at - 48) / 2] = value,
            58 => a.pre_instruction = value,
            60 => a.parameters[0] = value,
            62 => a.parameters[1] = value,
            _ => unreachable!(),
        }
        return true;
    }
    match at {
        0x801e => s.actors.extended.botwoon_head[0].pending_hit = value,
        0x8020 => s.actors.extended.botwoon_head[0].active = value,
        0x803c => s.actors.extended.botwoon_head[0].tail_ready = value,
        0x8818 => s.actors.extended.botwoon_path[0].previous_health = value,
        0x883e => s.actors.extended.botwoon_path[0].body_landed = value,
        0x7800..0x97dc => {
            *s.actors
                .extended
                .word_mut((at - 0x7800) / 0x800, ((at - 0x7800) % 0x800) / 2) = value
        }
        0x2000..0x3000 => {
            *s.actors
                .extended
                .word_mut(4 + (at - 0x2000) / 0x800, ((at - 0x2000) % 0x800) / 2) = value
        }
        _ => return false,
    }
    true
}
pub(crate) fn read_byte<S: State + ?Sized>(s: &S, at: usize) -> u8 {
    read_owned_at(s, at & !1)
        .map(|v| v.to_le_bytes()[at & 1])
        .unwrap_or_else(|| s.exceptional_byte(at))
}
pub(crate) fn store_byte<S: State + ?Sized>(s: &mut S, at: usize, value: u8) {
    if let Some(old) = read_owned_at(s, at & !1) {
        let mut bytes = old.to_le_bytes();
        bytes[at & 1] = value;
        if store_owned_at(s, at & !1, u16::from_le_bytes(bytes)) {
            return;
        }
    }
    s.set_exceptional_byte(at, value);
}
pub(crate) fn part_get<S: State + ?Sized>(s: &S, slot: i32, field: Field) -> u16 {
    let at = field_address(slot, field);
    read_owned_at(s, at)
        .unwrap_or_else(|| u16::from_le_bytes([read_byte(s, at), read_byte(s, at + 1)]))
}
pub(crate) fn part_set<S: State + ?Sized>(s: &mut S, slot: i32, field: Field, value: u16) {
    let at = field_address(slot, field);
    if store_owned_at(s, at, value) {
        return;
    }
    let [low, high] = value.to_le_bytes();
    store_byte(s, at, low);
    store_byte(s, at + 1, high);
}
pub(crate) fn part_byte<S: State + ?Sized>(s: &S, slot: i32, field: Field, lane: usize) -> u8 {
    read_byte(s, field_address(slot, field) + lane)
}
pub(crate) fn set_part_byte<S: State + ?Sized>(
    s: &mut S,
    slot: i32,
    field: Field,
    lane: usize,
    value: u8,
) {
    store_byte(s, field_address(slot, field) + lane, value);
}
macro_rules! kraid_part {
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr] = $value:expr) => {{
        let value: u8 = $value;
        $crate::actors::boss_fields_owned::set_part_byte($g, $slot, $crate::actors::boss_fields_owned::kraid_part!(@field $layout, $($field).+), $lane as usize, value)
    }};
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr]) => {
        $crate::actors::boss_fields_owned::part_byte(&*$g, $slot, $crate::actors::boss_fields_owned::kraid_part!(@field $layout, $($field).+), $lane as usize)
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) += $value:expr) => {{
        let value: u32 = $value;
        let low = u32::from($crate::actors::boss_fields_owned::kraid_part!($g, $layout[$slot] $(.$prefix)* .$lo));
        let high = u32::from($crate::actors::boss_fields_owned::kraid_part!($g, $layout[$slot] $(.$prefix)* .$hi));
        let sum = (low | (high << 16)).wrapping_add(value);
        $crate::actors::boss_fields_owned::kraid_part!($g, $layout[$slot] $(.$prefix)* .$lo = sum as u16);
        $crate::actors::boss_fields_owned::kraid_part!($g, $layout[$slot] $(.$prefix)* .$hi = (sum >> 16) as u16);
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)*; $($field:ident = $value:expr),+ $(,)?) => {
        $crate::actors::boss_fields_owned::kraid_part!(@set_fields $g, $layout[$slot] [$(.$prefix)*]; $($field = $value),+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr, $($rest:tt)+) => {
        $crate::actors::boss_fields_owned::kraid_part!($g, $layout[$slot] $($prefix)* .$field = $value);
        $crate::actors::boss_fields_owned::kraid_part!(@set_fields $g, $layout[$slot] [$($prefix)*]; $($rest)+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr) => {
        $crate::actors::boss_fields_owned::kraid_part!($g, $layout[$slot] $($prefix)* .$field = $value);
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+, |$old:ident| $value:expr) => {{
        let slot: i32 = $slot;
        let field = $crate::actors::boss_fields_owned::kraid_part!(@field $layout, $($field).+);
        let $old = $crate::actors::boss_fields_owned::part_get(&*$g, slot, field);
        let value: u16 = $value;
        $crate::actors::boss_fields_owned::part_set($g, slot, field, value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ = $value:expr) => {{
        let value: u16 = $value;
        $crate::actors::boss_fields_owned::part_set($g, $slot, $crate::actors::boss_fields_owned::kraid_part!(@field $layout, $($field).+), value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+) => {
        $crate::actors::boss_fields_owned::part_get(&*$g, $slot, $crate::actors::boss_fields_owned::kraid_part!(@field $layout, $($field).+))
    };
    ($g:ident,$layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident,$lo:ident)=$value:expr)=>{{
        let value:u32=$value;let [l0,l1,h0,h1]=value.to_le_bytes();
        $crate::actors::boss_fields_owned::kraid_part!($g,$layout[$slot] $(.$prefix)* .$lo=u16::from_le_bytes([l0,l1]));
        $crate::actors::boss_fields_owned::kraid_part!($g,$layout[$slot] $(.$prefix)* .$hi=u16::from_le_bytes([h0,h1]));
    }};
    ($g:ident,$layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident,$lo:ident))=>{
        (u32::from($crate::actors::boss_fields_owned::kraid_part!($g,$layout[$slot] $(.$prefix)* .$hi))<<16)|u32::from($crate::actors::boss_fields_owned::kraid_part!($g,$layout[$slot] $(.$prefix)* .$lo))
    };
    (@field $($tokens:tt)*) => { $crate::actors::boss_fields_owned::kraid_field!($($tokens)*) };
}
macro_rules! kraid_field {
    ($layout:ident,base.enemy_ptr) => {
        $crate::actors::boss_fields_owned::Field::BaseEnemyPtr
    };
    ($layout:ident,base.x_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseXPos
    };
    ($layout:ident,base.x_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseXSubpos
    };
    ($layout:ident,base.y_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseYPos
    };
    ($layout:ident,base.y_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseYSubpos
    };
    ($layout:ident,base.x_width) => {
        $crate::actors::boss_fields_owned::Field::BaseXWidth
    };
    ($layout:ident,base.y_height) => {
        $crate::actors::boss_fields_owned::Field::BaseYHeight
    };
    ($layout:ident,base.properties) => {
        $crate::actors::boss_fields_owned::Field::BaseProperties
    };
    ($layout:ident,base.extra_properties) => {
        $crate::actors::boss_fields_owned::Field::BaseExtraProperties
    };
    ($layout:ident,base.ai_handler_bits) => {
        $crate::actors::boss_fields_owned::Field::BaseAiHandlerBits
    };
    ($layout:ident,base.health) => {
        $crate::actors::boss_fields_owned::Field::BaseHealth
    };
    ($layout:ident,base.spritemap_pointer) => {
        $crate::actors::boss_fields_owned::Field::BaseSpritemapPointer
    };
    ($layout:ident,base.timer) => {
        $crate::actors::boss_fields_owned::Field::BaseTimer
    };
    ($layout:ident,base.current_instruction) => {
        $crate::actors::boss_fields_owned::Field::BaseCurrentInstruction
    };
    ($layout:ident,base.instruction_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInstructionTimer
    };
    ($layout:ident,base.palette_index) => {
        $crate::actors::boss_fields_owned::Field::BasePaletteIndex
    };
    ($layout:ident,base.vram_tiles_index) => {
        $crate::actors::boss_fields_owned::Field::BaseVramTilesIndex
    };
    ($layout:ident,base.flash_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseFlashTimer
    };
    ($layout:ident,base.invincibility_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInvincibilityTimer
    };
    ($layout:ident,base.shake_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseShakeTimer
    };
    ($layout:ident,base.frame_counter) => {
        $crate::actors::boss_fields_owned::Field::BaseFrameCounter
    };
    (EnemyBase,enemy_ptr) => {
        $crate::actors::boss_fields_owned::Field::BaseEnemyPtr
    };
    (EnemyBase,x_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseXPos
    };
    (EnemyBase,x_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseXSubpos
    };
    (EnemyBase,y_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseYPos
    };
    (EnemyBase,y_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseYSubpos
    };
    (EnemyBase,x_width) => {
        $crate::actors::boss_fields_owned::Field::BaseXWidth
    };
    (EnemyBase,y_height) => {
        $crate::actors::boss_fields_owned::Field::BaseYHeight
    };
    (EnemyBase,properties) => {
        $crate::actors::boss_fields_owned::Field::BaseProperties
    };
    (EnemyBase,extra_properties) => {
        $crate::actors::boss_fields_owned::Field::BaseExtraProperties
    };
    (EnemyBase,ai_handler_bits) => {
        $crate::actors::boss_fields_owned::Field::BaseAiHandlerBits
    };
    (EnemyBase,health) => {
        $crate::actors::boss_fields_owned::Field::BaseHealth
    };
    (EnemyBase,spritemap_pointer) => {
        $crate::actors::boss_fields_owned::Field::BaseSpritemapPointer
    };
    (EnemyBase,timer) => {
        $crate::actors::boss_fields_owned::Field::BaseTimer
    };
    (EnemyBase,current_instruction) => {
        $crate::actors::boss_fields_owned::Field::BaseCurrentInstruction
    };
    (EnemyBase,instruction_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInstructionTimer
    };
    (EnemyBase,palette_index) => {
        $crate::actors::boss_fields_owned::Field::BasePaletteIndex
    };
    (EnemyBase,vram_tiles_index) => {
        $crate::actors::boss_fields_owned::Field::BaseVramTilesIndex
    };
    (EnemyBase,flash_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseFlashTimer
    };
    (EnemyBase,invincibility_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInvincibilityTimer
    };
    (EnemyBase,shake_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseShakeTimer
    };
    (EnemyBase,frame_counter) => {
        $crate::actors::boss_fields_owned::Field::BaseFrameCounter
    };
    (Body,function) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyFunction
    };
    (Body,script) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyScript
    };
    (Body,script_delay) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyScriptDelay
    };
    (Body,mouth_hit_ready) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyMouthHitReady
    };
    (Body,timer) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyTimer
    };
    (Body,next) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyNext
    };
    (Body,unread_7802) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyUnread7802
    };
    (Body,unread_7804) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyUnread7804
    };
    (Body,thinking_timer) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyThinkingTimer
    };
    (Body,min_eject_y) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyMinEjectY
    };
    (Body,mouth_flags) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyMouthFlags
    };
    (Body,health_eighths) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyHealthEighths
    };
    (Body,target_x) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyTargetX
    };
    (Body,health_quarters) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyHealthQuarters
    };
    (Body,hurt_frame) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyHurtFrame
    };
    (Body,hurt_frame_timer) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyHurtFrameTimer
    };
    (Body,unread_782e) => {
        $crate::actors::boss_fields_owned::Field::KraidBodyUnread782e
    };
    (Part,function) => {
        $crate::actors::boss_fields_owned::Field::KraidPartFunction
    };
    (Part,progress) => {
        $crate::actors::boss_fields_owned::Field::KraidPartProgress
    };
    (Part,x_offset) => {
        $crate::actors::boss_fields_owned::Field::KraidPartXOffset
    };
    (Part,timer) => {
        $crate::actors::boss_fields_owned::Field::KraidPartTimer
    };
    (Part,next) => {
        $crate::actors::boss_fields_owned::Field::KraidPartNext
    };
    (Fingernail,function) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailFunction
    };
    (Fingernail,x_subspeed) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailXSubspeed
    };
    (Fingernail,x_speed) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailXSpeed
    };
    (Fingernail,y_subspeed) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailYSubspeed
    };
    (Fingernail,y_speed) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailYSpeed
    };
    (Fingernail,launched) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailLaunched
    };
    (Fingernail,next) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailNext
    };
    (Fingernail,x_subvelocity) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailsFingernailXSubvelocity
    };
    (Fingernail,function_timer) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailsFingernailFunctionTimer
    };
    (Fingernail,next_function) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailsFingernailNextFunction
    };
    (KraidLayout,kraid_var_a) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailFunction
    };
    (KraidLayout,kraid_var_b) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailXSubspeed
    };
    (KraidLayout,kraid_var_c) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailXSpeed
    };
    (KraidLayout,kraid_var_e) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailYSpeed
    };
    (KraidLayout,kraid_next) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailNext
    };
    (KraidLayout,kraid_var_d) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailYSubspeed
    };
    (KraidLayout,kraid_parameter_1) => {
        $crate::actors::boss_fields_owned::Field::KraidFingernailLaunched
    };
}
pub(crate) use {kraid_field, kraid_part};
macro_rules! crocomire_part {
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr] = $value:expr) => {{
        let value: u8 = $value;
        $crate::actors::boss_fields_owned::set_part_byte($g, $slot, $crate::actors::boss_fields_owned::crocomire_part!(@field $layout, $($field).+), $lane as usize, value)
    }};
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr]) => {
        $crate::actors::boss_fields_owned::part_byte(&*$g, $slot, $crate::actors::boss_fields_owned::crocomire_part!(@field $layout, $($field).+), $lane as usize)
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) += $value:expr) => {{
        let value: u32 = $value;
        let low = u32::from($crate::actors::boss_fields_owned::crocomire_part!($g, $layout[$slot] $(.$prefix)* .$lo));
        let high = u32::from($crate::actors::boss_fields_owned::crocomire_part!($g, $layout[$slot] $(.$prefix)* .$hi));
        let sum = (low | (high << 16)).wrapping_add(value);
        $crate::actors::boss_fields_owned::crocomire_part!($g, $layout[$slot] $(.$prefix)* .$lo = sum as u16);
        $crate::actors::boss_fields_owned::crocomire_part!($g, $layout[$slot] $(.$prefix)* .$hi = (sum >> 16) as u16);
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)*; $($field:ident = $value:expr),+ $(,)?) => {
        $crate::actors::boss_fields_owned::crocomire_part!(@set_fields $g, $layout[$slot] [$(.$prefix)*]; $($field = $value),+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr, $($rest:tt)+) => {
        $crate::actors::boss_fields_owned::crocomire_part!($g, $layout[$slot] $($prefix)* .$field = $value);
        $crate::actors::boss_fields_owned::crocomire_part!(@set_fields $g, $layout[$slot] [$($prefix)*]; $($rest)+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr) => {
        $crate::actors::boss_fields_owned::crocomire_part!($g, $layout[$slot] $($prefix)* .$field = $value);
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+, |$old:ident| $value:expr) => {{
        let slot: i32 = $slot;
        let field = $crate::actors::boss_fields_owned::crocomire_part!(@field $layout, $($field).+);
        let $old = $crate::actors::boss_fields_owned::part_get(&*$g, slot, field);
        let value: u16 = $value;
        $crate::actors::boss_fields_owned::part_set($g, slot, field, value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ = $value:expr) => {{
        let value: u16 = $value;
        $crate::actors::boss_fields_owned::part_set($g, $slot, $crate::actors::boss_fields_owned::crocomire_part!(@field $layout, $($field).+), value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+) => {
        $crate::actors::boss_fields_owned::part_get(&*$g, $slot, $crate::actors::boss_fields_owned::crocomire_part!(@field $layout, $($field).+))
    };
    ($g:ident,$layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident,$lo:ident)=$value:expr)=>{{
        let value:u32=$value;let [l0,l1,h0,h1]=value.to_le_bytes();
        $crate::actors::boss_fields_owned::crocomire_part!($g,$layout[$slot] $(.$prefix)* .$lo=u16::from_le_bytes([l0,l1]));
        $crate::actors::boss_fields_owned::crocomire_part!($g,$layout[$slot] $(.$prefix)* .$hi=u16::from_le_bytes([h0,h1]));
    }};
    ($g:ident,$layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident,$lo:ident))=>{
        (u32::from($crate::actors::boss_fields_owned::crocomire_part!($g,$layout[$slot] $(.$prefix)* .$hi))<<16)|u32::from($crate::actors::boss_fields_owned::crocomire_part!($g,$layout[$slot] $(.$prefix)* .$lo))
    };
    (@field $($tokens:tt)*) => { $crate::actors::boss_fields_owned::crocomire_field!($($tokens)*) };
}
macro_rules! crocomire_field {
    (Crocomire,death_sequence_index) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireDeathSequenceIndex
    };
    (Crocomire,fight_flags) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireFightFlags
    };
    (Crocomire,fight_function) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireFightFunction
    };
    (Crocomire,step_counter) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireStepCounter
    };
    (Crocomire,y_velocity) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireYVelocity
    };
    (Crocomire,projectile_counter) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireProjectileCounter
    };
    (Crocomire,unread_7800) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireUnread7800
    };
    (Crocomire,unread_7802) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireUnread7802
    };
    (Crocomire,melting_hdma_object) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireMeltingHdmaObject
    };
    (Crocomire,var_1f) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireMeltingHdmaObject
    };
    (Crocomire,acid_damage_sound_timer) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireAcidDamageSoundTimer
    };
    ($layout:ident,base.enemy_ptr) => {
        $crate::actors::boss_fields_owned::Field::BaseEnemyPtr
    };
    ($layout:ident,base.x_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseXPos
    };
    ($layout:ident,base.x_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseXSubpos
    };
    ($layout:ident,base.y_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseYPos
    };
    ($layout:ident,base.y_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseYSubpos
    };
    ($layout:ident,base.x_width) => {
        $crate::actors::boss_fields_owned::Field::BaseXWidth
    };
    ($layout:ident,base.y_height) => {
        $crate::actors::boss_fields_owned::Field::BaseYHeight
    };
    ($layout:ident,base.properties) => {
        $crate::actors::boss_fields_owned::Field::BaseProperties
    };
    ($layout:ident,base.extra_properties) => {
        $crate::actors::boss_fields_owned::Field::BaseExtraProperties
    };
    ($layout:ident,base.ai_handler_bits) => {
        $crate::actors::boss_fields_owned::Field::BaseAiHandlerBits
    };
    ($layout:ident,base.health) => {
        $crate::actors::boss_fields_owned::Field::BaseHealth
    };
    ($layout:ident,base.spritemap_pointer) => {
        $crate::actors::boss_fields_owned::Field::BaseSpritemapPointer
    };
    ($layout:ident,base.timer) => {
        $crate::actors::boss_fields_owned::Field::BaseTimer
    };
    ($layout:ident,base.current_instruction) => {
        $crate::actors::boss_fields_owned::Field::BaseCurrentInstruction
    };
    ($layout:ident,base.instruction_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInstructionTimer
    };
    ($layout:ident,base.palette_index) => {
        $crate::actors::boss_fields_owned::Field::BasePaletteIndex
    };
    ($layout:ident,base.vram_tiles_index) => {
        $crate::actors::boss_fields_owned::Field::BaseVramTilesIndex
    };
    ($layout:ident,base.flash_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseFlashTimer
    };
    ($layout:ident,base.invincibility_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInvincibilityTimer
    };
    ($layout:ident,base.shake_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseShakeTimer
    };
    ($layout:ident,base.frame_counter) => {
        $crate::actors::boss_fields_owned::Field::BaseFrameCounter
    };
    (EnemyBase,enemy_ptr) => {
        $crate::actors::boss_fields_owned::Field::BaseEnemyPtr
    };
    (EnemyBase,x_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseXPos
    };
    (EnemyBase,x_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseXSubpos
    };
    (EnemyBase,y_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseYPos
    };
    (EnemyBase,y_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseYSubpos
    };
    (EnemyBase,x_width) => {
        $crate::actors::boss_fields_owned::Field::BaseXWidth
    };
    (EnemyBase,y_height) => {
        $crate::actors::boss_fields_owned::Field::BaseYHeight
    };
    (EnemyBase,properties) => {
        $crate::actors::boss_fields_owned::Field::BaseProperties
    };
    (EnemyBase,extra_properties) => {
        $crate::actors::boss_fields_owned::Field::BaseExtraProperties
    };
    (EnemyBase,ai_handler_bits) => {
        $crate::actors::boss_fields_owned::Field::BaseAiHandlerBits
    };
    (EnemyBase,health) => {
        $crate::actors::boss_fields_owned::Field::BaseHealth
    };
    (EnemyBase,spritemap_pointer) => {
        $crate::actors::boss_fields_owned::Field::BaseSpritemapPointer
    };
    (EnemyBase,timer) => {
        $crate::actors::boss_fields_owned::Field::BaseTimer
    };
    (EnemyBase,current_instruction) => {
        $crate::actors::boss_fields_owned::Field::BaseCurrentInstruction
    };
    (EnemyBase,instruction_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInstructionTimer
    };
    (EnemyBase,palette_index) => {
        $crate::actors::boss_fields_owned::Field::BasePaletteIndex
    };
    (EnemyBase,vram_tiles_index) => {
        $crate::actors::boss_fields_owned::Field::BaseVramTilesIndex
    };
    (EnemyBase,flash_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseFlashTimer
    };
    (EnemyBase,invincibility_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInvincibilityTimer
    };
    (EnemyBase,shake_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseShakeTimer
    };
    (EnemyBase,frame_counter) => {
        $crate::actors::boss_fields_owned::Field::BaseFrameCounter
    };
    (EnemyLayout72,var_a) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireDeathSequenceIndex
    };
    (EnemyLayout72,var_b) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireFightFlags
    };
    (EnemyLayout72,var_c) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireFightFunction
    };
    (EnemyLayout72,var_d) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireStepCounter
    };
    (EnemyLayout72,var_e) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireYVelocity
    };
    (EnemyLayout72,var_f) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireProjectileCounter
    };
    (EnemyLayout72,var_00) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireUnread7800
    };
    (EnemyLayout72,var_01) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireUnread7802
    };
    (EnemyLayout72,var_1f) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireMeltingHdmaObject
    };
    (EnemyLayout72,var_20) => {
        $crate::actors::boss_fields_owned::Field::CrocomireAttackCrocomireAcidDamageSoundTimer
    };
}
pub(crate) use {crocomire_field, crocomire_part};
macro_rules! mother_brain_part {
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr] = $value:expr) => {{
        let value: u8 = $value;
        $crate::actors::boss_fields_owned::set_part_byte($g, $slot, $crate::actors::boss_fields_owned::mother_brain_part!(@field $layout, $($field).+), $lane as usize, value)
    }};
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr]) => {
        $crate::actors::boss_fields_owned::part_byte(&*$g, $slot, $crate::actors::boss_fields_owned::mother_brain_part!(@field $layout, $($field).+), $lane as usize)
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) += $value:expr) => {{
        let value: u32 = $value;
        let low = u32::from($crate::actors::boss_fields_owned::mother_brain_part!($g, $layout[$slot] $(.$prefix)* .$lo));
        let high = u32::from($crate::actors::boss_fields_owned::mother_brain_part!($g, $layout[$slot] $(.$prefix)* .$hi));
        let sum = (low | (high << 16)).wrapping_add(value);
        $crate::actors::boss_fields_owned::mother_brain_part!($g, $layout[$slot] $(.$prefix)* .$lo = sum as u16);
        $crate::actors::boss_fields_owned::mother_brain_part!($g, $layout[$slot] $(.$prefix)* .$hi = (sum >> 16) as u16);
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)*; $($field:ident = $value:expr),+ $(,)?) => {
        $crate::actors::boss_fields_owned::mother_brain_part!(@set_fields $g, $layout[$slot] [$(.$prefix)*]; $($field = $value),+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr, $($rest:tt)+) => {
        $crate::actors::boss_fields_owned::mother_brain_part!($g, $layout[$slot] $($prefix)* .$field = $value);
        $crate::actors::boss_fields_owned::mother_brain_part!(@set_fields $g, $layout[$slot] [$($prefix)*]; $($rest)+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr) => {
        $crate::actors::boss_fields_owned::mother_brain_part!($g, $layout[$slot] $($prefix)* .$field = $value);
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+, |$old:ident| $value:expr) => {{
        let slot: i32 = $slot;
        let field = $crate::actors::boss_fields_owned::mother_brain_part!(@field $layout, $($field).+);
        let $old = $crate::actors::boss_fields_owned::part_get(&*$g, slot, field);
        let value: u16 = $value;
        $crate::actors::boss_fields_owned::part_set($g, slot, field, value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ = $value:expr) => {{
        let value: u16 = $value;
        $crate::actors::boss_fields_owned::part_set($g, $slot, $crate::actors::boss_fields_owned::mother_brain_part!(@field $layout, $($field).+), value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+) => {
        $crate::actors::boss_fields_owned::part_get(&*$g, $slot, $crate::actors::boss_fields_owned::mother_brain_part!(@field $layout, $($field).+))
    };
    ($g:ident,$layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident,$lo:ident)=$value:expr)=>{{
        let value:u32=$value;let [l0,l1,h0,h1]=value.to_le_bytes();
        $crate::actors::boss_fields_owned::mother_brain_part!($g,$layout[$slot] $(.$prefix)* .$lo=u16::from_le_bytes([l0,l1]));
        $crate::actors::boss_fields_owned::mother_brain_part!($g,$layout[$slot] $(.$prefix)* .$hi=u16::from_le_bytes([h0,h1]));
    }};
    ($g:ident,$layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident,$lo:ident))=>{
        (u32::from($crate::actors::boss_fields_owned::mother_brain_part!($g,$layout[$slot] $(.$prefix)* .$hi))<<16)|u32::from($crate::actors::boss_fields_owned::mother_brain_part!($g,$layout[$slot] $(.$prefix)* .$lo))
    };
    (@field $($tokens:tt)*) => { $crate::actors::boss_fields_owned::mother_brain_field!($($tokens)*) };
}
macro_rules! mother_brain_field {
 ($layout:ident,base.enemy_ptr)=>{$crate::actors::boss_fields_owned::Field::BaseEnemyPtr};
 ($layout:ident,base.x_pos)=>{$crate::actors::boss_fields_owned::Field::BaseXPos};
 ($layout:ident,base.x_subpos)=>{$crate::actors::boss_fields_owned::Field::BaseXSubpos};
 ($layout:ident,base.y_pos)=>{$crate::actors::boss_fields_owned::Field::BaseYPos};
 ($layout:ident,base.y_subpos)=>{$crate::actors::boss_fields_owned::Field::BaseYSubpos};
 ($layout:ident,base.x_width)=>{$crate::actors::boss_fields_owned::Field::BaseXWidth};
 ($layout:ident,base.y_height)=>{$crate::actors::boss_fields_owned::Field::BaseYHeight};
 ($layout:ident,base.properties)=>{$crate::actors::boss_fields_owned::Field::BaseProperties};
 ($layout:ident,base.extra_properties)=>{$crate::actors::boss_fields_owned::Field::BaseExtraProperties};
 ($layout:ident,base.ai_handler_bits)=>{$crate::actors::boss_fields_owned::Field::BaseAiHandlerBits};
 ($layout:ident,base.health)=>{$crate::actors::boss_fields_owned::Field::BaseHealth};
 ($layout:ident,base.spritemap_pointer)=>{$crate::actors::boss_fields_owned::Field::BaseSpritemapPointer};
 ($layout:ident,base.timer)=>{$crate::actors::boss_fields_owned::Field::BaseTimer};
 ($layout:ident,base.current_instruction)=>{$crate::actors::boss_fields_owned::Field::BaseCurrentInstruction};
 ($layout:ident,base.instruction_timer)=>{$crate::actors::boss_fields_owned::Field::BaseInstructionTimer};
 ($layout:ident,base.palette_index)=>{$crate::actors::boss_fields_owned::Field::BasePaletteIndex};
 ($layout:ident,base.vram_tiles_index)=>{$crate::actors::boss_fields_owned::Field::BaseVramTilesIndex};
 ($layout:ident,base.flash_timer)=>{$crate::actors::boss_fields_owned::Field::BaseFlashTimer};
 ($layout:ident,base.invincibility_timer)=>{$crate::actors::boss_fields_owned::Field::BaseInvincibilityTimer};
 ($layout:ident,base.shake_timer)=>{$crate::actors::boss_fields_owned::Field::BaseShakeTimer};
 ($layout:ident,base.frame_counter)=>{$crate::actors::boss_fields_owned::Field::BaseFrameCounter};
 (EnemyBase,enemy_ptr)=>{$crate::actors::boss_fields_owned::Field::BaseEnemyPtr};
 (EnemyBase,x_pos)=>{$crate::actors::boss_fields_owned::Field::BaseXPos};
 (EnemyBase,x_subpos)=>{$crate::actors::boss_fields_owned::Field::BaseXSubpos};
 (EnemyBase,y_pos)=>{$crate::actors::boss_fields_owned::Field::BaseYPos};
 (EnemyBase,y_subpos)=>{$crate::actors::boss_fields_owned::Field::BaseYSubpos};
 (EnemyBase,x_width)=>{$crate::actors::boss_fields_owned::Field::BaseXWidth};
 (EnemyBase,y_height)=>{$crate::actors::boss_fields_owned::Field::BaseYHeight};
 (EnemyBase,properties)=>{$crate::actors::boss_fields_owned::Field::BaseProperties};
 (EnemyBase,extra_properties)=>{$crate::actors::boss_fields_owned::Field::BaseExtraProperties};
 (EnemyBase,ai_handler_bits)=>{$crate::actors::boss_fields_owned::Field::BaseAiHandlerBits};
 (EnemyBase,health)=>{$crate::actors::boss_fields_owned::Field::BaseHealth};
 (EnemyBase,spritemap_pointer)=>{$crate::actors::boss_fields_owned::Field::BaseSpritemapPointer};
 (EnemyBase,timer)=>{$crate::actors::boss_fields_owned::Field::BaseTimer};
 (EnemyBase,current_instruction)=>{$crate::actors::boss_fields_owned::Field::BaseCurrentInstruction};
 (EnemyBase,instruction_timer)=>{$crate::actors::boss_fields_owned::Field::BaseInstructionTimer};
 (EnemyBase,palette_index)=>{$crate::actors::boss_fields_owned::Field::BasePaletteIndex};
 (EnemyBase,vram_tiles_index)=>{$crate::actors::boss_fields_owned::Field::BaseVramTilesIndex};
 (EnemyBase,flash_timer)=>{$crate::actors::boss_fields_owned::Field::BaseFlashTimer};
 (EnemyBase,invincibility_timer)=>{$crate::actors::boss_fields_owned::Field::BaseInvincibilityTimer};
 (EnemyBase,shake_timer)=>{$crate::actors::boss_fields_owned::Field::BaseShakeTimer};
 (EnemyBase,frame_counter)=>{$crate::actors::boss_fields_owned::Field::BaseFrameCounter};
 (MotherBrainBody,function)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyFunction};
 (MotherBrainBody,brain_palette_counter)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBrainPaletteCounter};
 (MotherBrainBody,brain_palette_reload)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBrainPaletteReload};
 (MotherBrainBody,function_timer)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyFunctionTimer};
 (MotherBrainBody,cooldown)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyCooldown};
 (MotherBrainBody,explosion_timer)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyExplosionTimer};
 (MotherBrainBody,form)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyForm};
 (MotherBrainBody,pose)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyPose};
 (MotherBrainBody,hitboxes)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyHitboxes};
 (MotherBrainBody,attacks_disabled)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyAttacksDisabled};
 (MotherBrainBody,walk_counter)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyWalkCounter};
 (MotherBrainBody,hdma_object)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyHdmaObject};
 (MotherBrainBody,neck_palette)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyNeckPalette};
 (MotherBrainBody,brain_palette)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBrainPalette};
 (MotherBrainBody,room_palette_list)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyRoomPaletteList};
 (MotherBrainBody,room_palette_timer)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyRoomPaletteTimer};
 (MotherBrainBody,ring_volleys)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyRingVolleys};
 (MotherBrainBody,baby_metroid_cry)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBabyMetroidCry};
 (MotherBrainBody,beam_sounds_left)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamSoundsLeft};
 (MotherBrainBody,beam_sound_playing)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamSoundPlaying};
 (MotherBrainBody,hand_beam_phase)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyHandBeamPhase};
 (MotherBrainBody,attack_phase)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyAttackPhase};
 (MotherBrainBody,ring_target_angle)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyRingTargetAngle};
 (MotherBrainBody,delete_turrets)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyDeleteTurrets};
 (MotherBrainBody,corpse_state)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyCorpseState};
 (MotherBrainBody,brain_instruction_timer)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBrainInstructionTimer};
 (MotherBrainBody,brain_instructions)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBrainInstructions};
 (MotherBrainBody,beam_angle)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamAngle};
 (MotherBrainBody,beam_width)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamWidth};
 (MotherBrainBody,grey_transition)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyGreyTransition};
 (MotherBrainBody,fake_death_explosion_timer)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyFakeDeathExplosionTimer};
 (MotherBrainBody,fake_death_explosion_index)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyFakeDeathExplosionIndex};
 (MotherBrainBody,beam_right_edge)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamRightEdge};
 (MotherBrainBody,beam_left_edge)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamLeftEdge};
 (MotherBrainBody,beam_left_origin_x)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamLeftOriginX};
 (MotherBrainBody,beam_left_origin_y)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamLeftOriginY};
 (MotherBrainBody,beam_right_origin_x)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamRightOriginX};
 (MotherBrainBody,beam_right_origin_y)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBodyBeamRightOriginY};
 (MotherBrainBrain,function)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainFunction};
 (MotherBrainBrain,subfunction)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSubfunction};
 (MotherBrainBrain,subfunction_timer)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSubfunctionTimer};
 (MotherBrainBrain,shake_timer)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainShakeTimer};
 (MotherBrainBrain,beam_palette_index)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainBeamPaletteIndex};
 (MotherBrainBrain,unpause_hook)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainUnpauseHook};
 (MotherBrainBrain,bombs)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainBombs};
 (MotherBrainBrain,painful_walk_stage)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainPainfulWalkStage};
 (MotherBrainBrain,painful_walk_delay)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainPainfulWalkDelay};
 (MotherBrainBrain,painful_walk_function)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainPainfulWalkFunction};
 (MotherBrainBrain,painful_walk_timer)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainPainfulWalkTimer};
 (MotherBrainBrain,baby_metroid)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainBabyMetroid};
 (MotherBrainBrain,baby_metroid_released)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainBabyMetroidReleased};
 (MotherBrainBrain,brain_palette_enabled)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainBrainPaletteEnabled};
 (MotherBrainBrain,health_palette_enabled)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainHealthPaletteEnabled};
 (MotherBrainBrain,drool_enabled)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainDroolEnabled};
 (MotherBrainBrain,drool_parameter)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainDroolParameter};
 (MotherBrainBrain,breath_enabled)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainBreathEnabled};
 (MotherBrainBrain,breath_active)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainBreathActive};
 (MotherBrainBrain,samus_rainbow_function)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSamusRainbowFunction};
 (MotherBrainBrain,neck_function)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainNeckFunction};
 (MotherBrainBrain,walk_function)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainWalkFunction};
 (MotherBrainBrain,target_x)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainTargetX};
 (MotherBrainBrain,samus_rainbow_counter)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSamusRainbowCounter};
 (MotherBrainBrain,segment_1_x)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment1X};
 (MotherBrainBrain,segment_1_y)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment1Y};
 (MotherBrainBrain,segment_2_x)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment2X};
 (MotherBrainBrain,segment_2_y)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment2Y};
 (MotherBrainBrain,segment_3_x)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment3X};
 (MotherBrainBrain,segment_3_y)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment3Y};
 (MotherBrainBrain,segment_4_x)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment4X};
 (MotherBrainBrain,segment_4_y)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment4Y};
 (MotherBrainBrain,segment_5_x)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment5X};
 (MotherBrainBrain,segment_5_y)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainSegment5Y};
 (MotherBrainBrain,neck_moving)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainNeckMoving};
 (MotherBrainBrain,lower_motion)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainLowerMotion};
 (MotherBrainBrain,upper_motion)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainUpperMotion};
 (MotherBrainBrain,neck_speed)=>{$crate::actors::boss_fields_owned::Field::MotherBrainMotherBrainBrainNeckSpeed};
 (BabyMetroid,function)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidFunction};
 (BabyMetroid,x_velocity)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidXVelocity};
 (BabyMetroid,y_velocity)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidYVelocity};
 (BabyMetroid,palette_animation)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidPaletteAnimation};
 (BabyMetroid,palette_handler_delay)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidPaletteHandlerDelay};
 (BabyMetroid,function_timer)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidFunctionTimer};
 (BabyMetroid,contact_enabled)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidContactEnabled};
 (BabyMetroid,aggro_timer)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidAggroTimer};
 (BabyMetroid,proximity_counter)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidProximityCounter};
 (BabyMetroid,cry_enabled)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidCryEnabled};
 (BabyMetroid,cry_timer)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidBabyMetroidCryTimer};
 (BabyMetroidInCutscene,function)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneFunction};
 (BabyMetroidInCutscene,x_velocity)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneXVelocity};
 (BabyMetroidInCutscene,y_velocity)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneYVelocity};
 (BabyMetroidInCutscene,palette_animation)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutscenePaletteAnimation};
 (BabyMetroidInCutscene,palette_handler_delay)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutscenePaletteHandlerDelay};
 (BabyMetroidInCutscene,function_timer)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneFunctionTimer};
 (BabyMetroidInCutscene,cry_enabled)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneCryEnabled};
 (BabyMetroidInCutscene,flash_timer)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneFlashTimer};
 (BabyMetroidInCutscene,death_explosion_index)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneDeathExplosionIndex};
 (BabyMetroidInCutscene,death_explosion_timer)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneDeathExplosionTimer};
 (BabyMetroidInCutscene,health_palette_enabled)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneHealthPaletteEnabled};
 (BabyMetroidInCutscene,angle)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneAngle};
 (BabyMetroidInCutscene,speed)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneSpeed};
 (BabyMetroidInCutscene,fade_timer)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneFadeTimer};
 (BabyMetroidInCutscene,fade_index)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneFadeIndex};
 (BabyMetroidInCutscene,movement)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneMovement};
 (BabyMetroidInCutscene,palette_function)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutscenePaletteFunction};
 (BabyMetroidInCutscene,shake_origin_x)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneShakeOriginX};
 (BabyMetroidInCutscene,shake_origin_y)=>{$crate::actors::boss_fields_owned::Field::BabyMetroidCutsceneBabyMetroidInCutsceneShakeOriginY};
 (Corpse,drained_palette_index)=>{$crate::actors::boss_fields_owned::Field::CorpsesCorpseDrainedPaletteIndex};
 (Corpse,vram_transfers)=>{$crate::actors::boss_fields_owned::Field::CorpsesCorpseVramTransfers};
 (EnemyData,ai_var_b)=>{$crate::actors::boss_fields_owned::Field::MotherBrainEnemyDataAiVarB};
 (EnemyData,ai_var_c)=>{$crate::actors::boss_fields_owned::Field::MotherBrainEnemyDataAiVarC};
 (EnemyLayout72,var_3a)=>{$crate::actors::boss_fields_owned::Field::EscapeTypewriterTypewriterTilemapEntry};
 (EnemyLayout72,var_3b)=>{$crate::actors::boss_fields_owned::Field::EscapeTypewriterTypewriterText};
 (EnemyLayout72,var_3c)=>{$crate::actors::boss_fields_owned::Field::EscapeTypewriterTypewriterVramAddress};
 (EnemyLayout72,var_3d)=>{$crate::actors::boss_fields_owned::Field::EscapeTypewriterTypewriterTimer};
 (EnemyLayout72,var_3e)=>{$crate::actors::boss_fields_owned::Field::EscapeTypewriterTypewriterTimerReset};
 (EnemyLayout72,var_3f)=>{$crate::actors::boss_fields_owned::Field::EscapeTypewriterTypewriterStrokeTimer};
}
pub(crate) use {mother_brain_field, mother_brain_part};
macro_rules! mother_brain_effects_part {
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr] = $value:expr) => {{
        let value: u8 = $value;
        $crate::actors::boss_fields_owned::set_part_byte($g, $slot, $crate::actors::boss_fields_owned::mother_brain_effects_part!(@field $layout, $($field).+), $lane as usize, value)
    }};
    (byte $g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ [$lane:expr]) => {
        $crate::actors::boss_fields_owned::part_byte(&*$g, $slot, $crate::actors::boss_fields_owned::mother_brain_effects_part!(@field $layout, $($field).+), $lane as usize)
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident, $lo:ident) += $value:expr) => {{
        let value: u32 = $value;
        let low = u32::from($crate::actors::boss_fields_owned::mother_brain_effects_part!($g, $layout[$slot] $(.$prefix)* .$lo));
        let high = u32::from($crate::actors::boss_fields_owned::mother_brain_effects_part!($g, $layout[$slot] $(.$prefix)* .$hi));
        let sum = (low | (high << 16)).wrapping_add(value);
        $crate::actors::boss_fields_owned::mother_brain_effects_part!($g, $layout[$slot] $(.$prefix)* .$lo = sum as u16);
        $crate::actors::boss_fields_owned::mother_brain_effects_part!($g, $layout[$slot] $(.$prefix)* .$hi = (sum >> 16) as u16);
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$prefix:ident)*; $($field:ident = $value:expr),+ $(,)?) => {
        $crate::actors::boss_fields_owned::mother_brain_effects_part!(@set_fields $g, $layout[$slot] [$(.$prefix)*]; $($field = $value),+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr, $($rest:tt)+) => {
        $crate::actors::boss_fields_owned::mother_brain_effects_part!($g, $layout[$slot] $($prefix)* .$field = $value);
        $crate::actors::boss_fields_owned::mother_brain_effects_part!(@set_fields $g, $layout[$slot] [$($prefix)*]; $($rest)+);
    };
    (@set_fields $g:ident, $layout:ident[$slot:expr] [$($prefix:tt)*]; $field:ident = $value:expr) => {
        $crate::actors::boss_fields_owned::mother_brain_effects_part!($g, $layout[$slot] $($prefix)* .$field = $value);
    };
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+, |$old:ident| $value:expr) => {{
        let slot: i32 = $slot;
        let field = $crate::actors::boss_fields_owned::mother_brain_effects_part!(@field $layout, $($field).+);
        let $old = $crate::actors::boss_fields_owned::part_get(&*$g, slot, field);
        let value: u16 = $value;
        $crate::actors::boss_fields_owned::part_set($g, slot, field, value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+ = $value:expr) => {{
        let value: u16 = $value;
        $crate::actors::boss_fields_owned::part_set($g, $slot, $crate::actors::boss_fields_owned::mother_brain_effects_part!(@field $layout, $($field).+), value)
    }};
    ($g:ident, $layout:ident[$slot:expr] $(.$field:ident)+) => {
        $crate::actors::boss_fields_owned::part_get(&*$g, $slot, $crate::actors::boss_fields_owned::mother_brain_effects_part!(@field $layout, $($field).+))
    };
    ($g:ident,$layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident,$lo:ident)=$value:expr)=>{{
        let value:u32=$value;let [l0,l1,h0,h1]=value.to_le_bytes();
        $crate::actors::boss_fields_owned::mother_brain_effects_part!($g,$layout[$slot] $(.$prefix)* .$lo=u16::from_le_bytes([l0,l1]));
        $crate::actors::boss_fields_owned::mother_brain_effects_part!($g,$layout[$slot] $(.$prefix)* .$hi=u16::from_le_bytes([h0,h1]));
    }};
    ($g:ident,$layout:ident[$slot:expr] $(.$prefix:ident)* ($hi:ident,$lo:ident))=>{
        (u32::from($crate::actors::boss_fields_owned::mother_brain_effects_part!($g,$layout[$slot] $(.$prefix)* .$hi))<<16)|u32::from($crate::actors::boss_fields_owned::mother_brain_effects_part!($g,$layout[$slot] $(.$prefix)* .$lo))
    };
    (@field $($tokens:tt)*) => { $crate::actors::boss_fields_owned::mother_brain_effects_field!($($tokens)*) };
}
macro_rules! mother_brain_effects_field {
    ($layout:ident,base.enemy_ptr) => {
        $crate::actors::boss_fields_owned::Field::BaseEnemyPtr
    };
    ($layout:ident,base.x_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseXPos
    };
    ($layout:ident,base.x_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseXSubpos
    };
    ($layout:ident,base.y_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseYPos
    };
    ($layout:ident,base.y_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseYSubpos
    };
    ($layout:ident,base.x_width) => {
        $crate::actors::boss_fields_owned::Field::BaseXWidth
    };
    ($layout:ident,base.y_height) => {
        $crate::actors::boss_fields_owned::Field::BaseYHeight
    };
    ($layout:ident,base.properties) => {
        $crate::actors::boss_fields_owned::Field::BaseProperties
    };
    ($layout:ident,base.extra_properties) => {
        $crate::actors::boss_fields_owned::Field::BaseExtraProperties
    };
    ($layout:ident,base.ai_handler_bits) => {
        $crate::actors::boss_fields_owned::Field::BaseAiHandlerBits
    };
    ($layout:ident,base.health) => {
        $crate::actors::boss_fields_owned::Field::BaseHealth
    };
    ($layout:ident,base.spritemap_pointer) => {
        $crate::actors::boss_fields_owned::Field::BaseSpritemapPointer
    };
    ($layout:ident,base.timer) => {
        $crate::actors::boss_fields_owned::Field::BaseTimer
    };
    ($layout:ident,base.current_instruction) => {
        $crate::actors::boss_fields_owned::Field::BaseCurrentInstruction
    };
    ($layout:ident,base.instruction_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInstructionTimer
    };
    ($layout:ident,base.palette_index) => {
        $crate::actors::boss_fields_owned::Field::BasePaletteIndex
    };
    ($layout:ident,base.vram_tiles_index) => {
        $crate::actors::boss_fields_owned::Field::BaseVramTilesIndex
    };
    ($layout:ident,base.flash_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseFlashTimer
    };
    ($layout:ident,base.invincibility_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInvincibilityTimer
    };
    ($layout:ident,base.shake_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseShakeTimer
    };
    ($layout:ident,base.frame_counter) => {
        $crate::actors::boss_fields_owned::Field::BaseFrameCounter
    };
    (EnemyBase,enemy_ptr) => {
        $crate::actors::boss_fields_owned::Field::BaseEnemyPtr
    };
    (EnemyBase,x_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseXPos
    };
    (EnemyBase,x_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseXSubpos
    };
    (EnemyBase,y_pos) => {
        $crate::actors::boss_fields_owned::Field::BaseYPos
    };
    (EnemyBase,y_subpos) => {
        $crate::actors::boss_fields_owned::Field::BaseYSubpos
    };
    (EnemyBase,x_width) => {
        $crate::actors::boss_fields_owned::Field::BaseXWidth
    };
    (EnemyBase,y_height) => {
        $crate::actors::boss_fields_owned::Field::BaseYHeight
    };
    (EnemyBase,properties) => {
        $crate::actors::boss_fields_owned::Field::BaseProperties
    };
    (EnemyBase,extra_properties) => {
        $crate::actors::boss_fields_owned::Field::BaseExtraProperties
    };
    (EnemyBase,ai_handler_bits) => {
        $crate::actors::boss_fields_owned::Field::BaseAiHandlerBits
    };
    (EnemyBase,health) => {
        $crate::actors::boss_fields_owned::Field::BaseHealth
    };
    (EnemyBase,spritemap_pointer) => {
        $crate::actors::boss_fields_owned::Field::BaseSpritemapPointer
    };
    (EnemyBase,timer) => {
        $crate::actors::boss_fields_owned::Field::BaseTimer
    };
    (EnemyBase,current_instruction) => {
        $crate::actors::boss_fields_owned::Field::BaseCurrentInstruction
    };
    (EnemyBase,instruction_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInstructionTimer
    };
    (EnemyBase,palette_index) => {
        $crate::actors::boss_fields_owned::Field::BasePaletteIndex
    };
    (EnemyBase,vram_tiles_index) => {
        $crate::actors::boss_fields_owned::Field::BaseVramTilesIndex
    };
    (EnemyBase,flash_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseFlashTimer
    };
    (EnemyBase,invincibility_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseInvincibilityTimer
    };
    (EnemyBase,shake_timer) => {
        $crate::actors::boss_fields_owned::Field::BaseShakeTimer
    };
    (EnemyBase,frame_counter) => {
        $crate::actors::boss_fields_owned::Field::BaseFrameCounter
    };
    (Brain,explosion_timer) => {
        $crate::actors::boss_fields_owned::Field::MotherBrainEffectsBrainExplosionTimer
    };
    (Brain,explosion_index) => {
        $crate::actors::boss_fields_owned::Field::MotherBrainEffectsBrainExplosionIndex
    };
    (Brain,rainbow_palette_index) => {
        $crate::actors::boss_fields_owned::Field::MotherBrainEffectsBrainRainbowPaletteIndex
    };
}
pub(crate) use {mother_brain_effects_field, mother_brain_effects_part};
