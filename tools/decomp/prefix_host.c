// Additional read-only observation host; pinned gameplay and ordinary reset stay intact.
#define main baseline_main
#include "headless.c"
#undef main
#include "snes/input.h"

static FILE *events;
static unsigned long sequence, host_step, completed;
static bool changed_gameplay, causal_polling;
static unsigned scheduled_pad;
static FILE *causal_inputs;
static unsigned long causal_poll_count, causal_poll_limit, causal_completion_limit;
static bool inside_vblank_wait, omit_vblank_wait, omit_screen_wait;
static void Observe(const char *kind);
void __wrap_Vector_NMI(void);
static void PresentCausalPoll(void) {
  if (fscanf(causal_inputs, "%x", &scheduled_pad) != 1) exit(4);
  unsigned reversed = 0;
  for (unsigned bit = 0; bit < 16; bit++) reversed |= ((scheduled_pad >> bit) & 1) << (15-bit);
  g_snes->input1->currentState = reversed;
  ++causal_poll_count;
}
void __real_ScreenOn(void);
void __wrap_ScreenOn(void) {
  __real_ScreenOn();
  if (causal_polling && !omit_screen_wait) {
    inside_vblank_wait = true;
    Observe("within-dispatch-screen-on-wait");
    // Native ScreenOnWaitNMI calls WaitForNMI, which explicitly sets this flag.
    waiting_for_nmi = 1;
    __wrap_Vector_NMI();
    inside_vblank_wait = false;
  }
}
void __real_WaitUntilEndOfVblankAndClearHdma(void);
void __real_Vector_NMI(void);
void __real_ReadJoypadInputs(void);
void __wrap_WaitUntilEndOfVblankAndClearHdma(void) {
  if (causal_polling && !omit_vblank_wait) {
    inside_vblank_wait = true;
    Observe("within-dispatch-vblank-wait");
    __wrap_Vector_NMI();
    inside_vblank_wait = false;
  }
  __real_WaitUntilEndOfVblankAndClearHdma();
}

