// Native utility boundary used by the pinned configuration implementation.
#pragma once
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "native_util.h"

int sm_config_atoi(const char *value);
FILE *sm_config_stderr(void);
