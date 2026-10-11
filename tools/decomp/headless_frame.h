// Prepare native frame dispatch without altering the hardware or movie timeline.
static void HeadlessPrepareFrame(void) {
  // SoftReset requests Vector_RESET_Async's manual soft-reset position.
  // Enter that coroutine before the game-state table sees its ffff sentinel.
  if (game_state == 0xffff) {
    g_snes->cpu->k = 0x80;
    g_snes->cpu->pc = 0x8462;
    coroutine_state_0 = 3;
  }
}
