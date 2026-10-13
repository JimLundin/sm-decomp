
#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include "snes/snes.h"
#include "snes/ppu.h"
Snes *g_snes;
int g_new_ppu;
#include "ridley_ppu_contract.h"
int main(void) {
  Snes snes={0}; Ppu mine={0}, active={0};
  g_snes=&snes; snes.my_ppu=&mine; snes.snes_ppu=&active; snes.ppu=&active;
  mine.snes=active.snes=&snes;
  mine.m7matrix[0]=1234;mine.m7prev=19;mine.ppu1openBus=88;mine.ppu2openBus=99;
  snes.openBus=77;
  uint8_t before[40],after[40],restore[40];
  if(!RidleyPpuContractSave(before))return 1;
  ppu_write(snes.ppu,0x1b,0x00);ppu_write(snes.ppu,0x1b,0xff);ppu_write(snes.ppu,0x1c,2);
  uint8_t low=ppu_read(snes.ppu,0x35),high=ppu_read(snes.ppu,0x36);
  if(low!=254||high!=255)return 2;
  if(!RidleyPpuContractSave(after))return 3;
  if(memcmp(before+1,after+1,19)!=0)return 4;
  if(memcmp(before+20,after+20,19)==0)return 5;
  if(after[0]!=1||after[20]!=0||after[21]!=255||after[22]!=255||after[23]!=2||after[36]!=2||after[37]!=255)return 6;
  memset(&mine.m7matrix,0,sizeof(mine.m7matrix));mine.m7prev=mine.ppu1openBus=mine.ppu2openBus=0;
  memset(&active.m7matrix,0,sizeof(active.m7matrix));active.m7prev=active.ppu1openBus=active.ppu2openBus=0;
  snes.openBus=0;snes.ppu=&mine;
  if(!RidleyPpuContractLoad(after)||!RidleyPpuContractSave(restore)||memcmp(after,restore,40))return 7;
  restore[0]=2;
  if(RidleyPpuContractLoad(restore)||snes.ppu!=&active)return 8;
  before[0]=0;
  if(!RidleyPpuContractLoad(before)||snes.ppu!=&mine)return 9;
  puts("active Mode7 mutation, alternate preservation, signed reads, both-view exact restore and invalid discriminator refusal pass");
  return 0;
}