static void Observe(const char *kind) {
  fprintf(events, "%lu %lu %s %u",++sequence,host_step,kind,coroutine_state_0);
  const unsigned offsets[] = {0x998,0x79b,0xaf6,0xafa,0x9c2,0x9a4,0x9a2,0x9da,0x9dc,0x9de,0x9e0,0x5e5,0x8b,0x8f,0x97,0x5b6,0x5b8,0x617};
  for(unsigned i=0;i<sizeof(offsets)/sizeof(offsets[0]);i++)fprintf(events," %u",g_ram[offsets[i]]|(g_ram[offsets[i]+1]<<8));
  fprintf(events," %u\n",waiting_for_nmi);
  fflush(events);
}
void __real_RtlApuUpload(const uint8 *data);
void __wrap_RtlApuUpload(const uint8 *data) {
  Observe("upload-entry");
  __real_RtlApuUpload(data);
  Observe("upload-return");
}
void __real_ReadJoypadInputs(void);
void __wrap_ReadJoypadInputs(void) {
  if (causal_polling) { Observe("relocated-main-poll-suppressed"); return; }
  Observe("controller-entry");
  __real_ReadJoypadInputs();
  Observe("controller-return");
}
void __real_Vector_NMI(void);
void __wrap_Vector_NMI(void) {
  if (coroutine_state_0 == 0 && !inside_vblank_wait) {
    Observe("dispatch-complete");
    char name[80];
    snprintf(name,sizeof(name),"complete-%04lu.wram",++completed);
    FILE *state=fopen(name,"wb");
    if (!state || fwrite(g_ram,1,sizeof(g_ram),state)!=sizeof(g_ram) || fclose(state)) exit(2);
    if (causal_polling && causal_poll_count == causal_poll_limit && completed == causal_completion_limit) return;
  } else Observe("continuation");
  __real_Vector_NMI();
  if (causal_polling) {
    PresentCausalPoll();
    Observe("controller-entry");
    __real_ReadJoypadInputs();
    Observe("controller-return");
  }
  Observe("nmi-return");
}
uint16 __real_NextRandom(void);
uint16 __wrap_NextRandom(void) {
  uint16 result=__real_NextRandom();
  // Deliberate gameplay-error subject, selected only in a separate identified control process.
  if (changed_gameplay) random_number = result ^= 1;
  return result;
}
// Standalone polling seam: fixture loading is restricted to this offline subject.
// Gameplay replay below continues from its ordinary reset without state loading.
static int PollFixtures(const char *rom, const char *native, const char *cases, int control) {
  if (!SnesInit(rom)) return 2;
  FILE *list = fopen(cases, "r");
  if (!list) return 2;
  unsigned entry, leave, pad, stack_byte;
  uint8 expected[sizeof(g_ram)];
  while (fscanf(list, "%u %u %x %x", &entry, &leave, &pad, &stack_byte) == 4) {
    char path[4096];
    snprintf(path, sizeof(path), "%s/state-%u.wram", native, entry);
    FILE *file = fopen(path, "rb");
    if (!file || fread(g_ram, 1, sizeof(g_ram), file) != sizeof(g_ram) || fclose(file)) return 2;
    snprintf(path, sizeof(path), "%s/state-%u.wram", native, leave);
    file = fopen(path, "rb");
    if (!file || fread(expected, 1, sizeof(expected), file) != sizeof(expected) || fclose(file)) return 2;
    if (control == 1) pad ^= 0x80;
    if (stack_byte != 0xffff && (stack_byte < 0x1f00 || stack_byte >= 0x2000 || g_ram[stack_byte] != 0x94 || expected[stack_byte] != 0x16)) return 2;
    unsigned reversed = 0;
    for (unsigned bit = 0; bit < 16; bit++) reversed |= ((pad >> bit) & 1) << (15 - bit);
    g_snes->input1->currentState = reversed;
    // Invoke the real linked primitive, independently of boot/main/NMI scheduling.
    __real_ReadJoypadInputs();
    if (control == 2) g_ram[0x79b] ^= 1;
    unsigned differences = 0, gameplay_differences = 0;
    for (unsigned i = 0; i < sizeof(g_ram); i++) {
      differences += g_ram[i] != expected[i];
      if (i != stack_byte || g_ram[i] != 0x94 || expected[i] != 0x16) gameplay_differences += g_ram[i] != expected[i];
    }
    printf("poll %u %u %u %u %s %s %u %u 148 22\n",
           entry, leave, pad, differences, differences ? "false" : "true", gameplay_differences ? "false" : "true", gameplay_differences, stack_byte);
  }
  fclose(list);
  return 0;
}
int main(int argc,char **argv) {
  if (argc!=5 && argc!=6) return 2;
  if (!strcmp(argv[4], "poll-fixtures")) return PollFixtures(argv[1], argv[2], argv[3], 0);
  if (!strcmp(argv[4], "changed-poll-fixtures")) return PollFixtures(argv[1], argv[2], argv[3], 1);
  if (!strcmp(argv[4], "changed-state-poll-fixtures")) return PollFixtures(argv[1], argv[2], argv[3], 2);
  omit_screen_wait = !strcmp(argv[4], "omitted-causal-screen-wait");
  omit_vblank_wait = !strcmp(argv[4], "omitted-causal-wait");
  causal_polling = !strcmp(argv[4], "causal-polling") || !strcmp(argv[4], "changed-causal-rng") || omit_vblank_wait || omit_screen_wait;
  changed_gameplay = !strcmp(argv[4],"changed-rng") || !strcmp(argv[4], "changed-causal-rng");
  FILE *inputs=fopen(argv[2],"r");
  events=fopen("events.txt","w");
  if (!inputs || !events || !SnesInit(argv[1])) return 2;
  FILE *view=fopen("rom-view.bin","wb");
  if (!view || fwrite(g_rom,1,g_snes->cart->romSize,view)!=g_snes->cart->romSize || fclose(view)) return 2;
  g_spc_player=SpcPlayer_Create(); SpcPlayer_Initialize(g_spc_player);
  g_runmode=1; RtlReset(0);
  PpuBeginDrawing(g_snes->my_ppu,pixels,512*4,0);
  PpuBeginDrawing(g_snes->snes_ppu,pixels,512*4,0);
  causal_inputs = inputs;
  unsigned long steps=strtoul(argv[3],NULL,10);
  causal_poll_limit=steps;
  causal_completion_limit=argc==6 ? strtoul(argv[5],NULL,10) : 0;
  for (host_step=1;causal_polling ? (causal_poll_count<steps || completed<causal_completion_limit) : host_step<=steps;host_step++) {
    unsigned pad;
    if (causal_polling) pad=scheduled_pad;
    else if (fscanf(inputs,"%x",&pad)!=1 || pad>0xffff) return 2;
    scheduled_pad = pad;
    unsigned reversed=0;
    for (int bit=0;bit<16;bit++) reversed|=((pad>>bit)&1)<<(15-bit);
    HeadlessPrepareFrame();
    RtlRunFrame(reversed);
    if (g_fail) return 3;
  }
  FILE *state=fopen("terminal.bin","wb");
  if (!state || fwrite(g_ram,1,sizeof(g_ram),state)!=sizeof(g_ram) ||
      fwrite(g_sram,1,0x2000,state)!=0x2000 || fclose(state)) return 2;
  printf("summary %lu %lu 1 %lu %lu\n",steps,completed,fallback_calls,fallback_instructions);
  fclose(inputs); fclose(events); return 0;
}
