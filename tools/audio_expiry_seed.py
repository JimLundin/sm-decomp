"""Import a complete post-adjustment audio seed from frozen native observations.

Usage: audio_expiry_seed.py PRIVATE_NIGHT_ROOT OUTPUT_DIRECTORY
No reference or clean model is loaded or advanced. The output contains private assets
and observed state, so keep it outside the repository or in an encrypted bundle.
"""

import ast
from fractions import Fraction
import hashlib
import pathlib
import re
import runpy
import struct
import sys
import tomllib

_CODEC = runpy.run_path(str(pathlib.Path(__file__).with_name("audio_native_fixture.py")),
                       run_name="audio_observation_codec")
GROUPS, MIX_FIELDS, TRACK_FIELDS, instant, inline, voice = (
    _CODEC[name] for name in ("GROUPS", "MIX_FIELDS", "TRACK_FIELDS", "instant", "inline", "voice"))

CAPTURE = "audio-record/native-full-on-v19-music-expire"
CPU_FREQUENCY = 21477272
OSCILLATOR_FREQUENCY = 24607104
DENOMINATOR = 24 * CPU_FREQUENCY
SIZES = {"APURAM": 65536, "CPU_STATE": 152088, "SMP_STATE": 131208,
         "DSP_STATE": 17984, "PPU_STATE": 414352}


def require(condition, field):
    if not condition:
        raise ValueError(field)


def read_unsigned(data, offset, width=1):
    require(0 <= offset and offset + width <= len(data), "observation extent")
    return int.from_bytes(data[offset:offset + width], "little")


def validate_adjustment(before, after):
    for device, size in SIZES.items():
        require(len(before[device]) == len(after[device]) == size, f"{device} extent")
        changed = [i for i, (a, b) in enumerate(zip(before[device], after[device])) if a != b]
        permitted = {"APURAM": [128, 130], "SMP_STATE": [176, 178]}.get(device, [])
        require(changed == permitted, f"{device} adjustment ownership")
    require(after["SMP_STATE"][48:48 + 65536] == after["APURAM"], "SMP APURAM anchor")
    require(after["APURAM"][128] == after["APURAM"][130] == 1, "adjusted repeat observations")


def machine_phase(ppu, smp):
    require(read_unsigned(ppu, 12, 4) == CPU_FREQUENCY, "CPU frequency")
    require(read_unsigned(smp, 36, 4) == OSCILLATOR_FREQUENCY, "oscillator frequency")
    field, vertical, horizontal = ppu[29], *struct.unpack_from("<HH", ppu, 30)
    require(ppu[28] == 0 and field in (0, 1) and vertical < 262 and horizontal < 1364,
            "beam coordinates")
    return (field, 1364 * vertical + horizontal - (4 if field and vertical > 240 else 0),
            struct.unpack_from("<q", ppu, 16)[0], struct.unpack_from("<q", smp, 40)[0])


def cpu_elapsed(before, after):
    return (after[1] - before[1] + ((357368 - 4 * before[0]) if before[0] != after[0] else 0)
            - (after[2] - before[2]))


def cut_times(capture):
    base = machine_phase((capture / "loaded-PPU_STATE.bin").read_bytes(),
                         (capture / "loaded-SMP_STATE.bin").read_bytes())
    previous, cpu, times = base, 0, []
    for index in range(66):
        current = machine_phase((capture / f"cut-{index}-PPU_STATE.bin").read_bytes(),
                                (capture / f"cut-{index}-SMP_STATE.bin").read_bytes())
        cpu += cpu_elapsed(previous, current)
        times.append(Fraction(cpu * OSCILLATOR_FREQUENCY + current[3] - base[3], DENOMINATOR))
        previous = current
    return base, times


