const uint16 *EprojInstruction(uint16 slot,const uint16 *script) {
 const uint8 *p=(const uint8*)script;
 return (const uint16*)CallEprojInstr(GET_WORD(p)|0x860000,slot,p+2);
}
void EprojPreinstruction(uint16 slot,uint16 address) {
 CallEprojPreInstr(address|0x860000,slot);
}
