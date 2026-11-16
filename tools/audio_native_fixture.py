"""Decode the frozen paired audio evidence into private typed harness fixtures.

Usage: python tools/audio_native_fixture.py PRIVATE_AUDIO_SPEC OUTPUT_DIRECTORY
Only decoded assets/seeds and captured observations supply the fixture. The independent
observer definitions are loaded for their seed/content importer; no advance method is called.
The output is ROM-derived and must remain private or in an encrypted evidence bundle.
"""
import ast
import dataclasses
from fractions import Fraction
import hashlib
import pathlib
import runpy
import sys
import tomllib


NAMES = dict(zip(
    ("Footstep", "PlainSpin", "SpinEnded", "SamusLandedOrWallJumped", "PowerBeamFired",
     "SamusSilence", "BackgroundSilence", "CeresDoorOpening"),
    ("Footstep", "Spin", "EndSpin", "Land", "Beam", "SamusSilence", "BackgroundSilence", "Door")))
GROUPS = {1: "Action", 2: "Ambient", 3: "Contact"}
TRACK_FIELDS = (
    ("note", "note_remaining", "trackNoteTimers", 1, False),
    ("release", "gate_remaining", "trackNoteRingTimers", 1, False),
    ("duration", "duration", "trackNoteLengths", 1, False),
    ("articulation", "ratio", "trackNoteRingLengths", 1, False),
    ("repeats", "phrase_repeats", "trackRepeatedSubsectionCounters", 1, False),
    ("pitch", "pitch", "trackNotes", 1, False),
    ("loudness", "loudness", "trackVolumes", 2, False),
    ("ramp_remaining", "ramp_remaining", "trackDynamicVolumeTimers", 1, False),
    ("ramp_target", "ramp_target", "trackTargetVolumes", 1, False),
    ("ramp_increment", "ramp_delta", "trackVolumeDeltas", 2, True))
MIX_FIELDS = (
    ("percussion", "percussionInstrumentsBaseIndex", 1),
    ("master", "musicVolume", 2), ("echo_left", "echoVolumeLeft", 2),
    ("echo_right", "echoVolumeRight", 2), ("echo_delay", "echoDelay", 1),
    ("feedback", "echoFeedbackVolume", 1), ("filter", "i_echoFirFilterSet", 1))


def pending_writers(index, table, track, tables, publications, capture, fields, graphs):
    """Frozen attribution, admitted from preceding committed content, before comparisons."""
    if not table["pending_music_plan"]:
        return set()
    writers = {"note_remaining"}
    if track["note_remaining"] == 1 or track["gate_remaining"] > 0:
        writers.add("gate_remaining")
    if track["ramp_remaining"] > 0:
        writers.update(("ramp_remaining", "loudness"))
    if track["note_remaining"] == 1:
        assert index > 0 and not tables[index - 1]["pending_music_plan"]
        before = next(t for t in tables[index - 1]["track"] if t["slot"] == track["slot"])
        turns = sum(group == 0 for group, _, _ in publications[
                    tables[index - 1]["publication_count"]:table["publication_count"]])
        assert before["note_remaining"] > turns and before["note_remaining"] - turns == 1
        ram = (capture / f"cut-{index - 1}-APURAM.bin").read_bytes()
        offset = fields["trackPointers"] + 2 * track["slot"]
        pointer = int.from_bytes(ram[offset:offset + 2], "little")
        graph = next(g for g in graphs if g["slot"] == track["slot"])
        routes = [node["events"][i:] for node in graph["nodes"]
                  for i, event in enumerate(node["events"]) if event[2] == pointer]
        assert len(routes) == 1
        route = []
        for event in routes[0]:
            route.append(event[0])
            if event[0] in ("Tone", "Sustain", "Rest", "Percussion"):
                break
        assert route == ["Loudness", "LoudnessRamp", "Duration", "Sustain"]
        writers.update(("loudness", "ramp_remaining", "ramp_target", "ramp_delta", "duration"))
    return writers


def words(value):
    result = []
    while value:
        result.append(value % (1 << 32))
        value //= 1 << 32
    return result


