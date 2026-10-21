"""Install the pinned desktop loop with explicit native SM compatibility."""
import argparse
import hashlib
import os
from pathlib import Path
import re
import runpy
import subprocess
import urllib.request

REVISION = "45a149d8b4bb8098f53d1009f76e76e7d8b79bbf"
FILES = {
    "src/main.c": "88c97c79de7529c5f9f82caf17c9f5732f7a25586474aa4538f7c2522f88a9d2",
    "src/config.c": "d7c4f90eb30050919c329b1aba21f07c122ffa5220edeb059c3d9e3e7364b111",
    "src/config.h": "f428417ba93dd8df89fa0b1ecce7f15be1c14d50efaa9341fd6bac863b5dfc94",
    "src/types.h": "af3bbac047a9a6db0707f9241aa9e19767147ab3783b523cb22cfd55b0409e23",
    "LICENSE.txt": "78b243fa858f4bcb49f8ccca14fec262f62fcee4e44a1ce7565840bdb4ca2217",
}
NATIVE_INPUTS = ("sm/src/main.rs", "sm/src/native_desktop.c", "sm/src/native_desktop.h",
                 "sm/src/types.rs", "sm/src/platform/opengl.rs", "tools/install_native_desktop.py",
                 "tools/install_native_config.py")


def desktop_source(source):
    names = """ChangeWindowScale HitTestCallback DrawPpuFrameWithPerf AudioCallback
        SdlRenderer_Init SdlRenderer_Destroy SdlRenderer_BeginDraw SdlRenderer_EndDraw
        main RenderDigit RenderNumber HandleCommand HandleCommand_Locked HandleInput
        OpenOneGamepad RemapSdlButton HandleGamepadInput HandleVolumeAdjustment
        ApproximateAtan2 HandleGamepadAxisInput SwitchDirectory""".split()
    routines = {}
    for name in names:
        match = re.search(r"^(?:static )?\w+(?: SDLCALL)? " + name +
                          r"\([^\n]*\) \{\n.*?^\}", source, re.M | re.S)
        if not match:
            raise ValueError(f"Missing pinned desktop function: {name}")
        routines[name] = match[0]

    def replace(name, old, new):
        if routines[name].count(old) != 1:
            raise ValueError(f"Desktop patch no longer matches {name}: {old}")
        routines[name] = routines[name].replace(old, new)

    globals_ = source[source.index("enum {\n  kDefaultFullscreen"):source.index("void NORETURN Die(")]
    globals_ = globals_.replace('"The Legend of Zelda: A Link to the Past"', '"SuperMet"')
    globals_ += "\n" + source[source.index("static SDL_mutex *g_audio_mutex;"):source.index("static void SDLCALL AudioCallback(")]
    globals_ = globals_.replace("static SDL_mutex *g_audio_mutex;", "SDL_mutex *sm_desktop_audio_mutex;")
    globals_ += "\nstatic SDL_AudioDeviceID g_audio_device;\n"
    globals_ += "\n" + source[source.index("static SDL_Renderer *g_renderer;"):source.index("static bool SdlRenderer_Init(")]
    table = source[source.index("static const struct RendererFuncs kSdlRendererFuncs"):source.index("void OpenGLRenderer_Create(")]
    replace("ChangeWindowScale", "SDL_Rect bounds;", "SDL_Rect bounds = {0};")
    replace("ChangeWindowScale", "int bt = -1, bl, bb, br;", "int bt = -1, bl = 0, bb = 0, br = 0;")
    replace("ChangeWindowScale", "int mx, my;", "int mx = 0, my = 0;")
    replace("HitTestCallback", "int w, h;", "int w = 0, h = 0;")
    replace("SdlRenderer_Init", "SDL_RendererInfo renderer_info;", "SDL_RendererInfo renderer_info = {0};")
    replace("DrawPpuFrameWithPerf", "PpuGetCurrentRenderScale(g_zenv.ppu, g_ppu_render_flags)", "sm_host_render_scale(game, g_ppu_render_flags)")
    routines["DrawPpuFrameWithPerf"] = routines["DrawPpuFrameWithPerf"].replace("ZeldaDrawPpuFrame(", "sm_host_copy_frame(game, ")
    replace("DrawPpuFrameWithPerf", "g_curr_fps = average * (1.0f / 64);", "g_curr_fps = sm_host_fps(average * (1.0f / 64));")
    replace("DrawPpuFrameWithPerf", "g_renderer_funcs.BeginDraw(", "sm_host_begin_draw(&g_renderer_funcs, ")
    replace("DrawPpuFrameWithPerf", "  g_renderer_funcs.EndDraw();", """  int mismatches = sm_host_mismatch_count(game);
  if (mismatches != 0)
    RenderNumber(pixel_buffer + pitch * render_scale, pitch, mismatches, render_scale == 4);
  sm_host_end_draw(&g_renderer_funcs);""")
    replace("AudioCallback", "g_audiobuffer_end - g_audiobuffer_cur == 0", "(long)(g_audiobuffer_end - g_audiobuffer_cur) == 0")
    replace("AudioCallback", "IntMin(len, g_audiobuffer_end - g_audiobuffer_cur)", "IntMin(len, (int)(long)(g_audiobuffer_end - g_audiobuffer_cur))")
    replace("AudioCallback", "ZeldaRenderAudio(", "sm_host_render_audio(game, ")
    replace("AudioCallback", "  ZeldaDiscardUnusedAudioFrames();\n", "")
    # The owner gate surrounds the entire SDL callback; its internal SDL lock stays here.
    routines["AudioCallback"] = routines["AudioCallback"].replace("void *userdata, ", "")
    replace("HandleCommand", "  SDL_LockMutex(g_audio_mutex);\n  HandleCommand_Locked(j, pressed);\n  SDL_UnlockMutex(g_audio_mutex);", "  HandleCommand_Locked(j, pressed);")
    r = "HandleCommand_Locked"
    routines[r] = routines[r].replace("SaveLoadSlot(", "sm_host_save_load(game, ")
    replace(r, "case kKeys_CheatLife: PatchCommand('w'); break;", "case kKeys_CheatLife: sm_host_cheat(game, 'w'); break;")
    replace(r, "case kKeys_CheatEquipment: PatchCommand('W'); break;", "case kKeys_ToggleWhichFrame: sm_host_game_command(game, j, 0); break;")
    replace(r, "case kKeys_CheatKeys: PatchCommand('o'); break;", "case kKeys_CheatJump: sm_host_cheat(game, 'q'); break;")
    replace(r, "    case kKeys_CheatWalkThroughWalls: PatchCommand('E'); break;\n", "")
    replace(r, "case kKeys_ClearKeyLog: PatchCommand('k'); break;", "case kKeys_ClearKeyLog: sm_host_game_command(game, j, 0); break;")
    replace(r, "case kKeys_StopReplay: PatchCommand('l'); break;", "case kKeys_StopReplay: sm_host_game_command(game, j, 0); break;")
    replace(r, "ZeldaReset(true);", "sm_host_game_command(game, j, 0);")
    start = routines[r].index("    case kKeys_PauseDimmed:")
    end = routines[r].index("    case kKeys_ReplayTurbo:", start)
    routines[r] = routines[r][:start] + "    case kKeys_PauseDimmed: g_paused = !g_paused; break;\n" + routines[r][end:]
    replace(r, "case kKeys_ToggleRenderer: g_ppu_render_flags ^= kPpuRenderFlags_NewRenderer; break;", "case kKeys_ToggleRenderer: g_ppu_render_flags ^= kPpuRenderFlags_NewRenderer; sm_host_game_command(game, j, g_ppu_render_flags); break;")
    replace(r, "default: assert(0);", "default: sm_host_bad_command();")
    r = "HandleGamepadInput"
    routines[r] = routines[r].replace("1 << button", "1u << ((uint32)button & 31)")
    replace(r, "  if (pressed)\n    g_gamepad_last_cmd[button] = FindCmdForGamepadButton(button, g_gamepad_modifiers);", """  int command = pressed ? FindCmdForGamepadButton(button, g_gamepad_modifiers) : 0;
  if ((uint32)button >= 17) sm_host_gamepad_bounds(button);
  if (pressed) g_gamepad_last_cmd[button] = command;""")
    r = "HandleVolumeAdjustment"
    routines[r] = routines[r][:routines[r].index("#if SYSTEM_VOLUME_MIXER_AVAILABLE")] + routines[r][routines[r].index("#else\n") + 6:routines[r].index("#endif")] + "}"
    replace("SwitchDirectory", "char buf[4096];", "char buf[4096] = {0};")
    replace("SwitchDirectory", '"/zelda3.ini", 12', '"/sm.ini", 8')
    replace("SwitchDirectory", '"Found zelda3.ini in %s\\n"', '"Found sm.ini in %s\\n"')
    replace("RenderNumber", "char buf[32], *s;", "char buf[32] = {0}, *s;")
    # Retain the published event/window loop, replacing only game-owned phases.
    r = "main"
    replace(r, "int main(int argc, char** argv)", "int main(int argc, char** argv, const void *owner)")
    start = routines[r].index("  ParseConfigFile(config_file);")
    end = routines[r].index("  if (g_config.fullscreen == 1)", start)
    routines[r] = routines[r][:start] + """  if (argc >= 1 && strcmp(argv[0], "--debug") == 0) {
    sm_host_debug(owner);
    argc -= 1; argv += 1;
  }
  ParseConfigFile(config_file);
  g_snes_width = g_config.extended_aspect_ratio * 2 + 256;
  g_snes_height = 240;
  g_ppu_render_flags = g_config.new_renderer * kPpuRenderFlags_NewRenderer |
      g_config.enhanced_mode7 * kPpuRenderFlags_4x4Mode7 |
      g_config.extend_y * kPpuRenderFlags_Height240 |
      g_config.no_sprite_limits * kPpuRenderFlags_NoSpriteLimits;
""" + routines[r][end:]
    replace(r, "g_config.output_method == kOutputMethod_OpenGL ||\n      g_config.output_method == kOutputMethod_OpenGL_ES", "g_config.output_method == kOutputMethod_OpenGL")
    replace(r, "OpenGLRenderer_Create(&g_renderer_funcs, (g_config.output_method == kOutputMethod_OpenGL_ES));", "sm_host_OpenGLRenderer_Create(&g_renderer_funcs);")
    replace(r, "  SDL_Window* window = SDL_CreateWindow(", """  const char *filename = argv[0] ? argv[0] : "sm.smc";
  Snes *snes = sm_host_start_game(owner, filename);
  if (!snes) {
    char error[256] = {0};
    snprintf(error, sizeof(error), "unable to load rom: %s", filename);
    Die(error);
    return 1;
  }
  SDL_Window* window = SDL_CreateWindow(""")
    replace(r, "!g_renderer_funcs.Initialize(window)", "!sm_host_initialize_renderer(owner, &g_renderer_funcs, window)")
    replace(r, "SDL_AudioSpec want = { 0 }, have;", "SDL_AudioSpec want = {0}, have = {0};\n  want.userdata = have.userdata = (void *)owner;")
    replace(r, "  SDL_AudioDeviceID device = 0;\n", "")
    routines[r] = re.sub(r"\bdevice\b", "g_audio_device", routines[r])
    replace(r, "Failed to open audio g_audio_device:", "Failed to open audio device:")
    replace(r, "  if (g_config.enable_audio) {\n    want.freq", "  sm_host_create_player(owner);\n  sm_host_initialize_player(owner);\n  {\n    want.freq")
    replace(r, "want.freq = g_config.audio_freq;", "want.freq = 44100;")
    replace(r, "want.channels = g_config.audio_channels;", "want.channels = 2;")
    replace(r, "want.samples = g_config.audio_samples;", "want.samples = 2048;")
    replace(r, "want.callback = &AudioCallback;", "want.callback = &sm_host_audio_callback;")
    replace(r, "g_audio_channels = have.channels;", "g_audio_channels = 2;")
    start = routines[r].index("  if (argc >= 1 && !g_run_without_emu)")
    end = routines[r].index("  for (int i = 0; i < SDL_NumJoysticks(); i++)", start)
    routines[r] = routines[r][:start] + """  sm_host_prepare_pixels(snes);
#ifdef _WIN32
  _mkdir("saves");
#else
  mkdir("saves", 0755);
#endif
  sm_host_read_sram(owner);

""" + routines[r][end:]
    replace(r, "  SDL_Event event;\n", "")
    autoload = "  if (g_config.autosave)\n    HandleCommand(kKeys_Load + 0, true);\n"
    replace(r, autoload, "")
    replace(r, "  bool running = true;", autoload + "\n  bool running = true;")
    replace(r, "  bool audiopaused = true;", "  uint8 audiopaused = true;\n  bool has_bug_in_title = false;")
    replace(r, "  while(running) {", "  while(running) {\n    SDL_Event event = {0};")
    for name, adapter in (("HandleInput", "sm_host_key"), ("HandleGamepadInput", "sm_host_button"),
                          ("HandleGamepadAxisInput", "sm_host_axis"), ("HandleCommand", "sm_host_command")):
        routines[r] = routines[r].replace(name + "(", adapter + "(owner, ")
    start = routines[r].index("    SDL_LockMutex(g_audio_mutex);")
    end = routines[r].index("    // if vsync isn't working", start)
    routines[r] = routines[r][:start] + """    bool is_replay = sm_host_run_frame(owner, inputs);
    frameCtr++;
    bool disable_render = (g_turbo ^ (is_replay & g_replay_turbo)) &&
        (frameCtr & (g_turbo ? 0xf : 0x7f)) != 0;
    bool want_bug_in_title = sm_host_present(owner, &disable_render);
    if (want_bug_in_title != has_bug_in_title) {
      has_bug_in_title = want_bug_in_title;
      if (want_bug_in_title) {
        char title[60] = {0};
        snprintf(title, sizeof(title), "%s | BUG FOUND!", kWindowTitle);
        SDL_SetWindowTitle(g_window, title);
      } else SDL_SetWindowTitle(g_window, kWindowTitle);
    }

""" + routines[r][end:]
    replace(r, "if (!g_config.disable_frame_delay)", "if (!disable_render && !g_config.disable_frame_delay)")
    replace(r, "  if (g_config.enable_audio) {\n    SDL_PauseAudioDevice(g_audio_device, 1);\n    SDL_CloseAudioDevice(g_audio_device);\n  }", "  SDL_PauseAudioDevice(g_audio_device, 1);\n  SDL_CloseAudioDevice(g_audio_device);")
    replace(r, "g_renderer_funcs.Destroy();", "sm_host_destroy_renderer(owner, &g_renderer_funcs);")
    context = {"DrawPpuFrameWithPerf", "AudioCallback", "HandleCommand", "HandleCommand_Locked",
               "HandleInput", "HandleGamepadInput", "HandleGamepadAxisInput"}
    for name, text in routines.items():
        signature, body = text.split(" {", 1)
        if name in context:
            signature = signature.replace("()", "(Game *game)") if signature.endswith("()") else signature.replace("(", "(Game *game, ", 1)
        for called in context:
            body = re.sub(r"\b" + called + r"\((?!\))", called + "(game, ", body)
            body = body.replace(called + "()", called + "(game)")
        routines[name] = signature + " {" + body
    # These entries are called only inside their Rust owner/SDL callback boundaries.
    for name in context - {"HandleCommand_Locked"}:
        routines[name] = re.sub(r"^static ", "", routines[name]).replace(" SDLCALL ", " ")
    symbols = {name: "sm_desktop_" + name for name in names}
    symbols.update({"main": "sm_desktop_run", "Die": "sm_host_die", "g_config": "sm_config_state",
                    "g_audio_mutex": "sm_desktop_audio_mutex", "ParseConfigFile": "sm_host_parse_config",
                    "FindCmdForSdlKey": "sm_host_find_key", "FindCmdForGamepadButton": "sm_host_find_button"})
    prototypes = "\n".join(text[:text.index(" {")] + ";" for text in routines.values())
    result = globals_ + "\n#define RESIZE_BORDER 20\n" + prototypes + "\n" + table + "\n\n" + "\n\n".join(routines.values()) + "\n"
    result = re.sub(r"\b(?:" + "|".join(symbols) + r")\b", lambda m: symbols[m[0]], result)
    return result.replace("fprintf(stderr,", "fprintf(sm_host_stderr(),")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--zig", required=True, type=Path)
    parser.add_argument("--sdl-include", required=True, type=Path)
    parser.add_argument("--prefix", type=Path, default=Path(os.environ["LOCALAPPDATA"]) / "sm-decomp-tools/native-desktop-45a149d8")
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
    config_sources = runpy.run_path(str(root / "tools/install_native_config.py"))["config_sources"]
    _, header = config_sources((source / "src/config.c").read_text(), (source / "src/config.h").read_text())
    (library / "config.h").write_text(header, encoding="utf-8")
    (library / "native_desktop_selected.c").write_text(desktop_source((source / "src/main.c").read_text()), encoding="utf-8")
    blobs = [(root / name).read_bytes() for name in NATIVE_INPUTS]
    declarations = [f"static const unsigned char sm_desktop_source_{i}[] = {{" +
                    ",".join(str(byte) for byte in data) + "};" for i, data in enumerate(blobs)]
    declarations.append("static const unsigned char *const sm_desktop_source_bytes[] = {" +
                        ",".join(f"sm_desktop_source_{i}" for i in range(len(blobs))) + "};")
    declarations.append("static const size_t sm_desktop_source_lengths[] = {" +
                        ",".join(str(len(data)) for data in blobs) + "};")
    (library / "native_desktop_sources.h").write_text("\n".join(declarations) + "\n", encoding="ascii")
    if args.emit_only:
        return
    obj = library / "native_desktop.o"
    subprocess.run([str(args.zig), "cc", "-target", "x86_64-windows-gnu", "-O2", "-std=c11",
                    "-fexceptions", "-fwrapv", "-fno-strict-aliasing", "-ffp-contract=off", "-ffunction-sections", "-fdata-sections",
                    "-I", str(library), "-I", str(source / "src"), "-I", str(root / "sm/src"),
                    "-I", str(args.sdl_include), "-c", str(root / "sm/src/native_desktop.c"), "-o", str(obj)], check=True)
    subprocess.run([str(args.zig), "ar", "rcs", str(library / "libsm_desktop.a"), str(obj)], check=True)
    print(f"Installed native desktop in {library}; add this directory to LIBRARY_PATH.")


if __name__ == "__main__":
    main()
