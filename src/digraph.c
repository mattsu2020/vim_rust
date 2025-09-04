/* vi:set ts=8 sts=4 sw=4 noet:
 *
 * VIM - Vi IMproved	by Bram Moolenaar
 *
 * Do ":help uganda"  in Vim to read copying and usage conditions.
 * Do ":help credits" in Vim to see a list of people who contributed.
 * See README.txt for an overview of the Vim source code.
 */

/*
 * digraph.c: code for digraphs
 */

#include "vim.h"

#if defined(FEAT_DIGRAPHS) || defined(PROTO)

typedef int result_T;

typedef struct digraph
{
    char_u	char1;
    char_u	char2;
    result_T	result;
} digr_T;

static void printdigraph(digr_T *dp, result_T *previous);

// digraphs added by the user
static garray_T	user_digraphs = {0, 0, (int)sizeof(digr_T), 10, NULL};

/*
 * digraphs for Unicode from RFC1345
 * (also work for ISO-8859-1 aka latin1)
 *
 * Note: Characters marked with XX are not included literally, because some
 * compilers cannot handle them (Amiga SAS/C is the most picky one).
 */
extern digr_T digraphdefault[];
extern int rs_digraph_lookup(int char1, int char2);

#   define DG_START_LATIN 0xa1
#   define DG_START_GREEK 0x0386
#   define DG_START_CYRILLIC 0x0401
#   define DG_START_HEBREW 0x05d0
#   define DG_START_ARABIC 0x060c
#   define DG_START_LATIN_EXTENDED 0x1e02
#   define DG_START_GREEK_EXTENDED 0x1f00
#   define DG_START_PUNCTUATION 0x2002
#   define DG_START_SUB_SUPER 0x2070
#   define DG_START_CURRENCY 0x20a4
#   define DG_START_OTHER1 0x2103
#   define DG_START_ROMAN 0x2160
#   define DG_START_ARROWS 0x2190
#   define DG_START_MATH 0x2200
#   define DG_START_TECHNICAL 0x2302
#   define DG_START_OTHER2 0x2423
#   define DG_START_DRAWING 0x2500
#   define DG_START_BLOCK 0x2580
#   define DG_START_SHAPES 0x25a0
#   define DG_START_SYMBOLS 0x2605
#   define DG_START_DINGBATS 0x2713
#   define DG_START_CJK_SYMBOLS 0x3000
#   define DG_START_HIRAGANA 0x3041
#   define DG_START_KATAKANA 0x30a1
#   define DG_START_BOPOMOFO 0x3105
#   define DG_START_OTHER3 0x3220


/*
 * handle digraphs after typing a character
 */
    int
do_digraph(int c)
{
    static int	backspaced;	// character before K_BS
    static int	lastchar;	// last typed character

    if (c == -1)		// init values
    {
	backspaced = -1;
    }
    else if (p_dg)
    {
	if (backspaced >= 0)
	    c = digraph_get(backspaced, c, FALSE);
	backspaced = -1;
	if ((c == K_BS || c == Ctrl_H) && lastchar >= 0)
	    backspaced = lastchar;
    }
    lastchar = c;
    return c;
}

/*
 * Find a digraph for "val".  If found return the string to display it.
 * If not found return NULL.
 */
    char_u *
get_digraph_for_char(int val_arg)
{
    int		val = val_arg;
    int		i;
    int		use_defaults;
    digr_T	*dp;
    static      char_u      r[3];

#if defined(USE_UNICODE_DIGRAPHS)
    if (!enc_utf8)
    {
	char_u	    buf[6], *to;
	vimconv_T   vc;

	// convert the character from 'encoding' to Unicode
	i = mb_char2bytes(val, buf);
	vc.vc_type = CONV_NONE;
	if (convert_setup(&vc, p_enc, (char_u *)"utf-8") == OK)
	{
	    vc.vc_fail = TRUE;
	    to = string_convert(&vc, buf, &i);
	    if (to != NULL)
	    {
		val = utf_ptr2char(to);
		vim_free(to);
	    }
	    (void)convert_setup(&vc, NULL, NULL);
	}
    }
#endif

    for (use_defaults = 0; use_defaults <= 1; use_defaults++)
    {
	if (use_defaults == 0)
	    dp = (digr_T *)user_digraphs.ga_data;
	else
	    dp = digraphdefault;
	for (i = 0; use_defaults ? dp->char1 != NUL
					       : i < user_digraphs.ga_len; ++i)
	{
	    if (dp->result == val)
	    {
		r[0] = dp->char1;
		r[1] = dp->char2;
		r[2] = NUL;
		return r;
	    }
	    ++dp;
	}
    }
    return NULL;
}

