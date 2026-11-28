# `wax-c`: the C API (D136), as proposed

The building session's proposal for the shape and names of Wax's C API, which D136 leaves to it. Written 2026-11-25
by seal opus claude, and built on 2026-11-26 as `include/wax.h` and `src/lib.rs`. Where the two differ, the header is
what's built: door arguments are an array of plain tagged values, `wax_arg`, with numbers inline and big values as
lent handles (D152); every reader of a value reports the wrong kind as a host mistake; and messages are kept per
thread, so one thread never reads another's.

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

Every function starts `wax_`, and every type `wax_`. Handles are opaque pointers the host frees with the matching
`_free`. Every call that can go wrong returns a `wax_status` and puts its message where the host can read it.

```c
typedef struct wax_doors wax_doors;     /* a host's set of doors */
typedef struct wax_script wax_script;   /* one loaded script, with its tasks */
typedef struct wax_value wax_value;     /* a value crossing to or from a script */
typedef uint64_t wax_task;               /* a paused task */

typedef enum {
    WAX_OK,          /* done: a value, or nothing */
    WAX_PAUSED,      /* a door paused the script; resume or cancel the task */
    WAX_REFUSED,     /* refused while loading, or a call named something wrong */
    WAX_STOPPED,     /* a check failed or a limit was reached; the script is over */
    WAX_FAILED,      /* the procedure failed on purpose; the script carries on */
    WAX_HOST_MISTAKE /* the host used the API wrongly: a null, a freed handle, a wrong type */
} wax_status;

typedef struct { uint64_t steps; size_t memory; } wax_limits;

/* The kinds a door's values and result can be (D85: the host's own records and one_ofs are added by name). */
typedef enum { WAX_NOTHING, WAX_WHOLE, WAX_FLOAT, WAX_BOOL, WAX_TEXT, WAX_LIST, WAX_MAYBE, ... } wax_kind;
```

### Doors

```c
typedef wax_status (*wax_door_function)(void *host_data, const wax_value *const *values, size_t count,
                                          wax_value **result);

wax_doors *wax_doors_new(void);
void wax_doors_free(wax_doors *doors);
wax_status wax_doors_add(wax_doors *doors, const char *name, const wax_kind *values, size_t count,
                           wax_kind result, int can_pause, wax_door_function function, void *host_data);
wax_status wax_doors_labels(wax_doors *doors, const char *name, const char *const *labels, size_t count);
const char *wax_doors_message(const wax_doors *doors);
```

A door's function returns `WAX_OK` with its result, `WAX_PAUSED` to pause the script, or `WAX_FAILED` with the
failure's name in `result` as a text, one the door declared (D44).

### Scripts

```c
wax_status wax_script_load(const char *name, const char *source, size_t length, const wax_doors *doors,
                             wax_limits limits, wax_script **script);
void wax_script_free(wax_script *script);

wax_status wax_script_call(wax_script *script, const char *procedure, const char *const *labels,
                             const wax_value *const *values, size_t count, wax_value **result, wax_task *task);
wax_status wax_script_resume(wax_script *script, wax_task task, const wax_value *value, wax_value **result,
                               wax_task *paused_again);
wax_status wax_script_cancel(wax_script *script, wax_task task);

size_t wax_script_memory_in_use(const wax_script *script);
```

`wax_script_load` gives a script only with `WAX_OK`; with `WAX_REFUSED` there's no script, so its messages are
read as the next section says.

### Messages, whole and in parts (D151)

The messages of the last call on a thread that didn't give `WAX_OK` or `WAX_PAUSED`, kept until the next `wax_`
call on that thread, as C's `errno` is. A refusal may hold several; a stop or a failure holds one.

```c
typedef enum {
    WAX_REFUSAL, WAX_CHECK_FAILED, WAX_LIMIT_REACHED, WAX_NOT_DESIGNED_YET, WAX_NOT_BUILT_YET,
    WAX_FAILURE, WAX_MISTAKE
} wax_message_kind;

size_t wax_message_count(void);
wax_message_kind wax_message_kind_of(size_t index);
const char *wax_message_text(size_t index);   /* the whole message, laid out as Wax prints it */
const char *wax_message_what(size_t index);   /* each part, or NULL where the message has none */
const char *wax_message_why(size_t index);
const char *wax_message_fix(size_t index);
const char *wax_message_when(size_t index);
const char *wax_message_file(size_t index);
uint32_t wax_message_line(size_t index);      /* counted from 1; 0 where there's no place */
uint32_t wax_message_column(size_t index);
uint32_t wax_message_width(size_t index);
```

A failure on purpose gives its failure's name as `what`. Every text is UTF-8, ends with a zero, and stays readable
until the next `wax_` call on the thread.

### Values

```c
wax_value *wax_value_whole(int64_t whole);
wax_value *wax_value_float(double number);   /* refused unless finite (D18) */
wax_value *wax_value_bool(int truth);
wax_value *wax_value_text(const char *text, size_t length);   /* refused unless UTF-8 */
wax_value *wax_value_list(wax_value *const *items, size_t count);
void wax_value_free(wax_value *value);

wax_kind wax_value_kind(const wax_value *value);
int64_t wax_value_get_whole(const wax_value *value);
double wax_value_get_float(const wax_value *value);
int wax_value_get_bool(const wax_value *value);
const char *wax_value_get_text(const wax_value *value, size_t *length);
size_t wax_value_count(const wax_value *value);                   /* a list's items, a record's fields */
const wax_value *wax_value_item(const wax_value *value, size_t position);   /* counted from 0 */
```

`HugeWhole`, `Decimal` and `HugeDecimal` cross as text, exactly, with their own makers and readers.

## How it's kept safe

- **`unsafe` only in `wax-c`** (Building rule 2, with Tankun's OK of 2026-11-25), in as few blocks as it can be, each
  saying why it's sound: reading the host's pointers and strings, and handing back owned pointers.
- **No panic crosses into C:** every function catches one, and gives `WAX_HOST_MISTAKE` or `WAX_STOPPED` with a
  message, as the Rust host does (D14).
- **Every pointer is checked** for null; a text is checked to be UTF-8; a handle is checked to be one of Wax's own
  before it's used, so a freed or made-up one is refused, not followed.
- **Tested hard:** a C test program built with the API, and fuzzing through it, under AddressSanitizer where the
  platform has it.

## Questions it leaves for Tankun

1. (Answered by D151: every message whole and in parts, as above.)
2. Whether a door's values arrive as `wax_value` handles, as proposed, or plain C types where they're plain
   numbers, which is quicker but makes two ways to write a door. The proposal: handles, one way.
