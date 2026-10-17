// Scheduled Ceres actor/solid-setup adapters. Internal actor calls stay private.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <string.h>

typedef int (*CeresDoorHook)(void *, unsigned, uint16_t, uint16_t, uint16_t *);
static CeresDoorHook door_hook;
static void *door_context;
static unsigned active_door;
static uint16_t active_index;
static bool callback_active;
static bool spawn_active;
static uint64_t door_counts[87];

unsigned sm_native_ceres_door_active(void) { return active_door; }
void sm_native_set_ceres_door_hook(CeresDoorHook hook, void *context) {
  if (active_door || sm_native_projectile_active() || sm_native_scroll_active() || sm_native_room_transition_active() || sm_native_door_scrolling_active() || sm_native_camera_active() || sm_native_item_instruction_active())
    Die("changing Ceres door hook during an invocation");
  door_hook = hook;
  door_context = context;
}
void sm_native_ceres_door_counts(uint64_t *out) {
  memcpy(out, door_counts, sizeof(door_counts));
}
static bool IsDoor(uint16_t index) {
  return index < 0x800 && !(index & 0x3f) && *(uint16_t *)(g_ram + 0xf78 + index) == 0xe23f;
}
static unsigned Identity(unsigned entry, uint16_t index) {
  return (entry == 12 ? 0xba48u : 0xe23fu) << 16 | index;
}
static uint16_t InstructionOffset(const uint16_t *pointer) {
  uintptr_t value = (uintptr_t)pointer;
  if (value >= 0x10000) {
    uintptr_t start = (uintptr_t)RomPtr_A6(0x8000);
    if (value < start || value >= start + 0x8000)
      Die("Ceres door returned pointer outside A6 ROM");
    value = value - start + 0x8000;
  }
  if (value < 0x8000 || value & 1)
    Die("Ceres door invalid canonical instruction offset");
  return (uint16_t)value;
}
static int Dispatch(unsigned entry, uint16_t index, uint16_t instruction, uint16_t *output) {
  if (entry >= 13 || active_door || sm_native_projectile_active() || sm_native_scroll_active() || sm_native_door_scrolling_active() || sm_native_camera_active() || sm_native_item_instruction_active())
    Die("invalid or nested Ceres door entry");
  if (entry == 12) {
    if (index >= 80 || index & 1 || plm_header_ptr[index >> 1] != 0xba48)
      Die("Ceres solid setup outside BA48 scope");
  } else if (!IsDoor(index) || (entry < 2 && index != cur_enemy_index)) {
    Die("Ceres actor entry outside E23F scope");
  }
  sm_native_room_transition_child_begin(2, entry);
  active_door = entry + 1;
  active_index = index;
  sm_native_effect(14, entry, Identity(entry, index), instruction);
  if (!door_hook) {
    door_counts[entry]++;
    return 0;
  }
  door_counts[13 + entry]++;
  sm_native_room_transition_child_rust(2, 0);
  callback_active = true;
  int result = door_hook(door_context, entry, index, instruction, output);
  callback_active = false;
  if (result == 1) {
    door_counts[26 + entry]++;
    sm_native_room_transition_child_rust(2, 1);
    return 1;
  }
  if (result == 0) {
    door_counts[39 + entry]++;
    door_counts[entry]++;
    return 0;
  }
  Die("Rust Ceres door owned failure; C retry prohibited");
  return 1;
}
static void Done(unsigned entry, uint16_t index, uint16_t input, uint16_t output) {
  if (active_door != entry + 1 || active_index != index || callback_active || spawn_active)
    Die("Ceres door entry completion mismatch");
  if (entry == 7 && output == (uint16_t)(input + 2)) door_counts[54 + index / 64]++;
  if (entry == 12) {
    uint16_t at = plm_block_indices[index >> 1];
    bool solid = true;
    for (unsigned i = 0; i < 4; i++, at += room_width_in_blocks * 2)
      solid &= (level_data[at >> 1] & 0xf000) == 0x8000;
    if (solid) door_counts[86]++;
  }
  unsigned kind = entry < 2 ? 0 : entry == 12 ? 2 : 1;
  sm_native_effect(15, entry, Identity(entry, index), kind << 16 | output);
  active_door = 0;
  sm_native_room_transition_child_done(2, entry);
}
void sm_native_ceres_door_ai(unsigned entry, uint16_t index) {
  if (entry > 1) Die("invalid Ceres AI entry");
  uint16_t output = 0;
  if (!Dispatch(entry, index, 0, &output)) {
    if (entry == 0) CeresDoor_Init();
    else CeresDoor_Main();
  }
  if (output) Die("invalid Ceres AI output");
  Done(entry, index, 0, output);
}
const uint16_t *sm_native_ceres_door_instr(unsigned entry, uint16_t index, const uint16_t *instruction) {
  typedef const uint16_t *(*Instruction)(uint16_t, const uint16_t *);
  static const Instruction originals[10] = {
    CeresDoor_Instr_1, CeresDoor_Instr_2, CeresDoor_Instr_3, CeresDoor_Instr_4,
    CeresDoor_Instr_5, CeresDoor_Instr_6, CeresDoor_Instr_7, CeresDoor_Instr_8,
    CeresSteam_Instr_4, CeresSteam_Instr_5,
  };
  if (entry < 2 || entry >= 12) Die("invalid Ceres instruction entry");
  // The two property opcodes are shared with steam actors. They are scheduled
  // door roots only for E23F, never manufactured by private actor calls.
  if (entry >= 10 && !IsDoor(index)) return originals[entry - 2](index, instruction);
  uint16_t input = InstructionOffset(instruction), output = 0;
  if (!Dispatch(entry, index, input, &output))
    output = InstructionOffset(originals[entry - 2](index, instruction));
  if (output < 0x8000 || output & 1) Die("invalid Ceres instruction output");
  Done(entry, index, input, output);
  return INSTR_RETURN_ADDR(output);
}
uint8_t sm_native_ceres_door_solid_setup(uint16_t index) {
  // Keep other hypothetical users of the same shared setup outside the claim.
  if (index >= 80 || index & 1) Die("invalid Ceres PLM index");
  if (plm_header_ptr[index >> 1] != 0xba48) return sub_84B9F1(index);
  uint16_t output = 0;
  if (!Dispatch(12, index, 0, &output)) output = sub_84B9F1(index);
  if (output > 0xff) Die("invalid Ceres setup output");
  Done(12, index, 0, output);
  return (uint8_t)output;
}