def observed_publications(capture, base, stop):
    lines = (capture / "instant-events.txt").read_text(encoding="utf-8").splitlines()
    clocks = (capture / "instant-clocks.bin").read_bytes()
    require(len(clocks) == len(lines) * 128, "instant sidecar extent")
    previous, cpu, publications, inputs = base, 0, [], []
    for index, line in enumerate(lines):
        columns = line.split()
        require(columns[0::2][:3] == ["kind", "port", "value"], "instant record fields")
        kind, group, byte = columns[1], int(columns[3]), int(columns[5])
        current = machine_phase(clocks[index * 128:index * 128 + 64],
                                clocks[index * 128 + 64:(index + 1) * 128])
        cpu += cpu_elapsed(previous, current)
        previous = current
        at = Fraction(cpu * OSCILLATOR_FREQUENCY - base[3]
                      + (current[3] if kind == "spc-write" else 0), DENOMINATOR)
        if at <= stop:
            if kind == "spc-write":
                require(at.denominator == 1 and group in range(4), "physical publication")
                publications.append((index, group, byte, at))
            elif kind == "cpu-write":
                inputs.append((group, byte, at))
    require(publications and publications[-1][1] == 0, "completed music boundary")
    require(not any(group == 0 for group, _, _ in inputs), "stable resident music request")
    return publications, inputs


def timer(smp):
    stage0, stage1, stage2 = smp[131128:131131]
    unread = read_unsigned(smp, 131132, 4)
    require(stage0 < 192 and stage0 % 3 == 0 and stage1 in (0, 1) and stage2 < 16,
            "timer prescaler")
    require(unread < 16 and smp[131137] == 1 and smp[131138] == 16, "timer enabled target")
    require(read_unsigned(smp, 131172, 4) == 3 and smp[131176:131178] == bytes(2),
            "timer speed")
    return stage0 // 3 + 64 * stage1 + 128 * stage2, unread


