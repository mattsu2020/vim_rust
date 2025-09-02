/* vi:set ts=8 sts=4 sw=4 noet:
 *
 * VIM - Vi IMproved	by Bram Moolenaar
 *
 * Do ":help uganda"  in Vim to read copying and usage conditions.
 * Do ":help credits" in Vim to see a list of people who contributed.
 * See README.txt for an overview of the Vim source code.
 */
/*
 * search.c: code for normal mode searching commands
 */

#include "vim.h"
#include "search_rs.h"
#include "rust_search.h"

#ifdef FEAT_EVAL
static void set_vv_searchforward(void);
static int first_submatch(regmmatch_T *rp);
#ifdef FEAT_FIND_ID
static char_u *get_line_and_copy(linenr_T lnum, char_u *buf);
static void show_pat_in_path(char_u *, int, int, int, FILE *, linenr_T *, long);
#endif



#ifdef FEAT_SEARCH_EXTRA
static void save_incsearch_state(void);
static void restore_incsearch_state(void);
#endif
static int check_prevcol(char_u *linep, int col, int ch, int *prevcol);
static int find_rawstring_end(char_u *linep, pos_T *startpos, pos_T *endpos);
static void find_mps_values(int *initc, int *findc, int *backwards, int switchit);
static int is_zero_width(char_u *pattern, size_t patternlen, int move, pos_T *cur, int direction);
static void cmdline_search_stat(int dirc, pos_T *pos, pos_T *cursor_pos, int show_top_bot_msg, char_u *msgbuf, size_t msgbuflen, int recompute, int maxcount, long timeout);
static void update_search_stat(int dirc, pos_T *pos, pos_T *cursor_pos, searchstat_T *stat, int recompute, int maxcount, long timeout);

#define SEARCH_STAT_DEF_TIMEOUT 40L
// 'W ':  2 +
// '[>9999/>9999]': 13 + 1 (NUL)
#define SEARCH_STAT_BUF_LEN 16

/*
 * This file contains various searching-related routines. These fall into
 * three groups:
 * 1. string searches (for /, ?, n, and N)
 * 2. character searches within a single line (for f, F, t, T, etc)
 * 3. "other" kinds of searches like the '%' command, and 'word' searches.
 */

/*
 * String searches
 *
 * The string search functions are divided into two levels:
 * lowest:  searchit(); uses an pos_T for starting position and found match.
 * Highest: do_search(); uses curwin->w_cursor; calls searchit().
 *
 * The last search pattern is remembered for repeating the same search.
 * This pattern is shared between the :g, :s, ? and / commands.
 * This is in search_regcomp().
 *
 * The actual string matching is done using a heavily modified version of
 * Henry Spencer's regular expression library.  See regexp.c.
 */

/*
 * Two search patterns are remembered: One for the :substitute command and
 * one for other searches.  last_idx points to the one that was used the last
 * time.
 */
static spat_T spats[2] =
{
    {NULL, 0, TRUE, FALSE, {'/', 0, 0, 0L}},	// last used search pat
    {NULL, 0, TRUE, FALSE, {'/', 0, 0, 0L}}	// last used substitute pat
};

static int last_idx = 0;	// index in spats[] for RE_LAST

static char_u lastc[2] = {NUL, NUL};	// last character searched for
static int lastcdir = FORWARD;		// last direction of character search
static int last_t_cmd = TRUE;		// last search t_cmd
static char_u	lastc_bytes[MB_MAXBYTES + 1];
static int	lastc_bytelen = 1;	// >1 for multi-byte char

// copy of spats[], for keeping the search patterns while executing autocmds
static spat_T	    saved_spats[ARRAY_LENGTH(spats)];
static char_u	    *saved_mr_pattern = NULL;
static size_t	    saved_mr_patternlen = 0;
# ifdef FEAT_SEARCH_EXTRA
static int	    saved_spats_last_idx = 0;
static int	    saved_spats_no_hlsearch = 0;
# endif

// allocated copy of pattern used by search_regcomp()
static char_u	    *mr_pattern = NULL;
static size_t	    mr_patternlen = 0;

#ifdef FEAT_FIND_ID
/*
 * Type used by find_pattern_in_path() to remember which included files have
 * been searched already.
 */
typedef struct SearchedFile
{
    FILE	*fp;		// File pointer
    char_u	*name;		// Full name of file
    linenr_T	lnum;		// Line we were up to in file
    int		matched;	// Found a match in this file
} SearchedFile;
#endif

/*
 * translate search pattern for vim_regcomp()
 *
 * pat_save == RE_SEARCH: save pat in spats[RE_SEARCH].pat (normal search cmd)
 * pat_save == RE_SUBST: save pat in spats[RE_SUBST].pat (:substitute command)
 * pat_save == RE_BOTH: save pat in both patterns (:global command)
 * pat_use  == RE_SEARCH: use previous search pattern if "pat" is NULL
 * pat_use  == RE_SUBST: use previous substitute pattern if "pat" is NULL
 * pat_use  == RE_LAST: use last used pattern if "pat" is NULL
 * options & SEARCH_HIS: put search string in history
 * options & SEARCH_KEEP: keep previous search pattern
 *
 * returns FAIL if failed, OK otherwise.
 */
    int
search_regcomp(
    char_u	*pat,
    size_t	patlen,
    char_u	**used_pat,
    int		pat_save,
    int		pat_use,
    int		options,
    regmmatch_T	*regmatch)	// return: pattern and ignore-case flag
{
    int		magic;

    rc_did_emsg = FALSE;
    magic = magic_isset();

    /*
     * If no pattern given, use a previously defined pattern.
     */
    if (pat == NULL || *pat == NUL)
    {
	int i;

	if (pat_use == RE_LAST)
	    i = last_idx;
	else
	    i = pat_use;
	if (spats[i].pat == NULL)	// pattern was never defined
	{
	    if (pat_use == RE_SUBST)
		emsg(_(e_no_previous_substitute_regular_expression));
	    else
		emsg(_(e_no_previous_regular_expression));
	    rc_did_emsg = TRUE;
	    return FAIL;
	}
	pat = spats[i].pat;
	patlen = spats[i].patlen;
	magic = spats[i].magic;
	no_smartcase = spats[i].no_scs;
    }
    else if (options & SEARCH_HIS)	// put new pattern in history
	add_to_history(HIST_SEARCH, pat, patlen, TRUE, NUL);

    if (used_pat)
	*used_pat = pat;

    vim_free(mr_pattern);
#ifdef FEAT_RIGHTLEFT
    if (curwin->w_p_rl && *curwin->w_p_rlc == 's')
	mr_pattern = reverse_text(pat);
    else
#endif
	mr_pattern = vim_strnsave(pat, patlen);
    if (mr_pattern == NULL)
	mr_patternlen = 0;
    else
	mr_patternlen = patlen;

    /*
     * Save the currently used pattern in the appropriate place,
     * unless the pattern should not be remembered.
     */
    if (!(options & SEARCH_KEEP)
			       && (cmdmod.cmod_flags & CMOD_KEEPPATTERNS) == 0)
    {
	// search or global command
	if (pat_save == RE_SEARCH || pat_save == RE_BOTH)
	    save_re_pat(RE_SEARCH, pat, patlen, magic);
	// substitute or global command
	if (pat_save == RE_SUBST || pat_save == RE_BOTH)
	    save_re_pat(RE_SUBST, pat, patlen, magic);
    }

    regmatch->rmm_ic = ignorecase(pat);
    regmatch->rmm_maxcol = 0;
    regmatch->regprog = vim_regcomp(pat, magic ? RE_MAGIC : 0);
    if (regmatch->regprog == NULL)
	return FAIL;
    return OK;
}

/*
 * Get search pattern used by search_regcomp().
 */
    char_u *
get_search_pat(void)
{
    return mr_pattern;
}

    void
save_re_pat(int idx, char_u *pat, size_t patlen, int magic)
{
    if (spats[idx].pat == pat)
	return;

    vim_free(spats[idx].pat);
    spats[idx].pat = vim_strnsave(pat, patlen);
    if (spats[idx].pat == NULL)
	spats[idx].patlen = 0;
    else
	spats[idx].patlen = patlen;
    spats[idx].magic = magic;
    spats[idx].no_scs = no_smartcase;
    last_idx = idx;
#ifdef FEAT_SEARCH_EXTRA
    // If 'hlsearch' set and search pat changed: need redraw.
    if (p_hls)
	redraw_all_later(UPD_SOME_VALID);
    set_no_hlsearch(FALSE);
#endif
}

/*
 * Save the search patterns, so they can be restored later.
 * Used before/after executing autocommands and user functions.
 */
static int save_level = 0;

    void
save_search_patterns(void)
{
    int i;

    if (save_level++ != 0)
	return;

    for (i = 0; i < (int)ARRAY_LENGTH(spats); ++i)
    {
	saved_spats[i] = spats[i];
	if (spats[i].pat != NULL)
	{
	    saved_spats[i].pat = vim_strnsave(spats[i].pat, spats[i].patlen);
	    if (saved_spats[i].pat == NULL)
		saved_spats[i].patlen = 0;
	    else
		saved_spats[i].patlen = spats[i].patlen;
	}
    }
    if (mr_pattern == NULL)
	saved_mr_pattern = NULL;
    else
	saved_mr_pattern = vim_strnsave(mr_pattern, mr_patternlen);
    if (saved_mr_pattern == NULL)
	saved_mr_patternlen = 0;
    else
	saved_mr_patternlen = mr_patternlen;
#ifdef FEAT_SEARCH_EXTRA
    saved_spats_last_idx = last_idx;
    saved_spats_no_hlsearch = no_hlsearch;
#endif
}

    void
