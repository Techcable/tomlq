#ifndef TOMLQ_TOML2JSON
#define TOMLQ_TOML2JSON

#include "toml.h"
// The "simple dynamic strings library"
#include "sds.h"


/**
 * Escape the specified string.
 *
 * See the json spec for details: https://www.json.org/json-en.html
 *
 * Consumes ownership of the buffer, then returns the modfied version.
 */
sds tq_toml2json_escape_string(const char *input, sds buffer);

/**
 * Based on the input toml table,
 * this emits an json string of output.
 *
 * Everything is appropriately quoted and whathnot 
 *
 * The input is borrowed, the output is owned.
 */
sds tq_toml2json_table(toml_table_t* input);

#endif // TOMLQ_TOML2JSON
