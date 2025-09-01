#ifndef FOLD_RS_H
#define FOLD_RS_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct FoldState FoldState;

FoldState *rs_fold_state_new(void);
void rs_fold_state_free(FoldState *state);
void rs_fold_add(FoldState *state, long top, long len, unsigned char flags, unsigned char small);
void rs_fold_update(FoldState *state, long idx, long top, long len, unsigned char flags, unsigned char small);
long rs_fold_render(const FoldState *state);

#ifdef __cplusplus
}
#endif

#endif // FOLD_RS_H