restore_search_patterns(void)
{
    int i;

    if (--save_level != 0)
	return;

    for (i = 0; i < (int)ARRAY_LENGTH(spats); ++i)
    {
	vim_free(spats[i].pat);
	spats[i] = saved_spats[i];
    }
#if defined(FEAT_EVAL)
    set_vv_searchforward();
#endif
    vim_free(mr_pattern);
    mr_pattern = saved_mr_pattern;
    mr_patternlen = saved_mr_patternlen;
#ifdef FEAT_SEARCH_EXTRA
    last_idx = saved_spats_last_idx;
    set_no_hlsearch(saved_spats_no_hlsearch);
#endif
}

#if defined(EXITFREE) || defined(PROTO)
    void
free_search_patterns(void)
{
    int i;

    for (i = 0; i < (int)ARRAY_LENGTH(spats); ++i)
    {
	VIM_CLEAR(spats[i].pat);
	spats[i].patlen = 0;
    }
    VIM_CLEAR(mr_pattern);
    mr_patternlen = 0;
}
#endif

#ifdef FEAT_SEARCH_EXTRA
// copy of spats[RE_SEARCH], for keeping the search patterns while incremental
// searching
static spat_T	    saved_last_search_spat;
static int	    did_save_last_search_spat = 0;
static int	    saved_last_idx = 0;
static int	    saved_no_hlsearch = 0;
static int	    saved_search_match_endcol;
static int	    saved_search_match_lines;

/*
 * Save and restore the search pattern for incremental highlight search
 * feature.
 *
 * It's similar to but different from save_search_patterns() and
 * restore_search_patterns(), because the search pattern must be restored when
 * canceling incremental searching even if it's called inside user functions.
 */
    void
save_last_search_pattern(void)
{
    if (++did_save_last_search_spat != 1)
	// nested call, nothing to do
	return;

    saved_last_search_spat = spats[RE_SEARCH];
    if (spats[RE_SEARCH].pat != NULL)
    {
	saved_last_search_spat.pat = vim_strnsave(spats[RE_SEARCH].pat, spats[RE_SEARCH].patlen);
	if (saved_last_search_spat.pat == NULL)
	    saved_last_search_spat.patlen = 0;
	else
	    saved_last_search_spat.patlen = spats[RE_SEARCH].patlen;
    }
    saved_last_idx = last_idx;
    saved_no_hlsearch = no_hlsearch;
}

    void
restore_last_search_pattern(void)
{
    if (--did_save_last_search_spat > 0)
	// nested call, nothing to do
	return;
    if (did_save_last_search_spat != 0)
    {
	iemsg("restore_last_search_pattern() called more often than save_last_search_pattern()");
	return;
    }

    vim_free(spats[RE_SEARCH].pat);
    spats[RE_SEARCH] = saved_last_search_spat;
    saved_last_search_spat.pat = NULL;
    saved_last_search_spat.patlen = 0;
# if defined(FEAT_EVAL)
    set_vv_searchforward();
# endif
    last_idx = saved_last_idx;
    set_no_hlsearch(saved_no_hlsearch);
}

/*
 * Save and restore the incsearch highlighting variables.
 * This is required so that calling searchcount() at does not invalidate the
 * incsearch highlighting.
 */
    static void
save_incsearch_state(void)
{
    saved_search_match_endcol = search_match_endcol;
    saved_search_match_lines  = search_match_lines;
}

    static void
restore_incsearch_state(void)
{
    search_match_endcol = saved_search_match_endcol;
    search_match_lines  = saved_search_match_lines;
}

    char_u *
last_search_pattern(void)
{
    return spats[RE_SEARCH].pat;
}

    size_t
last_search_pattern_len(void)
{
    return spats[RE_SEARCH].patlen;
}
#endif

/*
 * Return TRUE when case should be ignored for search pattern "pat".
 * Uses the 'ignorecase' and 'smartcase' options.
 */
    int
ignorecase(char_u *pat)
{
    return ignorecase_opt(pat, p_ic, p_scs);
}

/*
 * As ignorecase() but pass the "ic" and "scs" flags.
 */
    int
ignorecase_opt(char_u *pat, int ic_in, int scs)
{
    int		ic = ic_in;

    if (ic && !no_smartcase && scs
			    && !(ctrl_x_mode_not_default() && curbuf->b_p_inf))
	ic = !pat_has_uppercase(pat);
    no_smartcase = FALSE;

    return ic;
}

/*
 * Return TRUE if pattern "pat" has an uppercase character.
 */
    int
pat_has_uppercase(char_u *pat)
{
    char_u *p = pat;
    magic_T magic_val = MAGIC_ON;

    // get the magicness of the pattern
    (void)skip_regexp_ex(pat, NUL, magic_isset(), NULL, NULL, &magic_val);

    while (*p != NUL)
    {
	int		l;

	if (has_mbyte && (l = (*mb_ptr2len)(p)) > 1)
	{
	    if (enc_utf8 && utf_isupper(utf_ptr2char(p)))
		return TRUE;
	    p += l;
	}
	else if (*p == '\\' && magic_val <= MAGIC_ON)
	{
	    if (p[1] == '_' && p[2] != NUL)  // skip "\_X"
		p += 3;
	    else if (p[1] == '%' && p[2] != NUL)  // skip "\%X"
		p += 3;
	    else if (p[1] != NUL)  // skip "\X"
		p += 2;
	    else
		p += 1;
	}
	else if ((*p == '%' || *p == '_') && magic_val == MAGIC_ALL)
	{
	    if (p[1] != NUL)  // skip "_X" and %X
		p += 2;
	    else
		p++;
	}
	else if (MB_ISUPPER(*p))
	    return TRUE;
	else
	    ++p;
    }
    return FALSE;
}

#if defined(FEAT_EVAL) || defined(PROTO)
    char_u *
last_csearch(void)
{
    return lastc_bytes;
}

    int
last_csearch_forward(void)
{
    return lastcdir == FORWARD;
}

    int
last_csearch_until(void)
{
    return last_t_cmd == TRUE;
}

    void
set_last_csearch(int c, char_u *s, int len)
{
    *lastc = c;
    lastc_bytelen = len;
    if (len)
	memcpy(lastc_bytes, s, len);
    else
	CLEAR_FIELD(lastc_bytes);
}
#endif

    void
set_csearch_direction(int cdir)
{
    lastcdir = cdir;
}

    void
set_csearch_until(int t_cmd)
{
    last_t_cmd = t_cmd;
}

    char_u *
last_search_pat(void)
{
    return spats[last_idx].pat;
}

/*
 * Reset search direction to forward.  For "gd" and "gD" commands.
 */
    void
reset_search_dir(void)
{
    spats[0].off.dir = '/';
#if defined(FEAT_EVAL)
    set_vv_searchforward();
#endif
}

#if defined(FEAT_EVAL) || defined(FEAT_VIMINFO)
/*
 * Set the last search pattern.  For ":let @/ =" and viminfo.
 * Also set the saved search pattern, so that this works in an autocommand.
 */
    void
set_last_search_pat(
    char_u	*s,
    int		idx,
    int		magic,
    int		setlast)
{
    vim_free(spats[idx].pat);
    // An empty string means that nothing should be matched.
    if (*s == NUL)
	spats[idx].pat = NULL;
    else
    {
	spats[idx].patlen = STRLEN(s);
	spats[idx].pat = vim_strnsave(s, spats[idx].patlen);
    }
    if (spats[idx].pat == NULL)
	spats[idx].patlen = 0;
    spats[idx].magic = magic;
    spats[idx].no_scs = FALSE;
    spats[idx].off.dir = '/';
#if defined(FEAT_EVAL)
    set_vv_searchforward();
#endif
    spats[idx].off.line = FALSE;
    spats[idx].off.end = FALSE;
    spats[idx].off.off = 0;
    if (setlast)
	last_idx = idx;
    if (save_level)
    {
	vim_free(saved_spats[idx].pat);
	saved_spats[idx] = spats[0];
	if (spats[idx].pat == NULL)
	    saved_spats[idx].pat = NULL;
	else
	    saved_spats[idx].pat = vim_strnsave(spats[idx].pat, spats[idx].patlen);
	if (saved_spats[idx].pat == NULL)
	    saved_spats[idx].patlen = 0;
	else
	    saved_spats[idx].patlen = spats[idx].patlen;
# ifdef FEAT_SEARCH_EXTRA
	saved_spats_last_idx = last_idx;
# endif
    }
# ifdef FEAT_SEARCH_EXTRA
    // If 'hlsearch' set and search pat changed: need redraw.
    if (p_hls && idx == last_idx && !no_hlsearch)
	redraw_all_later(UPD_SOME_VALID);
# endif
}
#endif

#ifdef FEAT_SEARCH_EXTRA
/*
 * Get a regexp program for the last used search pattern.
 * This is used for highlighting all matches in a window.
 * Values returned in regmatch->regprog and regmatch->rmm_ic.
 */
    void
