// Complete vertical roots, with one bounded synchronous original terrain call.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <limits.h>
#include <string.h>

typedef int (*MotionHook)(void *, unsigned, int32_t);
typedef int (*MotionObserver)(void *, unsigned, unsigned, int32_t, int32_t, const uint8_t *);
static MotionHook hook;
static MotionObserver observer;
static void *context, *observer_context;
static unsigned active_root, terrain_calls;
static int32_t root_amount, terrain_amount;
static bool callback_active, service_active, terrain_active, observing, admitted, poisoned, committed, rust_owned;
static const uint8_t *issued_ram;
static uint64_t generation, counts[23];

static bool SiblingActive(void) {
  return sm_native_projectile_active() || sm_native_scroll_active() || sm_native_ceres_door_active() ||
    sm_native_room_transition_active() || sm_native_door_scrolling_active() || sm_native_camera_active() ||
    sm_native_item_instruction_active() || sm_native_persistence_active();
}
unsigned sm_native_player_motion_active(void) { return active_root; }
uint64_t sm_native_player_motion_generation(void) { return generation; }
void sm_native_player_motion_counts(uint64_t *out) { memcpy(out, counts, sizeof(counts)); }
int sm_native_player_motion_pointer_clear(void) { return cur_coll_amt32 == NULL; }
static uint16_t Word(const uint8_t *ram, unsigned at) { return ram[at] | (uint16_t)ram[at + 1] << 8; }