/*
 * Get a digraph.  Used after typing CTRL-K on the command line or in normal
 * mode.
 * Returns composed character, or NUL when ESC was used.
 */
    int
get_digraph(
    int		cmdline)	// TRUE when called from the cmdline
{
    int		c, cc;

    ++no_mapping;
    ++allow_keys;
    c = plain_vgetc();
    --no_mapping;
    --allow_keys;

    if (c == ESC)		// ESC cancels CTRL-K
	return NUL;

    if (IS_SPECIAL(c))	// insert special key code
	return c;
    if (cmdline)
    {
	if (char2cells(c) == 1
#if defined(FEAT_CRYPT) || defined(FEAT_EVAL)
		&& cmdline_star == 0
#endif
	   )
	    putcmdline(c, TRUE);
    }
    else
	rs_add_to_showcmd(c);
    ++no_mapping;
    ++allow_keys;
    cc = plain_vgetc();
    --no_mapping;
    --allow_keys;
    if (cc != ESC)	    // ESC cancels CTRL-K
	return digraph_get(c, cc, TRUE);
    return NUL;
}

/*
 * Lookup the pair "char1", "char2" in the digraph tables.
 * If no match, return "char2".
 * If "meta_char" is TRUE and "char1" is a space, return "char2" | 0x80.
 */
    static int
getexactdigraph(int char1, int char2, int meta_char)
{
    int		i;
    int		retval = 0;
    digr_T	*dp;

    if (IS_SPECIAL(char1) || IS_SPECIAL(char2))
	return char2;

    /*
     * Search user digraphs first.
     */
    dp = (digr_T *)user_digraphs.ga_data;
    for (i = 0; i < user_digraphs.ga_len; ++i)
    {
	if ((int)dp->char1 == char1 && (int)dp->char2 == char2)
	{
	    retval = dp->result;
	    break;
	}
	++dp;
    }

    /*
     * Search default digraphs using the Rust-generated table.
     */
    if (retval == 0)
        retval = rs_digraph_lookup(char1, char2);
#ifdef USE_UNICODE_DIGRAPHS
    if (retval != 0 && !enc_utf8)
    {
	char_u	    buf[6], *to;
	vimconv_T   vc;

	/*
	 * Convert the Unicode digraph to 'encoding'.
	 */
	i = utf_char2bytes(retval, buf);
	retval = 0;
	vc.vc_type = CONV_NONE;
	if (convert_setup(&vc, (char_u *)"utf-8", p_enc) == OK)
	{
	    vc.vc_fail = TRUE;
	    to = string_convert(&vc, buf, &i);
	    if (to != NULL)
	    {
		retval = (*mb_ptr2char)(to);
		vim_free(to);
	    }
	    (void)convert_setup(&vc, NULL, NULL);
	}
    }
#endif

    // Ignore multi-byte characters when not in multi-byte mode.
    if (!has_mbyte && retval > 0xff)
	retval = 0;

    if (retval == 0)		// digraph deleted or not found
    {
	if (char1 == ' ' && meta_char)	// <space> <char> --> meta-char
	    return (char2 | 0x80);
	return char2;
    }
    return retval;
}

/*
 * Get digraph.
 * Allow for both char1-char2 and char2-char1
 */
    int
digraph_get(int char1, int char2, int meta_char)
{
    int	    retval;

    if (((retval = getexactdigraph(char1, char2, meta_char)) == char2)
	    && (char1 != char2)
	    && ((retval = getexactdigraph(char2, char1, meta_char)) == char1))
	return char2;
    return retval;
}

/*
 * Add a digraph to the digraph table.
 */
    static void
registerdigraph(int char1, int char2, int n)
{
    int		i;
    digr_T	*dp;

    // If the digraph already exists, replace "result".
    dp = (digr_T *)user_digraphs.ga_data;
    for (i = 0; i < user_digraphs.ga_len; ++i)
    {
	if ((int)dp->char1 == char1 && (int)dp->char2 == char2)
	{
	    dp->result = n;
	    return;
	}
	++dp;
    }

    // Add a new digraph to the table.
    if (ga_grow(&user_digraphs, 1) == FAIL)
	return;

    dp = (digr_T *)user_digraphs.ga_data + user_digraphs.ga_len;
    dp->char1 = char1;
    dp->char2 = char2;
    dp->result = n;
    ++user_digraphs.ga_len;
}