last_pat_prog(regmmatch_T *regmatch)
{
    if (spats[last_idx].pat == NULL)
    {
	regmatch->regprog = NULL;
	return;
    }
    ++emsg_off;		// So it doesn't beep if bad expr
    (void)search_regcomp((char_u *)"", 0, NULL, 0, last_idx, SEARCH_KEEP, regmatch);
    --emsg_off;
}
#endif

/*
 * Lowest level search function.
 * Search for 'count'th occurrence of pattern "pat" in direction "dir".
 * Start at position "pos" and return the found position in "pos".
 *
 * if (options & SEARCH_MSG) == 0 don't give any messages
 * if (options & SEARCH_MSG) == SEARCH_NFMSG don't give 'notfound' messages
 * if (options & SEARCH_MSG) == SEARCH_MSG give all messages
 * if (options & SEARCH_HIS) put search pattern in history
 * if (options & SEARCH_END) return position at end of match
 * if (options & SEARCH_START) accept match at pos itself
 * if (options & SEARCH_KEEP) keep previous search pattern
 * if (options & SEARCH_FOLD) match only once in a closed fold
 * if (options & SEARCH_PEEK) check for typed char, cancel search
 * if (options & SEARCH_COL) start at pos->col instead of zero
 *
 * Return FAIL (zero) for failure, non-zero for success.
 * When FEAT_EVAL is defined, returns the index of the first matching
 * subpattern plus one; one if there was none.
 */
int searchit(
    win_T *win,
    buf_T *buf,
    pos_T *pos,
    pos_T *end_pos,
    int dir,
    char_u *pat,
    size_t patlen,
    long count,
    int options,
    int pat_use,
    searchit_arg_T *extra_arg)
{
    return rust_searchit(win, buf, pos, end_pos, dir, pat, patlen, count,
                         options, pat_use, extra_arg);
}

#if defined(FEAT_EVAL) || defined(FEAT_PROTO)
    void
set_search_direction(int cdir)
{
    spats[0].off.dir = cdir;
}

    static void
set_vv_searchforward(void)
{
    set_vim_var_nr(VV_SEARCHFORWARD, (long)(spats[0].off.dir == '/'));
}

/*
 * Return the number of the first subpat that matched.
 * Return zero if none of them matched.
 */
    static int
first_submatch(regmmatch_T *rp)
{
    int		submatch;

    for (submatch = 1; ; ++submatch)
    {
	if (rp->startpos[submatch].lnum >= 0)
	    break;
	if (submatch == 9)
	{
	    submatch = 0;
	    break;
	}
    }
    return submatch;
}
#endif

/*
 * Highest level string search function.
 * Search for the 'count'th occurrence of pattern 'pat' in direction 'dirc'
 *		  If 'dirc' is 0: use previous dir.
 *    If 'pat' is NULL or empty : use previous string.
 *    If 'options & SEARCH_REV' : go in reverse of previous dir.
 *    If 'options & SEARCH_ECHO': echo the search command and handle options
 *    If 'options & SEARCH_MSG' : may give error message
 *    If 'options & SEARCH_OPT' : interpret optional flags
 *    If 'options & SEARCH_HIS' : put search pattern in history
 *    If 'options & SEARCH_NOOF': don't add offset to position
 *    If 'options & SEARCH_MARK': set previous context mark
 *    If 'options & SEARCH_KEEP': keep previous search pattern
 *    If 'options & SEARCH_START': accept match at curpos itself
 *    If 'options & SEARCH_PEEK': check for typed char, cancel search
 *
 * Careful: If spats[0].off.line == TRUE and spats[0].off.off == 0 this
 * makes the movement linewise without moving the match position.
 *
 * Return 0 for failure, 1 for found, 2 for found and line offset added.
 */
int do_search(
    oparg_T *oap,
    int dirc,
    int search_delim,
    char_u *pat,
    size_t patlen,
    long count,
    int options,
    searchit_arg_T *sia)
{
    return rust_do_search(curwin, curbuf, &curwin->w_cursor,
                           oap, dirc, search_delim, pat, patlen,
                           count, options, sia);
}

/*
 * search_for_exact_line(buf, pos, dir, pat)
 *
 * Search for a line starting with the given pattern (ignoring leading
 * white-space), starting from pos and going in direction "dir". "pos" will
 * contain the position of the match found.    Blank lines match only if
 * ADDING is set.  If p_ic is set then the pattern must be in lowercase.
 * Return OK for success, or FAIL if no line found.
 */
    int
search_for_exact_line(
    buf_T	*buf,
    pos_T	*pos,
    int		dir,
    char_u	*pat)
{
    linenr_T	start = 0;
    char_u	*ptr;
    char_u	*p;

    if (buf->b_ml.ml_line_count == 0)
	return FAIL;
    for (;;)
    {
	pos->lnum += dir;
	if (pos->lnum < 1)
	{
	    if (p_ws)
	    {
		pos->lnum = buf->b_ml.ml_line_count;
		if (!shortmess(SHM_SEARCH))
		    give_warning((char_u *)_(top_bot_msg), TRUE);
	    }
	    else
	    {
		pos->lnum = 1;
		break;
	    }
	}
	else if (pos->lnum > buf->b_ml.ml_line_count)
	{
	    if (p_ws)
	    {
		pos->lnum = 1;
		if (!shortmess(SHM_SEARCH))
		    give_warning((char_u *)_(bot_top_msg), TRUE);
	    }
	    else
	    {
		pos->lnum = 1;
		break;
	    }
	}
	if (pos->lnum == start)
	    break;
	if (start == 0)
	    start = pos->lnum;
	ptr = ml_get_buf(buf, pos->lnum, FALSE);
	p = skipwhite(ptr);
	pos->col = (colnr_T) (p - ptr);

	// when adding lines the matching line may be empty but it is not
	// ignored because we are interested in the next line -- Acevedo
	if (compl_status_adding() && !compl_status_sol())
	{
	    if ((p_ic ? MB_STRICMP(p, pat) : STRCMP(p, pat)) == 0)
		return OK;
	}
	else if (*p != NUL)	// ignore empty lines
	{	// expanding lines or words
	    if ((p_ic ? MB_STRNICMP(p, pat, ins_compl_len())
				   : STRNCMP(p, pat, ins_compl_len())) == 0)
		return OK;
	}
    }
    return FAIL;
}

/*
 * Character Searches
 */

/*
 * Search for a character in a line.  If "t_cmd" is FALSE, move to the
 * position of the character, otherwise move to just before the char.
 * Do this "cap->count1" times.
 * Return FAIL or OK.
 */
    int
searchc(cmdarg_T *cap, int t_cmd)
{
    int			c = cap->nchar;	// char to search for
    int			dir = cap->arg;	// TRUE for searching forward
    long		count = cap->count1;	// repeat count
    int			col;
    char_u		*p;
    int			len;
    int			stop = TRUE;

    if (c != NUL)	// normal search: remember args for repeat
    {
	if (!KeyStuffed)    // don't remember when redoing
	{
	    *lastc = c;
	    set_csearch_direction(dir);
	    set_csearch_until(t_cmd);
	    lastc_bytelen = (*mb_char2bytes)(c, lastc_bytes);
	    if (cap->ncharC1 != 0)
	    {
		lastc_bytelen += (*mb_char2bytes)(cap->ncharC1,
			lastc_bytes + lastc_bytelen);
		if (cap->ncharC2 != 0)
		    lastc_bytelen += (*mb_char2bytes)(cap->ncharC2,
			    lastc_bytes + lastc_bytelen);
	    }
	}
    }
    else		// repeat previous search
    {
	if (*lastc == NUL && lastc_bytelen <= 1)
	    return FAIL;
	if (dir)	// repeat in opposite direction
	    dir = -lastcdir;
	else
	    dir = lastcdir;
	t_cmd = last_t_cmd;
	c = *lastc;
	// For multi-byte re-use last lastc_bytes[] and lastc_bytelen.

	// Force a move of at least one char, so ";" and "," will move the
	// cursor, even if the cursor is right in front of char we are looking
	// at.
	if (vim_strchr(p_cpo, CPO_SCOLON) == NULL && count == 1 && t_cmd)
	    stop = FALSE;
    }

    if (dir == BACKWARD)
	cap->oap->inclusive = FALSE;
    else
	cap->oap->inclusive = TRUE;

    p = ml_get_curline();
    col = curwin->w_cursor.col;
    len = ml_get_curline_len();

    while (count--)
    {
	if (has_mbyte)
	{
	    for (;;)
	    {
		if (dir > 0)
		{
		    col += (*mb_ptr2len)(p + col);
		    if (col >= len)
			return FAIL;
		}
		else
		{
		    if (col == 0)
			return FAIL;
		    col -= (*mb_head_off)(p, p + col - 1) + 1;
		}
		if (lastc_bytelen <= 1)
		{
		    if (p[col] == c && stop)
			break;
		}
		else if (STRNCMP(p + col, lastc_bytes, lastc_bytelen) == 0
								       && stop)
		    break;
		stop = TRUE;
	    }
	}
	else
	{
	    for (;;)
	    {
		if ((col += dir) < 0 || col >= len)
		    return FAIL;
		if (p[col] == c && stop)
		    break;
		stop = TRUE;
	    }
	}
    }

    if (t_cmd)
    {
	// backup to before the character (possibly double-byte)
	col -= dir;
	if (has_mbyte)
	{
	    if (dir < 0)
		// Landed on the search char which is lastc_bytelen long
		col += lastc_bytelen - 1;
	    else
		// To previous char, which may be multi-byte.
		col -= (*mb_head_off)(p, p + col);
	}
    }
    curwin->w_cursor.col = col;

    return OK;
}

