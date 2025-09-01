#ifndef CHANNEL_RS_H
#define CHANNEL_RS_H

#ifdef __cplusplus
extern "C" {
#endif

typedef struct Channel Channel;
typedef void (*channel_callback)(const char* data, size_t len, void* userdata);

Channel* channel_open(const char* addr);
Channel* channel_spawn(const char* cmd);
int channel_send(Channel* chan, const char* data, size_t len);
ssize_t channel_receive(Channel* chan, char* buf, size_t len);
int channel_set_callback(Channel* chan, channel_callback cb, void* userdata);
int channel_job_wait(Channel* chan);
int channel_close_stdin(Channel* chan);
void channel_close(Channel* chan);

#ifdef __cplusplus
}
#endif

#endif // CHANNEL_RS_H
