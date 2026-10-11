static uint32_t PortPointerResult(const uint16_t *pointer,uint16_t enemy) {
 if(!pointer)return 0; uintptr_t p=(uintptr_t)pointer;
 unsigned bank=g_ram[0xf78+enemy+0x2e];
 if(p<0x10000){if(p<0x8000)exit(3);return (bank<<16)|p;}
 uintptr_t base=(uintptr_t)g_rom+((bank&0x7f)*0x8000);
 if(p<base || p>=base+0x8000)exit(3);
 return (bank<<16)|(0x8000+(p-base));
}
