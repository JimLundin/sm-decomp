// Scheduled room scrolling boundary and the real synchronous VRAM queue service.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <string.h>

typedef int (*ScrollHook)(void *, unsigned);
static ScrollHook scroll_hook;
static void *scroll_context;
static unsigned active_scroll;
static bool callback_active;
static bool flush_active;
static unsigned flush_owner;
static unsigned flush_mask;
static uint64_t flush_bytes;
// Layout is mirrored by scroll_hooks::counters. Ownership counters are never
// serialized as effects or emulator state; both implementations emit effects.
static uint64_t scroll_counts[42];

unsigned sm_native_scroll_active(void) { return active_scroll; }
void sm_native_set_scroll_hook(ScrollHook hook, void *context) {
  if (active_scroll || sm_native_projectile_active() || sm_native_ceres_door_active() || sm_native_room_transition_active() || sm_native_door_scrolling_active() || sm_native_camera_active() || sm_native_item_instruction_active() || flush_active)
    Die("changing scroll hook during an invocation");
  scroll_hook = hook;
  scroll_context = context;
}
void sm_native_scroll_counts(uint64_t *out) {
  memcpy(out, scroll_counts, sizeof(scroll_counts));
}
int sm_native_scroll_dispatch(unsigned entry) {
  if (entry >= 3 || active_scroll || sm_native_projectile_active() || sm_native_ceres_door_active() || sm_native_camera_active() || sm_native_item_instruction_active() || flush_active)
    Die("invalid or nested scroll entry");
  if (sm_native_door_scrolling_active()) sm_native_door_scrolling_child_begin(entry);
  else sm_native_room_transition_child_begin(1, entry);
  active_scroll = entry + 1;
  sm_native_effect(10, entry, 0, 0);
  if (!scroll_hook) {
    scroll_counts[entry]++;
    return 0;
  }
  scroll_counts[3 + entry]++;
  if (sm_native_door_scrolling_active()) sm_native_door_scrolling_child_rust(0);
  else sm_native_room_transition_child_rust(1, 0);
  callback_active = true;
  int result = scroll_hook(scroll_context, entry);
  callback_active = false;
  if (result == 1) {
    scroll_counts[6 + entry]++;
    if (sm_native_door_scrolling_active()) sm_native_door_scrolling_child_rust(1);
    else sm_native_room_transition_child_rust(1, 1);
    return 1;
  }
  if (result == 0) {
    scroll_counts[9 + entry]++;
    scroll_counts[entry]++;
    return 0;
  }
  Die("Rust scroll owned failure; C retry prohibited");
  return 1;
}
void sm_native_scroll_entry_done(unsigned entry) {
  if (active_scroll != entry + 1 || callback_active || flush_active)
    Die("scroll entry completion mismatch");
  sm_native_effect(11, entry, 0, 0);
  active_scroll = 0;
  if (sm_native_door_scrolling_active()) sm_native_door_scrolling_child_done(entry);
  else sm_native_room_transition_child_done(1, entry);
}

static unsigned ScrollEnableMask(void) {
  return (!!(uint8)bg1_update_col_enable) | (!!(uint8)bg2_update_col_enable << 1)
       | (!!(uint8)bg1_update_row_enable << 2) | (!!(uint8)bg2_update_row_enable << 3);
}
void sm_native_vram_flush_begin(void) {
  if (flush_active) Die("nested native VRAM queue flush");
  flush_active = true;
  flush_owner = active_scroll;
  flush_mask = ScrollEnableMask();
  flush_bytes = 0;
  scroll_counts[35]++;
  if (flush_owner) scroll_counts[14 + flush_owner - 1]++;
  // Owner 0 means ordinary NMI or another C caller, 1..3 scheduled scroll.
  sm_native_effect(12, flush_owner, vram_write_queue_tail, flush_mask);
}
void sm_native_vram_dma_byte(unsigned b_address) {
  if (flush_active && (b_address == 0x18 || b_address == 0x19)) flush_bytes++;
}
void sm_native_vram_flush_done(void) {
  if (!flush_active || vram_write_queue_tail || ScrollEnableMask())
    Die("native VRAM flush completion mismatch");
  scroll_counts[36]++;
  scroll_counts[41] += flush_bytes;
  if (flush_owner) {
    scroll_counts[17 + flush_owner - 1]++;
    scroll_counts[32 + flush_owner - 1] += flush_bytes;
  }
  for (unsigned layer = 0; layer < 2; layer++) {
    if (flush_mask & (1u << layer)) {
      scroll_counts[37 + layer]++;
      if (flush_owner) scroll_counts[20 + (flush_owner - 1) * 2 + layer]++;
    }
    if (flush_mask & (4u << layer)) {
      scroll_counts[39 + layer]++;
      if (flush_owner) scroll_counts[26 + (flush_owner - 1) * 2 + layer]++;
    }
  }
  // Low/high words preserve the exact byte count without truncating events.
  sm_native_effect(13, flush_owner, (uint32_t)flush_bytes, (uint32_t)(flush_bytes >> 32));
  flush_active = false;
}
int sm_native_flush_vram(uint8_t *ram) {
  scroll_counts[12]++; // A request is evidence even if context validation fails.
  if (!active_scroll || !callback_active || flush_active || sm_native_projectile_active() || sm_native_ceres_door_active())
    return -1;
  memcpy(g_ram, ram, sizeof(g_ram));
  NMI_ProcessVramWriteQueue();
  memcpy(ram, g_ram, sizeof(g_ram));
  scroll_counts[13]++;
  return 0;
}
