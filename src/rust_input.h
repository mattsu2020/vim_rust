#ifndef RUST_INPUT_H
#define RUST_INPUT_H

#include <stdint.h>
#include <stddef.h>

typedef struct InputContext InputContext;

InputContext *rs_input_context_new(void);
void rs_input_context_free(InputContext *ctx);
void rs_input_feed(InputContext *ctx, uint32_t key);
void rs_input_feed_str(InputContext *ctx, const char *s);
int32_t rs_input_get(InputContext *ctx);
void rs_redo_feed(InputContext *ctx, uint32_t key);
void rs_redo_feed_str(InputContext *ctx, const char *s);
int32_t rs_redo_get(InputContext *ctx);
size_t rs_redo_get_all(InputContext *ctx, char *buf, size_t len);
size_t rs_record_get(InputContext *ctx, char *buf, size_t len);

#endif // RUST_INPUT_H
