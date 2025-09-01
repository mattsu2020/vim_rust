#include "memfile_ffi.c"

memfile_T *mf_open(char_u *fname, int flags)
{
    return rs_mf_open(fname, flags);
}

bhdr_T *mf_new(memfile_T *mfp, int negative, int page_count)
{
    return rs_mf_new(mfp, negative, page_count);
}

bhdr_T *mf_get(memfile_T *mfp, blocknr_T nr, int page_count)
{
    return rs_mf_get(mfp, nr, page_count);
}

void mf_put(memfile_T *mfp, bhdr_T *hp, int dirty, int infile)
{
    rs_mf_put(mfp, hp, dirty, infile);
}
