// Scheduled bank84 instructions and synchronous services. Bodies stay in sm_84.c.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <string.h>

// These exported sm_84.c bodies are absent from the pinned funcs.h prototypes.
// Keep their exact source signatures; the originals remain the C reference.
const uint8 *PlmInstr_Delete(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_PreInstr(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_ClearPreInstr(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_Goto(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_SetTimer(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_GotoIfItemBitSet(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_SetItemBit(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_CollectAmmoMissileTank(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_SetLinkReg(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_Call(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_Return(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_ClearMusicQueueAndQueueTrack(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_LoadItemPlmGfx(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_DrawItemFrame0(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_DrawItemFrame1(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_PickupEquipmentAndShowMessage(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_CollectHealthEnergyTank(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_CollectHealthReserveTank(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_CollectAmmoSuperMissileTank(const uint8 *plmp, uint16 k);
const uint8 *PlmInstr_CollectAmmoPowerBombTank(const uint8 *plmp, uint16 k);
void PlmPreInstr_GotoLinkIfTriggered(uint16 k);

typedef int (*ItemHook)(void *, unsigned, uint16_t, uint16_t, uint32_t *);
typedef int (*ItemObserver)(void *, unsigned, unsigned, uint16_t, uint32_t, unsigned, unsigned, const uint8_t *);
static ItemHook item_hook;
static void *item_context;
static ItemObserver observer;
static void *observer_context;
static unsigned active_item, active_context, active_scope, active_group;
static uint16_t active_slot;
static bool callback_active, service_active, observing;
static uint64_t counts[961];

static bool SiblingActive(void) {
  return sm_native_projectile_active() || sm_native_scroll_active() ||
      sm_native_ceres_door_active() || sm_native_door_scrolling_active() || sm_native_camera_active();
}
unsigned sm_native_item_instruction_active(void) { return active_item; }
void sm_native_set_item_instruction_hook(ItemHook hook, void *context) {
  if (active_item || service_active || observing || SiblingActive() || sm_native_room_transition_active())
    Die("changing item hook during an invocation");
  item_hook = hook;
  item_context = context;
}
void sm_native_set_item_instruction_observer(ItemObserver hook, void *context) {
  if (active_item || service_active || observing || SiblingActive() || sm_native_room_transition_active())
    Die("changing item observer during an invocation");
  observer = hook;
  observer_context = context;
}
void sm_native_item_instruction_counts(uint64_t *out) { memcpy(out, counts, sizeof(counts)); }
static void Observe(unsigned phase, uint32_t value) {
  if (!observer) return;
  if (observing) Die("recursive item observer");
  observing = true;
  int result = observer(observer_context, phase, active_item ? active_item - 1 : UINT32_MAX,
      active_item ? active_slot : 0, value, active_item ? active_context : 0,
      active_item ? active_scope : 0, g_ram);
  observing = false;
  if (result) Die("item observer failed; evidence incomplete");
}
static uint16_t Offset(const uint8_t *pointer) {
  uintptr_t value = (uintptr_t)pointer, start = (uintptr_t)RomPtr_84(0x8000);
  if (value < start || value >= start + 0x8000) Die("item operand outside bank84 ROM");
  return (uint16_t)(value - start + 0x8000);
}
static uint32_t Output(const uint8_t *pointer) {
  uintptr_t value = (uintptr_t)pointer;
  if (!value) return 1u << 16;
  if (value < 0x10000) {
    if (value < 0x8000) Die("item returned invalid small pointer");
    return 2u << 16 | (uint32_t)value;
  }
  return 2u << 16 | Offset(pointer);
}
static void Count(unsigned kind, unsigned entry) {
  counts[kind * 21 + entry]++;
  counts[active_group + kind * 21 + entry]++;
}
static int Begin(unsigned entry, uint16_t slot, uint16_t operand, uint32_t *output) {
  if (entry >= 21 || slot >= 80 || (slot & 1) || active_item || service_active || observing || SiblingActive())
    Die("invalid or nested item entry");
  unsigned context = sm_native_room_transition_item_context();
  if (context > 1) Die("item entry outside room-transition service52");
  active_context = context;
  active_scope = plm_header_ptr[slot >> 1] == 0xef2f && plm_room_arguments[slot >> 1] == 0x22;
  if (plm_header_ptr[slot >> 1] == 0xef23 && plm_room_arguments[slot >> 1] == 0x1a) active_scope = 2;
  if (plm_room_arguments[slot >> 1] <= 0x1ff) {
    switch (plm_header_ptr[slot >> 1]) {
      case 0xeed7: case 0xeedb: case 0xeedf: case 0xeee3: case 0xef27: active_scope = 3; break;
    }
  }
  active_group = 105 * ((active_scope == 1 ? 1 : active_scope == 2 ? 5 : active_scope == 3 ? 7 : 3) + context);
  active_slot = slot;
  active_item = entry + 1;
  Count(0, entry);
  sm_native_effect(51, active_scope, context, 0);
  sm_native_effect(47, entry, slot, operand);
  Observe(0, operand);
  if (!item_hook) { Count(1, entry); return 0; }
  Count(2, entry);
  callback_active = true;
  int result = item_hook(item_context, entry, slot, operand, output);
  callback_active = false;
  if (result == 1) { Count(3, entry); return 1; }
  if (!result) { Count(4, entry); Count(1, entry); return 0; }
  Die("Rust item instruction owned failure; C retry prohibited");
  return 1;
}
static void Done(unsigned entry, uint16_t slot, uint32_t output) {
  if (active_item != entry + 1 || active_slot != slot || callback_active || service_active || observing)
    Die("item completion context mismatch");
  if (entry == 12 ? output != 0 : !(output == (1u << 16) || ((output >> 16) == 2 && (output & 0xffff) >= 0x8000)))
    Die("invalid item completion tag or offset");
  Observe(1, output);
  sm_native_effect(48, entry, slot, output);
  active_item = 0;
}
const uint8_t *sm_native_item_instruction(unsigned entry, const uint8_t *operand, uint16_t slot) {
  typedef const uint8 *(*Instruction)(const uint8 *, uint16);
  static const Instruction originals[21] = { PlmInstr_Delete, PlmInstr_PreInstr,
    PlmInstr_ClearPreInstr, PlmInstr_Goto, PlmInstr_SetTimer, PlmInstr_GotoIfItemBitSet,
    PlmInstr_SetItemBit, PlmInstr_CollectAmmoMissileTank, PlmInstr_SetLinkReg,
    PlmInstr_Call, PlmInstr_Return, PlmInstr_ClearMusicQueueAndQueueTrack, NULL,
    PlmInstr_LoadItemPlmGfx, PlmInstr_DrawItemFrame0, PlmInstr_DrawItemFrame1,
    PlmInstr_PickupEquipmentAndShowMessage, PlmInstr_CollectHealthEnergyTank,
    PlmInstr_CollectHealthReserveTank, PlmInstr_CollectAmmoSuperMissileTank,
    PlmInstr_CollectAmmoPowerBombTank };
  if (entry >= 21 || entry == 12) Die("invalid item instruction catalog entry");
  uint16_t input = Offset(operand);
  uint32_t output = 0;
  if (!Begin(entry, slot, input, &output)) output = Output(originals[entry](operand, slot));
  Done(entry, slot, output);
  return output == (1u << 16) ? NULL : INSTRB_RETURN_ADDR(output & 0xffff);
}
void sm_native_item_preinstruction(uint16_t slot) {
  uint32_t output = 0;
  if (!Begin(12, slot, 0, &output)) PlmPreInstr_GotoLinkIfTriggered(slot);
  Done(12, slot, output);
}

// Original selected-body call sites and Rust services share passive observation.
// Unrelated original-body users remain legal and do not become Rust requests.
void sm_native_item_instruction_native_service(unsigned id, uint16_t argument) {
  if (id >= 8 || service_active || observing) Die("invalid or nested item native service");
  service_active = true;
  sm_native_effect(49, id, argument, 0);
  Observe(2, id << 16 | argument);
  switch (id) {
    case 0: AddMissilesToHudTilemap(); break;
    case 1: PlayRoomMusicTrackAfterAFrames(argument); break;
    case 2: QueueMusic_Delayed8(argument); break;
    case 3: ProcessPlmDrawInstruction(argument); break;
    case 4: CalculatePlmBlockCoords(argument); break;
    case 5: DrawPlm(argument); break;
    case 6: AddSuperMissilesToHudTilemap(); break;
    case 7: AddPowerBombsToHudTilemap(); break;
  }
  Observe(3, id << 16 | argument);
  sm_native_effect(50, id, argument, 0);
  service_active = false;
}
static uint16_t RamWord(const uint8_t *ram, unsigned at) { return ram[at] | (uint16_t)ram[at + 1] << 8; }
static bool SafeDrawService(const uint8_t *ram, unsigned id, uint16_t slot) {
  if (slot >= 80 || (slot & 1)) return false;
  if (id == 3) {
    uint16_t block = RamWord(ram, 0x1c87 + slot);
    if (slot != active_slot || (block & 1) || block > 0xfffc) return false;
  } else if (id == 4 && slot != RamWord(ram, 0x1c27)) return false;
  if (id == 4) {
    uint16_t width = RamWord(ram, 0x7a5);
    return width != 0 && width <= 255;
  }
  uint16_t pointer = RamWord(ram, 0xde6c + slot);
  if (pointer < 0xa30f || pointer > 0xa339 || (pointer - 0xa30f) % 6) return false;
  if (id == 5) {
    uint16_t cursor = RamWord(ram, 0x1c25);
    if (cursor > 512 || cursor % 8 || RamWord(ram, 0x330) > 0x260) return false;
  }
  return true;
}
int sm_native_item_instruction_service(uint8_t *ram, unsigned id, uint16_t argument) {
  if (id < 8) counts[945 + id]++;
  if (!active_item || !callback_active || service_active || observing || SiblingActive() || !ram) return -1;
  if (!((active_item == 8 && id == 0 && argument == 0) ||
      ((active_item == 8 || (active_item >= 17 && active_item <= 21)) && id == 1 && argument == 0x168) ||
      (active_item == 12 && id == 2 && argument == 2) ||
      ((active_item == 15 || active_item == 16) && id >= 3 && id < 6 && SafeDrawService(ram, id, argument)) ||
      (active_item == 20 && id == 6 && argument == 0) || (active_item == 21 && id == 7 && argument == 0))) return -2;
  memcpy(g_ram, ram, sizeof(g_ram));
  sm_native_item_instruction_native_service(id, argument);
  memcpy(ram, g_ram, sizeof(g_ram));
  counts[953 + id]++;
  return 0;
}