def validate_clock_episode(capture, base, times, next_observe, phase):
    """Corroborate the decoded episode with independent prescaler and source-store traces."""
    initial_phase, _ = timer((capture / "loaded-SMP_STATE.bin").read_bytes())
    for index in (63, 64, 65):
        observed, _ = timer((capture / f"cut-{index}-SMP_STATE.bin").read_bytes())
        require((initial_phase + times[index]) % 2048 == observed, "native prescaler recurrence")
    now = times[64]
    require((initial_phase + now) % 2048 == phase, "post-adjustment prescaler recurrence")
    lines = (capture / "g2-watch-events.local.txt").read_text(encoding="utf-8").splitlines()
    clocks = (capture / "g2-watch-clocks.bin").read_bytes()
    require(len(clocks) == len(lines) * 128, "cadence watch extent")
    previous, cpu, following = base, 0, None
    for index, line in enumerate(lines):
        saved = clocks[index * 128:(index + 1) * 128]
        current = machine_phase(saved[:64], saved[64:])
        cpu += cpu_elapsed(previous, current)
        previous = current
        at = Fraction(cpu * OSCILLATOR_FREQUENCY + current[3] - base[3], DENOMINATOR)
        if line.split()[3] == "soundEffectsClock" and at > now:
            following = at
            break
    edge = now + 2048 - phase
    opportunities = max(0, -(-(edge - next_observe) // 8))
    first_nonempty = next_observe + 8 * opportunities
    require(following == first_nonempty + Fraction(53, 2), "next episode native distribution witness")


def semantic_cursor(events, address):
    matches = [index for index, event in enumerate(events) if event[2] == address]
    require(len(matches) == 1, "unique musical cursor")
    index = matches[0]
    require(events[index][0] != "Articulation", "cursor outside bundled articulation")
    return sum(event[0] != "Articulation" for event in events[:index])


def decode_song(ram, fields, graphs, fixture):
    timbres = []
    for graph in graphs:
        for node in graph["nodes"]:
            for kind, args, _, _ in node["events"]:
                if kind == "Timbre" and args[0] not in timbres:
                    timbres.append(args[0])
    tracks = []
    for graph in graphs:
        slot = graph["slot"]
        read = lambda name, width=1: read_unsigned(ram, fields[name] + 2 * slot, width)
        address = read("trackPointers", 2)
        nodes = graph["nodes"]
        matches = [index for index, node in enumerate(nodes)
                   if any(event[2] == address for event in node["events"])]
        require(len(matches) == 1, "unique musical phrase")
        phrase = matches[0]
        cursor = semantic_cursor(nodes[phrase]["events"], address)
        repeats = read("trackRepeatedSubsectionCounters")
        if phrase == 0:
            require(repeats == 0, "root repeat counter")
            position = dict(Root=dict(cursor=cursor))
        else:
            require(repeats > 0, "repeated phrase counter")
            continuation = semantic_cursor(nodes[0]["events"], read("trackRepeatedSubsectionAddresses", 2))
            target = read("trackRepeatedSubsectionReturnAddresses", 2)
            require(target == nodes[phrase]["events"][0][2], "repeated phrase target")
            root = fixture["content"]["tracks"][len(tracks)]["phrases"][0]
            require(continuation > 0 and isinstance(root[continuation - 1], dict)
                    and root[continuation - 1].get("Repeat", {}).get("phrase") == phrase,
                    "repeated phrase continuation")
            position = dict(Repeated=dict(phrase=phrase, cursor=cursor,
                                         remaining=repeats, continuation=continuation))
        track = {semantic: read(symbol, width) for semantic, _, symbol, width, signed in TRACK_FIELDS
                 if not signed}
        require(track["note"] > 0 and track["duration"] > 0, "positive musical duration")
        require(read("trackSubtransposes") == 0, "admitted zero pitch offset")
        code = read("trackInstrumentIndices")
        require(code in timbres, "resolved musical timbre")
        ramp = None
        if track["ramp_remaining"]:
            offset = fields["trackVolumeDeltas"] + 2 * slot
            ramp = dict(remaining=track["ramp_remaining"], target=track["ramp_target"],
                        increment=int.from_bytes(ram[offset:offset + 2], "little", signed=True))
        tracks.append(dict(position=position, note=track["note"], release=track["release"],
                           duration=track["duration"], articulation=track["articulation"],
                           pitch=track["pitch"], loudness=track["loudness"], ramp=ramp,
                           timbre=timbres.index(code), offset="Zero", strength=read("trackNoteVolume")))
    nonempty = [slot for slot in range(8) if read_unsigned(ram, fields["trackPointers"] + 2 * slot + 1)]
    require(nonempty == [graph["slot"] for graph in graphs], "complete song track roster")
    mix = {semantic: read_unsigned(ram, fields[symbol], width) for semantic, symbol, width in MIX_FIELDS}
    mix.update(echo_voices=[voice(i) for i in range(8) if ram[fields["echoEnableFlags"]] & (1 << i)],
               echo_enabled=not bool(ram[fields["flg"]] & 32))
    return dict(tracks=tracks, mix=mix)


def span_boundaries(ram, pointer, expected):
    """Validate source content and map byte offsets after spans to typed next-span cursors."""
    offset, spans, boundaries = 0, [], {0: 0}
    for _ in range(1000):
        code = read_unsigned(ram, pointer + offset)
        offset += 1
        if code in (255, 251):
            require(spans == expected, "effect source content identity")
            return boundaries
        if code in (249, 245, 248):
            offset += 2
        elif code == 254:
            offset += 1
        elif code in (253, 252):
            pass
        else:
            spans.append(tuple(ram[pointer + offset - 1:pointer + offset + 4]))
            offset += 4
            boundaries[offset] = len(spans)
    raise ValueError("bounded effect content")


def decode_groups(ram, cpu, fields, fixture, imported):
    aliases = {(row["group"], row["byte"]): row["cue"] for row in fixture["aliases"]}

    def request(group, byte):
        if byte == 0:
            return "Absent"
        require((group, byte) in aliases, "admitted effect request")
        return dict(Cue=aliases[group, byte])

    groups, requests, owned = {}, {}, set()
    source_clips = {group: {} for group in GROUPS}
    for (name, group, channels), alias in zip(imported, fixture["aliases"]):
        require(alias["group"] == group, "effect source alias group")
        source_clips[group][alias["cue"]] = channels
    for group, identity in GROUPS.items():
        read = lambda name, width=1: read_unsigned(ram, fields[name], width)
        prefix = f"sound{group}"
        history = []
        for part in range((1, 1, 2)[group - 1]):
            flag = read(f"{prefix}_channel{part}_releaseFlag")
            require(flag in (0, 255), "release history flag")
            count = read(f"{prefix}_channel{part}_releaseTimer")
            require(flag == 0 or count > 0, "positive active release history")
            history.append(dict(Active=count) if flag else "Inactive")
        cue_code, enabled = read(prefix), read(f"{prefix}_enabledVoices")
        playback = "Empty"
        if cue_code:
            cue = request(group, cue_code)["Cue"]
            initialization = read(f"{prefix}_initialisationFlag")
            require(initialization in (0, 255), "admitted initialization flag")
            if initialization == 0:
                require(enabled == 0, "prepared cue has no voice")
                playback = dict(Prepared=dict(cue=cue))
            else:
                parts, group_owned = [], set()
                source = source_clips[group][cue]
                for part, (spans, _) in enumerate(source):
                    part_read = lambda suffix, width=1: read(f"{prefix}_channel{part}_{suffix}", width)
                    disabled = part_read("disableByte")
                    require(disabled in (0, 255), "part enabled flag")
                    if disabled:
                        parts.append("Finished")
                        continue
                    index = part_read("voiceIndex")
                    require(index % 2 == 0 and 4 <= index <= 14, "effect voice identity")
                    physical = index // 2
                    require(physical not in owned and physical not in group_owned, "exclusive voice ownership")
                    group_owned.add(physical)
                    boundaries = span_boundaries(ram, part_read("p_instructionList", 2), spans)
                    byte_cursor = part_read("i_instructionList")
                    require(byte_cursor in boundaries, "effect semantic cursor")
                    pitch = part_read("note") & 127
                    require(pitch <= 71, "admitted effect pitch")
                    motion = None
                    flag = part_read("pitchSlideFlag")
                    require(flag in (0, 255), "motion flag")
                    if flag:
                        target, speed = part_read("targetNote") & 127, part_read("subnoteDelta")
                        require(0 < speed < 256 and pitch < target <= 71, "admitted active motion")
                        motion = dict(speed=speed, target=target)
                    parts.append(dict(Live=dict(voice=f"V{physical}", cursor=boundaries[byte_cursor],
                        instruction=part_read("instructionTimer"), pitch=pitch,
                        fraction=part_read("subnote"), motion=motion)))
                require(sum(1 << v for v in group_owned) == enabled, "complete effect ownership")
                owned.update(group_owned)
                playback = dict(Active=dict(cue=cue, parts=parts))
        else:
            require(enabled == 0, "empty effect ownership")
        reply = read(f"cpuIo{group}_write")
        require(ram[244 + group] == reply, "physical effect reply latch")
        groups[identity] = dict(history=history, handled=request(group, read(f"cpuIo{group}_read_prev")),
                               staged=request(group, read(f"cpuIo{group}_read")),
                               reply=request(group, reply), playback=playback)
        requests[identity] = request(group, cpu[152000 + group])
    require(sum(1 << v for v in owned) == ram[fields["enableSoundEffectVoices"]],
            "all reservations owned by decoded effects")
    return groups, requests


def verify_pins(root):
    manifest = tomllib.loads(pathlib.Path(__file__).with_name("audio_expiry_seed_pins.toml")
                             .read_text(encoding="utf-8"))
    for pin in manifest["file"]:
        path = root / pin["name"]
        require(path.stat().st_size == pin["size"], f"input extent {pin['name']}")
        with path.open("rb") as handle:
            require(hashlib.file_digest(handle, "sha256").hexdigest() == pin["sha256"],
                    f"input pin {pin['name']}")
    return manifest["file"]


def main():
    root, output = map(pathlib.Path, sys.argv[1:])
    require(not output.exists(), "unique private output")
    require(not output.resolve().is_relative_to(pathlib.Path(__file__).resolve().parent.parent),
            "private output outside repository")
    pins = verify_pins(root)
    capture, spec = root / CAPTURE, root / "audio-spec"
    before = {device: (capture / f"music-before-{device}.bin").read_bytes() for device in SIZES}
    after = {device: (capture / f"music-after-{device}.bin").read_bytes() for device in SIZES}
    require(all(before[device] == (capture / f"cut-64-{device}.bin").read_bytes() for device in SIZES),
            "cut64 precedes paused adjustment")
    validate_adjustment(before, after)
    require((capture / "music-memory-before-SMP_STATE.bin").read_bytes() == before["SMP_STATE"]
            and (capture / "music-memory-after-SMP_STATE.bin").read_bytes() == after["SMP_STATE"],
            "paused memory transaction snapshots")
    fixture_path = root / "audio-round/native-fixture-r5/expire.local.toml"
    fixture = tomllib.loads(fixture_path.read_text(encoding="utf-8"))
    fields = ast.literal_eval((spec / "ram-values.local.txt").read_text(encoding="utf-8"))
    # Source layout declarations only; no original instruction interpretation or execution.
    layout = (spec / "source/vanilla/ram.asm").read_text(encoding="utf-8")
    for match in re.finditer(r"%generate\(sound([123])_channel,\s*(\w+),\s*\$([0-9A-Fa-f]+),[^,]+,\s*(\d+)\)", layout):
        for part in range({1: 4, 2: 2, 3: 2}[int(match[1])]):
            fields.setdefault(f"sound{match[1]}_channel{part}{match[2]}",
                              int(match[3], 16) + part * int(match[4]))
    graphs = ast.literal_eval((spec / "imported-song-structure.local.txt").read_text(encoding="utf-8"))
    imported = ast.literal_eval((spec / "imported-effect-spans.local.txt").read_text(encoding="utf-8"))
    ram, cpu, smp = after["APURAM"], after["CPU_STATE"], after["SMP_STATE"]
    base, times = cut_times(capture)
    now = times[64]
    require(instant(now) == fixture["cuts"][64]["at"], "global cut64 origin")
    publications, inputs = observed_publications(capture, base, now)
    last_music = publications[-1][3]
    symbols = {match[2]: int(match[1], 16) for match in re.finditer(
        r"^[0-9a-fA-F]{2}:([0-9a-fA-F]{4})\s+(\S+)",
        (spec / "source-symbols.local.sym").read_text(encoding="utf-8"), re.M)}
    require(read_unsigned(smp, 4, 2) == symbols["main_branch_doUpdateDsp"], "completed return DSP boundary")
    require(last_music + Fraction(19, 2) < now < last_music + Fraction(1001, 2),
            "after stable resident pair before resumed timer observation")
    # Source engine170 publication, stable resident pair, loop DSP updates establish
    # the next episode's first timer read at publication+500.5. No CPU0 change exists.
    next_observe = last_music + Fraction(1001, 2)
    phase, unread = timer(smp)
    validate_clock_episode(capture, base, times, next_observe, phase)
    groups, requests = decode_groups(ram, cpu, fields, fixture, imported)
    effect = ram[fields["soundEffectsClock"]]
    require(effect % 32 == 0, "effect cadence remainder")
    resident = ram[fields["cpuIo0_write"]]
    require(resident == ram[244] == cpu[152000]
            and resident == ram[fields["cpuIo0_read"]] == ram[fields["cpuIo0_read_prev"]],
            "resident selection retained")
    seed = dict(now=instant(now), clock=dict(phase=instant(phase), unread=unread,
                lattice=instant(next_observe)), cadence=dict(effect=effect // 32,
                music=ram[fields["musicTrackClock"]]), music_due=False, groups=groups,
                song=decode_song(ram, fields, graphs, fixture), resident_music_reply=dict(
                    identity="resident-opening-forever", original_byte=resident), reservations=[],
                requests=requests, phase=dict(ObserveClock=dict(at=instant(next_observe))))
    physical_replies = [ram[244 + group] for group in range(4)]
    require(physical_replies == fixture["cuts"][64]["replies"], "seed physical replies")
    require(len(publications) == fixture["cuts"][64]["publications"], "seed publication offset")
    result = dict(schema="audio-expiry-post64-seed-v1", start_cut=64, next_cut=65, last_cut=183,
                  source_boundary="music-after", coordinate="AudioCycles since native loaded snapshot",
                  content_sha256=hashlib.sha256(inline(fixture["content"]).encode("utf-8")).hexdigest(),
                  fixture_sha256=hashlib.sha256(fixture_path.read_bytes()).hexdigest(),
                  publication_offset=len(publications), initial_replies=physical_replies,
                  seed=seed, pins=pins)
    output.mkdir(parents=True, exist_ok=False)
    path = output / "expire-post64-seed.local.toml"
    path.write_text("\n".join(f"{key} = {inline(value)}" for key, value in result.items()) + "\n",
                    encoding="utf-8", newline="\n")
    print("private_audio_expiry_seed schema audio-expiry-post64-seed-v1 start_cut64 next_cut65 last_cut183",
          "complete_state true pending_work false pins", len(pins),
          "sha256", hashlib.sha256(path.read_bytes()).hexdigest())


if __name__ == "__main__":
    main()
