//! Exact scoped patches for the three table entries and 27 retained routines.
use super::{PatchRecord, replace_once};
use std::fs;
use std::path::Path;

const ROUTINES: [&str; 27] = [
    "GameState_9_HitDoorBlock",
    "GameState_10_LoadingNextRoom_Async",
    "GameState_11_LoadingNextRoom_Async",
    "CallDoorTransitionFunction_Async",
    "DoorTransitionFunction_HandleElevator",
    "DoorTransitionFunction_Wait48frames",
    "DoorTransitionFunction_WaitForSoundsToFinish",
    "DoorTransitionFunction_FadeOutScreen",
    "DoorTransitionFunction_LoadDoorHeaderEtc",
    "DoorTransitionFunction_ScrollScreenToAlignment",
    "DoorTransitionFunction_FixDoorsMovingUp",
    "DoorTransitionFunction_SetupNewRoom",
    "DoorTransitionFunction_SetupScrolling",
    "DoorTransitionFunction_PlaceSamusLoadTiles",
    "DoorTransitionFunction_LoadMoreThings_Async",
    "DoorTransition_C_HandleAnimTiles",
    "DoorTransition_WaitForMusicToClear",
    "DoorTransition_HandleTransition",
    "DoorTransition_FadeInScreenAndFinish",
    "UpdateBackgroundCommand_0_Finish",
    "UpdateBackgroundCommand_2_TransferToVram",
    "UpdateBackgroundCommand_4_Decompression",
    "UpdateBackgroundCommand_6_ClearFxTilemap",
    "UpdateBackgroundCommand_8_TransferToVramAndSetBg3",
    "UpdateBackgroundCommand_A_ClearBg2Tilemap",
    "UpdateBackgroundCommand_C_ClearKraidBg2Tilemap",
    "UpdateBackgroundCommand_E_DoorDependentTransferToVRAM",
];
const SERVICES: [&str; 53] = [
    "DetermineWhichEnemiesToProcess",
    "EnemyMain",
    "DrawSamusEnemiesAndProjectiles",
    "EnsureSamusDrawnEachFrame",
    "LoaadDesinationRoomCreBitset",
    "DrawTimer",
    "ClearSoundsWhenGoingThroughDoor",
    "QueueSfx2_Max15",
    "LoadDoorHeader",
    "sub_8882AC",
    "CalculateLayer2PosAndScrollsWhenScrolling",
    "FixDoorsMovingUp",
    "SaveMapExploredifElevator",
    "LoadRoomHeader",
    "LoadStateHeader",
    "LoadMapExploredIfElevator",
    "InitializeSpecialEffectsForNewRoom",
    "LoadLevelScrollAndCre",
    "DoorTransitionScrollingSetup",
    "WaitUntilEndOfVblankAndEnableIrq",
    "DecompressToMem",
    "CopyToVramNow",
    "LoadEnemyGfxToVram",
    "LoadRoomMusic",
    "ClearEprojs",
    "ClearAnimtiles",
    "ClearPaletteFXObjects",
    "ClearPLMs",
    "CreatePlmsExecuteDoorAsmRoomSetup",
    "LoadFXHeader",
    "SpawnDoorClosingPLM",
    "UpdateBeamTilesAndPalette",
    "LoadColorsForSpritesBeamsAndEnemies",
    "LoadEnemies",
    "InitializeEnemies",
    "ResetProjectileData",
    "Samus_LoadSuitTargetPalette",
    "ClearFxTilemap",
    "SpawnBG3ScrollHdmaObject",
    "ClearBg2Tilemap",
    "ClearKraidBg2Tilemap",
    "AnimtilesHandler",
    "LoadNewMusicTrackIfChanged",
    "PointlessFunctionStupidToo",
    "SetLiquidPhysicsType",
    "EprojRunAll",
    "QueueEnemyBG2TilemapTransfers",
    "PlaySpinJumpSoundIfSpinJumping",
    "AdvancePaletteFadeForAllPalettes_0xc",
    "HasQueuedMusic",
    "CallSomeSamusCode",
    "WaitForNMI_Async",
    "PlmHandler_Async",
];

// The pinned definitions have no brace-containing strings. Strip comments and
// literals from structural scanning anyway, so exact extraction never consumes
// an adjacent routine because a diagnostic or comment contains punctuation.
pub(super) fn structural_end(source: &str, start: usize, open: u8, close: u8) -> Result<usize, String> {
    let bytes = source.as_bytes();
    let mut depth = 0;
    let mut at = start;
    let mut quote = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        if quote != 0 {
            if byte == b'\\' {
                at += 2;
                continue;
            }
            if byte == quote {
                quote = 0;
            }
        } else if byte == b'"' || byte == b'\'' {
            quote = byte;
        } else if bytes[at..].starts_with(b"//") {
            while at < bytes.len() && bytes[at] != b'\n' {
                at += 1;
            }
            continue;
        } else if bytes[at..].starts_with(b"/*") {
            let offset = source[at + 2..].find("*/").ok_or("unterminated C comment")?;
            at += offset + 4;
            continue;
        } else if byte == open {
            depth += 1;
        } else if byte == close {
            depth -= 1;
            if depth == 0 {
                return Ok(at + 1);
            }
        }
        at += 1;
    }
    Err("unterminated scoped C delimiter".into())
}