/*
 * "Other" Searches
 */

/*
 * findmatch - find the matching paren or brace
 *
 * Improvement over vi: Braces inside quotes are ignored.
 */
    pos_T *
findmatch(oparg_T *oap, int initc)
{
    return findmatchlimit(oap, initc, 0, 0);
}

/*
 * Return TRUE if the character before "linep[col]" equals "ch".
 * Return FALSE if "col" is zero.
 * Update "*prevcol" to the column of the previous character, unless "prevcol"
 * is NULL.
 * Handles multibyte string correctly.
 */
    static int
check_prevcol(
    char_u	*linep,
    int		col,
    int		ch,
    int		*prevcol)
{
    --col;
    if (col > 0 && has_mbyte)
	col -= (*mb_head_off)(linep, linep + col);
    if (prevcol)
	*prevcol = col;
    return (col >= 0 && linep[col] == ch) ? TRUE : FALSE;
}

/*
 * Raw string start is found at linep[startpos.col - 1].
 * Return TRUE if the matching end can be found between startpos and endpos.
 */
    static int
find_rawstring_end(char_u *linep, pos_T *startpos, pos_T *endpos)
{
    char_u	*p;
    char_u	*delim_copy;
    size_t	delim_len;
    linenr_T	lnum;
    int		found = FALSE;

    for (p = linep + startpos->col + 1; *p && *p != '('; ++p)
	;
    delim_len = (p - linep) - startpos->col - 1;
    delim_copy = vim_strnsave(linep + startpos->col + 1, delim_len);
    if (delim_copy == NULL)
	return FALSE;
    for (lnum = startpos->lnum; lnum <= endpos->lnum; ++lnum)
    {
	char_u *line = ml_get(lnum);

	for (p = line + (lnum == startpos->lnum
					    ? startpos->col + 1 : 0); *p; ++p)
	{
	    if (lnum == endpos->lnum && (colnr_T)(p - line) >= endpos->col)
		break;
	    if (*p == ')' && STRNCMP(delim_copy, p + 1, delim_len) == 0
			  && p[delim_len + 1] == '"')
	    {
		found = TRUE;
		break;
	    }
	}
	if (found)
	    break;
    }
    vim_free(delim_copy);
    return found;
}

/*
 * Check matchpairs option for "*initc".
 * If there is a match set "*initc" to the matching character and "*findc" to
 * the opposite character.  Set "*backwards" to the direction.
 * When "switchit" is TRUE swap the direction.
 */
    static void
find_mps_values(
    int	    *initc,
    int	    *findc,
    int	    *backwards,
    int	    switchit)
{
    char_u	*ptr;

    ptr = curbuf->b_p_mps;
    while (*ptr != NUL)
    {
	if (has_mbyte)
	{
	    char_u *prev;

	    if (mb_ptr2char(ptr) == *initc)
	    {
		if (switchit)
		{
		    *findc = *initc;
		    *initc = mb_ptr2char(ptr + mb_ptr2len(ptr) + 1);
		    *backwards = TRUE;
		}
		else
		{
		    *findc = mb_ptr2char(ptr + mb_ptr2len(ptr) + 1);
		    *backwards = FALSE;
		}
		return;
	    }
	    prev = ptr;
	    ptr += mb_ptr2len(ptr) + 1;
	    if (mb_ptr2char(ptr) == *initc)
	    {
		if (switchit)
		{
		    *findc = *initc;
		    *initc = mb_ptr2char(prev);
		    *backwards = FALSE;
		}
		else
		{
		    *findc = mb_ptr2char(prev);
		    *backwards = TRUE;
		}
		return;
	    }
	    ptr += mb_ptr2len(ptr);
	}
	else
	{
	    if (*ptr == *initc)
	    {
		if (switchit)
		{
		    *backwards = TRUE;
		    *findc = *initc;
		    *initc = ptr[2];
		}
		else
		{
		    *backwards = FALSE;
		    *findc = ptr[2];
		}
		return;
	    }
	    ptr += 2;
	    if (*ptr == *initc)
	    {
		if (switchit)
		{
		    *backwards = FALSE;
		    *findc = *initc;
		    *initc = ptr[-2];
		}
		else
		{
		    *backwards = TRUE;
		    *findc =  ptr[-2];
		}
		return;
	    }
	    ++ptr;
	}
	if (*ptr == ',')
	    ++ptr;
    }
}

/*
 * findmatchlimit -- find the matching paren or brace, if it exists within
 * maxtravel lines of the cursor.  A maxtravel of 0 means search until falling
 * off the edge of the file.
 *
 * "initc" is the character to find a match for.  NUL means to find the
 * character at or after the cursor. Special values:
 * '*'  look for C-style comment / *
 * '/'  look for C-style comment / *, ignoring comment-end
 * '#'  look for preprocessor directives
 * 'R'  look for raw string start: R"delim(text)delim" (only backwards)
 *
 * flags: FM_BACKWARD	search backwards (when initc is '/', '*' or '#')
 *	  FM_FORWARD	search forwards (when initc is '/', '*' or '#')
 *	  FM_BLOCKSTOP	stop at start/end of block ({ or } in column 0)
 *	  FM_SKIPCOMM	skip comments (not implemented yet!)
 *
 * "oap" is only used to set oap->motion_type for a linewise motion, it can be
 * NULL
 */
    pos_T *