/*
 * Check the characters are valid for a digraph.
 * If they are valid, returns TRUE; otherwise, give an error message and
 * returns FALSE.
 */
    static int
check_digraph_chars_valid(int char1, int char2)
{
    if (char2 == 0)
    {
	char_u msg[MB_MAXBYTES + 1];

	msg[mb_char2bytes(char1, msg)] = NUL;

	semsg(_(e_digraph_must_be_just_two_characters_str), msg);
	return FALSE;
    }
    if (char1 == ESC || char2 == ESC)
    {
	emsg(_(e_escape_not_allowed_in_digraph));
	return FALSE;
    }
    return TRUE;
}



/*
 * Add the digraphs in the argument to the digraph table.
 * format: {c1}{c2} char {c1}{c2} char ...
 */
    void
putdigraph(char_u *str)
{
    int		char1, char2, n;

    while (*str != NUL)
    {
	str = skipwhite(str);
	if (*str == NUL)
	    return;
	char1 = *str++;
	char2 = *str++;

	if (!check_digraph_chars_valid(char1, char2))
	    return;

	str = skipwhite(str);
	if (!VIM_ISDIGIT(*str))
	{
	    emsg(_(e_number_expected));
	    return;
	}
	n = getdigits(&str);

	registerdigraph(char1, char2, n);
    }
}

#if defined(USE_UNICODE_DIGRAPHS)
    static void
digraph_header(char *msg)
{
    if (msg_col > 0)
	msg_putchar('\n');
    msg_outtrans_attr((char_u *)msg, HL_ATTR(HLF_CM));
    msg_putchar('\n');
}
#endif

    void
listdigraphs(int use_headers)
{
    int		i;
    digr_T	*dp;
    result_T	previous = 0;

    msg_putchar('\n');

    dp = digraphdefault;
    while (dp->char1 != NUL && !got_int)
    {
#if defined(USE_UNICODE_DIGRAPHS)
	digr_T tmp;

	// May need to convert the result to 'encoding'.
	tmp.char1 = dp->char1;
	tmp.char2 = dp->char2;
	tmp.result = getexactdigraph(tmp.char1, tmp.char2, FALSE);
	if (tmp.result != 0 && tmp.result != tmp.char2
					  && (has_mbyte || tmp.result <= 255))
	    printdigraph(&tmp, use_headers ? &previous : NULL);
#else

	if (getexactdigraph(dp->char1, dp->char2, FALSE) == dp->result
		&& (has_mbyte || dp->result <= 255))
	    printdigraph(dp, use_headers ? &previous : NULL);
#endif
	++dp;
	ui_breakcheck();
    }

    dp = (digr_T *)user_digraphs.ga_data;
    for (i = 0; i < user_digraphs.ga_len && !got_int; ++i)
    {
#if defined(USE_UNICODE_DIGRAPHS)
	if (previous >= 0 && use_headers)
	    digraph_header(_("Custom"));
	previous = -1;
#endif
	printdigraph(dp, NULL);
	ui_breakcheck();
	++dp;
    }

    // clear screen, because some digraphs may be wrong, in which case we
    // messed up ScreenLines
    set_must_redraw(UPD_CLEAR);
}

    static void
digraph_getlist_appendpair(digr_T *dp, list_T *l)
{
    char_u	buf[30];
    char_u	*p;
    list_T	*l2;
    listitem_T	*li, *li2;


    li = listitem_alloc();
    if (li == NULL)
	return;
    list_append(l, li);
    li->li_tv.v_type = VAR_LIST;
    li->li_tv.v_lock = 0;

    l2 = list_alloc();
    li->li_tv.vval.v_list = l2;
    if (l2 == NULL)
	return;
    ++l2->lv_refcount;

    li2 = listitem_alloc();
    if (li2 == NULL)
	return;
    list_append(l2, li2);
    li2->li_tv.v_type = VAR_STRING;
    li2->li_tv.v_lock = 0;

    buf[0] = dp->char1;
    buf[1] = dp->char2;
    buf[2] = NUL;
    li2->li_tv.vval.v_string = vim_strsave(&buf[0]);

    li2 = listitem_alloc();
    if (li2 == NULL)
	return;
    list_append(l2, li2);
    li2->li_tv.v_type = VAR_STRING;
    li2->li_tv.v_lock = 0;

    p = buf;
    if (has_mbyte)
	p += (*mb_char2bytes)(dp->result, p);
    else
	*p++ = (char_u)dp->result;
    *p = NUL;

    li2->li_tv.vval.v_string = vim_strsave(buf);
}

    static void
