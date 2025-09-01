#ifndef TAG_RS_H
#define TAG_RS_H

#ifdef __cplusplus
extern "C" {
#endif

int rust_find_tags(const char *pat, int *num_matches, char ***matchesp,
                   int flags, int mincount, const char *buf_ffname);

#ifdef __cplusplus
}
#endif

#endif // TAG_RS_H