static bool PreparedTables(void) {
  static const uint8_t crumble[] = {0x37,0xce,0xf9,0xc9}, speed[] = {0xea,0xcd,0x51,0xc9}, sand[] = {0xc4,0xb4,0xe3,0xaa};
  static const uint8_t tc[] = {0x44,0xd0}, ts[] = {0x30,0xd0}, ta[] = {0x59,0x92}, tq[] = {0x2b,0xb7};
  return !memcmp(RomFixedPtr(0x84d044), crumble, 4) && !memcmp(RomFixedPtr(0x84d030), speed, 4) &&
    !memcmp(RomFixedPtr(0x84b72b), sand, 4) && !memcmp(RomFixedPtr(0x949139), tc, 2) &&
    !memcmp(RomFixedPtr(0x9491fd), ts, 2) && !memcmp(RomFixedPtr(0x9492e1), ta, 2) &&
    !memcmp(RomFixedPtr(0x9492f1), ta, 2) && !memcmp(RomFixedPtr(0x949259), tq, 2);
}
int sm_native_player_motion_prepared_tables(void) { return PreparedTables(); }
static bool Reaction(unsigned index) {
  unsigned kind = level_data[index] >> 12, bts = BTS[index];
  switch (kind) {
    case 0: case 1: case 2: case 4: case 6: case 8: case 12: case 14: return true;
    case 10: return bts < 16 && area_index < 8;
    case 7: case 15: return (bts & 0x80) != 0; // No unsealed bombable header lookup.
    case 3: case 11:
      return bts == 0 || bts == 0x62 || (bts == 0x80 && area_index == 4);
    case 9: {
      uint32_t address = (uint32_t)door_list_pointer + 2 * (bts & 0x7f);
      if (address < 0x8000 || address > 0xfffe) return false;
      const uint8_t *pointer = RomFixedPtr(0x8f0000 | address);
      unsigned door = pointer[0] | (unsigned)pointer[1] << 8;
      return door >= 0x8000 && door <= 0xfff4;
    }
    default: return false;
  }
}
// Initial subset: a nonwrapping span of at most two columns, one extension hop
// to a distinct terminal per column, and three exact PLM setup mappings. Setups
// alter only their terminal cell, so earlier reactions cannot invalidate a later
// admitted cell. Longer chains, shared targets and arbitrary PLMs are refused.
static bool TerrainAdmitted(int32_t amount) {
  if (!PreparedTables()) return false;
  if (!amount) return true;
  unsigned width = room_width_in_blocks, height = room_height_in_blocks;
  unsigned x = samus_x_pos, y = samus_y_pos, xr = samus_x_radius, yr = samus_y_radius;
  if (!width || width > 255 || !height || height > 255 || width * height > 0x3200 ||
      !xr || xr > 16 || !yr || yr > 16 || x < xr || x + xr > width * 16 ||
      amount < -0x100000 || amount > 0x100000) return false;
  int64_t destination = (int64_t)((uint32_t)y << 16 | samus_y_subpos) + amount;
  if (destination < 0 || destination > UINT32_MAX) return false;
  int ypos = destination >> 16;
  int edge = amount < 0 ? ypos - (int)yr : ypos + (int)yr - 1;
  if (edge < 0 || edge >= (int)(height * 16) || (edge >> 4) > 255) return false;
  unsigned left = (x - xr) >> 4, right = (x + xr - 1) >> 4;
  if (right - left > 1) return false;
  unsigned first = (edge >> 4) * width + left, last = first + right - left;
  unsigned terminals[2];
  for (unsigned i = first; i <= last; i++) {
    unsigned target = i, kind = level_data[i] >> 12;
    if (kind == 5 || kind == 13) {
      int offset = (int8_t)BTS[i];
      if (!offset) { terminals[i - first] = i; continue; }
      int next = (int)i + offset * (kind == 13 ? (int)width : 1);
      if (next < 0 || next >= (int)(width * height) || (next >= (int)first && next <= (int)last)) return false;
      target = next;
    }
    if (!Reaction(target)) return false;
    terminals[i - first] = target;
  }
  return first == last || terminals[0] != terminals[1];
}
int sm_native_player_motion_admitted(unsigned entry, int32_t amount) {
  return entry < 2 && !(entry == 0 && amount == INT32_MIN) && TerrainAdmitted(amount);
}
int sm_native_player_motion_available(void) { return active_root && callback_active && admitted; }
static bool ValidList(void) {
  if (!interactive_enemy_indexes_write_ptr) return true;
  for (unsigned i = 0; i < 32; i++) {
    unsigned slot = interactive_enemy_indexes[i];
    if (slot == 0xffff) return true;
    if (slot > 0x7c0 || (slot & 0x3f)) return false;
  }
  return false;
}
int sm_native_player_motion_fixture_admitted(unsigned entry, int32_t amount) {
  if (entry >= 7 || ((entry == 0 || entry == 3) && amount == INT32_MIN)) return 0;
  if (entry <= 2 && !ValidList()) return 0;
  if ((entry == 1 || (entry == 6 && (samus_collision_flag || (uint8_t)input_to_pose_calc != 5))) && samus_movement_type >= 28) return 0;
  return entry >= 2 || sm_native_player_motion_admitted(entry, amount);
}
void sm_native_set_player_motion_hook(MotionHook fn, void *ctx) {
  if (active_root || observing || SiblingActive() || cur_coll_amt32) Die("changing player-motion hook during invocation");
  hook = fn; context = ctx;
}
void sm_native_set_player_motion_observer(MotionObserver fn, void *ctx) {
  if (active_root || observing || SiblingActive() || cur_coll_amt32) Die("changing player-motion observer during invocation");
  observer = fn; observer_context = ctx;
}
static void Observe(unsigned phase, int32_t amount, int32_t result) {
  if (!active_root || observing || SiblingActive()) Die("invalid player-motion observation");
  sm_native_effect(56 + phase, active_root - 1, (uint32_t)amount, (uint32_t)result);
  if (!observer) return;
  observing = true;
  int status = observer(observer_context, phase, active_root - 1, amount, result, g_ram);
  observing = false;
  if (status) Die("player-motion observer failed; evidence incomplete");
}
int sm_native_player_motion_bind(const uint8_t *ram, size_t size, uint64_t token) {
  if (!active_root || !callback_active || observing || service_active || issued_ram || !ram || size != sizeof(g_ram) || token != generation) {
    poisoned = true; return -1;
  }
  issued_ram = ram;
  return 0;
}
void sm_native_player_motion_commit(const uint8_t *ram) {
  if (!active_root || !callback_active || service_active || terrain_active || observing || SiblingActive() ||
      !issued_ram || ram != issued_ram || cur_coll_amt32 || poisoned || committed)
    Die("invalid player-motion transaction commit");
  memcpy(g_ram, ram, sizeof(g_ram));
  committed = true;
}
int sm_native_player_motion_dispatch(unsigned entry, int32_t amount) {
  if (entry >= 2 || active_root || callback_active || service_active || terrain_active || observing || SiblingActive() || cur_coll_amt32)
    Die("recursive or cross-consumer player-motion root");
  if (entry == 0 && amount == INT32_MIN) Die("undefined player-motion negation");
  active_root = entry + 1; root_amount = amount; generation++;
  terrain_calls = 0; issued_ram = NULL; poisoned = committed = rust_owned = false;
  admitted = TerrainAdmitted(amount);
  counts[entry]++;
  if (!admitted) counts[10 + entry]++;
  Observe(0, amount, 0);
  if (hook) {
    counts[4 + entry]++;
    callback_active = true;
    int status = hook(context, entry, amount);
    callback_active = false;
    if (poisoned) Die("player-motion poisoned request; C retry prohibited");
    if (status == 1) {
      if (!committed) Die("player-motion handled without transaction commit");
      rust_owned = true; counts[8 + entry]++; return 1;
    }
    if (status != 0 || terrain_calls || committed) Die("Rust player-motion owned failure; C retry prohibited");
    counts[6 + entry]++;
  }
  counts[2 + entry]++;
  return 0;
}
void sm_native_player_motion_done(unsigned entry, int32_t amount) {
  if (active_root != entry + 1 || root_amount != amount || callback_active || service_active || terrain_active || observing || SiblingActive() || cur_coll_amt32)
    Die("player-motion root completion mismatch");
  if (rust_owned) {
    if (terrain_calls == 1) counts[17 + entry]++;
    else if (!terrain_calls && samus_collision_flag == 0xffff) { counts[15 + entry]++; counts[21 + entry]++; }
    else Die("player-motion handled without a completed enemy or terrain branch");
    counts[19 + entry]++;
  }
  unsigned at = entry ? 0xdb6 : 0xdb2;
  int32_t result = (uint32_t)Word(g_ram, at) << 16 | Word(g_ram, at + 2);
  Observe(1, amount, result);
  active_root = 0; issued_ram = NULL; generation++;
}
int sm_native_player_motion_terrain_begin(int32_t amount) {
  if (!active_root) return 0;
  if (terrain_active || observing || SiblingActive() || cur_coll_amt32 || terrain_calls || g_fail ||
      (callback_active && !service_active)) Die("recursive or unissued player-motion terrain");
  if (!service_active) counts[12]++;
  terrain_active = true; terrain_amount = amount;
  Observe(2, amount, 0);
  return 1;
}
void sm_native_player_motion_terrain_done(int32_t result) {
  if (!active_root || !terrain_active || observing || SiblingActive() || cur_coll_amt32 || g_fail)
    Die("player-motion terrain pointer or completion mismatch");
  counts[13]++; terrain_calls++;
  Observe(3, terrain_amount, result);
  terrain_active = false;
}
static bool InputUnchanged(const uint8_t *ram) {
  // Only original root direction/collision and no-hit checker index may differ
  // before terrain. All geometry, lists, level/BTS and PLM state stay frozen.
  return !memcmp(ram, g_ram, 0xb02) && !memcmp(ram + 0xb04, g_ram + 0xb04, 0xdd0 - 0xb04) &&
    !memcmp(ram + 0xdd2, g_ram + 0xdd2, 0x18a6 - 0xdd2) &&
    !memcmp(ram + 0x18a8, g_ram + 0x18a8, sizeof(g_ram) - 0x18a8);
}
int sm_native_player_motion_terrain(uint8_t *ram, size_t size, unsigned selector, int32_t amount, int32_t *result, uint64_t token) {
  counts[12]++; // Attempt precedes every validation, even a wrong buffer.
  if (!active_root || !callback_active || service_active || terrain_active || observing || SiblingActive() ||
      cur_coll_amt32 || !admitted || !issued_ram || ram != issued_ram || size != sizeof(g_ram) || !result ||
      selector || token != generation || terrain_calls || amount != root_amount ||
      Word(ram, 0xb02) != active_root + 1 || Word(ram, 0xdd0) || !InputUnchanged(ram)) {
    counts[14]++; poisoned = true; return -1;
  }
  service_active = true;
  memcpy(g_ram, ram, sizeof(g_ram));
  *result = Samus_MoveDown_NoSolidColl(amount);
  if (cur_coll_amt32) Die("player-motion terrain escaped collision pointer");
  memcpy(ram, g_ram, sizeof(g_ram));
  service_active = false;
  return 0;
}
void sm_native_invoke_player_motion(unsigned entry, int32_t amount, uint16_t *collision, int32_t *applied) {
  if (!collision || !applied || active_root || SiblingActive() || cur_coll_amt32 ||
      !sm_native_player_motion_fixture_admitted(entry, amount)) Die("unsafe player-motion fixture rejected before C");
  *collision = 0; *applied = 0;
  switch (entry) {
    case 0: Samus_MoveUp(amount); break;
    case 1: Samus_MoveDown(amount); break;
    case 2: { CheckEnemyColl_Result result = Samus_CheckSolidEnemyColl(amount); *collision = result.collision; *applied = result.amt; break; }
    case 3: Samus_MoveUp_NoColl(amount); break;
    case 4: Samus_MoveDown_NoColl(amount); break;
    case 5: Samus_MoveUp_SetPoseCalcInput(); break;
    case 6: Samus_MoveDown_SetPoseCalcInput(); break;
  }
}
