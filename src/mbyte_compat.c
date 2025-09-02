// Minimal wrappers to satisfy libvterm expectations when FEAT_TERMINAL
// is not enabled. Delegate to existing UTF helpers in mbyte.c.

#include "vim.h"

int utf_uint2cells(UINT32_T c)
{
    if (c >= 0x100 && utf_iscomposing((int)c))
        return 0;
    return utf_char2cells((int)c);
}

int utf_iscomposing_uint(UINT32_T c)
{
    return utf_iscomposing((int)c);
}