fn arguments(text: &str) -> Vec<&str> {
    if text.trim().is_empty() {
        return Vec::new();
    }
    let mut result = Vec::new();
    let mut depth = 0;
    let mut start = 0;
    for (at, byte) in text.bytes().enumerate() {
        match byte {
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => depth -= 1,
            b',' if depth == 0 => {
                result.push(text[start..at].trim());
                start = at + 1;
            }
            _ => {}
        }
    }
    result.push(text[start..].trim());
    result
}

fn instrument_calls(body: &str, seen: &mut [usize; 53]) -> Result<String, String> {
    let mut result = body.to_owned();
    for (id, name) in SERVICES.iter().enumerate() {
        let anchor = format!("{name}(");
        let mut cursor = 0;
        while let Some(relative) = result[cursor..].find(&anchor) {
            let start = cursor + relative;
            if start > 0
                && (result.as_bytes()[start - 1].is_ascii_alphanumeric() || result.as_bytes()[start - 1] == b'_')
            {
                cursor = start + anchor.len();
                continue;
            }
            let open = start + name.len();
            let end = structural_end(&result, open, b'(', b')')?;
            let args = arguments(&result[open + 1..end - 1]);
            let encoded = match id {
                7 | 50 if args.len() == 1 => format!("(uint16)({}), 0, 0", args[0]),
                20 if args.len() == 2 => format!("(uint32)({}), (uint16)((uint8 *)({}) - g_ram), 0", args[0], args[1]),
                21 if args.len() == 3 => format!("(uint16)({}), (uint32)({}), (uint16)({})", args[0], args[1], args[2]),
                _ if args.is_empty() => "0, 0, 0".into(),
                _ => return Err(format!("unexpected scoped {name} arguments")),
            };
            let replacement = format!("sm_native_room_transition_call({id}, {encoded})");
            result.replace_range(start..end, &replacement);
            cursor = start + replacement.len();
            seen[id] += 1;
        }
    }
    Ok(result)
}

pub(super) fn instrument(root: &Path, patches: &mut Vec<PatchRecord>) -> Result<(), String> {
    let path = "src/sm_82.c";
    replace_once(
        root,
        path,
        "  GameState_9_HitDoorBlock,\n  GameState_10_LoadingNextRoom_Async,\n  GameState_11_LoadingNextRoom_Async,",
        "  sm_native_room_transition_9,\n  sm_native_room_transition_10,\n  sm_native_room_transition_11,",
        patches,
    )?;
    let mut seen = [0; 53];
    for (id, name) in ROUTINES.iter().enumerate() {
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let return_type = if id < 19 { "CoroutineRet" } else { "uint16" };
        let declaration = format!("{return_type} {name}(");
        let start = source.find(&declaration).ok_or_else(|| format!("missing scoped routine {name}"))?;
        // Pinned definitions are uniquely identified, not merely function calls.
        if source.matches(&declaration).count() != 1 {
            return Err(format!("ambiguous scoped routine {name}"));
        }
        let open = source[start..].find('{').ok_or("missing scoped routine body")? + start;
        let end = structural_end(&source, open, b'{', b'}')?;
        let head = source[start..open].trim_end();
        let parenthesis = head.find('(').unwrap();
        let close = head.rfind(')').ok_or("missing scoped routine arguments")?;
        let params = &head[parenthesis + 1..close];
        let args = match id {
            3 => "ea",
            19 => "y",
            20..=26 => "j",
            _ => "",
        };
        let operand = if id == 3 {
            "(uint16)ea"
        } else if id >= 19 {
            args
        } else {
            "0"
        };
        let body = instrument_calls(&source[open + 1..end - 1], &mut seen)?;
        let replacement = format!(
            "static {return_type} NativeTransitionOriginal_{name}({params});\n\
             {return_type} {name}({params}) {{\n\
               sm_native_room_transition_routine(0, {id}, {operand});\n\
               {return_type} result = NativeTransitionOriginal_{name}({args});\n\
               sm_native_room_transition_routine(1, {id}, result);\n\
               return result;\n\
             }}\n\
             static {return_type} NativeTransitionOriginal_{name}({params}) {{{body}}}"
        );
        replace_once(root, path, &source[start..end], &replacement, patches)?;
    }
    if let Some((id, _)) = seen.iter().enumerate().find(|(_, n)| **n == 0) {
        return Err(format!("room-transition service {} is absent from selected source", SERVICES[id]));
    }
    Ok(())
}
