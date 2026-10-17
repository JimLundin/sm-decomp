// Ordinary Rio root interception and synchronous native services.
// Rust validates the complete pre-service and final WRAM contract. This layer
// owns allocation/generation binding, terrain admission and reentry exclusion.
#include "sm_cpu_infra.h"
#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
#include <string.h>
void CallRioFunc(uint32 ea);
typedef int (*RioHook)(void *, unsigned, uint16_t, uint32_t, uint16_t *);
static RioHook hook;
static void *context;
static unsigned active_root, calls, expected_service;
static uint16_t root_slot, state;
static bool callback_active, service_active, observing_service, poisoned, committed, admitted;
static unsigned observed_selector;
static const uint8_t *issued_ram;
static uint64_t generation, counts[67];
static bool SiblingActive(void) {
  return sm_native_projectile_active() || sm_native_scroll_active() || sm_native_ceres_door_active() ||
    sm_native_room_transition_active() || sm_native_door_scrolling_active() || sm_native_camera_active() ||
    sm_native_item_instruction_active() || sm_native_persistence_active() || sm_native_player_motion_active() ||
    sm_native_waver_active() || sm_native_ripper_active();
}
unsigned sm_native_rio_active(void) { return active_root; }
uint64_t sm_native_rio_generation(void) { return generation; }
void sm_native_rio_counts(uint64_t *out) { memcpy(out, counts, sizeof(counts)); }
static uint16_t Word(const uint8_t *ram, unsigned at) { return ram[at] | (uint16_t)ram[at+1] << 8; }
static bool Slot(unsigned k) { return k < 0x800 && !(k & 63); }
static bool Cell(unsigned index, unsigned cells) {
  unsigned kind = level_data[index] >> 12;
  if (kind==5 || kind==13) {
    int offset = (int8_t)BTS[index];
    if (!offset) return true;
    int next = (int)index + offset * (kind==13 ? (int)room_width_in_blocks : 1);
    if (next<0 || next>=(int)cells) return false;
    kind = level_data[next] >> 12; index = next;
    if (kind==5 || kind==13) return false;
  }
  return kind!=10 || (BTS[index]&0x7f)<16;
}
static bool TerrainAdmitted(void) {
  static const uint8_t table[32] = { [30]=0x94, [31]=0xd0 };
  static const uint8_t header[4] = {0xd4,0xb3,0x53,0xcd};
  // Native slope helpers read cur_enemy_index even when movement takes an
  // explicit k. The bounded rectangle must describe that same actor.
  if (!Slot(root_slot) || root_slot!=cur_enemy_index || memcmp(RomFixedPtr(0xa0c2da),table,sizeof(table)) ||
      memcmp(RomFixedPtr(0x84d094),header,sizeof(header))) return false;
  unsigned at=0xf78+root_slot, width=room_width_in_blocks, height=room_height_in_blocks;
  unsigned x=Word(g_ram,at+2), y=Word(g_ram,at+6), xr=Word(g_ram,at+10), yr=Word(g_ram,at+12);
  int dx=(int16_t)Word(g_ram,at+0x36), dy=(int16_t)Word(g_ram,at+0x34);
  // Enclose both movements (up to 16 pixels each), including the sine vector
  // installed after angle calculation. No native routine runs during admission.
  if (!width || width>255 || !height || height>255 || width*height>0x3200 ||
      !xr || xr>16 || !yr || yr>16 || dx < -4096 || dx>4096 || dy < -4096 || dy>4096 ||
      x<xr+16 || y<yr+16 || x+xr+16>width*16 || y+yr+16>height*16) return false;
  unsigned left=(x-xr-16)>>4, right=(x+xr+16-1)>>4, top=(y-yr-16)>>4, bottom=(y+yr+16-1)>>4;
  if (bottom>255) return false;
  for(unsigned row=top;row<=bottom;row++) for(unsigned col=left;col<=right;col++)
    if(!Cell(row*width+col,width*height)) return false;
  return true;
}
int sm_native_rio_available(void) { return callback_active && admitted && !poisoned; }
void sm_native_set_rio_hook(RioHook fn,void *ctx) {
  if(active_root || SiblingActive() || cur_coll_amt32) Die("changing Rio hook during invocation");
  hook=fn;context=ctx;
}
int sm_native_rio_bind(const uint8_t *ram,size_t size,uint64_t token) {
  if(!active_root || !callback_active || service_active || observing_service || SiblingActive() || cur_coll_amt32 ||
     poisoned || committed || issued_ram || !ram || size!=sizeof(g_ram) || token!=generation) {
    poisoned=true;return -1;
  }
  issued_ram=ram;return 0;
}
int sm_native_rio_commit(const uint8_t *ram,size_t size,uint64_t token) {
  if(!active_root || !callback_active || service_active || observing_service || SiblingActive() || !issued_ram || ram!=issued_ram ||
     size!=sizeof(g_ram) || token!=generation || cur_coll_amt32 || poisoned || committed || expected_service!=3) {
    poisoned=true;return -1;
  }
  memcpy(g_ram,ram,sizeof(g_ram));committed=true;return 0;
}
int sm_native_rio_service_begin(unsigned selector,uint16_t k,int32_t amount) {
  if(!active_root)return 0;
  if(observing_service || SiblingActive() || cur_coll_amt32 || g_fail || poisoned || committed ||
     (callback_active && !service_active) || selector>=3 || selector!=expected_service || k!=root_slot || calls>=3 ||
     (selector==2 ? amount!=0 : amount!=(int32_t)(int16_t)Word(g_ram,0xf78+k+(selector==0?0x36:0x34))*256))
    Die("recursive or unissued Rio native service");
  if(!service_active)counts[60+selector]++;
  observing_service=true;observed_selector=selector;
  sm_native_effect(70,(active_root-1)<<16|k,selector,(uint32_t)amount);
  return 1;
}
void sm_native_rio_service_done(unsigned selector,uint16_t result) {
  if(!active_root || !observing_service || selector!=observed_selector || SiblingActive() || cur_coll_amt32 || g_fail ||
     result>(selector==2?255:1))Die("Rio terminal native service failure");
  calls++;counts[63+selector]++;
  expected_service=selector==2?0:selector==1?3:(state==0xbc48 && result)?3:1;
  sm_native_effect(71,(active_root-1)<<16|root_slot,selector,result);
  observing_service=false;
}
int sm_native_rio_service(uint8_t *ram,size_t size,unsigned selector,uint16_t k,int32_t amount,uint16_t *result,uint64_t token) {
  if(selector<3)counts[60+selector]++;
  if(!active_root || !callback_active || service_active || observing_service || SiblingActive() || cur_coll_amt32 || !admitted ||
     !issued_ram || ram!=issued_ram || size!=sizeof(g_ram) || !result || token!=generation || poisoned || committed ||
     selector>=3 || selector!=expected_service || k!=root_slot || calls>=3 ||
     (selector==2 ? amount!=0 : amount!=(int32_t)(int16_t)Word(ram,0xf78+k+(selector==0?0x36:0x34))*256)) {
    counts[66]++;poisoned=true;return -1;
  }
  unsigned previous_calls=calls;
  service_active=true;memcpy(g_ram,ram,sizeof(g_ram));
  if(selector==0)*result=Enemy_MoveRight_IgnoreSlopes(k,amount);
  else if(selector==1)*result=Enemy_MoveDown(k,amount);
  else *result=CalculateAngleOfSamusFromEnemy(k);
  if(cur_coll_amt32 || g_fail || *result>(selector==2?255:1)) Die("Rio terminal native service failure");
  if(observing_service || calls!=previous_calls+1)Die("Rio missing or duplicated native service completion");
  memcpy(ram,g_ram,sizeof(g_ram));service_active=false;return 0;
}
static uint16_t Run(unsigned entry,uint16_t k,uint32_t arg) {
  if(entry>=10 || active_root || callback_active || service_active || observing_service || SiblingActive() || cur_coll_amt32)
    Die("recursive or cross-consumer Rio root");
  active_root=entry+1;root_slot=(entry>=3 && entry<=7)?k:cur_enemy_index;
  state=entry==1 && Slot(root_slot)?Word(g_ram,0xf78+root_slot+0x32):entry==9?(uint16_t)arg:0;
  static const uint16_t states[5]={0xbbed,0xbc32,0xbc48,0xbcb7,0xbcff};
  if(entry>=3 && entry<=7)state=states[entry-3];
  generation++;calls=0;issued_ram=NULL;poisoned=committed=false;
  bool movement=state==0xbc48 || state==0xbcb7 || (state==0xbcff && Slot(root_slot) &&
    (int16_t)(Word(g_ram,0xf78+root_slot+6)-samus_y_pos)<0);
  admitted=movement && TerrainAdmitted();
  expected_service=movement?(state==0xbcff?2:0):3;
  counts[entry]++;if(movement && !admitted)counts[50+entry]++;
  sm_native_effect(68,entry,k,arg);
  uint16_t result=0;int status=0;
  if(hook){
    counts[20+entry]++;callback_active=true;status=hook(context,entry,k,arg,&result);callback_active=false;
    if(poisoned)Die("Rio poisoned invocation; C retry prohibited");
    if(status==1){
      if(!committed || expected_service!=3 || result!=(entry==2?(uint16_t)arg:0))Die("Rio incomplete committed invocation");
      counts[40+entry]++;
    }else{
      if(status || calls || committed)Die("Rust Rio terminal failure; C retry prohibited");
      counts[30+entry]++;
    }
  }
  if(status!=1){
    counts[10+entry]++;
    switch(entry){
      case 0:Rio_Init();break;case 1:Rio_Main();break;
      case 2: {
        const uint16_t *jp=(const uint16_t *)(RomBankBase(0xa2)+arg);
        if(EnemyInstr_Rio_Instr_1(k,jp)!=jp)Die("Rio original continuation changed");
        result=(uint16_t)arg;break;
      }
      case 3:Rio_1(k);break;case 4:Rio_2(k);break;case 5:Rio_3(k);break;case 6:Rio_4(k);break;
      case 7:Rio_5(k);break;case 8:Rio_6((uint16_t)arg);break;case 9:CallRioFunc(arg);break;
    }
  }
  if(service_active || observing_service || SiblingActive() || cur_coll_amt32 || g_fail)Die("Rio root completion mismatch");
  sm_native_effect(69,entry,k,result);active_root=0;issued_ram=NULL;generation++;return result;
}
void sm_native_rio_ai(unsigned entry) { if(entry>1)Die("invalid Rio AI root");Run(entry,0,0); }
// Explicit diagnostic boundary for full-root controls. Never bypass terrain
// refusal by running unsafe C fixtures: validate before scheduler/native calls.
int sm_native_rio_fixture_admitted(unsigned entry,uint16_t k,uint32_t arg) {
  if(entry>=10 || active_root || SiblingActive() || cur_coll_amt32 || !Slot(cur_enemy_index))return 0;
  if(entry==2)return Slot(k) && arg>=0x8000 && arg<=0xfffe;
  if(entry==8)return arg<=0xffff;
  unsigned slot=(entry>=3 && entry<=7)?k:cur_enemy_index;
  if(!Slot(slot))return 0;
  static const uint16_t states[5]={0xbbed,0xbc32,0xbc48,0xbcb7,0xbcff};
  uint16_t selected=entry>=3 && entry<=7?states[entry-3]:entry==1?Word(g_ram,0xf78+slot+0x32):(uint16_t)arg;
  if(entry==1 || entry==9){
    if(entry==9 && (arg&0xffff0000)!=0xa20000)return 0;
    if(selected!=0xbbed && selected!=0xbc32 && selected!=0xbc48 && selected!=0xbcb7 && selected!=0xbcff)return 0;
  }
  if(entry==0)return 1;
  bool movement=selected==0xbc48 || selected==0xbcb7 || (selected==0xbcff &&
      (int16_t)(Word(g_ram,0xf78+slot+6)-samus_y_pos)<0);
  // Admission helper reads root_slot but has no native effects; restore it so
  // inspecting a fixture cannot change a subsequent invocation's root identity.
  uint16_t previous=root_slot;root_slot=slot;
  bool terrain=!movement || TerrainAdmitted();root_slot=previous;
  if(!terrain)return 0;
  if(selected==0xbbed){
    int delta=(int16_t)(samus_x_pos-Word(g_ram,0xf78+slot+2));
    if(delta<0)delta=-delta;
    if(delta<0xa0 && (g_ram[0x644]>=16 || g_ram[0x647]>=16))return 0;
  }
  return 1;
}
uint16_t sm_native_invoke_rio(unsigned entry,uint16_t k,uint32_t arg) {
  if(!sm_native_rio_fixture_admitted(entry,k,arg))Die("unsafe Rio fixture rejected before native effects");
  if(entry==0){CallEnemyAi(0xa2bbcd);return 0;}
  if(entry==1){CallEnemyAi(0xa2bbe3);return 0;}
  if(entry==2){
    const uint8_t *base=RomBankBase(0xa2);
    const uint16_t *jp=(const uint16_t *)(base+arg);
    if(CallEnemyInstr(0xa2bbc3,k,jp)!=jp)Die("Rio scheduler continuation mismatch");
    return (uint16_t)arg;
  }
  return Run(entry,k,arg);
}
const uint16_t *sm_native_rio_instruction(uint16_t k,const uint16_t *jp) {
  uintptr_t base=(uintptr_t)RomBankBase(0xa2), pointer=(uintptr_t)jp;
  if(pointer<base+0x8000 || pointer>base+0xfffe)Die("invalid Rio instruction pointer");
  return (const uint16_t *)(base+Run(2,k,(uint32_t)(pointer-base)));
}
