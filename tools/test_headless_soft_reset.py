"""Exercise the native soft-reset entry without power-cycling hardware."""
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


class SoftResetTests(unittest.TestCase):
    def test_soft_reset_enters_actual_vector_at_manual_position_three(self):
        source = (ROOT / 'deps/snesrev-sm/src/sm_80.c').read_text()
        reset = source.split('CoroutineRet Vector_RESET_Async(void) {', 1)[1].split('\n}\n', 1)[0]
        # Extract the real coroutine routing through the first hardware upload;
        # stop there, before unrelated reset services need a full SNES host.
        reset = reset.split('  APU_UploadBank(0xCF8000);', 1)[0]
        c = '''#include <stdint.h>
#include <string.h>
#include "types.h"
static uint8_t g_ram[0x20000],coroutine_state_0,reg_MEMSEL;
static uint16_t game_state,bug_fix_counter;
static unsigned io_writes,logo_calls,uploads;
struct Cpu{uint8_t k;uint16_t pc;} cpu;
struct Snes{struct Cpu *cpu;} snes={&cpu},*g_snes=&snes;
#define MEMSEL 0x420d
static void WriteReg(int a,int b){(void)a;(void)b;io_writes++;}
static uint8_t InitializeIoDisplayLogo_Async(void){logo_calls++;return 0;}
#include "headless_frame.h"
CoroutineRet Vector_RESET_Async(void){''' + reset + '''
 uploads++;return 0;}}
int main(void){
 memset(g_ram,0xa5,sizeof(g_ram));game_state=0xffff;coroutine_state_0=0;
 HeadlessPrepareFrame();Vector_RESET_Async();
 if(uploads!=1||logo_calls||io_writes||g_ram[0]!=0xa5||cpu.k!=0x80||cpu.pc!=0x8462)return 1;
 game_state=8;coroutine_state_0=12;HeadlessPrepareFrame();
 if(coroutine_state_0!=12)return 2;
 return 0;}
'''
        with tempfile.TemporaryDirectory() as temporary:
            run = Path(temporary)
            (run / 'control.c').write_text(c)
            result = subprocess.run(['cc', '-iquote', str(ROOT / 'deps/snesrev-sm/src'), '-iquote', str(ROOT / 'tools/decomp'),
                                     str(run / 'control.c'), '-o', str(run / 'control')], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
            result = subprocess.run([str(run / 'control')], capture_output=True, text=True)
            self.assertEqual(result.returncode, 0, result.stderr)
