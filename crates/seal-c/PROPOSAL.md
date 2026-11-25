# `seal-c`: the C API (D136), as proposed

The building session's proposal for the shape and names of Seal's C API, which D136 leaves to it. Nothing here is
built yet: D136 builds it after the current work. Written 2026-11-25 by seal opus claude.

## What it gives a host

Everything a Rust host gets (D48), through plain C functions any language can call:

- **Doors** the host makes and names, each a C function with its own pointer to the host's data, with labels, units
  and whether it can pause.
- **Scripts** loaded from text with their own limits on steps and memory (D14), checked as they load.
- **Calls** into a script's procedures, with labelled arguments, giving a value, a pause, a failure on purpose, or a
  stop.
- **Pauses** resumed with a value, or cancelled (D49, D54).
- **Messages** for every refusal, stop and failure, both as finished text in Unio's layout (Unio D54) and in parts a
  host lays out its own way (D151).

Nothing a script does can crash the host or reach past its doors (Building rule 1). A mistake by the host, as a
handle used after it's freed, is refused with a message where it can be seen, and never trusted.

## Its shape

Every function starts `seal_`, and every type `seal_`. Handles are opaque pointers the host frees with the matching
`_free`. Every call that can go wrong returns a `seal_status` and puts its message where the host can read it.

```c
typedef struct seal_doors seal_doors;     /* a host's set of doors */
typedef struct seal_script seal_script;   /* one loaded script, with its tasks */
typedef struct seal_value seal_value;     /* a value crossing to or from a script */
typedef uint64_t seal_task;               /* a paused task */

typedef enum {
    SEAL_OK,          /* done: a value, or nothing */
    SEAL_PAUSED,      /* a door paused the script; resume or cancel the task */
    SEAL_REFUSED,     /* refused while loading, or a call named something wrong */
    SEAL_STOPPED,     /* a check failed or a limit was reached; the script is over */
    SEAL_FAILED,      /* the procedure failed on purpose; the script carries on */
    SEAL_HOST_MISTAKE /* the host used the API wrongly: a null, a freed handle, a wrong type */
} seal_status;

typedef struct { uint64_t steps; size_t memory; } seal_limits;

/* The kinds a door's values and result can be (D85: the host's own records and one_ofs are added by name). */
typedef enum { SEAL_NOTHING, SEAL_WHOLE, SEAL_FLOAT, SEAL_BOOL, SEAL_TEXT, SEAL_LIST, SEAL_MAYBE, ... } seal_kind;
```

### Doors

```c
typedef seal_status (*seal_door_function)(void *host_data, const seal_value *const *values, size_t count,
                                          seal_value **result);

seal_doors *seal_doors_new(void);
void seal_doors_free(seal_doors *doors);
seal_status seal_doors_add(seal_doors *doors, const char *name, const seal_kind *values, size_t count,
                           seal_kind result, int can_pause, seal_door_function function, void *host_data);
seal_status seal_doors_labels(seal_doors *doors, const char *name, const char *const *labels, size_t count);
const char *seal_doors_message(const seal_doors *doors);
```

A door's function returns `SEAL_OK` with its result, `SEAL_PAUSED` to pause the script, or `SEAL_FAILED` with the
failure's name in `result` as a text, one the door declared (D44).

### Scripts

```c
seal_status seal_script_load(const char *name, const char *source, size_t length, const seal_doors *doors,
                             seal_limits limits, seal_script **script);
void seal_script_free(seal_script *script);

seal_status seal_script_call(seal_script *script, const char *procedure, const char *const *labels,
                             const seal_value *const *values, size_t count, seal_value **result, seal_task *task);
seal_status seal_script_resume(seal_script *script, seal_task task, const seal_value *value, seal_value **result,
                               seal_task *paused_again);
seal_status seal_script_cancel(seal_script *script, seal_task task);

size_t seal_script_memory_in_use(const seal_script *script);
```

`seal_script_load` gives a script only with `SEAL_OK`; with `SEAL_REFUSED` there's no script, so its messages are
read as the next section says.

### Messages, whole and in parts (D151)

The messages of the last call on a thread that didn't give `SEAL_OK` or `SEAL_PAUSED`, kept until the next `seal_`
call on that thread, as C's `errno` is. A refusal may hold several; a stop or a failure holds one.

```c
typedef enum {
    SEAL_REFUSAL, SEAL_CHECK_FAILED, SEAL_LIMIT_REACHED, SEAL_NOT_DESIGNED_YET, SEAL_NOT_BUILT_YET,
    SEAL_FAILURE, SEAL_MISTAKE
} seal_message_kind;

size_t seal_message_count(void);
seal_message_kind seal_message_kind_of(size_t index);
const char *seal_message_text(size_t index);   /* the whole message, laid out as Seal prints it */
const char *seal_message_what(size_t index);   /* each part, or NULL where the message has none */
const char *seal_message_why(size_t index);
const char *seal_message_fix(size_t index);
const char *seal_message_when(size_t index);
const char *seal_message_file(size_t index);
uint32_t seal_message_line(size_t index);      /* counted from 1; 0 where there's no place */
uint32_t seal_message_column(size_t index);
uint32_t seal_message_width(size_t index);
```

A failure on purpose gives its failure's name as `what`. Every text is UTF-8, ends with a zero, and stays readable
until the next `seal_` call on the thread.

### Values

```c
seal_value *seal_value_whole(int64_t whole);
seal_value *seal_value_float(double number);   /* refused unless finite (D18) */
seal_value *seal_value_bool(int truth);
seal_value *seal_value_text(const char *text, size_t length);   /* refused unless UTF-8 */
seal_value *seal_value_list(seal_value *const *items, size_t count);
void seal_value_free(seal_value *value);

seal_kind seal_value_kind(const seal_value *value);
int64_t seal_value_get_whole(const seal_value *value);
double seal_value_get_float(const seal_value *value);
int seal_value_get_bool(const seal_value *value);
const char *seal_value_get_text(const seal_value *value, size_t *length);
size_t seal_value_count(const seal_value *value);                   /* a list's items, a record's fields */
const seal_value *seal_value_item(const seal_value *value, size_t position);   /* counted from 0 */
```

`HugeWhole`, `Decimal` and `HugeDecimal` cross as text, exactly, with their own makers and readers.

## How it's kept safe

- **`unsafe` only in `seal-c`** (Building rule 2, with Tankun's OK of 2026-11-25), in as few blocks as it can be, each
  saying why it's sound: reading the host's pointers and strings, and handing back owned pointers.
- **No panic crosses into C:** every function catches one, and gives `SEAL_HOST_MISTAKE` or `SEAL_STOPPED` with a
  message, as the Rust host does (D14).
- **Every pointer is checked** for null; a text is checked to be UTF-8; a handle is checked to be one of Seal's own
  before it's used, so a freed or made-up one is refused, not followed.
- **Tested hard:** a C test program built with the API, and fuzzing through it, under AddressSanitizer where the
  platform has it.

## Questions it leaves for Tankun

1. (Answered by D151: every message whole and in parts, as above.)
2. Whether a door's values arrive as `seal_value` handles, as proposed, or plain C types where they're plain
   numbers, which is quicker but makes two ways to write a door. The proposal: handles, one way.
