#ifndef RUST_SOUND_H
#define RUST_SOUND_H

long rs_sound_playevent(const char *name);
long rs_sound_playfile(const char *path);
void rs_sound_stop(long id);
void rs_sound_clear(void);
int rs_has_any_sound_callback(void);

#endif // RUST_SOUND_H
