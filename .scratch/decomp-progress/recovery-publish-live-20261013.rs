use std::{fs, path::Path, process::Command};

fn main() {
    let root = Path::new("/home/jim/sm-decomp");
    let output = root.join(".scratch/decomp-progress");
    let scopes = [
        ("camera-scroll-root", "Complete camera scrolling frame", vec!["MainScrollingRoutine"], "94,425 exclusive Rust calls; 35,580 active; zero refusals per mixed subject."),
        ("room-scroll-roots", "Complete scheduled room scrolling roots", vec!["UpdateScrollVarsUpdateMap", "CalculateLayer2PosAndScrollsWhenScrolling"], "3,875/3,875 and 94,608/94,608 total/active exclusive Rust calls; zero refusals per mixed subject."),
        ("samus-projectile-frame-roots", "Complete scheduled Samus projectile update and render roots", vec!["HandleProjectile", "DrawPlayerExplosions2", "DrawBombAndProjectileExplosions"], "97,049/21,746, 102,632/21,558 and 96,336/20,998 total/active exclusive Rust calls; zero refusals per mixed subject."),
        ("observed-ridley-instructions-root", "Complete bounded Ridley attack interpreter", vec!["ProcessEnemyInstructions"], "2,393 exclusive Rust frames, 36 timer-due calls, 48 matching opcodes and sixteen 9642 allocations per mixed subject; 209,673 other frames remain explicitly C-owned."),
        ("observed-projectile-ownership-roots", "Complete admitted native projectile allocator and scheduler", vec!["SpawnEprojWithRoomGfx", "SpawnEprojWithGfx", "EprojRunAll"], "16, 8 and 95,325 exclusive Rust calls; 866 active scheduler calls, 617 9642 calls and all sixteen 9642 births; zero native137 refusals per mixed subject."),
        ("enemy-projectile-rendering-roots", "Both complete scheduled enemy projectile OAM render passes", vec!["DrawLowPriorityEprojs", "DrawHighPriorityEprojs"], "96,336 exclusive Rust calls per root and mixed subject; native C traversal and ordered output match."),
        ("enemy-projectile-contact-roots", "Complete enemy projectile Samus and projectile contact passes", vec!["EprojSamusCollDetect", "EprojProjCollDet"], "94,425 exclusive Rust calls per root and mixed subject; native C traversal and complete output match."),
    ];
    for (id, scope, routines, domain) in scopes {
        let symbols = routines.iter().map(|name| format!("{name:?}")).collect::<Vec<_>>().join(", ");
        let text = format!(
            "id = \"{id}-6053-live-20261013\"\nscope = {scope:?}\nagent = \"/root\"\nstage = \"live-exercised\"\nevidence_context = \"current-checkout\"\nsource_revision = \"578f90b3cc49557bb70060ad033bb90b8cf8ac50\"\ninventory_sha256 = \"7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25\"\nroutines = [{symbols}]\ncommit = \"3ce28ba58aba319b60bbc949153baafd4433132e\"\nlimitations = [\n{domain:?},\n\"Four fresh independent C/mixed subjects over 129,712 raw inputs; periodic/terminal WRAM/SRAM, ordered PPU/APU effects and full captures match with zero CPU fallback.\",\n\"The raw-input route ends in Ceres room DF8D/game state 8, not credits; original-ROM movie parity and causal controller scheduling remain separate.\",\n\"This observation reports installed and measured roots only. Other enemy scripts, whole Ridley AI, real boss/explosion and unmeasured internal helpers do not inherit live coverage.\",\n\"Gameplay source at observation matches integrated commit 3ce28ba. Exact verification CLI/source is retained at /tmp/sm-recovery-bounded-source; later source edits require their own relevant checks.\",\n]\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/out/recovery-session/composed-live-release/1791861180321696848-129445/verdict.toml\"\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/.scratch/decomp-port/recovery-composed-post-cleanup-authentication.toml\"\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/.scratch/decomp-port/recovery-composed-final-cleanup.toml\"\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/.scratch/decomp-port/recovery-composed-final-dedup.toml\"\n\n[[evidence]]\npath = \"/home/jim/sm-decomp/.scratch/decomp-port/recovery-live-completion-20261013.md\"\n"
        );
        let update = output.join(format!("{id}-6053-live-20261013.toml"));
        fs::write(&update, text).unwrap();
        let result = Command::new(root.join("target/debug/sm-pipeline"))
            .args(["progress", "update", "--root", "/home/jim/sm-decomp", "--update"])
            .arg(&update)
            .output()
            .unwrap();
        fs::write(output.join(format!("{id}-6053-live-20261013.stdout")), &result.stdout).unwrap();
        fs::write(output.join(format!("{id}-6053-live-20261013.stderr")), &result.stderr).unwrap();
        assert!(result.status.success(), "{id}: {}", String::from_utf8_lossy(&result.stderr));
        println!("Published {id}: {} roots", routines.len());
    }
}
