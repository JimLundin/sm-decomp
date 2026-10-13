use std::{fs, path::Path, process::Command};

const REPO: &str = "/home/jim/sm-decomp";
const REVISION: &str = "578f90b3cc49557bb70060ad033bb90b8cf8ac50";
const INVENTORY: &str = "7842e9035e7360eef10a93a96e6abc1cca601888b8423ea92fb554bbb15abb25";

struct Scope {
    family: &'static str,
    name: &'static str,
    owner: &'static str,
    commit: &'static str,
    expected_native: usize,
    report: &'static str,
    limitations: &'static [&'static str],
    extra_evidence: &'static [&'static str],
}

fn main() {
    let repo = Path::new(REPO);
    let scopes = [
        Scope {
            family: "projectile-trails",
            name: "Complete native projectile trail script and render frame",
            owner: "/root/independent_gameplay",
            commit: "2b62134bfbee7c5dcd4c81616edc5d91e781d44f",
            expected_native: 4,
            report: "recovery-integrated-projectile-trails-post-cleanup-authentication.toml",
            limitations: &[
                "Current frozen merged-source family authenticates all 3,316 controlled fixture pairs; earlier fourteen natural frozen-context captures remain separately retained.",
                "Direct movement opcodes had no hits in the recorded 9,000-input natural prefix. Creation/placement reuse projectile_flight; independent live parity remains pending.",
            ],
            extra_evidence: &[],
        },
        Scope {
            family: "samus-projectile-frame",
            name: "Complete scheduled Samus projectile update and render passes",
            owner: "/root/independent_gameplay",
            commit: "c711faea56f20d42113eb63393cc4a76d4940557",
            expected_native: 4,
            report: "recovery-integrated-samus-projectile-frame-post-cleanup-authentication.toml",
            limitations: &[
                "Current frozen merged-source family authenticates 476 whole-frame fixture pairs across all four scheduled boundaries.",
                "PLM/HDMA services remain explicit C dependencies; pinned C uninitialized-local Ceres drawing branches refuse. Independent live ownership remains pending.",
            ],
            extra_evidence: &[],
        },
        Scope {
            family: "camera-scroll",
            name: "Complete camera scrolling frame and directional helpers",
            owner: "/root/projectile_admission",
            commit: "7512b6479afe55da270931a9f3ae26c9d558e5a5",
            expected_native: 4,
            report: "recovery-integrated-camera-scroll-post-cleanup-authentication.toml",
            limitations: &[
                "Current frozen merged-source family authenticates 467 fixture pairs for the whole camera frame, directional helpers and Spore Spawn hook.",
                "Admitted table indices, room extent and hook domains remain bounded; room and Samus dependencies remain explicit. Independent live ownership remains pending.",
            ],
            extra_evidence: &[],
        },
        Scope {
            family: "room-scroll",
            name: "Complete normal room scrolling and tilemap queue subsystem",
            owner: "/root",
            commit: "97afab2f1dc712a2433a1ef35bb9dd8901ca6011",
            expected_native: 11,
            report: "recovery-integrated-room-scroll-post-cleanup-authentication.toml",
            limitations: &[
                "Current frozen merged-source family authenticates 1,166 fixture pairs across eleven normal scrolling, background and level tilemap queue routines.",
                "The declared valid input/ROM/queue domain and dependencies bound verification. Independent live state and ordered-effect ownership remain pending.",
            ],
            extra_evidence: &[],
        },
        Scope {
            family: "observed-ridley-instructions",
            name: "Complete bounded native Ridley attack instruction frames",
            owner: "/root/ridley_instructions",
            commit: "e947e968086a15c170f58472764bf9244b403a2a",
            expected_native: 8,
            report: "recovery-integrated-observed-ridley-instructions-post-cleanup-authentication.toml",
            limitations: &[
                "Current frozen merged-source family authenticates 1,804 controlled fixture pairs plus ninety retained natural complete-frame/callback captures.",
                "ProcessEnemyInstructions coverage is limited to the admitted native Ridley attack script. Other banks/scripts, enemy AI and the broader 220-definition lifecycle remain separate; typed C services and refused scripts remain explicit.",
                "Full attack live ownership, opcode/effect sequence parity and original-ROM parity remain pending.",
            ],
            extra_evidence: &[
                ".scratch/decomp-port/recovery-integrated-observed-ridley-instructions-captured-post-cleanup-authentication.toml",
                ".scratch/decomp-port/parallel-138-scope-ridley-subsystem.md",
            ],
        },
        Scope {
            family: "observed-projectile-ownership",
            name: "Complete native projectile allocation and scheduling boundaries",
            owner: "/root/projectile_admission",
            commit: "692805ce76eb73763a36cdf24082f829b98248cd",
            expected_native: 3,
            report: "progress-integrated-observed-projectile-ownership-authentication.toml",
            limitations: &[
                "Current frozen merged-source ownership family authenticates 1,840 fixture pairs plus the separately authenticated initializer, frame, rendering and contact prerequisites.",
                "The fourth declared entry EprojInstruction is a synthetic adapter control outside the C inventory and contributes no C-definition coverage; three actual native definitions are reported.",
                "Admitted nine-ID projectile trajectory and typed callback/host closure remain bounded. Historical resolved native137 evidence is preserved; fresh full composed live evidence remains pending.",
            ],
            extra_evidence: &[".scratch/decomp-port/progress-integrated-first-six-authentication-20261013.md"],
        },
    ];
    for scope in scopes {
        // The maintained registry exposes projected declarations. This lookup
        // uses its exact current lexical symbols and never decodes fixture data.
        let manifest = format!("tools/port/{}.toml", scope.family);
        let query = format!(".routines[] | select(.declared_in | any(. == {manifest:?})) | .symbol");
        let names = Command::new("jq")
            .arg("-r")
            .arg(query)
            .arg(repo.join(".scratch/decomp-progress/registry.json"))
            .output()
            .unwrap();
        assert!(names.status.success());
        let mut routines = String::from_utf8(names.stdout).unwrap().lines().map(str::to_owned).collect::<Vec<_>>();
        routines.sort();
        assert_eq!(routines.len(), scope.expected_native, "{} native declaration count", scope.family);
        let report = repo.join(".scratch/decomp-port").join(scope.report);
        assert!(fs::read_to_string(&report).unwrap().starts_with("status = \"offline-verified\""));
        for stage in ["translated", "verified", "integrated"] {
            let id = format!("{}-6053-current-{stage}-20261013", scope.family);
            let mut limits = scope.limitations.to_vec();
            limits.push("Installed gameplay commit is integrated into main baseline c711faea56f20d42113eb63393cc4a76d4940557; shared verifier/live wiring remains uncommitted at the authenticated exact frozen source. Final wiring commit remains coordinator work.");
            limits.push("Both run cleanup passes and stale implementer worktree retirement completed before retained authentication. Stage observations remain distinct; original-ROM parity is not tested.");
            let mut update = format!(
                "id = {id:?}\nscope = {:?}\nagent = {:?}\nstage = {stage:?}\nsource_revision = {REVISION:?}\ninventory_sha256 = {INVENTORY:?}\ncommit = {:?}\nroutines = {routines:?}\nlimitations = {limits:?}\n",
                scope.name, scope.owner, scope.commit
            );
            let mut evidence = vec![
                report.clone(),
                repo.join(".scratch/decomp-port/progress-current-integrated-stage-evidence-20261013.md"),
                repo.join(".scratch/decomp-port/recovery-integrated-final-authentication-summary.toml"),
                repo.join(".scratch/decomp-port/recovery-worktree-retirement-20261013.toml"),
            ];
            evidence.extend(scope.extra_evidence.iter().map(|path| repo.join(path)));
            for path in evidence {
                assert!(path.is_file());
                update.push_str(&format!("\n[[evidence]]\npath = {:?}\n", path.to_string_lossy()));
            }
            let update_path = repo.join(".scratch/decomp-port").join(format!("progress-{id}.toml"));
            fs::write(&update_path, update).unwrap();
            let result = Command::new(repo.join("target/release/sm-pipeline"))
                .args(["progress", "update", "--root", REPO, "--update"])
                .arg(update_path)
                .output()
                .unwrap();
            fs::write(repo.join(".scratch/decomp-port").join(format!("progress-{id}.stdout")), &result.stdout).unwrap();
            fs::write(repo.join(".scratch/decomp-port").join(format!("progress-{id}.stderr")), &result.stderr).unwrap();
            assert!(result.status.success(), "{id}: {}", String::from_utf8_lossy(&result.stderr));
            println!("published {stage} for {} native routines: {}", routines.len(), scope.family);
        }
    }
}
