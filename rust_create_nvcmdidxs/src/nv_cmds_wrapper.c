#include "ascii.h"
#include "keymap.h"
#include "macros.h"
#include "nv_cmds.h"

const int *rust_nv_cmds(void) {
    return nv_cmds;
}

int rust_nv_cmds_size(void) {
    return NV_CMDS_SIZE;
}
