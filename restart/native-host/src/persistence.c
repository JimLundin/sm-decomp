// Dedicated synchronous SRAM codec transport. Original bodies stay in sm_81.c.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <string.h>

typedef int (*PersistenceHook)(void *, unsigned, uint16_t, uint8_t *, uint8_t *, uint64_t, uint32_t *);
typedef int (*PersistenceObserver)(void *, unsigned, unsigned, uint16_t, uint32_t, const uint8_t *, const uint8_t *);
static PersistenceHook hook;
static PersistenceObserver observer;
static void *context, *observer_context;
static unsigned active_entry;
static uint16_t active_argument;
static bool callback_active, native_active, service_active, disk_active, observing, poisoned;
static unsigned disk_calls;
static uint64_t generation, counts[22];
static uint8_t scratch_ram[0x20000], scratch_sram[0x2000];

static bool SiblingActive(void) {
  return sm_native_projectile_active() || sm_native_scroll_active() || sm_native_ceres_door_active() ||
      sm_native_room_transition_active() || sm_native_door_scrolling_active() || sm_native_camera_active() ||
      sm_native_item_instruction_active();
}
unsigned sm_native_persistence_active(void) { return active_entry; }
uint64_t sm_native_persistence_generation(void) { return generation; }
const uint8_t *sm_native_sram(void) { return g_sram; }
void sm_native_persistence_counts(uint64_t *out) { memcpy(out, counts, sizeof(counts)); }
void sm_native_commit_persistence(const uint8_t *ram, const uint8_t *sram) {
  if (!ram || !sram || active_entry || observing || SiblingActive())
    Die("persistence commit outside idle fixture boundary");
  memcpy(g_ram, ram, sizeof(g_ram));
  memcpy(g_sram, sram, 0x2000);
  generation++;
}
void sm_native_set_persistence_hook(PersistenceHook fn, void *ctx) {
  if (active_entry || observing || SiblingActive()) Die("changing persistence hook during invocation");
  hook = fn;
  context = ctx;
}
void sm_native_set_persistence_observer(PersistenceObserver fn, void *ctx) {
  if (active_entry || observing || SiblingActive()) Die("changing persistence observer during invocation");
  observer = fn;
  observer_context = ctx;
}
static void Observe(unsigned phase, uint32_t output) {
  if (!active_entry || observing || SiblingActive()) Die("invalid persistence observation context");
  sm_native_effect(52 + phase, active_entry - 1, active_argument, output);
  if (!observer) return;
  observing = true;
  int status = observer(observer_context, phase, active_entry - 1, active_argument, output, g_ram, g_sram);
  observing = false;
  if (status) Die("persistence observer failed; evidence incomplete");
}
static bool ValidOutput(unsigned entry, uint32_t output) {
  return entry == 1 ? output == 0x10000 || output == 0x10001 : output == 0;
}
int sm_native_persistence_dispatch(unsigned entry, uint16_t argument, uint32_t *output) {
  if (entry >= 4 || !output || active_entry || callback_active || service_active || disk_active || observing || SiblingActive())
    Die("invalid or nested persistence entry");
  // The fourth ROM slot offset is 0x484a, beyond canonical SRAM. Never call C.
  if ((entry < 2 && (argument & 3) == 3) || (entry >= 2 && argument))
    Die("unsafe persistence argument rejected before native invocation");
  active_entry = entry + 1;
  active_argument = argument;
  generation++;
  disk_calls = 0;
  poisoned = false;
  counts[entry]++;
  Observe(0, UINT32_MAX);
  if (hook) {
    counts[8 + entry]++;
    memcpy(scratch_ram, g_ram, sizeof(g_ram));
    memcpy(scratch_sram, g_sram, 0x2000);
    callback_active = true;
    int status = hook(context, entry, argument, scratch_ram, scratch_sram, generation, output);
    callback_active = false;
    if (poisoned || service_active || disk_active || observing)
      Die("persistence invocation poisoned by service or observer failure");
    if (status == 1) {
      if (!ValidOutput(entry, *output) || disk_calls != (entry == 0 ? 1u : 0u))
        Die("invalid persistence handled output or disk completion");
      memcpy(g_ram, scratch_ram, sizeof(g_ram));
      memcpy(g_sram, scratch_sram, 0x2000);
      counts[16 + entry]++;
      return 1;
    }
    if (status != 0) Die("Rust persistence owned failure; C retry prohibited");
    if (disk_calls || memcmp(scratch_ram, g_ram, sizeof(g_ram)) || memcmp(scratch_sram, g_sram, 0x2000))
      Die("persistence dirty refusal or refusal after service; C retry prohibited");
    counts[12 + entry]++;
  }
  // Safe but unowned Save areas can reach the original after an atomic refusal.
  // Actual-C body fixtures only use the frozen areas 0..5.
  counts[4 + entry]++;
  native_active = true;
  return 0;
}
void sm_native_persistence_done(unsigned entry, uint16_t argument, uint32_t output) {
  if (active_entry != entry + 1 || argument != active_argument || callback_active || service_active || disk_active ||
      observing || SiblingActive() || !ValidOutput(entry, output) || disk_calls != (entry == 0 ? 1u : 0u))
    Die("persistence completion mismatch");
  native_active = false;
  Observe(1, output);
  active_entry = 0;
  generation++;
}
int sm_native_persistence_local(unsigned entry) {
  if (!active_entry) return 0;
  if (!native_active || callback_active || service_active || disk_active || observing || SiblingActive() ||
      !((active_entry == 1 && entry == 2) || (active_entry == 2 && entry == 3)))
    Die("recursive persistence helper");
  return 1;
}
int sm_native_persistence_disk_begin(void) {
  if (!active_entry) return 0; // Preserve unrelated original runtime disk calls.
  if (active_entry != 1 || observing || disk_active || SiblingActive() || disk_calls ||
      !(native_active || (callback_active && service_active)))
    Die("unissued or recursive persistence disk write");
  disk_active = true;
  disk_calls++;
  counts[20]++;
  Observe(2, UINT32_MAX);
  return 1;
}
void sm_native_persistence_disk_done(void) {
  if (active_entry != 1 || !disk_active || observing || SiblingActive()) Die("persistence disk completion mismatch");
  counts[21]++;
  Observe(3, UINT32_MAX);
  disk_active = false;
}
int sm_native_persistence_service(uint8_t *ram, uint8_t *sram, uint64_t issued_generation) {
  if (active_entry != 1 || !callback_active || native_active || service_active || disk_active || observing ||
      SiblingActive() || ram != scratch_ram || sram != scratch_sram || issued_generation != generation || disk_calls) {
    poisoned = true;
    return -1;
  }
  service_active = true;
  memcpy(g_ram, ram, sizeof(g_ram));
  memcpy(g_sram, sram, 0x2000);
  RtlWriteSram(); // Retained rename/open/write8192/close, including ignored I/O failures.
  memcpy(ram, g_ram, sizeof(g_ram));
  memcpy(sram, g_sram, 0x2000);
  service_active = false;
  return 0;
}
uint32_t sm_native_invoke_persistence(unsigned entry, uint16_t argument) {
  switch (entry) {
    case 0: SaveToSram(argument); return 0;
    case 1: return 0x10000 | LoadFromSram(argument);
    case 2:
      if (argument) Die("nonzero direct Pack argument");
      PackMapToSave(); return 0;
    case 3:
      if (argument) Die("nonzero direct Unpack argument");
      UnpackMapFromSave(); return 0;
    default: Die("invalid persistence fixture entry"); return 0;
  }
}
