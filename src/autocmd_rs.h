#ifndef AUTOCMD_RS_H
#define AUTOCMD_RS_H

#ifdef __cplusplus
extern "C" {
#endif

int rs_event_ignored(event_T event, char_u *ei);
int rs_check_ei(char_u *ei);

#ifdef __cplusplus
}
#endif

#endif // AUTOCMD_RS_H
