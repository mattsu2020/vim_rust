#ifndef CINDENT_RS_H
#define CINDENT_RS_H

#ifdef __cplusplus
extern "C" {
#endif

int cin_isfuncdecl(char_u **sp, linenr_T first_lnum, linenr_T min_lnum);

#ifdef __cplusplus
}
#endif

#endif // CINDENT_RS_H
