// Synchronous directional roots. Gameplay remains in recovered C or Rust.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <string.h>

typedef int (*DoorScrollingHook)(void *, unsigned);
static DoorScrollingHook hook;
static void *context;
static unsigned active_root, active_service, depth;
static unsigned routine_stack[13];
static bool rust_stack[13];
static bool callback_active, child_active, root_observed, root_finished;
static uint64_t counts[94];

unsigned sm_native_door_scrolling_active(void) { return active_root; }
static bool SiblingsActive(void) {
  return sm_native_projectile_active() || sm_native_ceres_door_active() || sm_native_scroll_active() || sm_native_camera_active() || sm_native_item_instruction_active();
}
void sm_native_set_door_scrolling_hook(DoorScrollingHook fn, void *ctx) {
  if (active_root || sm_native_room_transition_active() || SiblingsActive())
    Die("changing door-scrolling hook during invocation");
  hook = fn;
  context = ctx;
}
void sm_native_door_scrolling_counts(uint64_t *out) { memcpy(out, counts, sizeof(counts)); }
static uint32_t Pair(uint16_t lo, uint16_t hi) { return lo | (uint32_t)hi << 16; }
static void Snapshot(bool after) {
  sm_native_effect(after ? 36 : 35, Pair(layer1_x_pos, layer1_y_pos),
    Pair(layer2_x_pos, layer2_y_pos), Pair(reg_BG1HOFS, reg_BG1VOFS));
  sm_native_effect(after ? 38 : 37, Pair(reg_BG2HOFS, reg_BG2VOFS),
    Pair(door_destination_x_pos, door_destination_y_pos), door_transition_flag);
}
static bool ValidParent(unsigned routine) {
  if (!depth) return !root_observed && routine == active_root - 1;
  unsigned parent = routine_stack[depth - 1];
  if (parent == 0) return routine == 7 || routine == 12;
  if (parent == 1) return routine >= 3 && routine <= 6;
  if (parent == 2) return routine >= 9 && routine <= 12;
  if (parent >= 3 && parent <= 6) return routine == 7 || routine == 8 || routine == parent + 6;
  return (parent == 11 || parent == 12) && routine == 7;
}
static int Observe(unsigned event, unsigned routine, uint8_t result, uint16_t counter) {
  if (!active_root) return 0; // Retained private helpers can have other C callers.
  if (active_service || child_active || SiblingsActive() || event > 1 || routine >= 13) return -1;
  if (!event) {
    if (result || depth == 13 || !ValidParent(routine)) return -2;
    for (unsigned i = 0; i < depth; i++) if (routine_stack[i] == routine) return -3;
    if (!depth) root_observed = true;
    routine_stack[depth] = routine;
    rust_stack[depth++] = callback_active;
    counts[(callback_active ? 38 : 12) + routine]++;
    sm_native_effect(29, routine, 0, counter);
  } else {
    if (!depth || routine_stack[depth - 1] != routine || (routine < 9 && result)) return -4;
    counts[(rust_stack[--depth] ? 51 : 25) + routine]++;
    sm_native_effect(30, routine, result, counter);
    if (!depth) root_finished = true;
  }
  return 0;
}
void sm_native_door_scrolling_routine(unsigned event, unsigned routine, uint8_t result) {
  if (Observe(event, routine, result, door_transition_frame_counter))
    Die("invalid C door-scrolling routine observation");
}
int sm_native_door_scrolling_observe(unsigned event, unsigned routine, uint8_t result, uint16_t counter) {
  if (!active_root || !callback_active) return -1;
  return Observe(event, routine, result, counter);
}
void sm_native_door_scrolling_child_begin(unsigned entry) {
  if (!active_root || active_service != 4 || child_active || entry != 1 || SiblingsActive())
    Die("door-scrolling child outside update-scroll service or invalid entry");
  child_active = true;
  sm_native_effect(33, 3, entry, door_transition_frame_counter);
}
void sm_native_door_scrolling_child_done(unsigned entry) {
  if (!active_root || active_service != 4 || !child_active || entry != 1 || SiblingsActive())
    Die("door-scrolling child completion mismatch");
  sm_native_effect(34, 3, entry, door_transition_frame_counter);
  child_active = false;
}
void sm_native_door_scrolling_child_rust(unsigned completed) {
  if (!active_root || active_service != 4 || !child_active || completed > 1)
    Die("door-scrolling Rust child counter context mismatch");
  counts[(completed ? 84 : 80) + 3]++;
}
uint8_t sm_native_door_scrolling_call(unsigned id) {
  if (!active_root || !depth || active_service || child_active || SiblingsActive() || id >= 4)
    Die("invalid or recursive door-scrolling service");
  active_service = id + 1;
  counts[64 + id]++;
  sm_native_effect(31, id, 0, door_transition_frame_counter);
  uint8_t result = 0;
  switch (id) {
    case 0: result = CalculateLayer2Xpos(); break;
    case 1: result = CalculateLayer2Ypos(); break;
    case 2: CalculateBgScrollAndLayerPositionBlocks(); break;
    case 3: UpdateScrollVarsUpdateMap(); break;
  }
  if (child_active || SiblingsActive()) Die("door-scrolling service returned inside child");
  counts[68 + id]++;
  sm_native_effect(32, id, result, door_transition_frame_counter);
  active_service = 0;
  return result;
}
int sm_native_door_scrolling_service(uint8_t *ram, unsigned id, uint8_t *out) {
  if (id < 4) counts[72 + id]++;
  if (!active_root || !callback_active || !depth || active_service || child_active || SiblingsActive() || id >= 4)
    return -1;
  memcpy(g_ram, ram, sizeof(g_ram));
  *out = sm_native_door_scrolling_call(id);
  memcpy(ram, g_ram, sizeof(g_ram));
  counts[76 + id]++;
  return 0;
}
int sm_native_door_scrolling_dispatch(unsigned entry) {
  if (entry >= 3 || active_root || active_service || child_active || depth || SiblingsActive())
    Die("invalid or recursive door-scrolling root");
  sm_native_room_transition_directional_begin(entry);
  counts[(sm_native_room_transition_active() ? 88 : 91) + entry]++;
  active_root = entry + 1;
  root_observed = root_finished = false;
  sm_native_effect(27, entry, door_direction, Pair(door_transition_frame_counter, door_transition_flag));
  Snapshot(false);
  if (hook) {
    counts[3 + entry]++;
    callback_active = true;
    int result = hook(context, entry);
    callback_active = false;
    if (result == 1) { counts[6 + entry]++; return 1; }
    if (result != 0) Die("Rust door-scrolling owned failure; C retry prohibited");
    if (root_observed || depth) Die("door-scrolling refusal after observation");
    counts[9 + entry]++;
  }
  counts[entry]++;
  return 0;
}
void sm_native_door_scrolling_done(unsigned entry) {
  if (active_root != entry + 1 || callback_active || active_service || child_active || depth ||
      !root_observed || !root_finished || SiblingsActive())
    Die("door-scrolling root completion mismatch");
  Snapshot(true);
  sm_native_effect(28, entry, door_direction, Pair(door_transition_frame_counter, door_transition_flag));
  active_root = 0;
  sm_native_room_transition_directional_done(entry);
}