findmatchlimit(
    oparg_T	*oap,
    int		initc,
    int		flags,
    int		maxtravel)
{
    static pos_T pos;			// current search position
    int		findc = 0;		// matching brace
    int		c;
    int		count = 0;		// cumulative number of braces
    int		backwards = FALSE;	// init for gcc
    int		raw_string = FALSE;	// search for raw string
    int		inquote = FALSE;	// TRUE when inside quotes
    char_u	*linep;			// pointer to current line
    char_u	*ptr;
    int		do_quotes;		// check for quotes in current line
    int		at_start;		// do_quotes value at start position
    int		hash_dir = 0;		// Direction searched for # things
    int		comment_dir = 0;	// Direction searched for comments
    pos_T	match_pos;		// Where last slash-star was found
    int		start_in_quotes;	// start position is in quotes
    int		traveled = 0;		// how far we've searched so far
    int		ignore_cend = FALSE;    // ignore comment end
    int		cpo_match;		// vi compatible matching
    int		cpo_bsl;		// don't recognize backslashes
    int		match_escaped = 0;	// search for escaped match
    int		dir;			// Direction to search
    int		comment_col = MAXCOL;   // start of / / comment
    int		lispcomm = FALSE;	// inside of Lisp-style comment
    int		lisp = curbuf->b_p_lisp; // engage Lisp-specific hacks ;)

    pos = curwin->w_cursor;
    pos.coladd = 0;
    linep = ml_get(pos.lnum);

    cpo_match = (vim_strchr(p_cpo, CPO_MATCH) != NULL);
    cpo_bsl = (vim_strchr(p_cpo, CPO_MATCHBSL) != NULL);

    // Direction to search when initc is '/', '*' or '#'
    if (flags & FM_BACKWARD)
	dir = BACKWARD;
    else if (flags & FM_FORWARD)
	dir = FORWARD;
    else
	dir = 0;

    /*
     * if initc given, look in the table for the matching character
     * '/' and '*' are special cases: look for start or end of comment.
     * When '/' is used, we ignore running backwards into an star-slash, for
     * "[*" command, we just want to find any comment.
     */
    if (initc == '/' || initc == '*' || initc == 'R')
    {
	comment_dir = dir;
	if (initc == '/')
	    ignore_cend = TRUE;
	backwards = (dir == FORWARD) ? FALSE : TRUE;
	raw_string = (initc == 'R');
	initc = NUL;
    }
    else if (initc != '#' && initc != NUL)
    {
	find_mps_values(&initc, &findc, &backwards, TRUE);
	if (dir)
	    backwards = (dir == FORWARD) ? FALSE : TRUE;
	if (findc == NUL)
	    return NULL;
    }
    else
    {
	/*
	 * Either initc is '#', or no initc was given and we need to look
	 * under the cursor.
	 */
	if (initc == '#')
	{
	    hash_dir = dir;
	}
	else
	{
	    /*
	     * initc was not given, must look for something to match under
	     * or near the cursor.
	     * Only check for special things when 'cpo' doesn't have '%'.
	     */
	    if (!cpo_match)
	    {
		// Are we before or at #if, #else etc.?
		ptr = skipwhite(linep);
		if (*ptr == '#' && pos.col <= (colnr_T)(ptr - linep))
		{
		    ptr = skipwhite(ptr + 1);
		    if (   STRNCMP(ptr, "if", 2) == 0
			|| STRNCMP(ptr, "endif", 5) == 0
			|| STRNCMP(ptr, "el", 2) == 0)
			hash_dir = 1;
		}

		// Are we on a comment?
		else if (linep[pos.col] == '/')
		{
		    if (linep[pos.col + 1] == '*')
		    {
			comment_dir = FORWARD;
			backwards = FALSE;
			pos.col++;
		    }
		    else if (pos.col > 0 && linep[pos.col - 1] == '*')
		    {
			comment_dir = BACKWARD;
			backwards = TRUE;
			pos.col--;
		    }
		}
		else if (linep[pos.col] == '*')
		{
		    if (linep[pos.col + 1] == '/')
		    {
			comment_dir = BACKWARD;
			backwards = TRUE;
		    }
		    else if (pos.col > 0 && linep[pos.col - 1] == '/')
		    {
			comment_dir = FORWARD;
			backwards = FALSE;
		    }
		}
	    }

	    /*
	     * If we are not on a comment or the # at the start of a line, then
	     * look for brace anywhere on this line after the cursor.
	     */
	    if (!hash_dir && !comment_dir)
	    {
		/*
		 * Find the brace under or after the cursor.
		 * If beyond the end of the line, use the last character in
		 * the line.
		 */
		if (linep[pos.col] == NUL && pos.col)
		    --pos.col;
		for (;;)
		{
		    initc = PTR2CHAR(linep + pos.col);
		    if (initc == NUL)
			break;

		    find_mps_values(&initc, &findc, &backwards, FALSE);
		    if (findc)
			break;
		    pos.col += mb_ptr2len(linep + pos.col);
		}
		if (!findc)
		{
		    // no brace in the line, maybe use "  #if" then
		    if (!cpo_match && *skipwhite(linep) == '#')
			hash_dir = 1;
		    else
			return NULL;
		}
		else if (!cpo_bsl)
		{
		    int col, bslcnt = 0;

		    // Set "match_escaped" if there are an odd number of
		    // backslashes.
		    for (col = pos.col; check_prevcol(linep, col, '\\', &col);)
			bslcnt++;
		    match_escaped = (bslcnt & 1);
		}
	    }
	}
	if (hash_dir)
	{
	    /*
	     * Look for matching #if, #else, #elif, or #endif
	     */
	    if (oap != NULL)
		oap->motion_type = MLINE;   // Linewise for this case only
	    if (initc != '#')
	    {
		ptr = skipwhite(skipwhite(linep) + 1);
		if (STRNCMP(ptr, "if", 2) == 0 || STRNCMP(ptr, "el", 2) == 0)
		    hash_dir = 1;
		else if (STRNCMP(ptr, "endif", 5) == 0)
		    hash_dir = -1;
		else
		    return NULL;
	    }
	    pos.col = 0;
	    while (!got_int)
	    {
		if (hash_dir > 0)
		{
		    if (pos.lnum == curbuf->b_ml.ml_line_count)
			break;
		}
		else if (pos.lnum == 1)
		    break;
		pos.lnum += hash_dir;
		linep = ml_get(pos.lnum);
		line_breakcheck();	// check for CTRL-C typed
		ptr = skipwhite(linep);
		if (*ptr != '#')
		    continue;
		pos.col = (colnr_T) (ptr - linep);
		ptr = skipwhite(ptr + 1);
		if (hash_dir > 0)
		{
		    if (STRNCMP(ptr, "if", 2) == 0)
			count++;
		    else if (STRNCMP(ptr, "el", 2) == 0)
		    {
			if (count == 0)
			    return &pos;
		    }
		    else if (STRNCMP(ptr, "endif", 5) == 0)
		    {
			if (count == 0)
			    return &pos;
			count--;
		    }
		}
		else
		{
		    if (STRNCMP(ptr, "if", 2) == 0)
		    {
			if (count == 0)
			    return &pos;
			count--;
		    }
		    else if (initc == '#' && STRNCMP(ptr, "el", 2) == 0)
		    {
			if (count == 0)
			    return &pos;
		    }
		    else if (STRNCMP(ptr, "endif", 5) == 0)
			count++;
		}
	    }
	    return NULL;
	}
    }

#ifdef FEAT_RIGHTLEFT
    // This is just guessing: when 'rightleft' is set, search for a matching
    // paren/brace in the other direction.
    if (curwin->w_p_rl && vim_strchr((char_u *)"()[]{}<>", initc) != NULL)
	backwards = !backwards;
#endif

    do_quotes = -1;
    start_in_quotes = MAYBE;
    CLEAR_POS(&match_pos);

    // backward search: Check if this line contains a single-line comment
    if ((backwards && comment_dir) || lisp)
	comment_col = check_linecomment(linep);
    if (lisp && comment_col != MAXCOL && pos.col > (colnr_T)comment_col)
	lispcomm = TRUE;    // find match inside this comment

    while (!got_int)
    {
	/*
	 * Go to the next position, forward or backward. We could use
	 * inc() and dec() here, but that is much slower
	 */
	if (backwards)
	{
	    // char to match is inside of comment, don't search outside
	    if (lispcomm && pos.col < (colnr_T)comment_col)
		break;
	    if (pos.col == 0)		// at start of line, go to prev. one
	    {
		if (pos.lnum == 1)	// start of file
		    break;
		--pos.lnum;

		if (maxtravel > 0 && ++traveled > maxtravel)
		    break;

		linep = ml_get(pos.lnum);
		pos.col = ml_get_len(pos.lnum); // pos.col on trailing NUL
		do_quotes = -1;
		line_breakcheck();

		// Check if this line contains a single-line comment
		if (comment_dir || lisp)
		    comment_col = check_linecomment(linep);
		// skip comment
		if (lisp && comment_col != MAXCOL)
		    pos.col = comment_col;
	    }
	    else
	    {
		--pos.col;
		if (has_mbyte)
		    pos.col -= (*mb_head_off)(linep, linep + pos.col);
	    }
	}
	else				// forward search
	{
	    if (linep[pos.col] == NUL
		    // at end of line, go to next one
		    // For lisp don't search for match in comment
		    || (lisp && comment_col != MAXCOL
					   && pos.col == (colnr_T)comment_col))
	    {
		if (pos.lnum == curbuf->b_ml.ml_line_count  // end of file
			// line is exhausted and comment with it,
			// don't search for match in code
			 || lispcomm)
		    break;
		++pos.lnum;

		if (maxtravel && traveled++ > maxtravel)
		    break;

		linep = ml_get(pos.lnum);
		pos.col = 0;
		do_quotes = -1;
		line_breakcheck();
		if (lisp)   // find comment pos in new line
		    comment_col = check_linecomment(linep);
	    }
	    else
	    {
		if (has_mbyte)
		    pos.col += (*mb_ptr2len)(linep + pos.col);
		else
		    ++pos.col;
	    }
	}

	/*
	 * If FM_BLOCKSTOP given, stop at a '{' or '}' in column 0.
	 */
	if (pos.col == 0 && (flags & FM_BLOCKSTOP)
				       && (linep[0] == '{' || linep[0] == '}'))
	{
	    if (linep[0] == findc && count == 0)	// match!
		return &pos;
	    break;					// out of scope
	}

	if (comment_dir)
	{
	    // Note: comments do not nest, and we ignore quotes in them
	    // TODO: ignore comment brackets inside strings
	    if (comment_dir == FORWARD)
	    {
		if (linep[pos.col] == '*' && linep[pos.col + 1] == '/')
		{
		    pos.col++;
		    return &pos;
		}
	    }
	    else    // Searching backwards
	    {
		/*
		 * A comment may contain / * or / /, it may also start or end
		 * with / * /.	Ignore a / * after / / and after *.
		 */
		if (pos.col == 0)
		    continue;
		else if (raw_string)
		{
		    if (linep[pos.col - 1] == 'R'
			&& linep[pos.col] == '"'
			&& vim_strchr(linep + pos.col + 1, '(') != NULL)
		    {
			// Possible start of raw string. Now that we have the
			// delimiter we can check if it ends before where we
			// started searching, or before the previously found
			// raw string start.
			if (!find_rawstring_end(linep, &pos,
				  count > 0 ? &match_pos : &curwin->w_cursor))
			{
			    count++;
			    match_pos = pos;
			    match_pos.col--;
			}
			linep = ml_get(pos.lnum); // may have been released
		    }
		}
		else if (  linep[pos.col - 1] == '/'
			&& linep[pos.col] == '*'
			&& (pos.col == 1 || linep[pos.col - 2] != '*')
			&& (int)pos.col < comment_col)
		{
		    count++;
		    match_pos = pos;
		    match_pos.col--;
		}
		else if (linep[pos.col - 1] == '*' && linep[pos.col] == '/')
		{
		    if (count > 0)
			pos = match_pos;
		    else if (pos.col > 1 && linep[pos.col - 2] == '/'
					       && (int)pos.col <= comment_col)
			pos.col -= 2;
		    else if (ignore_cend)
			continue;
		    else
			return NULL;
		    return &pos;
		}
	    }
	    continue;
	}

	/*
	 * If smart matching ('cpoptions' does not contain '%'), braces inside
	 * of quotes are ignored, but only if there is an even number of
	 * quotes in the line.
	 */
	if (cpo_match)
	    do_quotes = 0;
	else if (do_quotes == -1)
	{
	    /*
	     * Count the number of quotes in the line, skipping \" and '"'.
	     * Watch out for "\\".
	     */
	    at_start = do_quotes;
	    for (ptr = linep; *ptr; ++ptr)
	    {
		if (ptr == linep + pos.col + backwards)
		    at_start = (do_quotes & 1);
		if (*ptr == '"'
			&& (ptr == linep || ptr[-1] != '\'' || ptr[1] != '\''))
		    ++do_quotes;
		if (*ptr == '\\' && ptr[1] != NUL)
		    ++ptr;
	    }
	    do_quotes &= 1;	    // result is 1 with even number of quotes

	    /*
	     * If we find an uneven count, check current line and previous
	     * one for a '\' at the end.
	     */
	    if (!do_quotes)
	    {
		inquote = FALSE;
		if (ptr[-1] == '\\')
		{
		    do_quotes = 1;
		    if (start_in_quotes == MAYBE)
		    {
			// Do we need to use at_start here?
			inquote = TRUE;
			start_in_quotes = TRUE;
		    }
		    else if (backwards)
			inquote = TRUE;
		}
		if (pos.lnum > 1)
		{
		    ptr = ml_get(pos.lnum - 1);
		    if (*ptr && *(ptr + ml_get_len(pos.lnum - 1) - 1) == '\\')
		    {
			do_quotes = 1;
			if (start_in_quotes == MAYBE)
			{
			    inquote = at_start;
			    if (inquote)
				start_in_quotes = TRUE;
			}
			else if (!backwards)
			    inquote = TRUE;
		    }

		    // ml_get() only keeps one line, need to get linep again
		    linep = ml_get(pos.lnum);
		}
	    }
	}
	if (start_in_quotes == MAYBE)
	    start_in_quotes = FALSE;

	/*
	 * If 'smartmatch' is set:
	 *   Things inside quotes are ignored by setting 'inquote'.  If we
	 *   find a quote without a preceding '\' invert 'inquote'.  At the
	 *   end of a line not ending in '\' we reset 'inquote'.
	 *
	 *   In lines with an uneven number of quotes (without preceding '\')
	 *   we do not know which part to ignore. Therefore we only set
	 *   inquote if the number of quotes in a line is even, unless this
	 *   line or the previous one ends in a '\'.  Complicated, isn't it?
	 */
	c = PTR2CHAR(linep + pos.col);
	switch (c)
	{
	case NUL:
	    // at end of line without trailing backslash, reset inquote
	    if (pos.col == 0 || linep[pos.col - 1] != '\\')
	    {
		inquote = FALSE;
		start_in_quotes = FALSE;
	    }
	    break;

	case '"':
	    // a quote that is preceded with an odd number of backslashes is
	    // ignored
	    if (do_quotes)
	    {
		int col;

		for (col = pos.col - 1; col >= 0; --col)
		    if (linep[col] != '\\')
			break;
		if ((((int)pos.col - 1 - col) & 1) == 0)
		{
		    inquote = !inquote;
		    start_in_quotes = FALSE;
		}
	    }
	    break;

	/*
	 * If smart matching ('cpoptions' does not contain '%'):
	 *   Skip things in single quotes: 'x' or '\x'.  Be careful for single
	 *   single quotes, eg jon's.  Things like '\233' or '\x3f' are not
	 *   skipped, there is never a brace in them.
	 *   Ignore this when finding matches for `'.
	 */
	case '\'':
	    if (!cpo_match && initc != '\'' && findc != '\'')
	    {
		if (backwards)
		{
		    if (pos.col > 1)
		    {
			if (linep[pos.col - 2] == '\'')
			{
			    pos.col -= 2;
			    break;
			}
			else if (linep[pos.col - 2] == '\\'
				  && pos.col > 2 && linep[pos.col - 3] == '\'')
			{
			    pos.col -= 3;
			    break;
			}
		    }
		}
		else if (linep[pos.col + 1])	// forward search
		{
		    if (linep[pos.col + 1] == '\\'
			   && linep[pos.col + 2] && linep[pos.col + 3] == '\'')
		    {
			pos.col += 3;
			break;
		    }
		    else if (linep[pos.col + 2] == '\'')
		    {
			pos.col += 2;
			break;
		    }
		}
	    }
	    // FALLTHROUGH

	default:
	    /*
	     * For Lisp skip over backslashed (), {} and [].
	     * (actually, we skip #\( et al)
	     */
	    if (curbuf->b_p_lisp
		    && vim_strchr((char_u *)"{}()[]", c) != NULL
		    && pos.col > 1
		    && check_prevcol(linep, pos.col, '\\', NULL)
		    && check_prevcol(linep, pos.col - 1, '#', NULL))
		break;

	    // Check for match outside of quotes, and inside of
	    // quotes when the start is also inside of quotes.
	    if ((!inquote || start_in_quotes == TRUE)
		    && (c == initc || c == findc))
	    {
		int	col, bslcnt = 0;

		if (!cpo_bsl)
		{
		    for (col = pos.col; check_prevcol(linep, col, '\\', &col);)
			bslcnt++;
		}
		// Only accept a match when 'M' is in 'cpo' or when escaping
		// is what we expect.
		if (cpo_bsl || (bslcnt & 1) == match_escaped)
		{
		    if (c == initc)
			count++;
		    else
		    {
			if (count == 0)
			    return &pos;
			count--;
		    }
		}
	    }
	}
    }

    if (comment_dir == BACKWARD && count > 0)
    {
	pos = match_pos;
	return &pos;
    }
    return (pos_T *)NULL;	// never found it
}

