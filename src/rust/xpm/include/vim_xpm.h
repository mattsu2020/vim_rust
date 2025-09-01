#ifndef VIM_XPM_H
#define VIM_XPM_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

uint32_t *xpm_load(const char *path, int *width, int *height);
void xpm_invert(uint32_t *data, size_t len);
void xpm_free(uint32_t *data, size_t len);

#ifdef __cplusplus
}
#endif

#endif // VIM_XPM_H