digraph_getlist_common(int list_all, typval_T *rettv)
{
    int		i;
    digr_T	*dp;

    if (rettv_list_alloc(rettv) == FAIL)
	return;

    if (list_all)
    {
	dp = digraphdefault;
	while (dp->char1 != NUL && !got_int)
	{
#ifdef USE_UNICODE_DIGRAPHS
	    digr_T tmp;

	    tmp.char1 = dp->char1;
	    tmp.char2 = dp->char2;
	    tmp.result = getexactdigraph(tmp.char1, tmp.char2, FALSE);
	    if (tmp.result != 0 && tmp.result != tmp.char2
					  && (has_mbyte || tmp.result <= 255))
		digraph_getlist_appendpair(&tmp, rettv->vval.v_list);
#else
	    if (getexactdigraph(dp->char1, dp->char2, FALSE) == dp->result
		    && (has_mbyte || dp->result <= 255))
		digraph_getlist_appendpair(dp, rettv->vval.v_list);
#endif
	    ++dp;
	}
    }

    dp = (digr_T *)user_digraphs.ga_data;
    for (i = 0; i < user_digraphs.ga_len && !got_int; ++i)
    {
	digraph_getlist_appendpair(dp, rettv->vval.v_list);
	++dp;
    }
}

static struct dg_header_entry {
    int	    dg_start;
    char    *dg_header;
} header_table[] = {
    {DG_START_LATIN, N_("Latin supplement")},
    {DG_START_GREEK, N_("Greek and Coptic")},
    {DG_START_CYRILLIC, N_("Cyrillic")},
    {DG_START_HEBREW, N_("Hebrew")},
    {DG_START_ARABIC, N_("Arabic")},
    {DG_START_LATIN_EXTENDED, N_("Latin extended")},
    {DG_START_GREEK_EXTENDED, N_("Greek extended")},
    {DG_START_PUNCTUATION, N_("Punctuation")},
    {DG_START_SUB_SUPER, N_("Super- and subscripts")},
    {DG_START_CURRENCY, N_("Currency")},
    {DG_START_OTHER1, N_("Other")},
    {DG_START_ROMAN, N_("Roman numbers")},
    {DG_START_ARROWS, N_("Arrows")},
    {DG_START_MATH, N_("Mathematical operators")},
    {DG_START_TECHNICAL, N_("Technical")},
    {DG_START_OTHER2, N_("Other")},
    {DG_START_DRAWING, N_("Box drawing")},
    {DG_START_BLOCK, N_("Block elements")},
    {DG_START_SHAPES, N_("Geometric shapes")},
    {DG_START_SYMBOLS, N_("Symbols")},
    {DG_START_DINGBATS, N_("Dingbats")},
    {DG_START_CJK_SYMBOLS, N_("CJK symbols and punctuation")},
    {DG_START_HIRAGANA, N_("Hiragana")},
    {DG_START_KATAKANA, N_("Katakana")},
    {DG_START_BOPOMOFO, N_("Bopomofo")},
    {DG_START_OTHER3, N_("Other")},
    {0xfffffff, NULL},
};

    static void
