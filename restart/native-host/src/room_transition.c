// Fixed room-transition scheduler adapter; no candidate gameplay lives here.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <string.h>

typedef int (*TransitionHook)(void *, unsigned, uint8_t *);
static TransitionHook transition_hook;
static void *transition_context;
static unsigned active_transition, active_service, active_child, child_entry;
static unsigned active_directional;
static bool callback_active, root_observed, root_finished;
static uint16_t root_result;
static unsigned depth;
static unsigned routine_stack[27];
static uint16_t operand_stack[27];
static bool rust_stack[27];
static uint64_t counts[709];
static const uint16_t phase_words[15] = {
  0xe17d, 0xe19f, 0xe29e, 0xe2db, 0xe2f7, 0xe310, 0xe353, 0xe36e,
  0xe38e, 0xe3c0, 0xe4a9, 0xe659, 0xe664, 0xe6a2, 0xe737
};
static bool ChildrenActive(void) {
  return sm_native_projectile_active() || sm_native_scroll_active() || sm_native_ceres_door_active() || sm_native_door_scrolling_active() || sm_native_camera_active() || sm_native_item_instruction_active();
}
unsigned sm_native_room_transition_item_context(void) {
  if (!active_transition) return 0;
  return active_service == 53 && depth && !active_child && !active_directional
      && !ChildrenActive() ? 1 : 2;
}
unsigned sm_native_room_transition_active(void) { return active_transition; }
void sm_native_set_room_transition_hook(TransitionHook hook, void *context) {
  if (active_transition || active_service || active_child || ChildrenActive())
    Die("changing room-transition hook during an invocation");
  transition_hook = hook;
  transition_context = context;
}
void sm_native_room_transition_counts(uint64_t *out) { memcpy(out, counts, sizeof(counts)); }

// Separate fixed slot: directional grandchildren must never enter the existing
// three-consumer matrices. Only these two named services admit a directional root.
void sm_native_room_transition_directional_begin(unsigned entry) {
  if (!active_transition) return;
  if (active_directional || active_child || ChildrenActive() ||
      !((active_service == 12 && entry == 0) || (active_service == 19 && entry == 1)))
    Die("directional root outside admitted coordinator service");
  active_directional = entry + 1;
}
void sm_native_room_transition_directional_done(unsigned entry) {
  if (!active_transition) return;
  if (active_directional != entry + 1 || active_child || ChildrenActive() ||
      !((active_service == 12 && entry == 0) || (active_service == 19 && entry == 1)))
    Die("coordinator directional completion mismatch");
  active_directional = 0;
}

