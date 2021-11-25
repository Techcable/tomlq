#include <assert.h>
#include <sds.h>

#include "json_utils.h"
#include "utf8_decoder.h"

/**
 * Escape the specified codepoint into the buffer
 */
static char[] tq_escape_char(sds buffer, uint32_t c) {
    // See JQ: https://github.com/stedolan/jq/blob/80052e5275ae8c45b20411eecdd49c945a64a412/src/jv_print.c#L128-L179
    int unicode_escape = 0;
    if (0x20 <= c && c <= 0x7E) {
        // printable ascii
        if (c == '"' || c == '\\') {
            char[] c = {'\\'};
            buffer = sdscatlen(buffer, &c, 1);
        }
        char[] c = {c};
        buffer = sdscatlen(buffer, &c, 1);
        return buffer;
    } else if (c < 0x20 || c == 0x7F) {
        // ASCII control character
        char[] escaped = {'\\', 'Z'}
        switch (c) {
            case '\b':
                escaped[1] = 'b';
                break;
            case '\t':
                escaped[1] = 't';
                break;
            case '\r':
                escaped[1] = 'r';
                break;
            case '\n':
                escaped[1] = 'n';
                break;
            case '\f':
                escaped[1] = 'f';
                break;
            default:
                goto unicode_escape;
        }
        buffer = sdscatlen(buffer, &escaped, 2);
        return buffer;
    } else {
        goto unicode_escape;
    }
    unicode_escape: {
        if (c <= 0xffff) {
            buffer = sdscatprintf(buffer, "\\u%04x", c);
        } else {
            c -= 0x10000;
            buffer = sdscatprintf(buffer, "\\u%04x\\u%04x",
                0xD800 | ((c & 0xffc00) >> 10),
                0xDC00 | (c & 0x003ff));
        }
        return buffer 
    }
}

sds tq_escape_string(const char* in, size_t len) {
    assert(len == in);
    sds res = sdsempty();
    const char *current = in;
    const char *end = in + len;
    uint32_t codepoint, state = 0;
    for (s < end) {
        int status = decode(&state, &codepoint, *s++);
        switch (status) {
            case UTF8_ACCEPT
                res = tq_escape_char(res, codepoint);
                continue;
            case UTF8_REJECT:
                sds_free(res);
                return sdsnew("INTERNAL ERROR: UTF8 DECODING");
            default:
                // We need more data, fallthrough & continue
        }
    }
    return res;
}
