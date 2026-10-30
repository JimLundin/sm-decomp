// ABI shared by the native utilities and their Rust/configuration callers.
#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>

char *sm_util_NextDelim(char **cursor, int separator);
bool sm_util_StringEqualsNoCase(const char *value, const char *expected);
const char *sm_util_StringStartsWithNoCase(const char *value, const char *prefix);
uint8_t *sm_util_ReadWholeFile(const char *name, size_t *length);
char *sm_util_NextLineStripComments(char **cursor);
char *sm_util_NextPossiblyQuotedString(char **cursor);
char *sm_util_SplitKeyValue(char *value);
const char *sm_util_SkipPrefix(const char *value, const char *prefix);
void sm_util_StrSet(char **target, const char *value);
void sm_util_die(const char *message);
bool sm_util_matches_source(uint32_t index, const uint8_t *bytes, size_t length);
