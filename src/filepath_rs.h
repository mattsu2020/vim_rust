#ifndef FILEPATH_RS_H
#define FILEPATH_RS_H

#ifdef __cplusplus
extern "C" {
#endif

int rs_is_path_sep(int ch);
char *rs_path_join(const char *a, const char *b);
void rs_path_free(char *s);

#ifdef __cplusplus
}
#endif

#endif // FILEPATH_RS_H