def instant(value):
    value = Fraction(value)
    assert value >= 0
    return dict(numerator=[int(value > 0), words(value.numerator)],
                denominator=[1, words(value.denominator)])


def inline(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, str):
        assert '"' not in value and "\\" not in value and "\n" not in value
        return '"' + value + '"'
    if isinstance(value, (list, tuple)):
        return "[" + ", ".join(map(inline, value)) + "]"
    if isinstance(value, dict):
        return "{ " + ", ".join(inline(k) + " = " + inline(v)
                                  for k, v in value.items() if v is not None) + " }"
    raise TypeError(type(value).__name__)


def line(group):
    return "Music" if group == 0 else dict(Group=GROUPS[group])


def voice(index):
    return ("FirstMusic" if index == 0 else "SecondMusic" if index == 1
            else dict(Effect=f"V{index}"))


def effect_pitch(original):
    """engine.asm289 discards the ordinary note marker before octave/key decoding."""
    assert 0 <= original <= 255 and original != 0xF6
    semitone = original & 0x7F
    assert semitone <= 71
    return semitone


def music_event(event):
    kind = type(event).__name__
    data = dataclasses.asdict(event)
    if kind == "Timing":
        return dict(Timing=dict(duration=data["duration"], articulation=data["articulation"],
                                extra=words(data["extra_handling_cycles"])))
    if kind in ("Tone", "Loudness", "Timbre", "MasterLoudness", "Percussion"):
        return {kind: next(iter(data.values()))}
    if kind == "Ramp":
        return dict(Ramp=data)
    if kind == "EchoMix":
        return dict(EchoMix=dict(voices=[voice(v) for v in sorted(event.voices)],
                                left=event.left, right=event.right))
    if kind == "EchoSpace":
        return dict(EchoSpace=dict(delay=event.delay, feedback=event.feedback, filter=event.filter))
    if kind == "RepeatPhrase":
        return dict(Repeat=dict(phrase=event.phrase, count=event.count))
    if kind == "CompletePhrase":
        return "Complete"
    if kind == "Sustain":
        return "Sustain"
    if kind == "PitchOffset":
        assert event.fraction == 0
        return dict(Offset="Zero")
    raise ValueError("unsupported music event")


def seed_song(song):
    tracks = []
    for index, saved in enumerate(song.tracks):
        track = saved.track
        assert track.note > 0 and track.duration > 0 and song.mix.tone_offsets[index] == 0
        position = (dict(Root=dict(cursor=saved.cursor)) if saved.phrase == 0 else
                    dict(Repeated=dict(phrase=saved.phrase, cursor=saved.cursor,
                                       remaining=track.repeats, continuation=saved.return_cursor)))
        tracks.append(dict(position=position, note=track.note, release=track.gate,
                           duration=track.duration, articulation=track.ratio, pitch=track.pitch,
                           loudness=track.volume,
                           ramp=None if track.ramp_remaining == 0 else dict(
                               remaining=track.ramp_remaining, target=track.ramp_target,
                               increment=track.ramp_delta),
                           timbre=song.mix.timbres[index], offset="Zero",
                           strength=song.mix.note_strengths[index]))
    mix = song.mix
    return dict(tracks=tracks, mix=dict(master=mix.master_loudness, percussion=mix.percussion_set,
                echo_voices=[voice(v) for v in sorted(mix.echo_voices)], echo_left=mix.echo_left,
                echo_right=mix.echo_right, echo_delay=mix.echo_delay, feedback=mix.feedback,
                filter=mix.filter_set, echo_enabled=mix.echo_writes_enabled))


def release_history(ram, release_decode):
    """All ordered admitted histories, including dormant parts; never indexed by voice."""
    history = {group: [] for group in GROUPS}
    for part in release_decode["part"]:
        active = ram[part["flag_address"]] == release_decode["active_flag"]
        count = ram[part["timer_address"]] if active else 0
        if active and count == 0:
            raise ValueError(f"active zero release history group{part['group']} part{part['index']}")
        entries = history[part["group"]]
        assert part["index"] == len(entries)
        entries.append(dict(Active=count) if active else "Inactive")
    assert [len(history[group]) for group in GROUPS] == [1, 1, 2]
    return history