// Both the private C Func6 call site and the Rust service use this same actual
// operation observer, so ordered behavior effects do not reveal ownership.
void sm_native_ceres_door_spawn(uint16_t header, uint16_t parameter) {
  if (active_door != 2 || spawn_active || header != 0xe509 || (parameter != 3 && parameter != 12))
    Die("Ceres puff spawn outside admitted main invocation");
  spawn_active = true;
  sm_native_effect(16, header, parameter, eproj_spawn_pt.x | (unsigned)eproj_spawn_pt.y << 16);
  SpawnEprojWithRoomGfx(header, parameter);
  sm_native_effect(17, header, parameter, eproj_spawn_pt.x | (unsigned)eproj_spawn_pt.y << 16);
  spawn_active = false;
}
int sm_native_ceres_door_spawn_eproj(uint8_t *ram, uint16_t header, uint16_t parameter, uint16_t x, uint16_t y) {
  door_counts[52]++; // Invalid native requests remain visible to diagnostics.
  if (active_door != 2 || !callback_active || spawn_active || sm_native_projectile_active()
      || sm_native_scroll_active() || header != 0xe509 || (parameter != 3 && parameter != 12))
    return -1;
  memcpy(g_ram, ram, sizeof(g_ram));
  eproj_spawn_pt = (Point16U){x, y};
  sm_native_ceres_door_spawn(header, parameter);
  memcpy(ram, g_ram, sizeof(g_ram));
  door_counts[53]++;
  return 0;
}
