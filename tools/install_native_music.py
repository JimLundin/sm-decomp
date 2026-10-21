"""Install selected Zelda music routines with the maintained SM compatibility changes."""
import argparse
import hashlib
import os
from pathlib import Path
import re
import subprocess
import urllib.request

REVISION = "45a149d8b4bb8098f53d1009f76e76e7d8b79bbf"
FILES = {
    "src/spc_player.c": "7cf1ddebb9ef85aaafd40cd5b4340726a79568be34240ae0a92326077b0e4644",
    "src/types.h": "af3bbac047a9a6db0707f9241aa9e19767147ab3783b523cb22cfd55b0409e23",
    "snes/dsp_regs.h": "2dcd8a034eb617579347173913c7d8970fc932c97d0b517562b730f766e3c306",
    "LICENSE.txt": "78b243fa858f4bcb49f8ccca14fec262f62fcee4e44a1ce7565840bdb4ca2217",
}
NATIVE_INPUTS = ("sm/src/native_music.c", "sm/src/native_music.h", "tools/install_native_music.py")


def native_hash(root):
    return hashlib.sha256(b"\0".join((root / name).read_bytes() for name in NATIVE_INPUTS)).hexdigest()


def music_source(source):
    names = """SpcDivHelper Chan_DoAnyFade SetupEchoParameter_EDL WriteVolumeToDsp
        WritePitch Music_ResetChan Channel_SetInstrument ComputePitchAdd PitchSlideToNote_Check
        HandleEffect CalcVibratoAddPitch HandlePanAndSweep HandleNoteTick Chan_HandleTick
        Port0_HandleMusic PlayNote""".split()
    routines = {}
    for name in names:
        match = re.search(r"^(?:static (?:inline )?)?\w+ " + name + r"\([^\n]*\) \{\n.*?^\}",
                          source, re.M | re.S)
        if not match:
            raise ValueError(f"Missing pinned function: {name}")
        routines[name] = match[0]

    def replace(name, old, new):
        if routines[name].count(old) != 1:
            raise ValueError(f"Compatibility patch no longer matches {name}: {old}")
        routines[name] = routines[name].replace(old, new)

    replace("SetupEchoParameter_EDL", "+ 0xd1", "+ 0x16")
    routines["WriteVolumeToDsp"] = routines["WriteVolumeToDsp"].replace("0x1178", "0x1e1d").replace("0x1179", "0x1e1e")
    # Keep the shared native pitch/instrument primitives used by SFX.
    pitch = routines["WritePitch"]
    pitch = pitch[:pitch.index("\n  uint8 pp")] + "\n  WritePitchInner(p, c, pitch);\n}"
    routines["WritePitch"] = re.sub(r"^  static const uint16 kBaseNoteFreqs.*\n", "", pitch, flags=re.M)
    instrument = routines["Channel_SetInstrument"]
    routines["Channel_SetInstrument"] = instrument[:instrument.index("\n  if (instrument")] + "\n  Channel_SetInstrument_NoSetId(p, c, instrument);\n}"
    replace("Music_ResetChan", "c->pitch_envelope_num_ticks = 0;", "c->pitch_envelope_num_ticks = 0;\n    c->cutk = 0;")
    replace("PitchSlideToNote_Check", """  if (p->cur_chan_bit & p->is_chan_on) {
    c->pattern_order_ptr_for_chan += 4;
    return;
  }
""", "")
    replace("HandleEffect", "kEffectByteLength[effect - 0xe0]", "EffectByteLength(effect)")
    replace("HandleEffect", "    if (!p->pause_music_ctr && !p->byte_3E1)\n      p->master_volume = arg << 8;",
            "    p->master_volume = arg << 8;")
    replace("HandleEffect", "p->reg_EON = p->echo_channels = arg;", "p->reg_EON = arg;")
    replace("HandleEffect", "  case 0xf4:", """  case 0xf1:
  case 0xf2:
    c->pitch_envelope_direction = effect == 0xf1;
    c->pitch_envelope_delay = arg;
    c->pitch_envelope_num_ticks = p->ram[c->pattern_order_ptr_for_chan++];
    c->pitch_envelope_slide_value = p->ram[c->pattern_order_ptr_for_chan++];
    break;
  case 0xf3:
    c->pitch_envelope_num_ticks = 0;
    break;
  case 0xf4:""")
    replace("HandleEffect", "  default:", """  case 0xfb:
    c->pattern_order_ptr_for_chan++;
    break;
  case 0xfc:
    c->cutk = arg + 1;
    break;
  case 0xfd:
    p->fast_forward = arg + 1;
    p->key_OFF |= ~p->is_chan_on;
    break;
  case 0xfe:
    p->fast_forward = arg;
    p->key_OFF |= ~p->is_chan_on;
    break;
  default:""")
    replace("HandlePanAndSweep", "  if (c->tremolo_depth) {\n    c->tremolo_hold_count = c->tremolo_delay_ticks;",
            "  if (c->tremolo_depth && c->tremolo_hold_count == c->tremolo_delay_ticks) {")
    replace("HandleNoteTick", "} else if (!(p->is_chan_on & p->cur_chan_bit)) {", "} else {")
    replace("HandleNoteTick", "      Chan_DoAnyFade(&c->pitch, c->pitch_add_per_tick, c->pitch_target, --c->pitch_slide_length);",
            """      c->pitch_slide_length--;
      c->pitch = c->pitch_slide_length ? c->pitch + c->pitch_add_per_tick :
          c->pitch_target << 8 | c->fine_tune;""")
    for ticks, field, add, target in (
        ("volume_fade_ticks", "channel_volume", "volume_fade_addpertick", "volume_fade_target"),
        ("pan_num_ticks", "pan_value", "pan_add_per_tick", "pan_target_value"),
    ):
        replace("Chan_HandleTick", f"    p->vol_dirty |= p->cur_chan_bit;\n    Chan_DoAnyFade(&c->{field}, c->{add}, c->{target}, true);",
                f"    Chan_DoAnyFade(&c->{field}, c->{add}, c->{target}, c->{ticks});\n    p->vol_dirty |= p->cur_chan_bit;")
    replace("Chan_HandleTick", "CalcTremolo(p, c);", "CalcTremolo(p, c, c->tremolo_count);")
    replace("Port0_HandleMusic", "  uint8 a = p->new_value_from_snes[0];", "  uint8 a = p->input_ports[0];\n  p->input_ports[0] = 255;")
    replace("Port0_HandleMusic", "  if (a == 0) {\nhandle_cmd_00:", """  if (a == 0xf0) {
    p->key_OFF |= ~p->is_chan_on;
    return;
  }
  if (a == 0xf1 || a == 255 || a == p->port_to_snes[0]) {""")
    replace("Port0_HandleMusic", "    if (p->pause_music_ctr != 0 && --p->pause_music_ctr == 0)\n      goto HandleCmd_0xf0_PauseMusic;\n", "")
    replace("Port0_HandleMusic", "      if (t == 0)\n        goto HandleCmd_0xf0_PauseMusic;", """      if (t == 0) {
        a = 0;
        goto new_song;
      }""")
    replace("Port0_HandleMusic", "!p->fast_forward && !(p->is_chan_on & p->cur_chan_bit)", "!(c->cutk | p->fast_forward)")
    # Discard Zelda pause/duck commands; SM consumes a new song through one tail.
    music = routines["Port0_HandleMusic"]
    music = music[:music.index("  } else if (a == 0xff)")] + "  } else {\nnew_song:\n" + music[music.index("    p->port_to_snes[0] = a;"):]
    routines["Port0_HandleMusic"] = music.replace("0xD000", "0x5820")
    prototypes = "\n".join(text[:text.index(" {")] + ";" for text in routines.values())
    return prototypes + "\n\n" + "\n\n".join(routines.values()) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--zig", required=True, type=Path)
    parser.add_argument("--prefix", type=Path, default=Path(os.environ["LOCALAPPDATA"]) / "sm-decomp-tools/native-music-45a149d8")
    parser.add_argument("--library-directory", type=Path)
    parser.add_argument("--emit-only", action="store_true")
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    source = args.prefix / "source"
    library = args.library_directory or args.prefix / "lib"
    library.mkdir(parents=True, exist_ok=True)
    for name, expected in FILES.items():
        path = source / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.exists():
            urllib.request.urlretrieve(f"https://raw.githubusercontent.com/snesrev/zelda3/{REVISION}/{name}", path)
        if hashlib.sha256(path.read_bytes()).hexdigest() != expected:
            raise ValueError(f"Upstream source checksum mismatch: {name}")
    selected = library / "native_music_selected.h"
    selected.write_text(music_source((source / "src/spc_player.c").read_text()), encoding="utf-8")
    stamp = library / "native_music_source.h"
    stamp.write_text(f'#define SM_MUSIC_SOURCE_HASH "{native_hash(root)}"\n', encoding="ascii")
    if args.emit_only:
        return
    obj = library / "native_music.o"
    subprocess.run([str(args.zig), "cc", "-target", "x86_64-windows-gnu", "-O2", "-std=c11",
                    "-fexceptions", "-fwrapv", "-fno-strict-aliasing", "-ffunction-sections", "-fdata-sections",
                    "-include", str(stamp), "-I", str(source), "-I", str(library),
                    "-c", str(root / "sm/src/native_music.c"), "-o", str(obj)], check=True)
    subprocess.run([str(args.zig), "ar", "rcs", str(library / "libsm_music.a"), str(obj)], check=True)
    print(f"Installed native music library in {library}; add this directory to LIBRARY_PATH.")


if __name__ == "__main__":
    main()
