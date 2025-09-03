# Autocommand API

The legacy `apply_autocmds_group()` helper has been removed. Use
`apply_autocmds()` to trigger autocommands for an event.

The available functions are:

```c
int apply_autocmds(event_T event, char_u *fname, char_u *fname_io, int force, buf_T *buf);
int apply_autocmds_exarg(event_T event, char_u *fname, char_u *fname_io, int force, buf_T *buf, exarg_T *eap);
int apply_autocmds_retval(event_T event, char_u *fname, char_u *fname_io, int force, buf_T *buf, int *retval);
```

Group-specific execution is not currently supported; all autocommands run in
the default group.

