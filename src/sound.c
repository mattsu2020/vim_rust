/* vi:set ts=8 sts=4 sw=4 et:
 *
 * VIM - Vi IMproved    by Bram Moolenaar
 *
 * Simplified sound support using Rust implementation.
 */

#include "vim.h"
#include "rust_sound.h"

#if defined(FEAT_SOUND) || defined(PROTO)

int has_any_sound_callback(void)
{
    return rs_has_any_sound_callback();
}

int has_sound_callback_in_queue(void)
{
    return FALSE;
}

void invoke_sound_callback(void)
{
    /* No callbacks supported in Rust implementation. */
}

static void sound_play_common(typval_T *argvars, typval_T *rettv, int playfile)
{
    if (in_vim9script() && check_for_string_arg(argvars, 0) == FAIL)
        return;
    char_u *name = tv_get_string(&argvars[0]);
    long id = playfile ? rs_sound_playfile((char *)name)
                       : rs_sound_playevent((char *)name);
    rettv->vval.v_number = id;
}

void f_sound_playevent(typval_T *argvars, typval_T *rettv)
{
    sound_play_common(argvars, rettv, FALSE);
}

void f_sound_playfile(typval_T *argvars, typval_T *rettv)
{
    sound_play_common(argvars, rettv, TRUE);
}

void f_sound_stop(typval_T *argvars, typval_T *rettv UNUSED)
{
    if (in_vim9script() && check_for_number_arg(argvars, 0) == FAIL)
        return;
    long id = tv_get_number(&argvars[0]);
    rs_sound_stop(id);
}

void f_sound_clear(typval_T *argvars UNUSED, typval_T *rettv UNUSED)
{
    rs_sound_clear();
}

# if defined(EXITFREE) || defined(PROTO)
void sound_free(void)
{
    rs_sound_clear();
}
# endif

#endif // FEAT_SOUND
