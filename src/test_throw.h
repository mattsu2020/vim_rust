/* vi:set ts=8 sts=4 sw=4 noet:
 *
 * VIM - Vi IMproved    by Bram Moolenaar
 *
 * Do ":help uganda"  in Vim to read copying and usage conditions.
 * Do ":help credits" in Vim to see a list of people who contributed.
 */

#ifndef TEST_THROW_H
#define TEST_THROW_H

// Expressions used for testing during the development phase.
# define THROW_ON_ERROR		(!eval_to_number("$VIMNOERRTHROW"))
# define THROW_ON_INTERRUPT	(!eval_to_number("$VIMNOINTTHROW"))
# define THROW_TEST

#endif // TEST_THROW_H
