/* Seal's C API (D136, D151, D152). Every function starts `seal_`. Handles are numbers Seal checks before it uses
 * them, so a freed or made-up one is refused with SEAL_HOST_MISTAKE, never followed; each belongs to the thread that
 * made it. A call that doesn't give SEAL_OK or SEAL_PAUSED leaves its messages to read with seal_message_*, kept
 * until the next seal_ call on the same thread, as C's errno is. Every text is UTF-8. */
#ifndef SEAL_H
#define SEAL_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct seal_doors seal_doors;   /* a host's set of doors */
typedef struct seal_script seal_script; /* one loaded script, with its tasks */
typedef struct seal_value seal_value;   /* a text, a list, a big number or another value that isn't plain */
typedef uint64_t seal_task;             /* a paused task */

typedef enum {
    SEAL_OK = 0,          /* done: a value, or nothing */
    SEAL_PAUSED = 1,      /* a door paused the script; resume or cancel the task */
    SEAL_REFUSED = 2,     /* refused while loading, or a call named something wrong */
    SEAL_STOPPED = 3,     /* a check failed or a limit was reached; the script is over */
    SEAL_FAILED = 4,      /* the procedure failed on purpose; the script carries on */
    SEAL_HOST_MISTAKE = 5 /* the host used the API wrongly: a null, a freed handle, a wrong kind */
} seal_status;

typedef enum {
    SEAL_NOTHING = 0,
    SEAL_WHOLE = 1,
    SEAL_FLOAT = 2,
    SEAL_BOOL = 3,
    SEAL_TEXT = 4,
    SEAL_HUGE_WHOLE = 5,
    SEAL_DECIMAL = 6,
    SEAL_HUGE_DECIMAL = 7,
    SEAL_LIST = 8,
    SEAL_MAYBE = 9,
    SEAL_DICT = 10,
    SEAL_RECORD = 11,
    SEAL_CASE = 12
} seal_kind;

/* One value crossing a door or a call (D152): a number or a Bool held in it, anything else as a handle. A door's
 * handles are lent for the call; a result's and a call's are the host's to free. */
typedef struct {
    seal_kind kind;
    union {
        int64_t whole;
        double number;
        int truth;
        seal_value *value;
    } as;
} seal_arg;

typedef struct {
    uint64_t steps;
    size_t memory;
} seal_limits;

/* A door: SEAL_OK with its result written, SEAL_PAUSED to pause the script (when it was opened able to), or
 * SEAL_FAILED with result->as.whole the place of its failure in the list it was opened with, counting from 0. */
typedef seal_status (*seal_door_function)(void *host_data, const seal_arg *args, size_t count, seal_arg *result);

/* ---- doors ---- */
seal_doors *seal_doors_new(void);
void seal_doors_free(seal_doors *doors);
/* Types are written as a script writes them: "Whole", "Float", "Bool", "Text", "HugeWhole", "Decimal",
 * "HugeDecimal", "List<Whole>", "Maybe<Text>", "Dict<key: Text, value: Whole>", and "Nothing" for no result. */
seal_status seal_doors_add(seal_doors *doors, const char *name, const char *const *value_types, size_t count,
                           const char *result_type, const char *const *failures, size_t failure_count,
                           int can_pause, seal_door_function function, void *host_data);
seal_status seal_doors_labels(seal_doors *doors, const char *name, const char *const *labels, size_t count);

/* ---- scripts ---- */
seal_status seal_script_load(const char *name, const char *source, size_t length, const seal_doors *doors,
                             seal_limits limits, seal_script **script);
seal_status seal_script_load_compiled(const uint8_t *bytes, size_t length, const char *name,
                                      const seal_doors *doors, seal_limits limits, seal_script **script);
void seal_script_free(seal_script *script);
/* labels may be NULL; otherwise one per argument, NULL for one with none. */
seal_status seal_script_call(seal_script *script, const char *procedure, const char *const *labels,
                             const seal_arg *args, size_t count, seal_arg *result, seal_task *task);
seal_status seal_script_resume(seal_script *script, seal_task task, const seal_arg *value, seal_arg *result,
                               seal_task *paused_again);
seal_status seal_script_cancel(seal_script *script, seal_task task);
size_t seal_script_memory_in_use(const seal_script *script);

/* ---- compiled files (D141, D153) ---- */
seal_status seal_build(const seal_doors *doors, const char *const *names, const char *const *sources, size_t count,
                       const uint8_t **bytes, size_t *length);
void seal_bytes_free(const uint8_t *bytes);

/* ---- messages, whole and in parts (D151) ---- */
typedef enum {
    SEAL_REFUSAL = 0,
    SEAL_CHECK_FAILED = 1,
    SEAL_LIMIT_REACHED = 2,
    SEAL_NOT_DESIGNED_YET = 3,
    SEAL_NOT_BUILT_YET = 4,
    SEAL_FAILURE = 5,
    SEAL_MISTAKE = 6
} seal_message_kind;

size_t seal_message_count(void);
seal_message_kind seal_message_kind_of(size_t index);
const char *seal_message_text(size_t index); /* the whole message, laid out as Seal prints it */
const char *seal_message_what(size_t index); /* each part, or NULL where the message has none */
const char *seal_message_why(size_t index);
const char *seal_message_fix(size_t index);
const char *seal_message_when(size_t index);
const char *seal_message_file(size_t index);
uint32_t seal_message_line(size_t index); /* counted from 1; 0 where there's no place */
uint32_t seal_message_column(size_t index);
uint32_t seal_message_width(size_t index);

/* ---- values that aren't plain ---- */
seal_value *seal_value_text(const char *text, size_t length); /* NULL, with a message, unless UTF-8 */
seal_value *seal_value_number(seal_kind kind, const char *written); /* a HugeWhole, Decimal or HugeDecimal */
seal_value *seal_value_list(const seal_arg *items, size_t count);   /* takes the items' handles over */
seal_value *seal_value_maybe(const seal_arg *inner);                /* NULL for none; takes its handle over */
void seal_value_free(seal_value *value);

/* Each reader gives SEAL_HOST_MISTAKE, with a message, for a value of another kind. */
seal_status seal_value_kind(const seal_value *value, seal_kind *kind);
seal_status seal_value_get_text(const seal_value *value, const char **text, size_t *length);
seal_status seal_value_count(const seal_value *value, size_t *count);
seal_status seal_value_item(const seal_value *value, size_t position, seal_arg *item); /* counted from 0 */

#ifdef __cplusplus
}
#endif

#endif
