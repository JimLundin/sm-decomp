// One-off Rust capture of inspected real owners. Does not mutate authority or queues.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};
const ROOT: &str = "/home/jim/sm-decomp";
const SELF: &str = "01a136a0-49bb-7690-a360-48def3c2c09b";
const OLD: &str = "01a13592-d02b-7143-8489-a15b03f19da2";
fn output(program: &str, args: &[&str]) -> String {
    let out = Command::new(program).args(args).output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap().trim().into()
}
fn hash(p: &Path) -> String {
    output("sha256sum", &[p.to_str().unwrap()]).split_whitespace().next().unwrap().into()
}
fn write(p: &Path, text: &str) {
    if p.exists() {
        assert_eq!(fs::read_to_string(p).unwrap(), text);
    } else {
        fs::write(p, text).unwrap();
    }
}
fn art(p: &Path) -> String {
    format!("path = {:?}\nsha256 = {:?}\n", p, hash(p))
}
fn reference(p: &Path) -> String {
    format!("{{\"path\":{:?},\"sha256\":{:?},\"bytes\":{}}}", p, hash(p), fs::metadata(p).unwrap().len())
}
fn main() {
    assert_eq!(std::env::var("CODEX_THREAD_ID").unwrap(), SELF);
    let root = PathBuf::from(std::env::args().nth(1).unwrap());
    let inspections = root.join("inspections");
    let stamp = inspections.join("observed-at.txt");
    if !stamp.exists() {
        write(&stamp, &output("date", &["-u", "+%Y-%m-%dT%H:%M:%SZ"]));
    }
    let now = fs::read_to_string(&stamp).unwrap();
    for (source, name) in [
        (".scratch/decomp-parallel/authority.toml", "authority-before.toml"),
        (".scratch/ceres-continuation/ledger.toml", "ceres-ledger-before.toml"),
        (".scratch/decomp-parallel/ledger.toml", "canonical-ledger-before.toml"),
        (".scratch/decomp-parallel/reconciliation/legacy-owner-bridge-v2.toml", "owner-bridge.toml"),
    ] {
        write(&inspections.join(name), &fs::read_to_string(Path::new(ROOT).join(source)).unwrap());
    }
    let replacement = inspections.join("replacement.toml");
    write(
        &replacement,
        &format!(
            "schema = \"decomp-session-observation-v1\"\nsession_id = {SELF:?}\nstatus = \"active\"\nobserved_at = {now:?}\n\n[[evidence]]\n{}",
            art(&inspections.join(format!("{SELF}.json")))
        ),
    );
    let entries = [
        (
            "ceres-continuation",
            "compact-controller",
            "48656a2-compact-v1",
            "ceres-20261013",
            "/root/ceres_scope",
            "01a133ec-4ef9-78b2-a8d7-248726023ce5",
            "completed",
        ),
        (
            "ceres-continuation",
            "shared-game-host",
            "48656a2-host-v1",
            "ceres-20261013",
            "/root/host_scope",
            "01a133ec-68e2-7033-9ee7-dba26d4db746",
            "completed",
        ),
        (
            "ceres-continuation",
            "integrate-shared-game-host",
            "48656a2-host-v1-1",
            "ceres-20261013",
            "/root",
            "01a133eb-76e2-7073-9aef-d97c03a812e4",
            "failed",
        ),
        (
            "decomp-parallel",
            "pause-equipment-render-offline",
            "53e916e7fc7edf90ffde5e953a18bcc43fb0ee931256ebd01e446757818f27f1",
            "decomp-run-20261013",
            "01a13623-3b41-7822-b0b3-a99f73df84ac",
            "01a13623-3b41-7822-b0b3-a99f73df84ac",
            "completed",
        ),
    ];
    let mut children = std::collections::BTreeMap::<String, String>::new();
    for (effort, id, revision, token, agent, session, status) in entries {
        let evidence = inspections.join(format!("{session}.json"));
        let process = inspections.join("processes.txt");
        let observed = inspections.join(format!("{id}-session.toml"));
        write(
            &observed,
            &format!(
                "schema = \"decomp-session-observation-v1\"\nsession_id = {session:?}\nstatus = {status:?}\nobserved_at = {now:?}\n\n[[evidence]]\n{}\n[[evidence]]\n{}",
                art(&evidence),
                art(&process)
            ),
        );
        let claim = format!("id = {id:?}\nrevision = {revision:?}\nattempt = 1\ncoordinator = {token:?}\n");
        let child = inspections.join(format!("{id}-child.toml"));
        write(
            &child,
            &format!(
                "schema = \"decomp-child-observation-v1\"\neffort = {effort:?}\nsession_id = {session:?}\nagent = {agent:?}\nstatus = {status:?}\nobserved_at = {now:?}\n\n[claim]\n{claim}\n[[evidence]]\n{}\n[[evidence]]\n{}",
                art(&evidence),
                art(&process)
            ),
        );
        children.entry(token.into()).or_default().push_str(&format!("\n[[children]]\neffort = {effort:?}\nagent = {agent:?}\nstatus = {status:?}\n[children.claim]\n{claim}\n[children.observation]\n{}",art(&child)));
        let json_claim = format!("{{\"id\":{id:?},\"revision\":{revision:?},\"attempt\":1,\"coordinator\":{token:?}}}");
        write(
            &inspections.join(format!("{id}-stop.json")),
            &format!(
                "{{\"schema\":\"translation-original-owner-stop-v1\",\"checkout\":{ROOT:?},\"effort\":{effort:?},\"claim\":{json_claim},\"agent\":{agent:?},\"observed_at\":{now:?},\"status\":{status:?},\"session\":{},\"evidence\":[{},{}],\"processes\":[]}}",
                reference(&observed),
                reference(&evidence),
                reference(&process)
            ),
        );
    }
    for (token, generation) in [("ceres-20261013", 2), ("decomp-run-20261013", 1)] {
        let transfer = inspections.join(format!("{token}-transfer.toml"));
        write(
            &transfer,
            &format!(
                "schema = \"decomp-session-transfer-v1\"\ntoken = {token:?}\nsession_id = {OLD:?}\ngeneration = {generation}\nstatus = \"completed\"\nobserved_at = {now:?}\nprocesses = []\n\n[[evidence]]\n{}\n[[evidence]]\n{}\n[[evidence]]\n{}\n[replacement]\n{}{}",
                art(&inspections.join(format!("{OLD}.json"))),
                art(&inspections.join("processes.txt")),
                art(&inspections.join("authority-before.toml")),
                art(&replacement),
                children[token]
            ),
        );
        write(
            &root.join(format!("{token}-transfer.request.toml")),
            &format!(
                "operation = \"transfer\"\ncheckout = {ROOT:?}\ncoordinator = {token:?}\n\n[authority]\nsession_id = {SELF:?}\ngeneration = {}\n\n[transfer]\ntoken = {token:?}\nsession_id = {SELF:?}\n\n[transfer.observation]\n{}",
                generation + 1,
                art(&transfer)
            ),
        );
    }
    println!("Captured 4 original owners and prepared 2 checked transfer requests; no authority changed.");
}
