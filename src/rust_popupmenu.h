#ifndef RUST_POPUPMENU_H
#define RUST_POPUPMENU_H

#include <stdint.h>

typedef struct {
    char *pum_text;
    char *pum_kind;
    char *pum_extra;
    char *pum_info;
    int32_t pum_cpt_source_idx;
    int32_t pum_user_abbr_hlattr;
    int32_t pum_user_kind_hlattr;
} PumItem;

void pum_display(const PumItem *array, int32_t size, int32_t selected);
void pum_clear(void);
int32_t pum_visible(void);
int32_t pum_get_height(void);

#endif // RUST_POPUPMENU_H
