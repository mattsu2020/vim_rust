#include "channel_rs.h"

// Thin C wrappers delegating to the Rust channel implementation.

Channel* vim_channel_open(const char* addr) {
    return channel_open(addr);
}

Channel* vim_channel_spawn(const char* cmd) {
    return channel_spawn(cmd);
}

int vim_channel_send(Channel* chan, const char* data, size_t len) {
    return channel_send(chan, data, len);
}

ssize_t vim_channel_receive(Channel* chan, char* buf, size_t len) {
    return channel_receive(chan, buf, len);
}

int vim_channel_set_callback(Channel* chan, channel_callback cb, void* userdata) {
    return channel_set_callback(chan, cb, userdata);
}

int vim_channel_job_wait(Channel* chan) {
    return channel_job_wait(chan);
}

int vim_channel_close_stdin(Channel* chan) {
    return channel_close_stdin(chan);
}

void vim_channel_close(Channel* chan) {
    channel_close(chan);
}

