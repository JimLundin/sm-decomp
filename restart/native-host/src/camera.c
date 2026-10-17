// Standalone normal-camera body boundary. This adapter performs no gameplay.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include <string.h>

typedef int (*CameraHook)(void *);
static CameraHook hook;
static void *context;
static bool active_camera, callback_active, root_observed, root_finished;
static unsigned depth, routine_stack[13];
static bool rust_stack[13];
static uint64_t counts[56];
unsigned sm_native_camera_active(void) { return active_camera; }
static bool OtherConsumerActive(void) {
  return sm_native_projectile_active() || sm_native_scroll_active() ||
    sm_native_ceres_door_active() || sm_native_room_transition_active() ||
    sm_native_door_scrolling_active() || sm_native_item_instruction_active();
}
void sm_native_set_camera_hook(CameraHook fn, void *ctx) {
  if (active_camera || OtherConsumerActive()) Die("changing camera hook during invocation");
  hook = fn;
  context = ctx;
}
void sm_native_camera_counts(uint64_t *out) { memcpy(out, counts, sizeof(counts)); }
static uint32_t Pair(uint16_t integer, uint16_t fraction) { return integer | (uint32_t)fraction << 16; }
static void Snapshot(bool after) {
  sm_native_effect(after ? 44 : 43, Pair(layer1_x_pos, layer1_x_subpos),
    Pair(layer1_y_pos, layer1_y_subpos), Pair(samus_x_pos, samus_x_subpos));
  sm_native_effect(after ? 46 : 45, Pair(samus_y_pos, samus_y_subpos),
    Pair(samus_prev_x_pos, samus_prev_x_subpos), Pair(samus_prev_y_pos, samus_prev_y_subpos));
}
static bool ValidParent(unsigned routine) {
  if (!depth) return !root_observed && routine == 0;
  switch (routine_stack[depth - 1]) {
    case 0: return routine == 1 || (routine >= 3 && routine <= 7) || routine == 10;
    case 1: return routine == 2;
    case 3: return routine >= 7 && routine <= 9;
    case 4: return routine >= 10 && routine <= 12;
    default: return false;
  }
}
static int Observe(unsigned event, unsigned routine, uint16_t operand) {
  if (!active_camera) return 0; // Distance/constraint helpers have other C callers.
  if (OtherConsumerActive() || event > 1 || routine >= 13) return -1;
  if (!event) {
    if (depth == 13 || !ValidParent(routine) || (routine == 1 ? operand != 0x9589 : operand != 0)) return -2;
    for (unsigned i = 0; i < depth; i++) if (routine_stack[i] == routine) return -3;
    if (!depth) root_observed = true;
    routine_stack[depth] = routine;
    rust_stack[depth++] = callback_active;
    counts[(callback_active ? 30 : 4) + routine]++;
    sm_native_effect(41, routine, operand, 0);
  } else {
    if (!depth || routine_stack[depth - 1] != routine || operand) return -4;
    counts[(rust_stack[--depth] ? 43 : 17) + routine]++;
    sm_native_effect(42, routine, 0, 0);
    if (!depth) root_finished = true;
  }
  return 0;
}
void sm_native_camera_routine(unsigned event, unsigned routine, uint16_t operand) {
  if (Observe(event, routine, operand)) Die("invalid C camera routine observation");
}
int sm_native_camera_observe(unsigned event, unsigned routine, uint16_t operand) {
  if (!active_camera || !callback_active) return -1;
  return Observe(event, routine, operand);
}
int sm_native_camera_dispatch(void) {
  if (active_camera || depth || OtherConsumerActive()) Die("recursive or cross-consumer camera root");
  active_camera = true;
  root_observed = root_finished = false;
  sm_native_effect(39, game_state, room_ptr, slow_grabble_scrolling_flag);
  Snapshot(false);
  if (hook) {
    counts[1]++;
    callback_active = true;
    int result = hook(context);
    callback_active = false;
    if (result == 1) { counts[2]++; return 1; }
    if (result != 0) Die("Rust camera owned failure; C retry prohibited");
    if (root_observed || depth) Die("camera refusal after observation");
    counts[3]++;
  }
  counts[0]++;
  return 0;
}
void sm_native_camera_done(void) {
  if (!active_camera || callback_active || depth || !root_observed || !root_finished || OtherConsumerActive())
    Die("camera root completion mismatch");
  Snapshot(true);
  sm_native_effect(40, game_state, room_ptr, slow_grabble_scrolling_flag);
  active_camera = false;
}
