//! One deterministic subsystem batch: lifecycle, scripts, tail and frame chains.
use crate::digest;
use crate::family_model::{Asset, Case, Manifest, Routine, Write};
use anyhow::{Context, Result, ensure};
use std::path::Path;

const ROM_SHA256: &str = "12b77c4bc9c1832cee8881244659065ee1d84c70c3d29e6eaf92e6798cc2ca72";
fn write(offset: usize, value: u16) -> Write {
    Write { offset, data: value.to_le_bytes().to_vec() }
}
fn base(area: u16, orientation: u16) -> Vec<Write> {
    vec![
        write(0xe54, 0),
        write(0xf78, if area == 2 { 0xe17f } else { 0xe13f }),
        write(0xfa6, 0xa6),
        write(0xf7a, 160),
        write(0xf7e, 160),
        write(0xf82, 16),
        write(0xf84, 24),
        write(0xf86, 0x2000),
        write(0xf8c, 10000),
        write(0xf92, 0xe538),
        write(0xf94, 1),
        write(0xf96, 3584),
        write(0xfa4, 2),
        write(0xfa8, 0xa354),
        write(0x79f, area),
        write(0x7a5, 32),
        write(0x7a7, 32),
        write(0xaf6, 128),
        write(0xafa, 256),
        write(0xafe, 8),
        write(0xb00, 16),
        write(0xa1f, 0),
        write(0x9c2, 99),
        write(0x5e5, 0x4abc),
        write(0x7802, 1),
        write(0x7804, 1),
        write(0x7806, 0),
        write(0x780a, 0xda71),
        write(0x780c, 1),
        write(0x780e, 5),
        write(0x7812, 32),
        write(0x7818, 3584),
        write(0x7820, orientation),
        write(0x7824, 0),
        write(0x7838, 15),
        write(0x783a, 0),
        write(0x7840, 0),
        write(0x7842, 416),
        write(0x7844, 32),
        write(0x7846, 224),
        write(0x784e, 0),
        write(0x8800, 0xbe9c),
        write(0x2000, 1),
        write(0x2012, 240),
        write(0x2014, 1),
        write(0x2016, 16368),
        write(0x2018, 16448),
        write(0x201a, 0xffff),
        write(0x201c, 0xffff),
        write(0x201e, 16),
        write(0x202a, 0x4000),
        write(0x203e, 0x4000),
        write(0x2052, 0x4000),
        write(0x2066, 0x4000),
        write(0x207a, 0x4000),
        write(0x208e, 0x4000),
        write(0x20a2, 0x4000),
    ]
}
fn host() -> Vec<u16> {
    vec![0x5aa5, 0xa55a, 11, 22, 33, 44, 55, 66]
}
fn routine(symbol: &str, args: &[&str], returns: &str) -> Routine {
    Routine {
        symbol: symbol.into(),
        args: args.len(),
        arg_types: Some(args.iter().map(|v| (*v).into()).collect()),
        returns: returns.into(),
        live_eligible: Some(false),
        live_limitation: Some("Subsystem acceptance requires the independent live Ridley frame gate.".into()),
        ..Default::default()
    }
}
fn case(name: String, area: u16, orientation: u16, updates: &[Write]) -> Case {
    let mut writes = base(area, orientation);
    writes.extend_from_slice(updates);
    Case { name, writes, host_words: Some(host()), ..Default::default() }
}
fn add(routines: &mut [Routine], symbol: &str, case: Case) {
    routines.iter_mut().find(|r| r.symbol == symbol).expect("registered subsystem entry").cases.push(case);
}