printdigraph(digr_T *dp, result_T *previous)
{
    char_u	buf[30];
    char_u	*p;

    int		list_width;

    if ((dy_flags & DY_UHEX) || has_mbyte)
	list_width = 13;
    else
	list_width = 11;

    if (dp->result == 0)
	return;

#if defined(USE_UNICODE_DIGRAPHS)
    if (previous != NULL)
    {
	int i;

	for (i = 0; header_table[i].dg_header != NULL; ++i)
	    if (*previous < header_table[i].dg_start
		    && dp->result >= header_table[i].dg_start
		    && dp->result < header_table[i + 1].dg_start)
	    {
		digraph_header(_(header_table[i].dg_header));
		break;
	    }
	*previous = dp->result;
    }
#endif
    if (msg_col > Columns - list_width)
	msg_putchar('\n');
    if (msg_col)
	while (msg_col % list_width != 0)
	    msg_putchar(' ');

    p = buf;
    *p++ = dp->char1;
    *p++ = dp->char2;
    *p++ = ' ';
    *p = NUL;
    msg_outtrans(buf);
    p = buf;
    if (has_mbyte)
    {
	// add a space to draw a composing char on
	if (enc_utf8 && utf_iscomposing(dp->result))
	    *p++ = ' ';
	p += (*mb_char2bytes)(dp->result, p);
    }
    else
	*p++ = (char_u)dp->result;
    *p = NUL;
    msg_outtrans_attr(buf, HL_ATTR(HLF_8));
    p = buf;
    if (char2cells(dp->result) == 1)
	*p++ = ' ';
    vim_snprintf((char *)p, sizeof(buf) - (p - buf), " %3d", dp->result);
    msg_outtrans(buf);
}

# ifdef FEAT_EVAL
/*
 * Get the two digraph characters from a typval.
 * Return OK or FAIL.
 */
    static int
get_digraph_chars(typval_T *arg, int *char1, int *char2)
{
    char_u	buf_chars[NUMBUFLEN];
    char_u	*chars = tv_get_string_buf_chk(arg, buf_chars);
    char_u	*p = chars;

    if (p != NULL)
    {
	if (*p != NUL)
	{
	    *char1 = mb_cptr2char_adv(&p);
	    if (*p != NUL)
	    {
		*char2 = mb_cptr2char_adv(&p);
		if (*p == NUL)
		{
		    if (check_digraph_chars_valid(*char1, *char2))
			return OK;
		    return FAIL;
		}
	    }
	}
    }
    semsg(_(e_digraph_must_be_just_two_characters_str), chars);
    return FAIL;
}

    static int
digraph_set_common(typval_T *argchars, typval_T *argdigraph)
{
    int		char1, char2;
    char_u	*digraph;
    char_u	*p;
    char_u	buf_digraph[NUMBUFLEN];
    varnumber_T n;

    if (get_digraph_chars(argchars, &char1, &char2) == FAIL)
	return FALSE;

    digraph = tv_get_string_buf_chk(argdigraph, buf_digraph);
    if (digraph == NULL)
	return FALSE;
    p = digraph;
    n = mb_cptr2char_adv(&p);
    if (*p != NUL)
    {
	semsg(_(e_digraph_argument_must_be_one_character_str), digraph);
	return FALSE;
    }

    registerdigraph(char1, char2, (int)n);
    return TRUE;
}
# endif

#endif // FEAT_DIGRAPHS

#if defined(FEAT_EVAL) || defined(PROTO)
/*
 * "digraph_get()" function
 */
    void
f_digraph_get(typval_T *argvars, typval_T *rettv)
{
# ifdef FEAT_DIGRAPHS
    int		code;
    char_u	buf[NUMBUFLEN];
    char_u	*digraphs;

    rettv->v_type = VAR_STRING;
    rettv->vval.v_string = NULL;  // Return empty string for failure

    if (in_vim9script() && check_for_string_arg(argvars, 0) == FAIL)
	return;

    digraphs = tv_get_string_chk(&argvars[0]);

    if (digraphs == NULL)
	return;
    else if (STRLEN(digraphs) != 2)
    {
	semsg(_(e_digraph_must_be_just_two_characters_str), digraphs);
	return;
    }
    code = digraph_get(digraphs[0], digraphs[1], FALSE);

    if (has_mbyte)
	buf[(*mb_char2bytes)(code, buf)] = NUL;
    else
    {
	buf[0] = code;
	buf[1] = NUL;
    }

    rettv->vval.v_string = vim_strsave(buf);
# else
    emsg(_(e_no_digraphs_version));
# endif
}

/*
 * "digraph_getlist()" function
 */
    void
f_digraph_getlist(typval_T *argvars, typval_T *rettv)
{
# ifdef FEAT_DIGRAPHS
    int     flag_list_all;

    if (check_for_opt_bool_arg(argvars, 0) == FAIL)
	return;

    if (argvars[0].v_type == VAR_UNKNOWN)
	flag_list_all = FALSE;
    else
    {
	varnumber_T flag = tv_get_bool(&argvars[0]);

	flag_list_all = flag ? TRUE : FALSE;
    }

    digraph_getlist_common(flag_list_all, rettv);
# else
    emsg(_(e_no_digraphs_version));
# endif
}

