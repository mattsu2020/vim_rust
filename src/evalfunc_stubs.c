#include "vim.h"

// Helper to set number return to 0 (false)
static void ret_zero(typval_T *rettv)
{
    if (rettv != NULL)
    {
        rettv->v_type = VAR_NUMBER;
        rettv->vval.v_number = 0;
    }
}

void f_assert_beeps(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_equal(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_equalfile(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_exception(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_fails(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_false(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_inrange(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_match(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_nobeep(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_notequal(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_notmatch(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_report(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_assert_true(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_ch_log(typval_T *argvars UNUSED, typval_T *rettv) { /* no-op */ ret_zero(rettv); }
void f_ch_logfile(typval_T *argvars UNUSED, typval_T *rettv) { /* no-op */ ret_zero(rettv); }
void f_test_alloc_fail(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_autochdir(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_feedinput(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_garbagecollect_now(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_garbagecollect_soon(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_getvalue(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_gui_event(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_ignore_error(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_mswin_event(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_null_blob(typval_T *argvars UNUSED, typval_T *rettv) { rettv->v_type = VAR_BLOB; rettv->vval.v_blob = NULL; }
void f_test_null_dict(typval_T *argvars UNUSED, typval_T *rettv) { rettv->v_type = VAR_DICT; rettv->vval.v_dict = NULL; }
void f_test_null_function(typval_T *argvars UNUSED, typval_T *rettv) { rettv->v_type = VAR_FUNC; rettv->vval.v_string = NULL; }
void f_test_null_list(typval_T *argvars UNUSED, typval_T *rettv) { rettv->v_type = VAR_LIST; rettv->vval.v_list = NULL; }
void f_test_null_partial(typval_T *argvars UNUSED, typval_T *rettv) { rettv->v_type = VAR_PARTIAL; rettv->vval.v_partial = NULL; }
void f_test_null_string(typval_T *argvars UNUSED, typval_T *rettv) { rettv->v_type = VAR_STRING; rettv->vval.v_string = NULL; }
void f_test_null_tuple(typval_T *argvars UNUSED, typval_T *rettv) { rettv->v_type = VAR_TUPLE; rettv->vval.v_tuple = NULL; }
void f_test_option_not_set(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_override(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_refcount(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_setmouse(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_settime(typval_T *argvars UNUSED, typval_T *rettv) { ret_zero(rettv); }
void f_test_unknown(typval_T *argvars UNUSED, typval_T *rettv) { rettv->v_type = VAR_UNKNOWN; }
void f_test_void(typval_T *argvars UNUSED, typval_T *rettv) { rettv->v_type = VAR_VOID; }