/*
 * Check if line[] contains a / / comment.
 * Return MAXCOL if not, otherwise return the column.
 */
    int
check_linecomment(char_u *line)
{
    char_u  *p;

    p = line;
    // skip Lispish one-line comments
    if (curbuf->b_p_lisp)
    {
	if (vim_strchr(p, ';') != NULL) // there may be comments
	{
	    int in_str = FALSE;	// inside of string

	    p = line;		// scan from start
	    while ((p = vim_strpbrk(p, (char_u *)"\";")) != NULL)
	    {
		if (*p == '"')
		{
		    if (in_str)
		    {
			if (*(p - 1) != '\\') // skip escaped quote
			    in_str = FALSE;
		    }
		    else if (p == line || ((p - line) >= 2
				      // skip #\" form
				      && *(p - 1) != '\\' && *(p - 2) != '#'))
			in_str = TRUE;
		}
		else if (!in_str && ((p - line) < 2
				    || (*(p - 1) != '\\' && *(p - 2) != '#'))
			       && !is_pos_in_string(line, (colnr_T)(p - line)))
		    break;	// found!
		++p;
	    }
	}
	else
	    p = NULL;
    }
    else
	while ((p = vim_strchr(p, '/')) != NULL)
	{
	    // Accept a double /, unless it's preceded with * and followed by
	    // *, because * / / * is an end and start of a C comment.  Only
	    // accept the position if it is not inside a string.
	    if (p[1] == '/' && (p == line || p[-1] != '*' || p[2] != '*')
			       && !is_pos_in_string(line, (colnr_T)(p - line)))
		break;
	    ++p;
	}

    if (p == NULL)
	return MAXCOL;
    return (int)(p - line);
}

/*
 * Move cursor briefly to character matching the one under the cursor.
 * Used for Insert mode and "r" command.
 * Show the match only if it is visible on the screen.
 * If there isn't a match, then beep.
 */
    void
showmatch(
    int		c)	    // char to show match for
{
    pos_T	*lpos, save_cursor;
    pos_T	mpos;
    colnr_T	vcol;
    long	save_so;
    long	save_siso;
#ifdef CURSOR_SHAPE
    int		save_state;
#endif
    colnr_T	save_dollar_vcol;
    char_u	*p;
    long	*so = curwin->w_p_so >= 0 ? &curwin->w_p_so : &p_so;
    long	*siso = curwin->w_p_siso >= 0 ? &curwin->w_p_siso : &p_siso;

    /*
     * Only show match for chars in the 'matchpairs' option.
     */
    // 'matchpairs' is "x:y,x:y"
    for (p = curbuf->b_p_mps; *p != NUL; ++p)
    {
#ifdef FEAT_RIGHTLEFT
	if (PTR2CHAR(p) == c && (curwin->w_p_rl ^ p_ri))
	    break;
#endif
	p += mb_ptr2len(p) + 1;
	if (PTR2CHAR(p) == c
#ifdef FEAT_RIGHTLEFT
		&& !(curwin->w_p_rl ^ p_ri)
#endif
	   )
	    break;
	p += mb_ptr2len(p);
	if (*p == NUL)
	    return;
    }
    if (*p == NUL)
	return;

    if ((lpos = findmatch(NULL, NUL)) == NULL)	    // no match, so beep
    {
	vim_beep(BO_MATCH);
	return;
    }

    if (lpos->lnum < curwin->w_topline || lpos->lnum >= curwin->w_botline)
	return;

    if (!curwin->w_p_wrap)
	getvcol(curwin, lpos, NULL, &vcol, NULL);

    int col_visible = (curwin->w_p_wrap
	    || (vcol >= curwin->w_leftcol
		&& vcol < curwin->w_leftcol + curwin->w_width));
    if (!col_visible)
	return;

    mpos = *lpos;    // save the pos, update_screen() may change it
    save_cursor = curwin->w_cursor;
    save_so = *so;
    save_siso = *siso;
    // Handle "$" in 'cpo': If the ')' is typed on top of the "$",
    // stop displaying the "$".
    if (dollar_vcol >= 0 && dollar_vcol == curwin->w_virtcol)
	dollar_vcol = -1;
    ++curwin->w_virtcol;	// do display ')' just before "$"
    update_screen(UPD_VALID);	// show the new char first

    save_dollar_vcol = dollar_vcol;
#ifdef CURSOR_SHAPE
    save_state = State;
    State = MODE_SHOWMATCH;
    ui_cursor_shape();		// may show different cursor shape
#endif
    curwin->w_cursor = mpos;	// move to matching char
    *so = 0;			// don't use 'scrolloff' here
    *siso = 0;			// don't use 'sidescrolloff' here
    showruler(FALSE);
    setcursor();
    cursor_on();		// make sure that the cursor is shown
    out_flush_cursor(TRUE, FALSE);

    // Restore dollar_vcol(), because setcursor() may call curs_rows()
    // which resets it if the matching position is in a previous line
    // and has a higher column number.
    dollar_vcol = save_dollar_vcol;

    /*
     * brief pause, unless 'm' is present in 'cpo' and a character is
     * available.
     */
    if (vim_strchr(p_cpo, CPO_SHOWMATCH) != NULL)
	ui_delay(p_mat * 100L + 8, TRUE);
    else if (!char_avail())
	ui_delay(p_mat * 100L + 9, FALSE);
    curwin->w_cursor = save_cursor;	// restore cursor position
    *so = save_so;
    *siso = save_siso;
#ifdef CURSOR_SHAPE
    State = save_state;
    ui_cursor_shape();		// may show different cursor shape
#endif
}

