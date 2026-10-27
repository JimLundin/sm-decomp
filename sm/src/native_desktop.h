#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <SDL.h>
typedef struct Game Game;
typedef struct RendererFuncs {
  bool (*Initialize)(SDL_Window *);
  void (*Destroy)(void);
  void (*BeginDraw)(int, int, uint8_t **, int *);
  void (*EndDraw)(void);
} RendererFuncs;
enum { kPpuRenderFlags_NewRenderer = 1, kPpuRenderFlags_4x4Mode7 = 2,
       kPpuRenderFlags_Height240 = 4, kPpuRenderFlags_NoSpriteLimits = 8,
       kSaveLoad_Save = 1, kSaveLoad_Load = 2, kSaveLoad_Replay = 3 };
void sm_host_die(const char *);
FILE *sm_host_stderr(void);
void sm_host_parse_config(void *, const char *);
int sm_host_find_key(int, int);
int sm_host_find_button(int, uint32_t);
void sm_host_debug(const void *);
bool sm_host_start_game(const void *, const char *);
bool sm_host_initialize_renderer(const void *, const RendererFuncs *, SDL_Window *);
void sm_desktop_close(void);
void sm_host_begin_draw(const RendererFuncs *, int, int, uint8_t **, int *);
void sm_host_end_draw(const RendererFuncs *);
void sm_host_create_player(const void *);
void sm_host_initialize_player(const void *);
void sm_host_prepare_pixels(const void *);
void sm_host_read_sram(const void *);
void sm_host_key(const void *, int, int, bool);
void sm_host_button(const void *, int, bool);
void sm_host_axis(const void *, int, int, int);
void sm_host_command(const void *, uint32_t, bool);
bool sm_host_run_frame(const void *, int);
bool sm_host_present(const void *, bool *);
int sm_host_render_scale(Game *, uint32_t);
void sm_host_copy_frame(Game *, uint8_t *, size_t, uint32_t);
int sm_host_mismatch_count(Game *);
int sm_host_fps(float);
void sm_host_save_load(Game *, int, int);
void sm_host_cheat(Game *, char);
void sm_host_game_command(Game *, uint32_t, int);
void sm_host_gamepad_bounds(int);
void sm_host_bad_command(void);
void sm_host_render_audio(Game *, int16_t *, int, int);
void SDLCALL sm_host_audio_callback(void *, uint8_t *, int);
void sm_host_OpenGLRenderer_Create(RendererFuncs *);
