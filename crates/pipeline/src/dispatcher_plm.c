#include "sm_rtl.h"
#include "variables.h"
#include "funcs.h"
extern const uint8 *CallPlmInstr(uint32,const uint8*,uint16);
extern CoroutineRet PlmHandler_Async(void);
const uint16 *PlmInstruction(uint16 slot,const uint16 *script) {
 const uint8 *p=(const uint8*)script;
 return (const uint16*)CallPlmInstr(GET_WORD(p)|0x840000,p+2,slot);
}
uint32 PlmControlFrame(void) {return PlmHandler_Async();}
uint32 PlmFrame(void) {return PlmHandler_Async();}
