#ifndef CHANNEL_RS_H
#define CHANNEL_RS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Channel Channel;

typedef void (*channel_callback)(void *userdata, const char *data, size_t len);

Channel* channel_open(const char* addr);
int channel_send(Channel* chan, const char* data, size_t len);
ssize_t channel_receive(Channel* chan, char* buf, size_t len);
void channel_set_callback(Channel* chan, channel_callback cb, void *userdata);
void channel_poll(Channel* chan);
void channel_close(Channel* chan);

#ifdef __cplusplus
}
#endif

#endif // CHANNEL_RS_H
