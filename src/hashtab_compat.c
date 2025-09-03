// Minimal stub for hash_clear_all when FEAT_SPELL/FEAT_TERMINAL are disabled.
#include "vim.h"

#if !defined(FEAT_SPELL) && !defined(FEAT_TERMINAL)
void hash_clear_all(hashtab_T *ht, int off)
{
    (void)off;
    // Fallback: just clear the hashtable array without freeing keys.
    // This matches the weaker behavior when the full helper isn't built.
    hash_clear(ht);
}
#endif