static bool ValidParent(unsigned routine, uint16_t operand) {
  if (!depth) return !root_observed && routine == active_transition - 1 && operand == 0;
  unsigned parent = routine_stack[depth - 1];
  if ((parent == 0 || parent == 2) && routine == 3) return true;
  if (parent == 3 && routine >= 4 && routine < 19)
    return operand_stack[depth - 1] == phase_words[routine - 4];
  if ((parent == 4 || parent == 5) && routine == 1) return true;
  if (parent == 4 && routine == 5) return true;
  if (parent == 14 && routine >= 19) return true;
  return (parent == 23 || parent == 26) && routine == 20;
}
static int ObserveRoutine(unsigned event, unsigned routine, uint16_t value) {
  if (!active_transition) return 0; // Shared background helpers also serve initial room loading.
  if (active_service || active_child || ChildrenActive() || routine >= 27 || event > 1) return -1;
  if (!event) {
    if (depth == 27 || !ValidParent(routine, value)) return -2;
    if ((routine != 3 && routine < 19 && value) || (routine >= 19 && value < 0x8000)) return -3;
    if (routine == 3) {
      bool valid = false;
      for (unsigned i = 0; i < 15; i++) valid |= value == phase_words[i];
      if (!valid) return -4;
    }
    for (unsigned i = 0; i < depth; i++) if (routine_stack[i] == routine) return -5;
    if (!depth) root_observed = true;
    routine_stack[depth] = routine;
    operand_stack[depth] = value;
    rust_stack[depth++] = callback_active;
    counts[(callback_active ? 72 : 18) + routine]++;
    sm_native_effect(20, routine, value, 0);
  } else {
    if (!depth || routine_stack[depth - 1] != routine || (routine < 19 && value > 255)) return -6;
    counts[(rust_stack[--depth] ? 99 : 45) + routine]++;
    sm_native_effect(21, routine, value, 0);
    if (!depth) { root_finished = true; root_result = value; }
  }
  return 0;
}
void sm_native_room_transition_routine(unsigned event, unsigned routine, uint16_t value) {
  if (ObserveRoutine(event, routine, value)) Die("invalid C room-transition routine observation");
}
int sm_native_room_transition_observe(unsigned event, unsigned routine, uint16_t value) {
  if (!active_transition || !callback_active) return -1;
  return ObserveRoutine(event, routine, value);
}
void sm_native_room_transition_child_begin(unsigned consumer, unsigned entry) {
  if (!active_transition) return;
  if (!active_service || active_child || active_directional || consumer >= 3)
    Die("room-transition child outside named service or recursive child");
  active_child = consumer + 1;
  child_entry = entry;
  sm_native_effect(25, active_service - 1, consumer, entry);
}
void sm_native_room_transition_child_done(unsigned consumer, unsigned entry) {
  if (!active_transition) return;
  if (!active_service || active_child != consumer + 1 || child_entry != entry || ChildrenActive())
    Die("room-transition child completion mismatch");
  sm_native_effect(26, active_service - 1, consumer, entry);
  active_child = 0;
}
void sm_native_room_transition_child_rust(unsigned consumer, unsigned completed) {
  if (!active_transition) return;
  if (!active_service || active_child != consumer + 1 || completed > 1)
    Die("room-transition Rust child counter context mismatch");
  counts[(completed ? 550 : 391) + (active_service - 1) * 3 + consumer]++;
}
static bool ValidService(unsigned id, uint32_t a, uint32_t b, uint32_t c) {
  if (id >= 53) return false;
  if (id == 7 || id == 50) return a <= 0xffff && b == 0 && c == 0;
  if (id == 20) return a <= 0xffffff && b <= 0xffff && c == 0;
  if (id == 21) return a <= 0xffff && b <= 0xffffff && c <= 0xffff;
  return a == 0 && b == 0 && c == 0;
}
static uint16_t OriginalService(unsigned id, uint32_t a, uint32_t b, uint32_t c) {
  switch (id) {
    case 0: DetermineWhichEnemiesToProcess(); return 0;
    case 1: EnemyMain(); return 0;
    case 2: DrawSamusEnemiesAndProjectiles(); return 0;
    case 3: EnsureSamusDrawnEachFrame(); return 0;
    case 4: LoaadDesinationRoomCreBitset(); return 0;
    case 5: DrawTimer(); return 0;
    case 6: ClearSoundsWhenGoingThroughDoor(); return 0;
    case 7: QueueSfx2_Max15((uint16_t)a); return 0;
    case 8: LoadDoorHeader(); return 0;
    case 9: sub_8882AC(); return 0;
    case 10: CalculateLayer2PosAndScrollsWhenScrolling(); return 0;
    case 11: FixDoorsMovingUp(); return 0;
    case 12: SaveMapExploredifElevator(); return 0;
    case 13: LoadRoomHeader(); return 0;
    case 14: LoadStateHeader(); return 0;
    case 15: LoadMapExploredIfElevator(); return 0;
    case 16: InitializeSpecialEffectsForNewRoom(); return 0;
    case 17: LoadLevelScrollAndCre(); return 0;
    case 18: DoorTransitionScrollingSetup(); return 0;
    case 19: WaitUntilEndOfVblankAndEnableIrq(); return 0;
    case 20: DecompressToMem(a, g_ram + (uint16_t)b); return 0;
    case 21: CopyToVramNow((uint16_t)a, b, (uint16_t)c); return 0;
    case 22: LoadEnemyGfxToVram(); return 0;
    case 23: LoadRoomMusic(); return 0;
    case 24: ClearEprojs(); return 0;
    case 25: ClearAnimtiles(); return 0;
    case 26: ClearPaletteFXObjects(); return 0;
    case 27: ClearPLMs(); return 0;
    case 28: CreatePlmsExecuteDoorAsmRoomSetup(); return 0;
    case 29: LoadFXHeader(); return 0;
    case 30: SpawnDoorClosingPLM(); return 0;
    case 31: UpdateBeamTilesAndPalette(); return 0;
    case 32: LoadColorsForSpritesBeamsAndEnemies(); return 0;
    case 33: LoadEnemies(); return 0;
    case 34: InitializeEnemies(); return 0;
    case 35: ResetProjectileData(); return 0;
    case 36: Samus_LoadSuitTargetPalette(); return 0;
    case 37: ClearFxTilemap(); return 0;
    case 38: SpawnBG3ScrollHdmaObject(); return 0;
    case 39: ClearBg2Tilemap(); return 0;
    case 40: ClearKraidBg2Tilemap(); return 0;
    case 41: AnimtilesHandler(); return 0;
    case 42: LoadNewMusicTrackIfChanged(); return 0;
    case 43: PointlessFunctionStupidToo(); return 0;
    case 44: SetLiquidPhysicsType(); return 0;
    case 45: EprojRunAll(); return 0;
    case 46: QueueEnemyBG2TilemapTransfers(); return 0;
    case 47: PlaySpinJumpSoundIfSpinJumping(); return 0;
    case 48: return AdvancePaletteFadeForAllPalettes_0xc();
    case 49: return HasQueuedMusic();
    case 50: return CallSomeSamusCode((uint16_t)a);
    case 51: return WaitForNMI_Async();
    case 52: return PlmHandler_Async();
    default: Die("invalid room-transition service ID"); return 0;
  }
}
uint16_t sm_native_room_transition_call(unsigned id, uint32_t a, uint32_t b, uint32_t c) {
  if (!ValidService(id, a, b, c)) Die("invalid C room-transition service arguments");
  if (!active_transition) return OriginalService(id, a, b, c);
  if (active_service || active_child || ChildrenActive() || !depth)
    Die("recursive room-transition service or missing routine context");
  active_service = id + 1;
  counts[126 + id]++;
  sm_native_effect(22, id, a, b);
  sm_native_effect(23, id, c, 0);
  uint16_t result = OriginalService(id, a, b, c);
  if (active_child || active_directional || ChildrenActive()) Die("room-transition service returned inside child");
  counts[179 + id]++;
  if ((id == 51 || id == 52) && result) counts[338 + id]++;
  sm_native_effect(24, id, result, 0);
  active_service = 0;
  return result;
}
int sm_native_room_transition_service(uint8_t *ram, unsigned id, uint32_t a, uint32_t b, uint32_t c, uint16_t *out) {
  if (id < 53) counts[232 + id]++;
  if (!active_transition || !callback_active || active_service || active_child || ChildrenActive() || !depth)
    return -1;
  if (!ValidService(id, a, b, c)) return -2;
  memcpy(g_ram, ram, sizeof(g_ram));
  *out = sm_native_room_transition_call(id, a, b, c);
  memcpy(ram, g_ram, sizeof(g_ram));
  counts[285 + id]++;
  return 0;
}
static uint8_t Dispatch(unsigned entry) {
  if (entry >= 3 || active_transition || active_service || active_child || depth || ChildrenActive())
    Die("invalid or recursive room-transition root");
  active_transition = entry + 1;
  root_observed = root_finished = false;
  root_result = 0;
  if (coroutine_state_1) counts[15 + entry]++;
  sm_native_effect(18, entry, door_transition_function, coroutine_state_1);
  uint8_t result = 0;
  int handled = 0;
  if (transition_hook) {
    counts[3 + entry]++;
    callback_active = true;
    handled = transition_hook(transition_context, entry, &result);
    callback_active = false;
    if (handled == 1) counts[6 + entry]++;
    else if (!handled) {
      if (root_observed || depth) Die("room-transition refusal after observation");
      counts[9 + entry]++;
    } else Die("Rust room-transition owned failure; C retry prohibited");
  }
  if (!handled) {
    counts[entry]++;
    switch (entry) {
      case 0: result = GameState_9_HitDoorBlock(); break;
      case 1: result = GameState_10_LoadingNextRoom_Async(); break;
      case 2: result = GameState_11_LoadingNextRoom_Async(); break;
    }
  }
  if (depth || active_service || active_child || ChildrenActive() || !root_finished || root_result != result)
    Die("room-transition root return/observation mismatch");
  if (result) counts[12 + entry]++;
  sm_native_effect(19, entry, result, coroutine_state_1);
  active_transition = 0;
  return result;
}
uint8_t sm_native_room_transition_9(void) { return Dispatch(0); }
uint8_t sm_native_room_transition_10(void) { return Dispatch(1); }
uint8_t sm_native_room_transition_11(void) { return Dispatch(2); }

