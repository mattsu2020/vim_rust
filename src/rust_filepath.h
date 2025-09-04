#ifndef RUST_FILEPATH_H
#define RUST_FILEPATH_H

int rs_is_path_sep(int ch);
char *rs_path_join(const char *a, const char *b);
void rs_path_free(char *s);
char *rs_select_file_console(const char *initdir);

#endif // RUST_FILEPATH_H
