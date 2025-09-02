#ifndef OS_UNIX_RS_H
#define OS_UNIX_RS_H

int rs_chdir(const char *path);
int rs_setenv(const char *name, const char *value, int overwrite);
int rs_unsetenv(const char *name);

#endif // OS_UNIX_RS_H