fn manifest_toml(manifest: &Manifest) -> Result<String> {
    let mut document = toml::to_string_pretty(manifest)?.parse::<toml_edit::DocumentMut>()?;
    for routine in document["routines"].as_array_of_tables_mut().expect("serialized routines").iter_mut() {
        for case in routine["cases"].as_array_of_tables_mut().expect("serialized cases").iter_mut() {
            let Some(writes) = case.get("writes").and_then(toml_edit::Item::as_array_of_tables) else { continue };
            let mut array = toml_edit::Array::new();
            for write in writes {
                let mut record = toml_edit::InlineTable::new();
                for (key, item) in write {
                    let mut value = item.as_value().expect("scalar fixture write").clone();
                    value.decor_mut().clear();
                    if let Some(data) = value.as_array_mut() {
                        data.fmt();
                    }
                    record.insert(key, value);
                }
                record.fmt();
                array.push(record);
            }
            array.fmt();
            case["writes"] = toml_edit::value(array);
        }
    }
    Ok(document.to_string())
}

pub fn generate(checkout: &Path, rom: &Path, output: &Path) -> Result<Manifest> {
    ensure!(digest(rom)? == ROM_SHA256, "whole Ridley fixture ROM identity differs");
    let entries = [
        "CeresRidley_Init",
        "CeresRidley_Main",
        "CeresRidley_Hurt",
        "Ridley_Main",
        "Ridley_Hurt",
        "Ridley_Func_2",
        "Ridley_Shot",
        "Ridley_Powerbomb",
        "sub_A6DF59",
        "sub_A6DF60",
        "Ridley_A2F2",
        "HandleCeresRidleyGetawayCutscene",
        "RidleysExplosion_Init",
        "RidleysExplosion_Main",
        "Ridley_Func_70",
        "ProcessEnemyInstructions",
    ];
    let mut routines = entries.into_iter().map(|s| routine(s, &[], "void")).collect::<Vec<_>>();
    routines.push(routine("CallRidleyFunc", &["uint32"], "void"));
    for index in 1..=16 {
        routines.push(routine(&format!("Ridley_Instr_{index}"), &["uint16", "rom_ptr"], "rom_ptr"));
    }
    for area in [2, 6] {
        for orientation in [0, 1, 2] {
            for symbol in [
                "CeresRidley_Init",
                "CeresRidley_Main",
                "CeresRidley_Hurt",
                "Ridley_Main",
                "Ridley_Hurt",
                "Ridley_Func_2",
                "Ridley_Shot",
                "Ridley_Powerbomb",
                "sub_A6DF59",
                "sub_A6DF60",
                "Ridley_A2F2",
            ] {
                add(
                    &mut routines,
                    symbol,
                    case(format!("entry-{symbol}-a{area}-o{orientation}"), area, orientation, &[]),
                );
            }
            for defeated in [0, 1] {
                add(
                    &mut routines,
                    "CeresRidley_Init",
                    case(
                        format!("init-defeated{defeated}-a{area}-o{orientation}"),
                        area,
                        orientation,
                        &[write(0xd828 + usize::from(area), defeated)],
                    ),
                );
            }
        }
    }
    // Every state is exercised as one subsystem entry and again inside complete AI frames.
    let states = [
        0xa354, 0xa35b, 0xa377, 0xa389, 0xa3df, 0xa455, 0xa478, 0xa6af, 0xa6c8, 0xa6e8, 0xa782, 0xa7f9, 0xa83c, 0xa84e,
        0xa88d, 0xa8a4, 0xa8d4, 0xa8f8, 0xa923, 0xa947, 0xa971, 0xa9a0, 0xaa11, 0xaa4f, 0xaa50, 0xb2f3, 0xb321, 0xb3ec,
        0xb3f8, 0xb441, 0xb455, 0xb493, 0xb4d1, 0xb516, 0xb554, 0xb594, 0xb5c4, 0xb5e5, 0xb613, 0xb6a7, 0xb6dd, 0xb70e,
        0xb7b9, 0xbab7, 0xbb8f, 0xbbc4, 0xbbf1, 0xbc2e, 0xbd4e, 0xbd9a, 0xbdbc, 0xbdf2, 0xbe03, 0xc04e, 0xc538, 0xc53e,
        0xc551, 0xc588, 0xc5a8, 0xc5c8, 0xc5da, 0xc600,
    ];
    for state in states {
        for area in [2, 6] {
            for orientation in [0, 2] {
                for timer in [0, 1, 32, 0xffff] {
                    let mut fixture = case(
                        format!("state-{state:04x}-a{area}-o{orientation}-t{timer:04x}"),
                        area,
                        orientation,
                        &[
                            write(0xfa8, state),
                            write(0xfae, 0),
                            write(0xfb2, timer),
                            write(0x7800, 0),
                            write(0x7834, 0),
                            write(0x8836, 0),
                        ],
                    );
                    if state == 0xc04e {
                        fixture.writes.push(write(0xfb2, 0));
                    }
                    fixture.a = i64::from(0xa60000u32 | u32::from(state));
                    add(&mut routines, "CallRidleyFunc", fixture.clone());
                    fixture.a = 0;
                    fixture.name = format!("main-{}", fixture.name);
                    add(&mut routines, if area == 2 { "Ridley_Main" } else { "CeresRidley_Main" }, fixture);
                }
            }
        }
    }
    for state in 0..9 {
        for delay in [0, 1, 2, 0xffff] {
            for active in [0, 0x8000] {
                for orientation in [0, 1, 2] {
                    let mut updates =
                        vec![write(0x2000, state), write(0x2010, delay), write(0x2014, 8), write(0x781c, 15)];
                    for i in (16..=76).step_by(10) {
                        updates.extend([
                            write(0x2000 + i * 2, active),
                            write(0x2000 + (i + 1) * 2, 17),
                            write(0x2000 + (i + 2) * 2, 0x8000),
                            write(0x2000 + (i + 3) * 2, 0x1800),
                        ]);
                    }
                    add(
                        &mut routines,
                        "Ridley_Func_70",
                        case(
                            format!("tail-s{state}-d{delay:04x}-f{active:04x}-o{orientation}"),
                            2,
                            orientation,
                            &updates,
                        ),
                    );
                }
            }
        }
    }
    for index in 1..=16 {
        for orientation in [0, 1, 2] {
            for carry in [0, 1] {
                let mut fixture = case(
                    format!("callback-{index}-o{orientation}-carry{carry}"),
                    2,
                    orientation,
                    &[write(0x7836, carry)],
                );
                fixture.a = 64;
                // Actual script arguments containing branches/frame-safe operands.
                fixture.b = 0xa6e73c;
                add(&mut routines, &format!("Ridley_Instr_{index}"), fixture);
            }
        }
    }
    for parameter in (0..=22).step_by(2) {
        for orientation in [0, 2] {
            add(
                &mut routines,
                "RidleysExplosion_Init",
                case(
                    format!("explosion-p{parameter}-o{orientation}"),
                    2,
                    orientation,
                    &[write(0xf78, 0xe1bf), write(0xfb4, parameter)],
                ),
            );
        }
    }
    for timer in [0, 1, 2, 0xffff] {
        add(
            &mut routines,
            "RidleysExplosion_Main",
            case(format!("explosion-timer{timer:04x}"), 2, 0, &[write(0xf78, 0xe1bf), write(0xfb2, timer)]),
        );
    }
    for frame in [0, 2, 100, 208, 220] {
        add(
            &mut routines,
            "HandleCeresRidleyGetawayCutscene",
            case(format!("getaway-frame{frame}"), 2, 0, &[write(0x8026, frame), write(0x802a, 32), write(0x8014, 1)]),
        );
    }
    // Branch-linked chains run the actual scripts without continuation patches.
    let mut order = Vec::new();
    for script in [0xe538, 0xe548, 0xe658, 0xe690, 0xe6c8, 0xe6f0, 0xe706, 0xe73a, 0xe91d] {
        for orientation in [0, 2] {
            let name = format!("script-{script:04x}-o{orientation}-000");
            add(
                &mut routines,
                "ProcessEnemyInstructions",
                case(name.clone(), 2, orientation, &[write(0xf92, script), write(0xf94, 1)]),
            );
            order.push(name.clone());
            let mut previous = name;
            for frame in 1..=224 {
                let name = format!("script-{script:04x}-o{orientation}-{frame:03}");
                add(
                    &mut routines,
                    "ProcessEnemyInstructions",
                    Case { name: name.clone(), continue_from: Some(previous), ..Default::default() },
                );
                order.push(name.clone());
                previous = name;
            }
        }
    }
    let cases = routines.iter().map(|r| r.cases.len()).sum::<usize>();
    let mut native_order = routines
        .iter()
        .flat_map(|r| r.cases.iter().filter(|c| !c.name.starts_with("script-")).map(|c| c.name.clone()))
        .collect::<Vec<_>>();
    native_order.extend(order);
    let manifest = Manifest {
        family: "ridley-subsystem".into(),
        bank: "sm_a6.c".into(),
        rust_module: "crates/port/src/ridley_subsystem.rs".into(),
        rust_imports: vec![
            "audio".into(),
            "enemy".into(),
            "enemy_projectiles".into(),
            "observed_projectile_ownership".into(),
            "ridley_enemy_instructions".into(),
        ],
        rust_dependencies: vec![
            "crates/port/src/ridley_movement.rs".into(),
            "crates/port/src/ridley_rendering.rs".into(),
            "crates/port/src/ridley_cutscene.rs".into(),
            "crates/port/src/ridley_subsystem_instructions.rs".into(),
            "crates/port/src/ridley_subsystem_admission.rs".into(),
        ],
        projectile_host: true,
        callback_adapter: Some("ridley-subsystem".into()),
        rom: Some(Asset { file: rom.into(), sha256: ROM_SHA256.into() }),
        routines,
        case_order: Some(native_order),
        scope: Some(format!(
            "Whole Ridley/Ceres lifecycle and script batch, {cases} fixture pairs; typed actual-C services remain explicit dependencies. Live ownership and original-ROM parity require separate acceptance."
        )),
        ..Default::default()
    };
    // Keep fixture writes reviewable without a million repetitive table lines.
    // The typed round trip must match before this manifest is published.
    let text = manifest_toml(&manifest)?;
    ensure!(toml::from_str::<Manifest>(&text)? == manifest, "compact Ridley manifest round trip differs");
    let pending = output.with_extension(format!("pending-{}", std::process::id()));
    let result = (|| -> Result<()> {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new().create_new(true).write(true).open(&pending)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&pending, output)?;
        std::fs::File::open(output.parent().context("Ridley manifest parent missing")?)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&pending);
    }
    result?;
    ensure!(checkout.join(&manifest.rust_module).is_file(), "whole Ridley maintained module missing");
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn canonical_state_records_do_not_alias_host_or_audio_cursors() {
        let writes = base(2, 2);
        assert!(writes.iter().any(|w| w.offset == 0x7820 && w.data == [2, 0]));
        assert!(writes.iter().all(|w| ![0x590, 0x639, 0x63b, 0x643, 0x646].contains(&w.offset)));
        assert!(writes.iter().any(|w| w.offset == 0x780a && w.data == [0x71, 0xda]));
    }

    #[test]
    fn compact_writes_preserve_typed_fixture_contract() {
        let mut entry = routine("Ridley_Main", &[], "void");
        entry.cases.push(case("main".into(), 2, 2, &[write(0x330, 0xffff)]));
        let manifest = Manifest { routines: vec![entry], ..Default::default() };
        let text = manifest_toml(&manifest).unwrap();
        assert!(!text.contains("[[routines.cases.writes]]"));
        assert!(text.contains("offset = 816"));
        assert_eq!(toml::from_str::<Manifest>(&text).unwrap(), manifest);
    }
}
