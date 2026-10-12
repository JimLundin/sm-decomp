#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "sm_rtl.h"
#include "funcs.h"

extern uint8_t g_ram[0x20000]; extern const uint8_t *g_rom;
static uint32_t PortPointerResult(const uint16_t *pointer,uint16_t bank) {
 if(!pointer)return 0; uintptr_t p=(uintptr_t)pointer;
 uintptr_t base=(uintptr_t)g_rom+((bank&0x7f)*0x8000);
 if(p<0x10000){if(p<0x8000)exit(3);return ((uint32_t)bank<<16)|p;}
 if(p<base||p>=base+0x8000)exit(3);
 return ((uint32_t)bank<<16)|(0x8000+(p-base));
}

extern const uint16_t * EprojInstruction(uint16_t, const uint16_t *);
extern void SpawnEprojWithRoomGfx(uint16_t, uint16_t);
extern void SpawnEprojWithGfx(uint16_t,uint16_t,uint16_t);
extern void EprojRunAll(void);
void PortHostLoad(uint16_t*); void PortHostSave(uint16_t*);
int PortPrototypeInvoke(int argc,char **argv){
 if(argc!=9)return 2;
 uint16_t host[8]; FILE *hf=fopen(argv[7],"rb"); if(!hf||fread(host,2,8,hf)!=8||fgetc(hf)!=EOF||fclose(hf))return 2; PortHostLoad(host);
 int64_t a=strtoll(argv[2],0,10),b=strtoll(argv[3],0,10); uint32_t result=0;

 FILE *f=fopen(argv[4],"rb"); if(!f||fread(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fgetc(f)!=EOF||fclose(f))return 2;
 if (!strcmp(argv[1], "EprojInstruction")) { const uint16_t *pointer=EprojInstruction((uint16_t)a, (const uint16_t*)(g_rom+((((uint32_t)b>>16)<<15)|((uint32_t)b&0x7fff))%0x400000)); result=PortPointerResult(pointer,134); } else if (!strcmp(argv[1], "SpawnEprojWithRoomGfx")) { SpawnEprojWithRoomGfx((uint16_t)a, (uint16_t)b); result=0; } else if (!strcmp(argv[1], "SpawnEprojWithGfx")) { SpawnEprojWithGfx((uint16_t)a,(uint16_t)b,(uint16_t)((uint32_t)b>>16)); result=0; } else if (!strcmp(argv[1], "EprojRunAll")) { EprojRunAll(); result=0; } else return 2;
 f=fopen(argv[5],"wb"); if(!f||fwrite(g_ram,1,sizeof(g_ram),f)!=sizeof(g_ram)||fclose(f))return 2;
 PortHostSave(host);hf=fopen(argv[8],"wb");if(!hf||fwrite(host,2,8,hf)!=8||fclose(hf))return 2;
 printf("%u\n",result); return 0;
}
