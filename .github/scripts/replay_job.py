#!/usr/bin/env python3
"""Restore and run one verify.yml matrix job (JOB, as JSON).

Rows run in up to WORKERS verifier processes at once: one engine replay uses one
core and each row starts from its own seed. Rows with a checkpoint prefix stay
together in one process, because they can reuse each other's checked prefixes.
Raw verifier output stays private on the runner in the modes that filter it.
"""
import concurrent.futures, json, os, pathlib, re, subprocess, sys, threading, urllib.request

ROOT = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "tools"))
import scenario  # noqa: E402

JOB = json.loads(os.environ["JOB"])
TEMP = pathlib.Path(os.environ["RUNNER_TEMP"])
SUMMARY = os.environ["GITHUB_STEP_SUMMARY"]
WORKERS = int(os.environ.get("WORKERS") or os.cpu_count() or 1)
LOCK_NAMES = re.compile(r"^(SEGMENT|PASS|FAIL|UNLOCKED|KNOWN|REGRESSION|Corpus|job |Verifier total|Lock|"
                        r"[A-Za-z0-9-]+: (lock |lockstep |free run |Free run|Free-run|@|no current|localization))")
SUMMARY_NAMES = re.compile(r"^(PASS|FAIL|Corpus|RESULT|DOMAIN|DIAGNOSTIC|EXIT|"
                           r"[A-Za-z0-9-]+: (lock |lockstep |free run |Free run|@))")
# Only fixed diagnostic categories survive; raw exception text stays private.
AUDIO_DIAGNOSTIC = re.compile(r'^DIAGNOSTIC stage=(post-replay|audio-trace-validation|post-replay-provenance|corpus-post-replay) category=(required-probes|required-facts|probe-follow-up|fact-follow-up|credits-fact|audio-output-completion|audio-footer-write|audio-footer-flush|checkpoint-origin-read|checkpoint-origin-changed|checkpoint-save|trace-read|trace-format|header-missing|header-provenance|footer-duplicate|output-stream|output-kind|footer-incomplete|output-counts|source-read|source-changed|rom-input-read|rom-input-changed|dll-read|dll-changed|inputs-changed|checkpoint-read|checkpoint-frame) tick=(unknown|[0-9]+)$')
ROOM_LOGS = {
    "SM_CROSS_RENDER_LOG": ("Cross-renderer", "lines differ", 5),
    "SM_NATIVE_LINES_LOG": ("Native PPU lines", "differing lines or resyncs", None),
    "SM_HDMA_TABLE_LOG": ("HDMA tables", "frames differ from their frame-start table", None),
    "SM_IRQ_TABLE_LOG": ("IRQ tables", "frames differ from their planned interrupts", None),
}
print_lock = threading.Lock()


def script(name, *arguments, **environment):
    subprocess.run([sys.executable, "-B", str(ROOT / ".github/scripts" / name), *arguments],
                   cwd=ROOT, check=True, env=dict(os.environ, **environment))


def restore():
    if JOB["kinds"] == "room":
        bundle = TEMP / "chain.tar.gz.enc"
        urllib.request.urlretrieve(
            f"https://github.com/JimLundin/sm-decomp/releases/download/ci-data/{JOB['asset']}", bundle)
        script("full_route.py", "restore", str(bundle), FULL_ROUTE=JOB["route"])
    elif "shards" in JOB:
        # One seed bundle or movie route per shard, restored in turn.
        for shard in json.loads(JOB["shards"]):
            script("prepare_corpus_inputs.py", KIND=shard["kind"], SEED_ASSET=shard["seed_asset"],
                   SEED_SHA256=shard["seed_sha256"], SEGMENTS=shard["segments"],
                   ROUTE=shard["route"], INPUTS=shard["inputs"])


def rom_import():
    """A record that rejects the cartridge would leave the game on its old ROM reads.

    The log can hold ROM-derived values, so only pass/fail, the rejected record
    names and the failing assertion's source location are printed."""
    result = subprocess.run(["cargo", "test", "--locked", "-p", "sm_data", "--test", "rom_import"], cwd=ROOT,
                            env=dict(os.environ, SM_ROM=str(ROOT / "rom/sm.smc")),
                            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, errors="replace")
    if not result.returncode:
        print("sm_data ROM import check: PASS")
        return 0
    print("sm_data ROM import check: FAIL")
    for pattern in (r"rejected the original ROM: \[[^]]*\]", r"panicked at [^ ]+:[0-9]+:[0-9]+"):
        for match in re.findall(pattern, result.stdout)[:5]:
            print(match)
    return 1


def partition(ids):
    """Split rows into balanced worker groups, keeping prefix rows together."""
    frames = {row["id"]: row for row in scenario.rows(ROOT)}
    cost = lambda name: frames[name].get("frames", 1)
    prefixed = [name for name in ids if frames[name].get("start_frame", 0)]
    groups = [[] for _ in range(max(1, min(WORKERS, len(ids))))]
    if prefixed:
        groups[0].extend(prefixed)
    for name in sorted(set(ids) - set(prefixed), key=lambda name: (-cost(name), name)):
        min(groups, key=lambda group: sum(cost(item) for item in group)).append(name)
    return [group for group in groups if group]