/*
 * Check if the pattern is zero-width.
 * If move is TRUE, check from the beginning of the buffer, else from position
 * "cur".
 * "direction" is FORWARD or BACKWARD.
 * Returns TRUE, FALSE or -1 for failure.
 */
    static int
is_zero_width(
    char_u	*pattern,
    size_t	patternlen,
    int		move,
    pos_T	*cur,
    int		direction)
{
    regmmatch_T	regmatch;
    int		nmatched = 0;
    int		result = -1;
    pos_T	pos;
    int		called_emsg_before = called_emsg;
    int		flag = 0;

    if (pattern == NULL)
    {
	pattern = spats[last_idx].pat;
	patternlen = spats[last_idx].patlen;
    }

    if (search_regcomp(pattern, patternlen, NULL, RE_SEARCH, RE_SEARCH,
					      SEARCH_KEEP, &regmatch) == FAIL)
	return -1;

    // init startcol correctly
    regmatch.startpos[0].col = -1;
    // move to match
    if (move)
    {
	CLEAR_POS(&pos);
    }
    else
    {
	pos = *cur;
	// accept a match at the cursor position
	flag = SEARCH_START;
    }

    if (searchit(curwin, curbuf, &pos, NULL, direction, pattern, patternlen, 1,
				  SEARCH_KEEP + flag, RE_SEARCH, NULL) != FAIL)
    {
	// Zero-width pattern should match somewhere, then we can check if
	// start and end are in the same position.
	do
	{
	    regmatch.startpos[0].col++;
	    nmatched = vim_regexec_multi(&regmatch, curwin, curbuf,
			       pos.lnum, regmatch.startpos[0].col, NULL);
	    if (nmatched != 0)
		break;
	} while (regmatch.regprog != NULL
		&& direction == FORWARD ? regmatch.startpos[0].col < pos.col
				      : regmatch.startpos[0].col > pos.col);

	if (called_emsg == called_emsg_before)
	{
	    result = (nmatched != 0
		&& regmatch.startpos[0].lnum == regmatch.endpos[0].lnum
		&& regmatch.startpos[0].col == regmatch.endpos[0].col);
	}
    }

    vim_regfree(regmatch.regprog);
    return result;
}


/*
 * Find next search match under cursor, cursor at end.
 * Used while an operator is pending, and in Visual mode.
 */
    int
current_search(
    long	count,
    int		forward)	// TRUE for forward, FALSE for backward
{
    pos_T	start_pos;	// start position of the pattern match
    pos_T	end_pos;	// end position of the pattern match
    pos_T	orig_pos;	// position of the cursor at beginning
    pos_T	pos;		// position after the pattern
    int		i;
    int		dir;
    int		result;		// result of various function calls
    char_u	old_p_ws = p_ws;
    int		flags = 0;
    pos_T	save_VIsual = VIsual;
    int		zero_width;
    int		skip_first_backward;

    // Correct cursor when 'selection' is exclusive
    if (VIsual_active && *p_sel == 'e' && LT_POS(VIsual, curwin->w_cursor))
	dec_cursor();

    // When searching forward and the cursor is at the start of the Visual
    // area, skip the first search backward, otherwise it doesn't move.
    skip_first_backward = forward && VIsual_active
					   && LT_POS(curwin->w_cursor, VIsual);

    orig_pos = pos = curwin->w_cursor;
    if (VIsual_active)
    {
	if (forward)
	    incl(&pos);
	else
	    decl(&pos);
    }

    // Is the pattern is zero-width?, this time, don't care about the direction
    zero_width = is_zero_width(spats[last_idx].pat, spats[last_idx].patlen,
						TRUE, &curwin->w_cursor, FORWARD);
    if (zero_width == -1)
	return FAIL;  // pattern not found

    /*
     * The trick is to first search backwards and then search forward again,
     * so that a match at the current cursor position will be correctly
     * captured.  When "forward" is false do it the other way around.
     */
    for (i = 0; i < 2; i++)
    {
	if (forward)
	{
	    if (i == 0 && skip_first_backward)
		continue;
	    dir = i;
	}
	else
	    dir = !i;

	flags = 0;
	if (!dir && !zero_width)
	    flags = SEARCH_END;
	end_pos = pos;

	// wrapping should not occur in the first round
	if (i == 0)
	    p_ws = FALSE;

	result = searchit(curwin, curbuf, &pos, &end_pos,
		(dir ? FORWARD : BACKWARD),
		spats[last_idx].pat, spats[last_idx].patlen, (long) (i ? count : 1),
		SEARCH_KEEP | flags, RE_SEARCH, NULL);

	p_ws = old_p_ws;

	// First search may fail, but then start searching from the
	// beginning of the file (cursor might be on the search match)
	// except when Visual mode is active, so that extending the visual
	// selection works.
	if (i == 1 && !result) // not found, abort
	{
	    curwin->w_cursor = orig_pos;
	    if (VIsual_active)
		VIsual = save_VIsual;
	    return FAIL;
	}
	else if (i == 0 && !result)
	{
	    if (forward)
	    {
		// try again from start of buffer
		CLEAR_POS(&pos);
	    }
	    else
	    {
		// try again from end of buffer
		// searching backwards, so set pos to last line and col
		pos.lnum = curwin->w_buffer->b_ml.ml_line_count;
		pos.col  = ml_get_len(curwin->w_buffer->b_ml.ml_line_count);
	    }
	}
    }

    start_pos = pos;

    if (!VIsual_active)
	VIsual = start_pos;

    // put the cursor after the match
    curwin->w_cursor = end_pos;
    if (LT_POS(VIsual, end_pos) && forward)
    {
	if (skip_first_backward)
	    // put the cursor on the start of the match
	    curwin->w_cursor = pos;
	else
	    // put the cursor on last character of match
	    dec_cursor();
    }
    else if (VIsual_active && LT_POS(curwin->w_cursor, VIsual) && forward)
	curwin->w_cursor = pos;   // put the cursor on the start of the match
    VIsual_active = TRUE;
    VIsual_mode = 'v';

    if (*p_sel == 'e')
    {
	// Correction for exclusive selection depends on the direction.
	if (forward && LTOREQ_POS(VIsual, curwin->w_cursor))
	    inc_cursor();
	else if (!forward && LTOREQ_POS(curwin->w_cursor, VIsual))
	    inc(&VIsual);
    }

#ifdef FEAT_FOLDING
    if (fdo_flags & FDO_SEARCH && KeyTyped)
	foldOpenCursor();
#endif

    may_start_select('c');
    setmouse();
#ifdef FEAT_CLIPBOARD
    // Make sure the clipboard gets updated.  Needed because start and
    // end are still the same, and the selection needs to be owned
    clip_star.vmode = NUL;
#endif
    redraw_curbuf_later(UPD_INVERTED);
    showmode();

    return OK;
}

/*
 * return TRUE if line 'lnum' is empty or has white chars only.
 */
    int
linewhite(linenr_T lnum)
{
    char_u  *p;

    p = skipwhite(ml_get(lnum));
    return (*p == NUL);
}

/*
 * Add the search count "[3/19]" to "msgbuf".
 * See update_search_stat() for other arguments.
 */
    static void
