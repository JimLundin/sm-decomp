// Waver actor interception; native scheduler, terrain and PLM setup stay native.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <string.h>

typedef int (*WaverHook)(void *, unsigned, uint16_t, uint16_t, uint16_t *);
typedef int (*WaverObserver)(void *, unsigned, unsigned, uint16_t, uint16_t, unsigned, int32_t, uint16_t, uint16_t, const uint8_t *);
static WaverHook hook;
static WaverObserver observer;
static void *context, *observer_context;
static unsigned active_root, completed_services, active_service;
static uint16_t root_k, root_jp, request_k, request_auxiliary, horizontal_collision, sine_result;
static int32_t request_argument;
static bool callback_active, service_active, observing, admitted, poisoned, committed;
static const uint8_t *issued_ram;
static uint64_t generation, counts[31];
static bool SiblingActive(void) {
  return sm_native_projectile_active() || sm_native_scroll_active() || sm_native_ceres_door_active() ||
    sm_native_room_transition_active() || sm_native_door_scrolling_active() || sm_native_camera_active() ||
    sm_native_item_instruction_active() || sm_native_persistence_active() || sm_native_player_motion_active();
}
unsigned sm_native_waver_active(void) { return active_root; }
uint64_t sm_native_waver_generation(void) { return generation; }
void sm_native_waver_counts(uint64_t *out) { memcpy(out, counts, sizeof(counts)); }
static uint16_t Word(const uint8_t *ram, unsigned at) { return ram[at] | (uint16_t)ram[at + 1] << 8; }
static bool Slot(unsigned k) { return k < 0x800 && !(k & 63); }
static unsigned Record(void) { return 0xf78 + cur_enemy_index; }
static int32_t HorizontalAmount(void) { unsigned at = Record(); return (uint32_t)Word(g_ram, at + 0x32) << 16 | Word(g_ram, at + 0x30); }
static bool PreparedTables(void) {
  static const uint8_t table[32] = { [30] = 0x94, [31] = 0xd0 };
  static const uint8_t header[4] = { 0xd4, 0xb3, 0x53, 0xcd };
  return !memcmp(RomFixedPtr(0xa0c2da), table, sizeof(table)) && !memcmp(RomFixedPtr(0x84d094), header, sizeof(header));
}
int sm_native_waver_prepared_tables(void) { return PreparedTables(); }
// One extension hop, a zero extension, or a concrete terminal. The only PLM
// terminal is spike index15; its actual D094 setup clears the same cell's type.
// Shared targets remain safe: that mutation can only replace a terminal by air.
static bool Cell(unsigned index, unsigned cells) {
  unsigned kind = level_data[index] >> 12;
  if (kind == 5 || kind == 13) {
    int offset = (int8_t)BTS[index];
    if (!offset) return true;
    int next = (int)index + offset * (kind == 13 ? (int)room_width_in_blocks : 1);
    if (next < 0 || next >= (int)cells) return false;
    index = next; kind = level_data[index] >> 12;
    if (kind == 5 || kind == 13) return false;
  }
  return kind != 10 || (BTS[index] & 0x7f) < 16;
}
// Preflight a rectangle enclosing every horizontal scan and every possible
// subsequent sine-derived vertical scan. No native movement runs during this
// check. Source-valid states outside this finite subset retain the C fallback.
static bool TerrainAdmitted(void) {
  if (!Slot(cur_enemy_index) || !PreparedTables()) return false;
  unsigned width = room_width_in_blocks, height = room_height_in_blocks, at = Record();
  unsigned x = Word(g_ram, at + 2), y = Word(g_ram, at + 6), xr = Word(g_ram, at + 10), yr = Word(g_ram, at + 12);
  int32_t amount = HorizontalAmount();
  if (!width || width > 255 || !height || height > 255 || width * height > 0x3200 ||
      !xr || xr > 16 || !yr || yr > 16 || amount < -0x100000 || amount > 0x100000 ||
      x < xr + 16 || y < yr + 4 || x + xr + 16 > width * 16 || y + yr + 4 > height * 16) return false;
  unsigned left = (x - xr - 16) >> 4, right = (x + xr + 16 - 1) >> 4;
  unsigned top = (y - yr - 4) >> 4, bottom = (y + yr + 4 - 1) >> 4;
  if (bottom > 255) return false; // Native Mult8x8 consumes a byte row.
  for (unsigned row = top; row <= bottom; row++)
    for (unsigned col = left; col <= right; col++)
      if (!Cell(row * width + col, width * height)) return false;
  return true;
}
int sm_native_waver_fixture_admitted(unsigned entry, uint16_t k, uint16_t jp) {
  if (entry >= 4 || !Slot(cur_enemy_index)) return 0;
  unsigned at = Record(), f = Word(g_ram, at + 0x3a), c = Word(g_ram, at + 0x34);
  if (entry == 2) return Slot(k) && jp >= 0x8000 && jp <= 0xfffe;
  // No-argument actor entries ignore fixture k/jp, just like the Rust API.
  if (entry == 0) return 1; // Init overwrites old F before indexing.
  if (entry == 3) return f == c || f < 4; // No-op precedes lookup.
  return f < 4 && TerrainAdmitted();
}
int sm_native_waver_available(void) { return active_root == 2 && callback_active && admitted; }
void sm_native_set_waver_hook(WaverHook fn, void *ctx) {
  if (active_root || observing || SiblingActive() || cur_coll_amt32) Die("changing Waver hook during invocation");
  hook = fn; context = ctx;
}
void sm_native_set_waver_observer(WaverObserver fn, void *ctx) {
  if (active_root || observing || SiblingActive() || cur_coll_amt32) Die("changing Waver observer during invocation");
  observer = fn; observer_context = ctx;
}
static void Observe(unsigned phase, unsigned service, uint16_t k, int32_t argument, uint16_t auxiliary, uint16_t result) {
  if (!active_root || observing || SiblingActive()) Die("invalid Waver observation");
  sm_native_effect(60 + phase, (active_root - 1) << 24 | service << 16 | k, (uint32_t)argument,
    phase == 0 || phase == 2 ? auxiliary : result);
  if (!observer) return;
  observing = true;
  int status = observer(observer_context, phase, active_root - 1, k, root_jp, service, argument, auxiliary, result, g_ram);
  observing = false;
  if (status) Die("Waver observer failed; evidence incomplete");
}
int sm_native_waver_bind(const uint8_t *ram, size_t size, uint64_t token) {
  if (!active_root || !callback_active || service_active || active_service || observing || issued_ram || !ram || size != sizeof(g_ram) || token != generation) {
    poisoned = true; return -1;
  }
  issued_ram = ram; return 0;
}
void sm_native_waver_commit(const uint8_t *ram) {
  if (!active_root || !callback_active || service_active || active_service || observing || SiblingActive() ||
      !issued_ram || ram != issued_ram || cur_coll_amt32 || poisoned || committed) Die("invalid Waver transaction commit");
  memcpy(g_ram, ram, sizeof(g_ram)); committed = true;
}
static bool ServiceArguments(unsigned selector, uint16_t k, int32_t argument, uint16_t auxiliary) {
  if (active_root != 2 || !Slot(cur_enemy_index) || selector != completed_services) return false;
  switch (selector) {
    case 0: return k == cur_enemy_index && !auxiliary && argument == HorizontalAmount();
    case 1: return !horizontal_collision && !k && auxiliary == 4 && argument == (Word(g_ram, Record() + 0x36) & 255);
    case 2: return !horizontal_collision && k == cur_enemy_index && !auxiliary && argument == (int32_t)(int16_t)sine_result * 65536;
    default: return false;
  }
}
int sm_native_waver_service_begin(unsigned selector, uint16_t k, int32_t argument, uint16_t auxiliary) {
  if (!active_root) return 0;
  if (active_service || observing || SiblingActive() || cur_coll_amt32 || g_fail ||
      (callback_active && !service_active) || !ServiceArguments(selector, k, argument, auxiliary)) Die("recursive or unissued Waver service");
  if (!service_active) counts[24 + selector]++;
  active_service = selector + 1; request_k = k; request_argument = argument; request_auxiliary = auxiliary;
  Observe(2, selector, k, argument, auxiliary, 0);
  return 1;
}
void sm_native_waver_service_done(unsigned selector, uint16_t result) {
  if (active_service != selector + 1 || observing || SiblingActive() || cur_coll_amt32 || g_fail ||
      (selector != 1 && result > 1)) Die("Waver service completion mismatch");
  if (selector == 0) horizontal_collision = result;
  if (selector == 1) sine_result = result;
  counts[27 + selector]++; completed_services++;
  Observe(3, selector, request_k, request_argument, request_auxiliary, result);
  active_service = 0;
}
int sm_native_waver_service(uint8_t *ram, size_t size, unsigned selector, uint16_t k, int32_t argument, uint16_t auxiliary, uint16_t *result, uint64_t token) {
  if (selector < 3) counts[24 + selector]++; // All attempts precede validation.
  if (!active_root || !callback_active || service_active || active_service || observing || SiblingActive() ||
      cur_coll_amt32 || !admitted || !issued_ram || ram != issued_ram || size != sizeof(g_ram) || !result ||
      token != generation || selector >= 3 || !ServiceArguments(selector, k, argument, auxiliary) ||
      memcmp(ram, g_ram, sizeof(g_ram))) {
    counts[30]++; poisoned = true; return -1;
  }
  service_active = true;
  memcpy(g_ram, ram, sizeof(g_ram));
  switch (selector) {
    case 0: *result = Enemy_MoveRight_IgnoreSlopes(k, argument); break;
    case 1: *result = SineMult8bit(argument, auxiliary); break;
    case 2: *result = Enemy_MoveDown(k, argument); break;
  }
  if (cur_coll_amt32 || g_fail) Die("Waver native service failure");
  memcpy(ram, g_ram, sizeof(g_ram)); service_active = false; return 0;
}
static uint16_t Run(unsigned entry, uint16_t k, uint16_t jp) {
  if (entry >= 4 || active_root || callback_active || service_active || active_service || observing || SiblingActive() || cur_coll_amt32)
    Die("recursive or cross-consumer Waver root");
  active_root = entry + 1; root_k = k; root_jp = jp; generation++;
  completed_services = horizontal_collision = sine_result = 0;
  issued_ram = NULL; poisoned = committed = false;
  admitted = entry == 1 && TerrainAdmitted();
  counts[entry]++; if (entry == 1 && !admitted) counts[20 + entry]++;
  Observe(0, 3, k, jp, 0, 0);
  uint16_t result = 0; int status = 0;
  if (hook) {
    counts[8 + entry]++; callback_active = true;
    status = hook(context, entry, k, jp, &result);
    callback_active = false;
    if (poisoned) Die("Waver poisoned request; C retry prohibited");
    if (status == 1) {
      if (!committed || result != (entry == 2 ? jp : 0)) Die("invalid Waver committed result");
      if ((entry == 1 && !(completed_services == 3 || (completed_services == 1 && horizontal_collision))) ||
          (entry != 1 && completed_services)) Die("Waver incomplete owned service sequence");
      counts[16 + entry]++;
    } else {
      if (status || committed || completed_services) Die("Rust Waver owned failure; C retry prohibited");
      counts[12 + entry]++;
    }
  }
  if (status != 1) {
    counts[4 + entry]++;
    switch (entry) {
      case 0: Waver_Init(); break;
      case 1: Waver_Main(); break;
      case 2: {
        const uint16_t *pointer = Waver_Instr_1(k, (const uint16_t *)(RomBankBase(0xa3) + jp));
        if ((const uint8_t *)pointer != RomBankBase(0xa3) + jp) Die("Waver original continuation changed");
        result = jp; break;
      }
      case 3: Waver_Func_1(); break;
    }
  }
  if (callback_active || service_active || active_service || observing || SiblingActive() || cur_coll_amt32 || g_fail)
    Die("Waver root completion mismatch");
  Observe(1, 3, root_k, root_jp, 0, result);
  active_root = 0; issued_ram = NULL; generation++; return result;
}
void sm_native_waver_ai(unsigned entry) {
  if (entry > 1) Die("invalid Waver AI entry");
  Run(entry, 0, 0);
}
const uint16_t *sm_native_waver_instruction(uint16_t k, const uint16_t *jp) {
  uintptr_t base = (uintptr_t)RomBankBase(0xa3), pointer = (uintptr_t)jp;
  if (pointer < base + 0x8000 || pointer > base + 0xfffe) Die("invalid Waver instruction pointer");
  return (const uint16_t *)(base + Run(2, k, pointer - base));
}
uint16_t sm_native_invoke_waver(unsigned entry, uint16_t k, uint16_t jp) {
  if (active_root || SiblingActive() || !sm_native_waver_fixture_admitted(entry, k, jp)) Die("unsafe Waver fixture rejected before C");
  switch (entry) {
    case 0: CallEnemyAi(0xa386ed); return 0;
    case 1: CallEnemyAi(0xa3874c); return 0;
    case 2: {
      const uint8_t *base = RomBankBase(0xa3);
      const uint16_t *result = CallEnemyInstr(0xa386e3, k, (const uint16_t *)(base + jp));
      if ((const uint8_t *)result != base + jp) Die("Waver scheduler continuation mismatch");
      return (const uint8_t *)result - base;
    }
    default: return Run(3, 0, 0); // Diagnostic helper entry, never a scheduler slot.
  }
}