/*
 * "digraph_set()" function
 */
    void
f_digraph_set(typval_T *argvars, typval_T *rettv)
{
# ifdef FEAT_DIGRAPHS
    rettv->v_type = VAR_BOOL;
    rettv->vval.v_number = VVAL_FALSE;

    if (in_vim9script()
	    && (check_for_string_arg(argvars, 0) == FAIL
		|| check_for_string_arg(argvars, 1) == FAIL))
	return;

    if (!digraph_set_common(&argvars[0], &argvars[1]))
	return;

    rettv->vval.v_number = VVAL_TRUE;
# else
    emsg(_(e_no_digraphs_version));
# endif
}

/*
 * "digraph_setlist()" function
 */
    void
f_digraph_setlist(typval_T * argvars, typval_T *rettv)
{
# ifdef FEAT_DIGRAPHS
    list_T	*pl, *l;
    listitem_T	*pli;

    rettv->v_type = VAR_BOOL;
    rettv->vval.v_number = VVAL_FALSE;

    if (argvars[0].v_type != VAR_LIST)
    {
	emsg(_(e_digraph_setlist_argument_must_be_list_of_lists_with_two_items));
	return;
    }

    pl = argvars[0].vval.v_list;
    if (pl == NULL)
    {
	// Empty list always results in success.
	rettv->vval.v_number = VVAL_TRUE;
	return;
    }

    FOR_ALL_LIST_ITEMS(pl, pli)
    {
	if (pli->li_tv.v_type != VAR_LIST)
	{
	    emsg(_(e_digraph_setlist_argument_must_be_list_of_lists_with_two_items));
	    return;
	}

	l = pli->li_tv.vval.v_list;
	if (l == NULL || l->lv_len != 2)
	{
	    emsg(_(e_digraph_setlist_argument_must_be_list_of_lists_with_two_items));
	    return;
	}

	if (!digraph_set_common(&l->lv_first->li_tv,
						 &l->lv_first->li_next->li_tv))
	    return;
    }
    rettv->vval.v_number = VVAL_TRUE;
# else
    emsg(_(e_no_digraphs_version));
# endif
}

#endif // FEAT_EVAL


#if defined(FEAT_KEYMAP) || defined(PROTO)

// structure used for b_kmap_ga.ga_data
typedef struct
{
    char_u	*from;
    char_u	*to;
} kmap_T;

#define KMAP_MAXLEN 20	    // maximum length of "from" or "to"

static void keymap_unload(void);

/*
 * Set up key mapping tables for the 'keymap' option.
 * Returns NULL if OK, an error message for failure.  This only needs to be
 * used when setting the option, not later when the value has already been
 * checked.
 */
    char *
keymap_init(void)
{
    curbuf->b_kmap_state &= ~KEYMAP_INIT;

    if (*curbuf->b_p_keymap == NUL)
    {
	// Stop any active keymap and clear the table.  Also remove
	// b:keymap_name, as no keymap is active now.
	keymap_unload();
	do_cmdline_cmd((char_u *)"unlet! b:keymap_name");
    }
    else
    {
	char_u	*buf;
	size_t  buflen;

	// Source the keymap file.  It will contain a ":loadkeymap" command
	// which will call ex_loadkeymap() below.
	buflen = STRLEN(curbuf->b_p_keymap) + STRLEN(p_enc) + 14;
	buf = alloc(buflen);
	if (buf == NULL)
	    return e_out_of_memory;

	// try finding "keymap/'keymap'_'encoding'.vim"  in 'runtimepath'
	vim_snprintf((char *)buf, buflen, "keymap/%s_%s.vim",
						   curbuf->b_p_keymap, p_enc);
	if (source_runtime(buf, 0) == FAIL)
	{
	    // try finding "keymap/'keymap'.vim" in 'runtimepath'
	    vim_snprintf((char *)buf, buflen, "keymap/%s.vim",
							  curbuf->b_p_keymap);
	    if (source_runtime(buf, 0) == FAIL)
	    {
		vim_free(buf);
		return N_(e_keymap_file_not_found);
	    }
	}
	vim_free(buf);
    }

    return NULL;
}

