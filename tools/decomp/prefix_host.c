// Additional read-only observation host; pinned gameplay and ordinary reset stay intact.
#define main baseline_main
#include "headless.c"
#undef main

static FILE *events;
static unsigned long sequence, host_step, completed;
static bool changed_gameplay;

static void Observe(const char *kind) {
  fprintf(events, "{\"event\":%lu,\"host_step\":%lu,\"kind\":\"%s\",\"coroutine\":%u,\"state\":{",
          ++sequence, host_step, kind, coroutine_state_0);
  const unsigned offsets[] = {0x998,0x79b,0xaf6,0xafa,0x9c2,0x9a4,0x9a2,0x9da,0x9dc,0x9de,0x9e0,0x5e5,0x8b,0x8f,0x97,0x5b6,0x5b8,0x617};
  const char *names[] = {"game_state","room","samus_x","samus_y","health","items","equipped_items","clock_frames","clock_seconds","clock_minutes","clock_hours","rng","held","pressed","previous","accepted_nmi","all_nmi","upload"};
  for (unsigned i=0; i<sizeof(offsets)/sizeof(offsets[0]); i++)
    fprintf(events, "%s\"%s\":%u", i ? "," : "", names[i], g_ram[offsets[i]] | (g_ram[offsets[i]+1]<<8));
  fprintf(events, ",\"waiting\":%u}}\n", waiting_for_nmi);
  fflush(events);
}
void __real_ReadJoypadInputs(void);
void __wrap_ReadJoypadInputs(void) {
  Observe("controller-entry");
  __real_ReadJoypadInputs();
  Observe("controller-return");
}
void __real_Vector_NMI(void);
void __wrap_Vector_NMI(void) {
  if (coroutine_state_0 == 0) {
    Observe("dispatch-complete");
    char name[80];
    snprintf(name,sizeof(name),"complete-%04lu.wram",++completed);
    FILE *state=fopen(name,"wb");
    if (!state || fwrite(g_ram,1,sizeof(g_ram),state)!=sizeof(g_ram) || fclose(state)) exit(2);
  } else Observe("continuation");
  __real_Vector_NMI();
  Observe("nmi-return");
}
uint16 __real_NextRandom(void);
uint16 __wrap_NextRandom(void) {
  uint16 result=__real_NextRandom();
  // Deliberate gameplay-error subject, selected only in a separate identified control process.
  if (changed_gameplay) random_number = result ^= 1;
  return result;
}
int main(int argc,char **argv) {
  if (argc!=5) return 2;
  changed_gameplay = !strcmp(argv[4],"changed-rng");
  FILE *inputs=fopen(argv[2],"r");
  events=fopen("events.jsonl","w");
  if (!inputs || !events || !SnesInit(argv[1])) return 2;
  FILE *view=fopen("rom-view.bin","wb");
  if (!view || fwrite(g_rom,1,g_snes->cart->romSize,view)!=g_snes->cart->romSize || fclose(view)) return 2;
  g_spc_player=SpcPlayer_Create(); SpcPlayer_Initialize(g_spc_player);
  g_runmode=1; RtlReset(0);
  PpuBeginDrawing(g_snes->my_ppu,pixels,512*4,0);
  PpuBeginDrawing(g_snes->snes_ppu,pixels,512*4,0);
  unsigned long steps=strtoul(argv[3],NULL,10);
  for (host_step=1;host_step<=steps;host_step++) {
    unsigned pad;
    if (fscanf(inputs,"%x",&pad)!=1 || pad>0xffff) return 2;
    unsigned reversed=0;
    for (int bit=0;bit<16;bit++) reversed|=((pad>>bit)&1)<<(15-bit);
    RtlRunFrame(reversed);
    if (g_fail) return 3;
  }
  FILE *state=fopen("terminal.bin","wb");
  if (!state || fwrite(g_ram,1,sizeof(g_ram),state)!=sizeof(g_ram) ||
      fwrite(g_sram,1,0x2000,state)!=0x2000 || fclose(state)) return 2;
  printf("{\"steps\":%lu,\"complete\":%lu,\"initializations\":1,\"fallback_calls\":%lu,\"fallback_instructions\":%lu}\n",steps,completed,fallback_calls,fallback_instructions);
  fclose(inputs); fclose(events); return 0;
}