def main():
    root, output = map(pathlib.Path, sys.argv[1:])
    # The definitions resolve their frozen source inputs through this exact private root.
    import os
    assert root.resolve() == (pathlib.Path(os.environ["TEMP"]) /
                              "collab/night-20261115/audio-spec").resolve()
    roster = root.parent / "audio-record/roster-request-review-pins.txt"
    roster_hash = "7a9305e8f4a582254da3c91304e535523739617376cf98bb9874d1b04e1d6a60"
    assert hashlib.sha256(roster.read_bytes()).hexdigest() == roster_hash
    roster_lines = roster.read_text(encoding="utf-8").splitlines()
    assert len(roster_lines) == 7206
    for entry in roster_lines:
        expected, path = entry.split(" ", 1)
        with pathlib.Path(path).open("rb") as handle:
            assert hashlib.file_digest(handle, "sha256").hexdigest() == expected
    manifest = root / "music-pair-frozen-pins.local.toml"
    assert hashlib.sha256(manifest.read_bytes()).hexdigest() == \
        "4978d98baa1b5c73d16add026e0d5f23cf1f5a886d184f13ed8f1d9c3b42667f"
    pins = tomllib.loads(manifest.read_text(encoding="utf-8"))["file"]
    assert len(pins) == 27
    for pin in pins:
        data = (root / pin["name"]).read_bytes()
        assert len(data) == pin["size"] and hashlib.sha256(data).hexdigest() == pin["sha256"]
    definitions = runpy.run_path(str(root / "successor/transport_reference.py"), run_name="fixture_import")
    audio = definitions["AdmittedAudio"]()
    typed = definitions["TYPED"]
    aliases = tomllib.loads((root / "effect-aliases.local.toml").read_text(encoding="utf-8"))["cue"]
    cue_codes = {(row["group"], row["original_request"]): NAMES[row["name"]] for row in aliases}

    def request(group, value):
        return "Absent" if value == 0 else dict(Cue=cue_codes[group, value])

    fields = ast.literal_eval((root / "ram-values.local.txt").read_text(encoding="utf-8"))
    release_decode = tomllib.loads(pathlib.Path(__file__).with_name(
        "audio_native_decode.toml").read_text(encoding="utf-8"))["release"]
    effects = {}
    timbre_codes = list(typed["TIMBRE_CODES"])
    for (group, original), (name, channels) in definitions["BASE"]["ns"]["ens"]["content"].items():
        parts = []
        for spans, controls in channels:
            assert all(kind in ("ConnectedPitchSlide", "RepeatStart", "RepeatForever")
                       for kind, _, _ in controls)
            for span in spans:
                if span[0] not in timbre_codes:
                    timbre_codes.append(span[0])
            motion = next((dict(speed=args[0], target=effect_pitch(args[1])) for kind, args, _ in controls
                           if kind == "ConnectedPitchSlide"), None)
            if motion is not None:
                original_target = next(args[1] for kind, args, _ in controls if kind == "ConnectedPitchSlide")
                assert all((span[3] & 0x80) == (original_target & 0x80) for span in spans)
            loop_start = next((index for kind, _, index in controls if kind == "RepeatStart"), 0)
            looping = any(kind == "RepeatForever" for kind, _, _ in controls)
            parts.append(dict(spans=[dict(duration=span[-1], pitch=effect_pitch(span[3]),
                                         timbre=timbre_codes.index(span[0]), motion=motion)
                                    for span in spans], loop_start=loop_start if looping else None))
        effects[NAMES[name]] = dict(parts=parts)
    content = dict(effects=effects, tracks=[dict(phrases=[[music_event(e) for e in phrase]
                     for phrase in track.phrases]) for track in typed["TRACKS"]])
    assert all(not audio.groups[g] and audio.pending[g] is None for g in GROUPS)
    assert audio.reserved == 0
    seed = dict(now=instant(0), clock=dict(phase=instant(audio.phi), unread=0,
                lattice=instant(Fraction(811, 2))), cadence=dict(effect=audio.effect_fraction,
                music=audio.music_fraction), music_due=False,
                groups={GROUPS[g]: dict(handled=request(g, audio.handled[g]),
                    staged=request(g, audio.accepted[g]), reply=request(g, audio.reply[g]),
                    playback="Empty") for g in GROUPS}, song=seed_song(audio.tracks[0]),
                reservations=[], requests={GROUPS[g]: request(g, audio.seed_in[g]) for g in GROUPS},
                phase=dict(ObserveClock=dict(at=instant(Fraction(811, 2)))))
    reader_path = root.parent / "audio-record/measure_native_ramp.py"
    reader = dict(__name__="fixture_import", __file__=str(reader_path))
    source = reader_path.read_text(encoding="utf-8-sig").replace("range(73)", "range(184)")
    exec(compile(source, str(reader_path), "exec"), reader)
    control_dir = root.parent / "audio-record/native-full-on-v19-music-control"
    _, cut_times = reader["data"](control_dir)
    # Both arms compare the fixed control CPU cuts, never expiry's shifted SMP frontiers.
    cut_times = [Fraction(time, reader["DEN"]) for time in cut_times]
    graphs = ast.literal_eval((root / "imported-song-structure.local.txt").read_text(encoding="utf-8"))
    frozen_seed = definitions["BASE"]["ns"]["ens"]["seed"]
    output.mkdir(parents=True, exist_ok=False)
    for arm in ("control", "expire"):
        capture = root.parent / f"audio-record/native-full-on-v19-music-{arm}"
        loaded = (capture / "loaded-APURAM.bin").read_bytes()
        assert loaded == frozen_seed
        histories = release_history(loaded, release_decode)
        arm_seed = dict(seed, groups={GROUPS[g]: dict(seed["groups"][GROUPS[g]],
            history=histories[g]) for g in GROUPS}, resident_music_reply=dict(identity="resident-opening-forever",
            original_byte=loaded[definitions["BASE"]["ns"]["ens"]["f"]["cpuIo0_write"]]))
        raw, _ = reader["data"](capture)
        actual = [(g, Fraction(t, reader["DEN"]), value) for kind, g, value, t, _ in raw
                  if kind == "spc-write" and Fraction(t, reader["DEN"]) <= cut_times[-1]]
        inputs = [dict(group=g, at=instant(Fraction(t, reader["DEN"])), byte=value)
                  for kind, g, value, t, _ in raw if kind == "cpu-write"
                  and Fraction(t, reader["DEN"]) <= cut_times[-1]]
        tables = tomllib.loads((root / f"music-pair-{arm}-cuts.local.toml").read_text(encoding="utf-8"))["cut"]
        timings = tomllib.loads((root / f"music-pair-{arm}-timings.local.toml").read_text(encoding="utf-8"))["publication"]
        values = (root / f"music-pair-{arm}-bytes.local.bin").read_bytes()
        frozen = [(e["group"], Fraction(e["time_numerator"], e["time_denominator"]), value)
                  for e, value in zip(timings, values)]
        assert actual == frozen
        publications = [dict(line=line(g), byte=value, at=instant(time)) for g, time, value in actual]
        fixture = dict(seed=arm_seed, content=content, inputs=inputs, publications=publications,
                       initial_replies=[audio.reply[g] for g in range(4)], cuts=[], expire=arm == "expire",
                       aliases=[dict(group=row["group"], byte=row["original_request"], cue=NAMES[row["name"]])
                                for row in aliases])
        masked = inactive = 0
        masks = []
        diagnostic_history = []
        last = fixture["initial_replies"].copy()
        cursor = 0
        for index, (stop, table) in enumerate(zip(cut_times, tables)):
            ram = (capture / f"cut-{index}-APURAM.bin").read_bytes()
            observed = []
            history_observed = []
            histories = release_history(ram, release_decode)
            for g, entries in histories.items():
                for part, entry in enumerate(entries):
                    active = isinstance(entry, dict)
                    prefix = f"effect.{GROUPS[g]}.part{part}.release_"
                    history_observed.append(dict(field=prefix + "active", value=int(active)))
                    history_observed.append(dict(field=prefix + "remaining", value=entry["Active"] if active else 0))
            # Source scratch may be within an unpublished effect result at a fixed CPU cut.
            # Retain this probe separately; it is not committed semantic State evidence.
            diagnostic_history.append(dict(index=index, fields=history_observed))
            while cursor < len(actual) and actual[cursor][1] <= stop:
                group, _, value = actual[cursor]
                last[group] = value
                cursor += 1
            assert cursor == table["publication_count"]
            for track_index, track in enumerate(table["track"]):
                writers = pending_writers(index, table, track, tables, actual, capture, fields, graphs)
                for semantic, frozen, symbol, width, signed in TRACK_FIELDS:
                    if frozen in writers:
                        masked += 1
                        writer = {
                            "note_remaining": "music157",
                            "gate_remaining": "music913-914/196-197",
                            "duration": "music175",
                            "loudness": "music510" if track["note_remaining"] == 1 else "music785-795",
                            "ramp_remaining": "music517-520" if track["note_remaining"] == 1 else "music785-795",
                            "ramp_target": "music517-520",
                            "ramp_delta": "music517-520",
                        }[frozen]
                        masks.append(dict(cut=index, field=f"track{track_index}.{semantic}", writer=writer))
                        continue
                    if frozen in ("ramp_target", "ramp_delta") and track["ramp_remaining"] == 0:
                        inactive += 1
                        continue
                    offset = fields[symbol] + 2 * track["slot"]
                    value = int.from_bytes(ram[offset:offset + width], "little", signed=signed)
                    observed.append(dict(field=f"track{track_index}.{semantic}", value=value))
                for semantic, symbol in (("timbre", "trackInstrumentIndices"),
                                         ("offset", "trackSubtransposes"),
                                         ("strength", "trackNoteVolume")):
                    value = ram[fields[symbol] + 2 * track["slot"]]
                    if semantic == "timbre":
                        value = typed["TIMBRE_CODES"].index(value)
                    observed.append(dict(field=f"track{track_index}.{semantic}", value=value))
            for semantic, symbol, width in MIX_FIELDS:
                offset = fields[symbol]
                observed.append(dict(field=f"mix.{semantic}",
                                     value=int.from_bytes(ram[offset:offset + width], "little")))
            for voice_index in range(8):
                observed.append(dict(field=f"mix.echo_voice{voice_index}",
                    value=int(bool(ram[definitions["BASE"]["ns"]["ens"]["f"]["echoEnableFlags"]] & (1 << voice_index)))))
            observed.append(dict(field="mix.echo_enabled", value=int(not bool(
                ram[definitions["BASE"]["ns"]["ens"]["f"]["flg"]] & 32))))
            fixture["cuts"].append(dict(index=index, at=instant(stop),
                publications=table["publication_count"], pending_music=table["pending_music_plan"],
                replies=last.copy(), fields=observed))
        assert masked == 77 and len(actual) == 764 and len(fixture["cuts"]) == 184
        assert sum(len(cut["fields"]) for cut in fixture["cuts"]) == 7157
        fixture["source_writer_masks"] = masked
        fixture["masks"] = masks
        fixture["inactive_ramp_observations"] = inactive
        fixture["pins"] = pins
        fixture["roster_sha256"] = roster_hash
        fixture["roster_files"] = len(roster_lines)
        fixture["timbres"] = len(timbre_codes)
        path = output / f"{arm}.local.toml"
        path.write_text("\n".join(f"{key} = {inline(value)}" for key, value in fixture.items()) + "\n",
                        encoding="utf-8", newline="\n")
        diagnostic = output / f"raw-history-{arm}.local.toml"
        diagnostic.write_text("cut = " + inline(diagnostic_history) + "\n", encoding="utf-8", newline="\n")
        print("private_audio_fixture", arm, "cuts", len(fixture["cuts"]),
              "publications", len(publications), "sha256", hashlib.sha256(path.read_bytes()).hexdigest())


if __name__ == "__main__":
    main()
