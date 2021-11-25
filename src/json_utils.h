#ifndef TOMLQ_JSON_UTILS
#define TOMLQ_JSON_UTILS

#include "sds.h"

/**
 * Escape a string to make it suitable for printing in json.
 *
 * Removes any special characters.
 *
 * Does not add in the quotes (you'll have to do that)
 *
 * For example "foo\n" => "foo\\n"
 */
sds tq_escape_string(const char* in);




