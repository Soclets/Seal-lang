/* Wax's C API (D136, D151, D152). Every function starts `wax_`. Handles are numbers Wax checks before it uses
 * them, so a freed or made-up one is refused with WAX_HOST_MISTAKE, never followed; each belongs to the thread that
 * made it. A call that doesn't give WAX_OK or WAX_PAUSED leaves its messages to read with wax_message_*, kept
 * until the next wax_ call on the same thread, as C's errno is. Every text is UTF-8. */
#ifndef WAX_H
#define WAX_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct wax_doors wax_doors;   /* a host's set of doors */
typedef struct wax_script wax_script; /* one loaded script, with its tasks */
typedef struct wax_value wax_value;   /* a text, a list, a big number or another value that isn't plain */
typedef uint64_t wax_task;             /* a paused task */

typedef enum {
    WAX_OK = 0,          /* done: a value, or nothing */
    WAX_PAUSED = 1,      /* a door paused the script; resume or cancel the task */
    WAX_REFUSED = 2,     /* refused while loading, or a call named something wrong */
    WAX_STOPPED = 3,     /* a check failed or a limit was reached; the script is over */
    WAX_FAILED = 4,      /* the procedure failed on purpose; the script carries on */
    WAX_HOST_MISTAKE = 5 /* the host used the API wrongly: a null, a freed handle, a wrong kind */
} wax_status;

typedef enum {
    WAX_NOTHING = 0,
    WAX_WHOLE = 1,
    WAX_FLOAT = 2,
    WAX_BOOL = 3,
    WAX_TEXT = 4,
    WAX_HUGE_WHOLE = 5,
    WAX_DECIMAL = 6,
    WAX_HUGE_DECIMAL = 7,
    WAX_LIST = 8,
    WAX_MAYBE = 9,
    WAX_DICT = 10,
    WAX_RECORD = 11,
    WAX_CASE = 12
} wax_kind;

/* One value crossing a door or a call (D152): a number or a Bool held in it, anything else as a handle. A door's
 * handles are lent for the call; a result's and a call's are the host's to free. */
typedef struct {
    wax_kind kind;
    union {
        int64_t whole;
        double number;
        int truth;
        wax_value *value;
    } as;
} wax_arg;

typedef struct {
    uint64_t steps;
    size_t memory;
} wax_limits;

/* A door: WAX_OK with its result written, WAX_PAUSED to pause the script (when it was opened able to), or
 * WAX_FAILED with result->as.whole the place of its failure in the list it was opened with, counting from 0. */
typedef wax_status (*wax_door_function)(void *host_data, const wax_arg *args, size_t count, wax_arg *result);

/* ---- doors ---- */
wax_doors *wax_doors_new(void);
void wax_doors_free(wax_doors *doors);
/* Types are written as a script writes them: "Whole", "Float", "Bool", "Text", "HugeWhole", "Decimal",
 * "HugeDecimal", "List<Whole>", "Maybe<Text>", "Dict<key: Text, value: Whole>", and "Nothing" for no result. */
wax_status wax_doors_add(wax_doors *doors, const char *name, const char *const *value_types, size_t count,
                           const char *result_type, const char *const *failures, size_t failure_count,
                           int can_pause, wax_door_function function, void *host_data);
wax_status wax_doors_labels(wax_doors *doors, const char *name, const char *const *labels, size_t count);
/* D169: every .waxc file loaded with these doors must be signed with the secret key matching this ML-DSA-44 public
 * key, as `wax keygen` makes it (1312 bytes). A file that isn't is refused. */
wax_status wax_doors_require_signature(wax_doors *doors, const uint8_t *public_key, size_t length);

/* ---- scripts ---- */
wax_status wax_script_load(const char *name, const char *source, size_t length, const wax_doors *doors,
                             wax_limits limits, wax_script **script);
wax_status wax_script_load_compiled(const uint8_t *bytes, size_t length, const char *name,
                                      const wax_doors *doors, wax_limits limits, wax_script **script);
void wax_script_free(wax_script *script);
/* labels may be NULL; otherwise one per argument, NULL for one with none. */
wax_status wax_script_call(wax_script *script, const char *procedure, const char *const *labels,
                             const wax_arg *args, size_t count, wax_arg *result, wax_task *task);
wax_status wax_script_resume(wax_script *script, wax_task task, const wax_arg *value, wax_arg *result,
                               wax_task *paused_again);
wax_status wax_script_cancel(wax_script *script, wax_task task);
size_t wax_script_memory_in_use(const wax_script *script);

/* ---- compiled files (D141, D153) ---- */
wax_status wax_build(const wax_doors *doors, const char *const *names, const char *const *sources, size_t count,
                       const uint8_t **bytes, size_t *length);
void wax_bytes_free(const uint8_t *bytes);

/* ---- messages, whole and in parts (D151) ---- */
typedef enum {
    WAX_REFUSAL = 0,
    WAX_CHECK_FAILED = 1,
    WAX_LIMIT_REACHED = 2,
    WAX_NOT_DESIGNED_YET = 3,
    WAX_NOT_BUILT_YET = 4,
    WAX_FAILURE = 5,
    WAX_MISTAKE = 6
} wax_message_kind;

size_t wax_message_count(void);
wax_message_kind wax_message_kind_of(size_t index);
const char *wax_message_text(size_t index); /* the whole message, laid out as Wax prints it */
const char *wax_message_what(size_t index); /* each part, or NULL where the message has none */
const char *wax_message_why(size_t index);
const char *wax_message_fix(size_t index);
const char *wax_message_when(size_t index);
const char *wax_message_file(size_t index);
uint32_t wax_message_line(size_t index); /* counted from 1; 0 where there's no place */
uint32_t wax_message_column(size_t index);
uint32_t wax_message_width(size_t index);

/* ---- values that aren't plain ---- */
wax_value *wax_value_text(const char *text, size_t length); /* NULL, with a message, unless UTF-8 */
wax_value *wax_value_number(wax_kind kind, const char *written); /* a HugeWhole, Decimal or HugeDecimal */
wax_value *wax_value_list(const wax_arg *items, size_t count);   /* takes the items' handles over */
wax_value *wax_value_maybe(const wax_arg *inner);                /* NULL for none; takes its handle over */
void wax_value_free(wax_value *value);

/* Each reader gives WAX_HOST_MISTAKE, with a message, for a value of another kind. */
wax_status wax_value_kind(const wax_value *value, wax_kind *kind);
wax_status wax_value_get_text(const wax_value *value, const char **text, size_t *length);
wax_status wax_value_count(const wax_value *value, size_t *count);
wax_status wax_value_item(const wax_value *value, size_t position, wax_arg *item); /* counted from 0 */

#ifdef __cplusplus
}
#endif

#endif