def console_line(line, state, mode, audio):
    """What this mode may print, or None. Audio traces and locks keep raw text private."""
    if mode == "lock":
        return line if LOCK_NAMES.match(line) else None
    if not audio:
        return line
    match = re.match(r"^SEGMENT ([A-Za-z0-9-]+)", line)
    if match:
        state["row"] = match[1]
        return None
    match = AUDIO_DIAGNOSTIC.match(line)
    if match:
        return f"DIAGNOSTIC row={state.get('row', '')} stage={match[1]} category={match[2]} tick={match[3]}"
    match = re.match(r"^FAIL ([A-Za-z0-9-]+): verifier exit ([0-9]+)$", line)
    if match:
        return f"FAIL {match[1]}\nEXIT row={match[1]} code={match[2]}"
    match = re.match(r"^(PASS|FAIL) ([A-Za-z0-9-]+)", line)
    if match:
        return f"{match[1]} {match[2]}"
    match = re.match(r"^Verified ([0-9]+) frames: ([0-9]+) failing frames", line)
    if match:
        return f"RESULT {state.get('row', '')}: {match[1]} frames, {match[2]} failing frames"
    match = re.match(r"^@([0-9]+): (?:Observation|Free-run) divergence in ([A-Za-z0-9_.\[\]-]+)", line)
    if match:
        return f"DOMAIN {state.get('row', '')}: tick {match[1]}, field {match[2]}"
    return line if line.startswith("Corpus") else None


def worker(index, group, mode, audio, summary):
    arguments = [sys.executable, "tools/verify_dll.py", "rom/sm.smc", "--corpus", "scenarios", "--no-build", "--direct"]
    for name in group:
        arguments += ["--segment", name]
    environment = dict(os.environ)
    logs = {}
    if mode == "lock":
        arguments += ["--mode", "check-lock", "--lock", "scenarios/lock.toml"]
    else:
        environment["SM_ORACLE"] = os.environ.get("ORACLE") or "patched"
        if audio:
            arguments += ["--audio-trace-directory", f"audio-traces-{index}"]
        if JOB["kinds"] == "room" and not os.environ.get("LLVM_PROFILE_FILE"):
            # Redraw and model checks: every 30th frame on pull requests, every frame nightly.
            nightly = os.environ.get("GITHUB_EVENT_NAME") == "schedule"
            environment.update(SM_CROSS_RENDER="30", SM_PPU_MODEL_CHECK="1" if nightly else "30",
                               SM_HDMA_TABLE="1", SM_IRQ_TABLE="1")
            if nightly:
                environment["SM_NATIVE_LINES"] = "1"
            for name in ROOM_LOGS:
                logs[name] = TEMP / f"{name.lower()}-{index}.csv"
                environment[name] = str(logs[name])
    if JOB["kinds"] == "room":
        environment["FULL_ROUTE"] = JOB["route"]
    state = {}
    with (TEMP / f"replay-{index}.log").open("w", encoding="utf-8", errors="replace") as raw:
        process = subprocess.Popen(arguments, cwd=ROOT, env=environment, stdout=subprocess.PIPE,
                                   stderr=subprocess.STDOUT, text=True, encoding="utf-8", errors="replace")
        for line in process.stdout:
            raw.write(line)
            shown = console_line(line.rstrip("\n"), state, mode, audio)
            if shown is None:
                continue
            with print_lock:
                print(shown, flush=True)
                if SUMMARY_NAMES.match(shown):
                    summary.append(shown)
    code = process.wait()
    for name, path in logs.items():
        if path.exists():
            # Only frames, lines, modes, counts and positions: pixel values are ROM-derived.
            title, what, columns = ROOM_LOGS[name]
            lines = path.read_text().splitlines()
            with print_lock:
                print(f"{title}: {len(lines)} {what}:")
                for entry in lines[:20]:
                    print(",".join(entry.split(",")[:columns]) if columns else entry)
            code = code or 1
    return code


def replay():
    mode, audio = JOB["mode"], bool(os.environ.get("AUDIO_TRACE"))
    groups = partition(JOB["segments"].split())
    print(f"job {JOB['name']}: {len(JOB['segments'].split())} rows in {len(groups)} processes", flush=True)
    summary = []
    with concurrent.futures.ThreadPoolExecutor(len(groups)) as pool:
        codes = list(pool.map(lambda item: worker(item[0], item[1], mode, audio, summary), enumerate(groups)))
    with open(SUMMARY, "a") as out:
        out.write(f"### {JOB['name']}\n```\n" + "".join(line + "\n" for line in summary) + "```\n")
    return 1 if any(codes) else 0


def main():
    restore()
    if JOB["mode"] == "rom-import":
        return rom_import()
    if JOB["mode"] == "record":
        # Only ids, ticks, domains and hashes reach the console or summary.
        result = subprocess.run([sys.executable, "-B", ".github/scripts/lock_gate.py", "record-oracle"], cwd=ROOT,
                                env=dict(os.environ, SEGMENTS=JOB["segments"]), stdout=subprocess.PIPE,
                                stderr=subprocess.STDOUT, text=True, errors="replace")
        print(result.stdout, end="")
        with open(SUMMARY, "a") as out:
            out.write(result.stdout)
        return result.returncode
    return replay()


if __name__ == "__main__":
    sys.exit(main())
