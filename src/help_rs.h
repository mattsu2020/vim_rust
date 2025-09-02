#ifndef HELP_RS_H
#define HELP_RS_H

#ifdef __cplusplus
extern "C" {
#endif

int rs_find_help_tags(const char *pat, int *num_matches, char ***matchesp,
                      int keep_lang);
void *rs_help_open_window(int width, int height);
void rs_help_close_window(void *ptr);

#ifdef __cplusplus
}
#endif

#endif // HELP_RS_H
