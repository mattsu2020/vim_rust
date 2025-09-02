#ifndef PATH_RS_H
#define PATH_RS_H

#ifdef __cplusplus
extern "C" {
#endif

char *rs_normalize_path(const char *path);
char *rs_find_in_path(const char *name, const char *paths);

#ifdef __cplusplus
}
#endif

#endif // PATH_RS_H