cmdline_search_stat(
    int		dirc,
    pos_T	*pos,
    pos_T	*cursor_pos,
    int		show_top_bot_msg,
    char_u	*msgbuf,
    size_t	msgbuflen,
    int		recompute,
    int		maxcount,
    long	timeout)
{
    searchstat_T stat;

    update_search_stat(dirc, pos, cursor_pos, &stat, recompute, maxcount,
								      timeout);
    if (stat.cur <= 0)
	return;

    char	t[SEARCH_STAT_BUF_LEN];
    size_t	len;

#ifdef FEAT_RIGHTLEFT
    if (curwin->w_p_rl && *curwin->w_p_rlc == 's')
    {
	if (stat.incomplete == 1)
	    len = vim_snprintf(t, SEARCH_STAT_BUF_LEN, "[?/??]");
	else if (stat.cnt > maxcount && stat.cur > maxcount)
	    len = vim_snprintf(t, SEARCH_STAT_BUF_LEN, "[>%d/>%d]",
		    maxcount, maxcount);
	else if (stat.cnt > maxcount)
	    len = vim_snprintf(t, SEARCH_STAT_BUF_LEN, "[>%d/%d]",
		    maxcount, stat.cur);
	else
	    len = vim_snprintf(t, SEARCH_STAT_BUF_LEN, "[%d/%d]",
		    stat.cnt, stat.cur);
    }
    else
#endif
    {
	if (stat.incomplete == 1)
	    len = vim_snprintf(t, SEARCH_STAT_BUF_LEN, "[?/??]");
	else if (stat.cnt > maxcount && stat.cur > maxcount)
	    len = vim_snprintf(t, SEARCH_STAT_BUF_LEN, "[>%d/>%d]",
		    maxcount, maxcount);
	else if (stat.cnt > maxcount)
	    len = vim_snprintf(t, SEARCH_STAT_BUF_LEN, "[%d/>%d]",
		    stat.cur, maxcount);
	else
	    len = vim_snprintf(t, SEARCH_STAT_BUF_LEN, "[%d/%d]",
		    stat.cur, stat.cnt);
    }

    if (show_top_bot_msg && len + 2 < SEARCH_STAT_BUF_LEN)
    {
	mch_memmove(t + 2, t, len);
	t[0] = 'W';
	t[1] = ' ';
	len += 2;
    }

    if (len > msgbuflen)
	len = msgbuflen;
    mch_memmove(msgbuf + msgbuflen - len, t, len);

    if (dirc == '?' && stat.cur == maxcount + 1)
	stat.cur = -1;

    // keep the message even after redraw, but don't put in history
    msg_hist_off = TRUE;
    give_warning(msgbuf, FALSE);
    msg_hist_off = FALSE;
}

/*
 * Add the search count information to "stat".
 */
static void
update_search_stat(
    int                 dirc,
    pos_T               *pos,
    pos_T               *cursor_pos,
    searchstat_T        *stat,
    int                 recompute,
    int                 maxcount,
    long                timeout UNUSED)
{
    char_u      *line;

    (void)dirc;
    (void)cursor_pos;
    (void)recompute;
    (void)maxcount;
    (void)timeout;

    if (stat == NULL || spats[last_idx].pat == NULL)
        return;

    line = ml_get_buf(curbuf, pos->lnum, FALSE);
    if (line == NULL)
        return;

    rust_search_update_stat((const char *)spats[last_idx].pat,
                            (const char *)line,
                            stat);
}

#if defined(FEAT_FIND_ID) || defined(PROTO)

/*
 * Get line "lnum" and copy it into "buf[LSIZE]".
 * The copy is made because the regexp may make the line invalid when using a
 * mark.
 */
    static char_u *
get_line_and_copy(linenr_T lnum, char_u *buf)
{
    char_u *line = ml_get(lnum);

    vim_strncpy(buf, line, LSIZE - 1);
    return buf;
}

/*
 * Find identifiers or defines in included files.
 * If p_ic && compl_status_sol() then ptr must be in lowercase.
 */
void find_pattern_in_path(
    char_u *ptr,
    int dir,
    int len,
    int whole,
    int skip_comments,
    int type,
    long count,
    int action,
    linenr_T start_lnum,
    linenr_T end_lnum,
    int forceit,
    int silent)
{
    rust_find_pattern_in_path(ptr, dir, len, whole, skip_comments, type,
                              count, action, start_lnum, end_lnum,
                              forceit, silent);
}
#endif

    static void
show_pat_in_path(
    char_u  *line,
    int	    type,
    int	    did_show,
    int	    action,
    FILE    *fp,
    linenr_T *lnum,
    long    count)
{
    char_u  *p;
    size_t  linelen;

    if (did_show)
	msg_putchar('\n');	// cursor below last one
    else if (!msg_silent)
	gotocmdline(TRUE);	// cursor at status line
    if (got_int)		// 'q' typed at "--more--" message
	return;
    linelen = STRLEN(line);
    for (;;)
    {
	p = line + linelen - 1;
	if (fp != NULL)
	{
	    // We used fgets(), so get rid of newline at end
	    if (p >= line && *p == '\n')
		--p;
	    if (p >= line && *p == '\r')
		--p;
	    *(p + 1) = NUL;
	}
	if (action == ACTION_SHOW_ALL)
	{
	    sprintf((char *)IObuff, "%3ld: ", count);	// show match nr
	    msg_puts((char *)IObuff);
	    sprintf((char *)IObuff, "%4ld", *lnum);	// show line nr
						// Highlight line numbers
	    msg_puts_attr((char *)IObuff, HL_ATTR(HLF_N));
	    msg_puts(" ");
	}
	msg_prt_line(line, FALSE);
	out_flush();			// show one line at a time

	// Definition continues until line that doesn't end with '\'
	if (got_int || type != FIND_DEFINE || p < line || *p != '\\')
	    break;

	if (fp != NULL)
	{
	    if (vim_fgets(line, LSIZE, fp)) // end of file
		break;
	    linelen = STRLEN(line);
	    ++*lnum;
	}
	else
	{
	    if (++*lnum > curbuf->b_ml.ml_line_count)
		break;
	    line = ml_get(*lnum);
	    linelen = ml_get_len(*lnum);
	}
	msg_putchar('\n');
    }
}
#endif

#ifdef FEAT_VIMINFO
/*
 * Return the last used search pattern at "idx".
 */
    spat_T *
get_spat(int idx)
{
    return &spats[idx];
}

/*
 * Return the last used search pattern index.
 */
    int
get_spat_last_idx(void)
{
    return last_idx;
}
#endif

#if defined(FEAT_EVAL) || defined(FEAT_PROTO)
/*
 * "searchcount()" function
 */
    void
f_searchcount(typval_T *argvars, typval_T *rettv)
{
    pos_T		pos = curwin->w_cursor;
    char_u		*pattern = NULL;
    int			maxcount = p_msc;
    long		timeout = SEARCH_STAT_DEF_TIMEOUT;
    int			recompute = TRUE;
    searchstat_T	stat;

    if (rettv_dict_alloc(rettv) == FAIL)
	return;

    if (in_vim9script() && check_for_opt_dict_arg(argvars, 0) == FAIL)
	return;

    if (shortmess(SHM_SEARCHCOUNT))	// 'shortmess' contains 'S' flag
	recompute = TRUE;

    if (argvars[0].v_type != VAR_UNKNOWN)
    {
	dict_T		*dict;
	dictitem_T	*di;
	listitem_T	*li;
	int		error = FALSE;

	if (check_for_nonnull_dict_arg(argvars, 0) == FAIL)
	    return;
	dict = argvars[0].vval.v_dict;
	di = dict_find(dict, (char_u *)"timeout", -1);
	if (di != NULL)
	{
	    timeout = (long)tv_get_number_chk(&di->di_tv, &error);
	    if (error)
		return;
	}
	di = dict_find(dict, (char_u *)"maxcount", -1);
	if (di != NULL)
	{
	    maxcount = (int)tv_get_number_chk(&di->di_tv, &error);
	    if (error)
		return;
	}
	recompute = dict_get_bool(dict, "recompute", recompute);
	di = dict_find(dict, (char_u *)"pattern", -1);
	if (di != NULL)
	{
	    pattern = tv_get_string_chk(&di->di_tv);
	    if (pattern == NULL)
		return;
	}
	di = dict_find(dict, (char_u *)"pos", -1);
	if (di != NULL)
	{
	    if (di->di_tv.v_type != VAR_LIST)
	    {
		semsg(_(e_invalid_argument_str), "pos");
		return;
	    }
	    if (list_len(di->di_tv.vval.v_list) != 3)
	    {
		semsg(_(e_invalid_argument_str), "List format should be [lnum, col, off]");
		return;
	    }
	    li = list_find(di->di_tv.vval.v_list, 0L);
	    if (li != NULL)
	    {
		pos.lnum = tv_get_number_chk(&li->li_tv, &error);
		if (error)
		    return;
	    }
	    li = list_find(di->di_tv.vval.v_list, 1L);
	    if (li != NULL)
	    {
		pos.col = tv_get_number_chk(&li->li_tv, &error) - 1;
		if (error)
		    return;
	    }
	    li = list_find(di->di_tv.vval.v_list, 2L);
	    if (li != NULL)
	    {
		pos.coladd = tv_get_number_chk(&li->li_tv, &error);
		if (error)
		    return;
	    }
	}
    }

    save_last_search_pattern();
#ifdef FEAT_SEARCH_EXTRA
    save_incsearch_state();
#endif
    if (pattern != NULL)
    {
	if (*pattern == NUL)
	    goto the_end;
	vim_free(spats[last_idx].pat);
	spats[last_idx].patlen = STRLEN(pattern);
	spats[last_idx].pat = vim_strnsave(pattern, spats[last_idx].patlen);
	if (spats[last_idx].pat == NULL)
	    spats[last_idx].patlen = 0;
    }
    if (spats[last_idx].pat == NULL || *spats[last_idx].pat == NUL)
	goto the_end;	// the previous pattern was never defined

    update_search_stat(0, &pos, &pos, &stat, recompute, maxcount, timeout);

    dict_add_number(rettv->vval.v_dict, "current", stat.cur);
    dict_add_number(rettv->vval.v_dict, "total", stat.cnt);
    dict_add_number(rettv->vval.v_dict, "exact_match", stat.exact_match);
    dict_add_number(rettv->vval.v_dict, "incomplete", stat.incomplete);
    dict_add_number(rettv->vval.v_dict, "maxcount", stat.last_maxcount);

the_end:
    restore_last_search_pattern();
#ifdef FEAT_SEARCH_EXTRA
    restore_incsearch_state();
#endif
}
#endif
