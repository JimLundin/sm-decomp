// Four native-scheduled Ripper initialization/patrol roots and one terrain service.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <string.h>

// Exact pristine/runtime A28187..A2838E binding,65 rows (including endpoint64).
// SHA256 b8c7ce12514576f554227bdf8a4e22d00bee41cd4b7b3aeea5dac2a594ecbcfb.
static const uint8_t linear_table[520] = {
  0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10,
  0xff, 0xff, 0x00, 0xf0, 0x00, 0x00, 0x00, 0x20, 0xff, 0xff, 0x00, 0xe0,
  0x00, 0x00, 0x00, 0x30, 0xff, 0xff, 0x00, 0xd0, 0x00, 0x00, 0x00, 0x40,
  0xff, 0xff, 0x00, 0xc0, 0x00, 0x00, 0x00, 0x50, 0xff, 0xff, 0x00, 0xb0,
  0x00, 0x00, 0x00, 0x60, 0xff, 0xff, 0x00, 0xa0, 0x00, 0x00, 0x00, 0x70,
  0xff, 0xff, 0x00, 0x90, 0x00, 0x00, 0x00, 0x80, 0xff, 0xff, 0x00, 0x80,
  0x00, 0x00, 0x00, 0x90, 0xff, 0xff, 0x00, 0x70, 0x00, 0x00, 0x00, 0xa0,
  0xff, 0xff, 0x00, 0x60, 0x00, 0x00, 0x00, 0xb0, 0xff, 0xff, 0x00, 0x50,
  0x00, 0x00, 0x00, 0xc0, 0xff, 0xff, 0x00, 0x40, 0x00, 0x00, 0x00, 0xd0,
  0xff, 0xff, 0x00, 0x30, 0x00, 0x00, 0x00, 0xe0, 0xff, 0xff, 0x00, 0x20,
  0x00, 0x00, 0x00, 0xf0, 0xff, 0xff, 0x00, 0x10, 0x01, 0x00, 0x00, 0x00,
  0xff, 0xff, 0x00, 0x00, 0x01, 0x00, 0x00, 0x10, 0xfe, 0xff, 0x00, 0xf0,
  0x01, 0x00, 0x00, 0x20, 0xfe, 0xff, 0x00, 0xe0, 0x01, 0x00, 0x00, 0x30,
  0xfe, 0xff, 0x00, 0xd0, 0x01, 0x00, 0x00, 0x40, 0xfe, 0xff, 0x00, 0xc0,
  0x01, 0x00, 0x00, 0x50, 0xfe, 0xff, 0x00, 0xb0, 0x01, 0x00, 0x00, 0x60,
  0xfe, 0xff, 0x00, 0xa0, 0x01, 0x00, 0x00, 0x70, 0xfe, 0xff, 0x00, 0x90,
  0x01, 0x00, 0x00, 0x80, 0xfe, 0xff, 0x00, 0x80, 0x01, 0x00, 0x00, 0x90,
  0xfe, 0xff, 0x00, 0x70, 0x01, 0x00, 0x00, 0xa0, 0xfe, 0xff, 0x00, 0x60,
  0x01, 0x00, 0x00, 0xb0, 0xfe, 0xff, 0x00, 0x50, 0x01, 0x00, 0x00, 0xc0,
  0xfe, 0xff, 0x00, 0x40, 0x01, 0x00, 0x00, 0xd0, 0xfe, 0xff, 0x00, 0x30,
  0x01, 0x00, 0x00, 0xe0, 0xfe, 0xff, 0x00, 0x20, 0x01, 0x00, 0x00, 0xf0,
  0xfe, 0xff, 0x00, 0x10, 0x02, 0x00, 0x00, 0x00, 0xfe, 0xff, 0x00, 0x00,
  0x02, 0x00, 0x00, 0x10, 0xfd, 0xff, 0x00, 0xf0, 0x02, 0x00, 0x00, 0x20,
  0xfd, 0xff, 0x00, 0xe0, 0x02, 0x00, 0x00, 0x30, 0xfd, 0xff, 0x00, 0xd0,
  0x02, 0x00, 0x00, 0x40, 0xfd, 0xff, 0x00, 0xc0, 0x02, 0x00, 0x00, 0x50,
  0xfd, 0xff, 0x00, 0xb0, 0x02, 0x00, 0x00, 0x60, 0xfd, 0xff, 0x00, 0xa0,
  0x02, 0x00, 0x00, 0x70, 0xfd, 0xff, 0x00, 0x90, 0x02, 0x00, 0x00, 0x80,
  0xfd, 0xff, 0x00, 0x80, 0x02, 0x00, 0x00, 0x90, 0xfd, 0xff, 0x00, 0x70,
  0x02, 0x00, 0x00, 0xa0, 0xfd, 0xff, 0x00, 0x60, 0x02, 0x00, 0x00, 0xb0,
  0xfd, 0xff, 0x00, 0x50, 0x02, 0x00, 0x00, 0xc0, 0xfd, 0xff, 0x00, 0x40,
  0x02, 0x00, 0x00, 0xd0, 0xfd, 0xff, 0x00, 0x30, 0x02, 0x00, 0x00, 0xe0,
  0xfd, 0xff, 0x00, 0x20, 0x02, 0x00, 0x00, 0xf0, 0xfd, 0xff, 0x00, 0x10,
  0x03, 0x00, 0x00, 0x00, 0xfd, 0xff, 0x00, 0x00, 0x03, 0x00, 0x00, 0x10,
  0xfc, 0xff, 0x00, 0xf0, 0x03, 0x00, 0x00, 0x20, 0xfc, 0xff, 0x00, 0xe0,
  0x03, 0x00, 0x00, 0x30, 0xfc, 0xff, 0x00, 0xd0, 0x03, 0x00, 0x00, 0x40,
  0xfc, 0xff, 0x00, 0xc0, 0x03, 0x00, 0x00, 0x50, 0xfc, 0xff, 0x00, 0xb0,
  0x03, 0x00, 0x00, 0x60, 0xfc, 0xff, 0x00, 0xa0, 0x03, 0x00, 0x00, 0x70,
  0xfc, 0xff, 0x00, 0x90, 0x03, 0x00, 0x00, 0x80, 0xfc, 0xff, 0x00, 0x80,
  0x03, 0x00, 0x00, 0x90, 0xfc, 0xff, 0x00, 0x70, 0x03, 0x00, 0x00, 0xa0,
  0xfc, 0xff, 0x00, 0x60, 0x03, 0x00, 0x00, 0xb0, 0xfc, 0xff, 0x00, 0x50,
  0x03, 0x00, 0x00, 0xc0, 0xfc, 0xff, 0x00, 0x40, 0x03, 0x00, 0x00, 0xd0,
  0xfc, 0xff, 0x00, 0x30, 0x03, 0x00, 0x00, 0xe0, 0xfc, 0xff, 0x00, 0x20,
  0x03, 0x00, 0x00, 0xf0, 0xfc, 0xff, 0x00, 0x10, 0x04, 0x00, 0x00, 0x00,
  0xfc, 0xff, 0x00, 0x00
};
typedef int (*RipperHook)(void *, unsigned);
typedef int (*RipperObserver)(void *, unsigned, unsigned, uint16_t, int32_t, uint8_t, const uint8_t *);
static RipperHook hook;
static RipperObserver observer;
static void *context, *observer_context;
static unsigned active_root, service_calls;
static uint16_t root_slot;
static int32_t root_amount;
static uint8_t collision;
static bool callback_active, service_active, movement_active, observing, admitted, poisoned, committed;
static const uint8_t *issued_ram;
static uint64_t generation, counts[27];
static bool SiblingActive(void) {
  return sm_native_projectile_active() || sm_native_scroll_active() || sm_native_ceres_door_active() ||
    sm_native_room_transition_active() || sm_native_door_scrolling_active() || sm_native_camera_active() ||
    sm_native_item_instruction_active() || sm_native_persistence_active() || sm_native_player_motion_active() || sm_native_waver_active();
}
unsigned sm_native_ripper_active(void) { return active_root; }
uint64_t sm_native_ripper_generation(void) { return generation; }
void sm_native_ripper_counts(uint64_t *out) { memcpy(out, counts, sizeof(counts)); }
static uint16_t Word(const uint8_t *ram, unsigned at) { return ram[at] | (uint16_t)ram[at + 1] << 8; }
static bool Slot(unsigned k) { return k < 0x800 && !(k & 63); }
static bool Main(unsigned entry) { return entry == 1 || entry == 3; }
static unsigned Record(void) { return 0xf78 + cur_enemy_index; }
static int32_t Amount(void) { unsigned at = Record(); return (uint32_t)Word(g_ram, at + 0x36) << 16 | Word(g_ram, at + 0x34); }
static bool PreparedTables(void) {
  static const uint8_t table[32] = { [30] = 0x94, [31] = 0xd0 };
  static const uint8_t header[4] = {0xd4,0xb3,0x53,0xcd};
  return !memcmp(RomFixedPtr(0xa28187), linear_table, sizeof(linear_table)) &&
    !memcmp(RomFixedPtr(0xa0c2da), table, sizeof(table)) && !memcmp(RomFixedPtr(0x84d094), header, sizeof(header));
}
int sm_native_ripper_prepared_tables(void) { return PreparedTables(); }
static bool RowAdmitted(unsigned entry) {
  if (!Slot(cur_enemy_index)) return false;
  unsigned row = Main(entry) ? Word(g_ram, Record() + 0x38) : (uint16_t)(8 * Word(g_ram, Record() + 0x3c));
  return row <= 0x200 && !(row & 7);
}
// The same finite terrain facts as Waver: one hop/zero extension, concrete
// terminal reactions, and spike0..15 with the actual D094 clearing setup.
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
static bool TerrainAdmitted(void) {
  if (!Slot(cur_enemy_index) || !PreparedTables()) return false;
  unsigned width = room_width_in_blocks, height = room_height_in_blocks, at = Record();
  unsigned x = Word(g_ram,at+2), y = Word(g_ram,at+6), xr = Word(g_ram,at+10), yr = Word(g_ram,at+12);
  int32_t amount = Amount();
  if (!width || width > 255 || !height || height > 255 || width * height > 0x3200 ||
      !xr || xr > 16 || !yr || yr > 16 || amount < -0x100000 || amount > 0x100000 ||
      x < xr + 16 || y < yr + 4 || x + xr + 16 > width * 16 || y + yr + 4 > height * 16) return false;
  unsigned left = (x-xr-16)>>4, right = (x+xr+16-1)>>4, top = (y-yr-4)>>4, bottom = (y+yr+4-1)>>4;
  if (bottom > 255) return false;
  for (unsigned row=top; row<=bottom; row++) for (unsigned col=left; col<=right; col++)
    if (!Cell(row*width+col,width*height)) return false;
  return true;
}
int sm_native_ripper_fixture_admitted(unsigned entry) {
  return entry < 4 && PreparedTables() && RowAdmitted(entry) && (!Main(entry) || TerrainAdmitted());
}
int sm_native_ripper_available(void) { return active_root && Main(active_root-1) && callback_active && admitted; }
void sm_native_set_ripper_hook(RipperHook fn, void *ctx) {
  if (active_root || observing || SiblingActive() || cur_coll_amt32) Die("changing Ripper hook during invocation");
  hook=fn;context=ctx;
}
void sm_native_set_ripper_observer(RipperObserver fn, void *ctx) {
  if (active_root || observing || SiblingActive() || cur_coll_amt32) Die("changing Ripper observer during invocation");
  observer=fn;observer_context=ctx;
}
static void Observe(unsigned phase, uint8_t result) {
  if (!active_root || observing || SiblingActive()) Die("invalid Ripper observation");
  sm_native_effect(64+phase,(active_root-1)<<16|root_slot,(uint32_t)root_amount,result);
  if (!observer) return;
  observing=true;
  int status=observer(observer_context,phase,active_root-1,root_slot,root_amount,result,g_ram);
  observing=false;
  if (status) Die("Ripper observer failed; evidence incomplete");
}
int sm_native_ripper_bind(const uint8_t *ram,size_t size,uint64_t token) {
  if (!active_root || !callback_active || service_active || movement_active || observing || issued_ram || !ram || size!=sizeof(g_ram) || token!=generation) {
    poisoned=true;return -1;
  }
  issued_ram=ram;return 0;
}
void sm_native_ripper_commit(const uint8_t *ram) {
  if (!active_root || !callback_active || service_active || movement_active || observing || SiblingActive() ||
      !issued_ram || ram!=issued_ram || cur_coll_amt32 || poisoned || committed) Die("invalid Ripper transaction commit");
  memcpy(g_ram,ram,sizeof(g_ram));committed=true;
}
int sm_native_ripper_movement_begin(uint16_t slot,int32_t amount) {
  if (!active_root) return 0; // Preserve Waver's passive wrapper without crosscounts.
  if (!Main(active_root-1) || movement_active || observing || SiblingActive() || cur_coll_amt32 || g_fail ||
      service_calls || (callback_active && !service_active) || slot!=root_slot || amount!=root_amount)
    Die("recursive or unissued Ripper service");
  if (!service_active) counts[24]++;
  movement_active=true;Observe(2,0);return 1;
}
void sm_native_ripper_movement_done(uint8_t result) {
  if (!movement_active || observing || SiblingActive() || cur_coll_amt32 || g_fail || result>1)
    Die("Ripper invalid collision or service completion");
  collision=result;service_calls++;counts[25]++;Observe(3,result);movement_active=false;
}
int sm_native_ripper_service(uint8_t *ram,size_t size,unsigned selector,uint16_t slot,int32_t amount,uint8_t *result,uint64_t token) {
  counts[24]++;
  if (!active_root || !callback_active || service_active || movement_active || observing || SiblingActive() ||
      cur_coll_amt32 || !admitted || !issued_ram || ram!=issued_ram || size!=sizeof(g_ram) || !result ||
      token!=generation || selector || service_calls || slot!=root_slot || amount!=root_amount ||
      memcmp(ram,g_ram,sizeof(g_ram))) {counts[26]++;poisoned=true;return -1;}
  service_active=true;memcpy(g_ram,ram,sizeof(g_ram));
  *result=Enemy_MoveRight_IgnoreSlopes(slot,amount);
  if (cur_coll_amt32 || g_fail) Die("Ripper native service failure");
  memcpy(ram,g_ram,sizeof(g_ram));service_active=false;return 0;
}
void sm_native_ripper_ai(unsigned entry) {
  if (entry>=4 || active_root || callback_active || service_active || movement_active || observing || SiblingActive() || cur_coll_amt32)
    Die("recursive or cross-consumer Ripper root");
  if (!PreparedTables()) Die("Ripper runtime table identity mismatch");
  active_root=entry+1;root_slot=cur_enemy_index;root_amount=Main(entry)&&Slot(root_slot)?Amount():0;generation++;
  service_calls=collision=0;issued_ram=NULL;poisoned=committed=false;
  admitted=Main(entry)&&RowAdmitted(entry)&&TerrainAdmitted();
  counts[entry]++;if(Main(entry)&&!admitted)counts[20+entry]++;
  Observe(0,0);int status=0;
  if(hook){
    counts[8+entry]++;callback_active=true;status=hook(context,entry);callback_active=false;
    if(poisoned)Die("Ripper poisoned request; C retry prohibited");
    if(status==1){
      if(!committed || service_calls!=(Main(entry)?1u:0u))Die("Ripper incomplete owned invocation");
      counts[16+entry]++;
    }else{
      if(status || committed || service_calls)Die("Rust Ripper owned failure; C retry prohibited");
      counts[12+entry]++;
    }
  }
  if(status!=1){
    counts[4+entry]++;
    switch(entry){case 0:JetPowerRipper_Init();break;case 1:JetPowerRipper_Main();break;case 2:Ripper_Init();break;case 3:Ripper_Main();break;}
  }
  if(callback_active || service_active || movement_active || observing || SiblingActive() || cur_coll_amt32 || g_fail)
    Die("Ripper root completion mismatch");
  Observe(1,collision);active_root=0;issued_ram=NULL;generation++;
}
void sm_native_invoke_ripper(unsigned entry) {
  if(active_root || SiblingActive() || !sm_native_ripper_fixture_admitted(entry))Die("unsafe Ripper fixture rejected before C");
  static const uint32_t addresses[4]={0xa2e318,0xa2e353,0xa2e49f,0xa2e4da};
  CallEnemyAi(addresses[entry]);
}
