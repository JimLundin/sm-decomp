//! Exact dispatcher and dependency-call patches; original instruction bodies remain.
use super::{PatchRecord, replace_once};
use std::fs;
use std::path::Path;

pub(super) fn instrument(root: &Path, patches: &mut Vec<PatchRecord>) -> Result<(), String> {
    let path = "src/sm_84.c";
    let names = [
        "Delete",
        "PreInstr",
        "ClearPreInstr",
        "Goto",
        "SetTimer",
        "GotoIfItemBitSet",
        "SetItemBit",
        "CollectAmmoMissileTank",
        "SetLinkReg",
        "Call",
        "Return",
        "ClearMusicQueueAndQueueTrack",
        "LoadItemPlmGfx",
        "DrawItemFrame0",
        "DrawItemFrame1",
        "PickupEquipmentAndShowMessage",
        "CollectHealthEnergyTank",
        "CollectHealthReserveTank",
        "CollectAmmoSuperMissileTank",
        "CollectAmmoPowerBombTank",
    ];
    for (entry, name) in names.iter().enumerate() {
        let entry = if entry >= 12 { entry + 1 } else { entry }; // 12 is the preinstruction.
        replace_once(
            root,
            path,
            &format!("  case fnPlmInstr_{name}: return PlmInstr_{name}(j, k);"),
            &format!("  case fnPlmInstr_{name}: return sm_native_item_instruction({entry}, j, k);"),
            patches,
        )?;
    }
    replace_once(
        root,
        path,
        "  case fnPlmPreInstr_GotoLinkIfTriggered: PlmPreInstr_GotoLinkIfTriggered(k); return;",
        "  case fnPlmPreInstr_GotoLinkIfTriggered: sm_native_item_preinstruction(k); return;",
        patches,
    )?;
    for (name, calls) in [
        (
            "CollectAmmoMissileTank",
            vec![
                ("AddMissilesToHudTilemap();", "sm_native_item_instruction_native_service(0, 0);"),
                ("PlayRoomMusicTrackAfterAFrames(0x168);", "sm_native_item_instruction_native_service(1, 0x168);"),
            ],
        ),
        (
            "ClearMusicQueueAndQueueTrack",
            vec![("QueueMusic_Delayed8(plmp[0]);", "sm_native_item_instruction_native_service(2, plmp[0]);")],
        ),
        (
            "PickupEquipmentAndShowMessage",
            vec![("PlayRoomMusicTrackAfterAFrames(0x168);", "sm_native_item_instruction_native_service(1, 0x168);")],
        ),
        (
            "CollectHealthEnergyTank",
            vec![("PlayRoomMusicTrackAfterAFrames(0x168);", "sm_native_item_instruction_native_service(1, 0x168);")],
        ),
        (
            "CollectHealthReserveTank",
            vec![("PlayRoomMusicTrackAfterAFrames(0x168);", "sm_native_item_instruction_native_service(1, 0x168);")],
        ),
        (
            "CollectAmmoSuperMissileTank",
            vec![
                ("AddSuperMissilesToHudTilemap();", "sm_native_item_instruction_native_service(6, 0);"),
                ("PlayRoomMusicTrackAfterAFrames(0x168);", "sm_native_item_instruction_native_service(1, 0x168);"),
            ],
        ),
        (
            "CollectAmmoPowerBombTank",
            vec![
                ("AddPowerBombsToHudTilemap();", "sm_native_item_instruction_native_service(7, 0);"),
                ("PlayRoomMusicTrackAfterAFrames(0x168);", "sm_native_item_instruction_native_service(1, 0x168);"),
            ],
        ),
        (
            "DrawItemFrame_Common",
            vec![
                ("ProcessPlmDrawInstruction(k);", "sm_native_item_instruction_native_service(3, k);"),
                ("CalculatePlmBlockCoords(plm_id);", "sm_native_item_instruction_native_service(4, plm_id);"),
                ("DrawPlm(v3);", "sm_native_item_instruction_native_service(5, v3);"),
            ],
        ),
    ] {
        let source = fs::read_to_string(root.join(path)).map_err(|e| e.to_string())?;
        let declaration = format!("const uint8 *PlmInstr_{name}(");
        if source.matches(&declaration).count() != 1 {
            return Err(format!("ambiguous item body {name}"));
        }
        let start = source.find(&declaration).unwrap();
        let open = start + source[start..].find('{').ok_or("missing item body")?;
        let end = super::room_transition_prepare::structural_end(&source, open, b'{', b'}')?;
        let original = &source[start..end];
        let mut updated = original.to_owned();
        for (before, after) in calls {
            if updated.matches(before).count() != 1 {
                return Err(format!("missing exact item service {name}: {before}"));
            }
            updated = updated.replacen(before, after, 1);
        }
        replace_once(root, path, original, &updated, patches)?;
    }
    Ok(())
}
