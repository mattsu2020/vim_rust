#include <stdint.h>

typedef unsigned char char_u;
typedef int64_t blocknr_T;

typedef struct MemFile memfile_T;
typedef struct BlockHdr bhdr_T;

extern memfile_T *rs_mf_open(const char_u *fname, int flags);
extern bhdr_T *rs_mf_new(memfile_T *mfp, int negative, int page_count);
extern bhdr_T *rs_mf_get(memfile_T *mfp, blocknr_T nr, int page_count);
extern void rs_mf_put(memfile_T *mfp, bhdr_T *hp, int dirty, int infile);