/*
 * ":loadkeymap" command: load the following lines as the keymap.
 */
    void
ex_loadkeymap(exarg_T *eap)
{
    char_u	*line;
    char_u	*p;
    char_u	*s;
    kmap_T	*kp;
#define KMAP_LLEN   200	    // max length of "to" and "from" together
    char_u	buf[KMAP_LLEN + 11];
    int		i;
    char_u	*save_cpo = p_cpo;

    if (!sourcing_a_script(eap))
    {
	emsg(_(e_using_loadkeymap_not_in_sourced_file));
	return;
    }

    /*
     * Stop any active keymap and clear the table.
     */
    keymap_unload();

    curbuf->b_kmap_state = 0;
    ga_init2(&curbuf->b_kmap_ga, sizeof(kmap_T), 20);

    // Set 'cpoptions' to "C" to avoid line continuation.
    p_cpo = (char_u *)"C";

    /*
     * Get each line of the sourced file, break at the end.
     */
    for (;;)
    {
	line = eap->ea_getline(0, eap->cookie, 0, TRUE);
	if (line == NULL)
	    break;

	p = skipwhite(line);
	if (*p != '"' && *p != NUL && ga_grow(&curbuf->b_kmap_ga, 1) == OK)
	{
	    kp = (kmap_T *)curbuf->b_kmap_ga.ga_data + curbuf->b_kmap_ga.ga_len;
	    s = skiptowhite(p);
	    kp->from = vim_strnsave(p, s - p);
	    p = skipwhite(s);
	    s = skiptowhite(p);
	    kp->to = vim_strnsave(p, s - p);

	    if (kp->from == NULL || kp->to == NULL
		    || STRLEN(kp->from) + STRLEN(kp->to) >= KMAP_LLEN
		    || *kp->from == NUL || *kp->to == NUL)
	    {
		if (kp->to != NULL && *kp->to == NUL)
		    emsg(_(e_empty_keymap_entry));
		vim_free(kp->from);
		vim_free(kp->to);
	    }
	    else
		++curbuf->b_kmap_ga.ga_len;
	}
	vim_free(line);
    }

    /*
     * setup ":lnoremap" to map the keys
     */
    for (i = 0; i < curbuf->b_kmap_ga.ga_len; ++i)
    {
	vim_snprintf((char *)buf, sizeof(buf), "<buffer> %s %s",
				((kmap_T *)curbuf->b_kmap_ga.ga_data)[i].from,
				 ((kmap_T *)curbuf->b_kmap_ga.ga_data)[i].to);
	(void)do_map(MAPTYPE_NOREMAP, buf, MODE_LANGMAP, FALSE);
    }

    p_cpo = save_cpo;

    curbuf->b_kmap_state |= KEYMAP_LOADED;
    status_redraw_curbuf();
}

/*
 * Stop using 'keymap'.
 */
    static void
keymap_unload(void)
{
    char_u	buf[KMAP_MAXLEN + 10];
    int		i;
    char_u	*save_cpo = p_cpo;
    kmap_T	*kp;

    if (!(curbuf->b_kmap_state & KEYMAP_LOADED))
	return;

    // Set 'cpoptions' to "C" to avoid line continuation.
    p_cpo = (char_u *)"C";

    // clear the ":lmap"s
    kp = (kmap_T *)curbuf->b_kmap_ga.ga_data;
    for (i = 0; i < curbuf->b_kmap_ga.ga_len; ++i)
    {
	vim_snprintf((char *)buf, sizeof(buf), "<buffer> %s", kp[i].from);
	(void)do_map(MAPTYPE_UNMAP, buf, MODE_LANGMAP, FALSE);
    }
    keymap_clear(&curbuf->b_kmap_ga);

    p_cpo = save_cpo;

    ga_clear(&curbuf->b_kmap_ga);
    curbuf->b_kmap_state &= ~KEYMAP_LOADED;
    status_redraw_curbuf();
}

    void
keymap_clear(garray_T *kmap)
{
    int	    i;
    kmap_T  *kp = (kmap_T *)kmap->ga_data;

    for (i = 0; i < kmap->ga_len; ++i)
    {
	vim_free(kp[i].from);
	vim_free(kp[i].to);
    }
}
#endif // FEAT_KEYMAP
