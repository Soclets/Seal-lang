/* Wax's C API tested from C (D136, D151, D152), built and run by tests/c_api.rs under AddressSanitizer where the
 * system's compiler has it. Every check that fails prints what and where, and the program exits with 1. With the
 * argument `bench N`, it times N door calls through the API instead and prints the time each took. */
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "wax.h"

static int failures = 0;

#define CHECK(condition)                                                                                   \
    do {                                                                                                   \
        if (!(condition)) {                                                                                \
            fprintf(stderr, "api_test.c:%d: check failed: %s\n", __LINE__, #condition);                    \
            failures++;                                                                                    \
        }                                                                                                  \
    } while (0)

static const wax_limits LIMITS = {1000000, 1 << 20};

/* The host's own data: what its doors were given. */
typedef struct {
    int64_t hurt_total;
    int calls;
    char last_said[64];
} game;

static wax_status damage(void *data, const wax_arg *args, size_t count, wax_arg *result) {
    game *the_game = data;
    if (count != 2 || args[0].kind != WAX_WHOLE || args[1].kind != WAX_WHOLE) return WAX_HOST_MISTAKE;
    the_game->hurt_total += args[0].as.whole * args[1].as.whole;
    the_game->calls++;
    result->kind = WAX_NOTHING;
    return WAX_OK;
}

/* Text in, text out: the argument is lent for the call, the result is a new value Wax takes over. */
static wax_status shout(void *data, const wax_arg *args, size_t count, wax_arg *result) {
    game *the_game = data;
    const char *text = NULL;
    size_t length = 0;
    if (count != 1 || args[0].kind != WAX_TEXT) return WAX_HOST_MISTAKE;
    if (wax_value_get_text(args[0].as.value, &text, &length) != WAX_OK) return WAX_HOST_MISTAKE;
    snprintf(the_game->last_said, sizeof the_game->last_said, "%.*s", (int)length, text);
    char loud[80];
    int written = snprintf(loud, sizeof loud, "%.*s!", (int)length, text);
    result->kind = WAX_TEXT;
    result->as.value = wax_value_text(loud, (size_t)written);
    /* A lent value is the caller's: freeing it does nothing. */
    wax_value_free(args[0].as.value);
    return WAX_OK;
}

/* Fails with its first failure, `too_high`, past 10. */
static wax_status pick(void *data, const wax_arg *args, size_t count, wax_arg *result) {
    (void)data;
    if (count != 1) return WAX_HOST_MISTAKE;
    if (args[0].as.whole > 10) {
        result->as.whole = 0;
        return WAX_FAILED;
    }
    result->kind = WAX_WHOLE;
    result->as.whole = args[0].as.whole * 2;
    return WAX_OK;
}

static wax_status wait_door(void *data, const wax_arg *args, size_t count, wax_arg *result) {
    (void)data;
    (void)args;
    (void)count;
    (void)result;
    return WAX_PAUSED;
}

static wax_doors *make_doors(game *the_game) {
    wax_doors *doors = wax_doors_new();
    const char *two_wholes[] = {"Whole", "Whole"};
    const char *one_text[] = {"Text"};
    const char *one_whole[] = {"Whole"};
    const char *pick_failures[] = {"too_high"};
    CHECK(wax_doors_add(doors, "damage", two_wholes, 2, "Nothing", NULL, 0, 0, damage, the_game) == WAX_OK);
    CHECK(wax_doors_add(doors, "shout", one_text, 1, "Text", NULL, 0, 0, shout, the_game) == WAX_OK);
    CHECK(wax_doors_add(doors, "pick", one_whole, 1, "Whole", pick_failures, 1, 0, pick, the_game) == WAX_OK);
    CHECK(wax_doors_add(doors, "wait", one_whole, 1, "Nothing", NULL, 0, 1, wait_door, the_game) == WAX_OK);
    const char *labels[] = {"seconds"};
    CHECK(wax_doors_labels(doors, "wait", labels, 1) == WAX_OK);
    return doors;
}

static const char *SCRIPT =
    "proc fight[rounds: Whole] -> Whole\n"
    "    mut total = 0\n"
    "    for round from 1 to rounds\n"
    "        damage(round, 3)\n"
    "        total = total + round\n"
    "    end\n"
    "    return total\n"
    "end\n"
    "\n"
    "proc greet[name: Text] -> Text\n"
    "    return shout(\"hi {name}\")\n"
    "end\n"
    "\n"
    "proc doubled[n: Whole] -> Whole ! [too_high]\n"
    "    return pick(n)?\n"
    "end\n"
    "\n"
    "proc patrol\n"
    "    wait(seconds: 2)\n"
    "    damage(1, 1)\n"
    "end\n"
    "\n"
    "proc names -> List<Text>\n"
    "    return [\"Ann\", \"Bo\"]\n"
    "end\n";

static void test_calls(void) {
    game the_game = {0};
    wax_doors *doors = make_doors(&the_game);
    wax_script *script = NULL;
    CHECK(wax_script_load("game.wax", SCRIPT, strlen(SCRIPT), doors, LIMITS, &script) == WAX_OK);

    wax_arg four = {WAX_WHOLE, {.whole = 4}};
    wax_arg result;
    wax_task task = 0;
    CHECK(wax_script_call(script, "fight", NULL, &four, 1, &result, &task) == WAX_OK);
    CHECK(result.kind == WAX_WHOLE && result.as.whole == 10);
    CHECK(the_game.hurt_total == 30 && the_game.calls == 4);

    /* Text through a door both ways. */
    wax_arg name = {WAX_TEXT, {.value = wax_value_text("Ann", 3)}};
    CHECK(wax_script_call(script, "greet", NULL, &name, 1, &result, &task) == WAX_OK);
    CHECK(result.kind == WAX_TEXT);
    const char *text = NULL;
    size_t length = 0;
    CHECK(wax_value_get_text(result.as.value, &text, &length) == WAX_OK);
    CHECK(length == 7 && memcmp(text, "hi Ann!", 7) == 0);
    CHECK(strcmp(the_game.last_said, "hi Ann") == 0);
    /* A reader on the wrong kind says so, rather than giving something. */
    size_t count = 0;
    CHECK(wax_value_count(result.as.value, &count) == WAX_HOST_MISTAKE);
    CHECK(wax_message_count() == 1 && wax_message_kind_of(0) == WAX_MISTAKE);
    /* The message names the function and the parameter, so the host's programmer can find the line. */
    CHECK(strcmp(wax_message_what(0), "`wax_value_count`: `value` is `WAX_TEXT`, which has no items") == 0);
    CHECK(wax_value_count(result.as.value, NULL) == WAX_HOST_MISTAKE);
    wax_value_free(result.as.value);
    wax_value_free(name.as.value);
    /* A freed handle is refused, never followed, and so is one made up. */
    CHECK(wax_value_get_text(name.as.value, &text, &length) == WAX_HOST_MISTAKE);
    CHECK(wax_value_get_text((wax_value *)(uintptr_t)0xdeadbeef, &text, &length) == WAX_HOST_MISTAKE);
    wax_arg freed = {WAX_TEXT, {.value = name.as.value}};
    CHECK(wax_script_call(script, "greet", NULL, &freed, 1, &result, &task) == WAX_HOST_MISTAKE);
    CHECK(strcmp(wax_message_what(0), "`wax_script_call`: `args[0]` is a value handle that's freed, or was never one") == 0);

    /* A failure on purpose: the script carries on. */
    wax_arg twelve = {WAX_WHOLE, {.whole = 12}};
    CHECK(wax_script_call(script, "doubled", NULL, &twelve, 1, &result, &task) == WAX_FAILED);
    CHECK(wax_message_count() == 1 && wax_message_kind_of(0) == WAX_FAILURE);
    CHECK(wax_message_what(0) != NULL && strcmp(wax_message_what(0), "too_high") == 0);
    wax_arg three = {WAX_WHOLE, {.whole = 3}};
    CHECK(wax_script_call(script, "doubled", NULL, &three, 1, &result, &task) == WAX_OK);
    CHECK(result.as.whole == 6);

    /* A pause, resumed, then another, cancelled. */
    CHECK(wax_script_call(script, "patrol", NULL, NULL, 0, &result, &task) == WAX_PAUSED);
    wax_task again = 0;
    CHECK(wax_script_resume(script, task, NULL, &result, &again) == WAX_OK);
    CHECK(the_game.hurt_total == 31);
    CHECK(wax_script_call(script, "patrol", NULL, NULL, 0, &result, &task) == WAX_PAUSED);
    CHECK(wax_script_cancel(script, task) == WAX_OK);
    CHECK(wax_script_cancel(script, task) != WAX_OK);

    /* A list, read item by item. */
    CHECK(wax_script_call(script, "names", NULL, NULL, 0, &result, &task) == WAX_OK);
    CHECK(result.kind == WAX_LIST && wax_value_count(result.as.value, &count) == WAX_OK && count == 2);
    wax_arg item;
    CHECK(wax_value_item(result.as.value, 1, &item) == WAX_OK && item.kind == WAX_TEXT);
    CHECK(wax_value_get_text(item.as.value, &text, &length) == WAX_OK && length == 2 && memcmp(text, "Bo", 2) == 0);
    CHECK(wax_value_item(result.as.value, 2, &item) == WAX_HOST_MISTAKE);
    wax_value_free(result.as.value);
    CHECK(wax_script_memory_in_use(script) > 0);

    /* Built into a file, loaded from it, the same. */
    const char *names[] = {"game.wax"};
    const char *sources[] = {SCRIPT};
    const uint8_t *bytes = NULL;
    size_t size = 0;
    CHECK(wax_build(doors, names, sources, 1, &bytes, &size) == WAX_OK && size > 0);
    wax_script *compiled = NULL;
    CHECK(wax_script_load_compiled(bytes, size, "game.wax", doors, LIMITS, &compiled) == WAX_OK);
    CHECK(wax_script_call(compiled, "fight", NULL, &four, 1, &result, &task) == WAX_OK && result.as.whole == 10);
    wax_script_free(compiled);
    /* D169: doors that require a signature refuse a key of the wrong size, and then any file not signed with it. */
    static const uint8_t short_key[10] = {0};
    CHECK(wax_doors_require_signature(doors, short_key, sizeof short_key) == WAX_REFUSED);
    static uint8_t key[1312];
    for (size_t at = 0; at < sizeof key; at++) key[at] = (uint8_t)(at * 7);
    if (wax_doors_require_signature(doors, key, sizeof key) == WAX_OK) {
        CHECK(wax_script_load_compiled(bytes, size, "game.wax", doors, LIMITS, &compiled) == WAX_REFUSED);
    }
    wax_bytes_free(bytes);

    wax_script_free(script);
    CHECK(wax_script_call(script, "fight", NULL, &four, 1, &result, &task) == WAX_HOST_MISTAKE);
    wax_doors_free(doors);
}

static void test_messages(void) {
    game the_game = {0};
    wax_doors *doors = make_doors(&the_game);
    wax_script *script = NULL;
    const char *bad = "x = 1\nx = 2\n";
    CHECK(wax_script_load("bad.wax", bad, strlen(bad), doors, LIMITS, &script) == WAX_REFUSED);
    CHECK(script == NULL);
    CHECK(wax_message_count() >= 1);
    CHECK(wax_message_kind_of(0) == WAX_REFUSAL);
    /* With a fix, the whole text shows `fix:` in place of `what:` (Unio D101); the part is there all the same. */
    CHECK(wax_message_what(0) != NULL && strstr(wax_message_what(0), "can't be given a new value") != NULL);
    CHECK(wax_message_text(0) != NULL && strstr(wax_message_text(0), "fix:") != NULL);
    CHECK(wax_message_fix(0) != NULL && strcmp(wax_message_fix(0), "mut x = 1") == 0);
    CHECK(wax_message_why(0) != NULL);
    CHECK(wax_message_file(0) != NULL && strcmp(wax_message_file(0), "bad.wax") == 0);
    CHECK(wax_message_line(0) == 1 && wax_message_column(0) == 1 && wax_message_width(0) == 1);
    CHECK(wax_message_what(99) == NULL && wax_message_line(99) == 0);
    /* A script that stops: a failed check, with its place. */
    const char *stops = "proc boom -> Whole\n    return 9223372036854775807 + 1\nend\n";
    CHECK(wax_script_load("stops.wax", stops, strlen(stops), doors, LIMITS, &script) == WAX_OK);
    wax_arg result;
    wax_task task;
    CHECK(wax_script_call(script, "boom", NULL, NULL, 0, &result, &task) == WAX_STOPPED);
    CHECK(wax_message_kind_of(0) == WAX_CHECK_FAILED && wax_message_line(0) == 2);
    CHECK(wax_message_when(0) != NULL);
    /* The next call clears them, as errno is cleared. */
    CHECK(wax_script_call(script, "boom", NULL, NULL, 0, &result, &task) == WAX_STOPPED);
    wax_script_free(script);
    CHECK(wax_message_count() == 0);
    /* Mistakes in the host's own calls. */
    CHECK(wax_script_load(NULL, bad, strlen(bad), doors, LIMITS, &script) == WAX_HOST_MISTAKE);
    const char *nothing[] = {"Record"};
    CHECK(wax_doors_add(doors, "odd", nothing, 1, "Nothing", NULL, 0, 0, damage, &the_game) == WAX_HOST_MISTAKE);
    CHECK(wax_doors_add(doors, "damage", nothing, 0, "Nothing", NULL, 0, 0, damage, &the_game) == WAX_REFUSED);
    wax_doors_free(doors);
}

/* Each thread loads and refuses scripts of its own; its messages are its own. */
static void *thread_main(void *argument) {
    long number = (long)argument;
    game the_game = {0};
    wax_doors *doors = make_doors(&the_game);
    for (int round = 0; round < 200; round++) {
        char bad[64];
        int length = snprintf(bad, sizeof bad, "x%ld = 1\nx%ld = 2\n", number, number);
        wax_script *script = NULL;
        CHECK(wax_script_load("thread.wax", bad, (size_t)length, doors, LIMITS, &script) == WAX_REFUSED);
        char wanted[32];
        snprintf(wanted, sizeof wanted, "`x%ld`", number);
        CHECK(wax_message_count() >= 1 && strstr(wax_message_what(0), wanted) != NULL);
    }
    wax_doors_free(doors);
    return NULL;
}

static void test_threads(void) {
    pthread_t threads[4];
    for (long number = 0; number < 4; number++) pthread_create(&threads[number], NULL, thread_main, (void *)number);
    for (int number = 0; number < 4; number++) pthread_join(threads[number], NULL);
}

static double seconds(void) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    return (double)now.tv_sec + (double)now.tv_nsec / 1e9;
}

/* The door-call row: a script calling a door that takes two whole numbers, `count` times. */
static int bench(long count) {
    game the_game = {0};
    wax_doors *doors = make_doors(&the_game);
    const char *source = "proc run[n: Whole]\n    for round from 1 to n\n        damage(round, 2)\n    end\nend\n";
    wax_script *script = NULL;
    wax_limits limits = {UINT64_MAX, 1 << 24};
    if (wax_script_load("bench.wax", source, strlen(source), doors, limits, &script) != WAX_OK) return 1;
    wax_arg n = {WAX_WHOLE, {.whole = count}};
    wax_arg result;
    wax_task task;
    double best = 1e9;
    for (int run = 0; run < 7; run++) {
        the_game.hurt_total = 0;
        double started = seconds();
        if (wax_script_call(script, "run", NULL, &n, 1, &result, &task) != WAX_OK) return 1;
        double took = seconds() - started;
        if (took < best) best = took;
    }
    printf("result %lld\n", (long long)the_game.hurt_total);
    printf("door_call_each %.12f\n", best / (double)count);
    wax_script_free(script);
    wax_doors_free(doors);
    return 0;
}

int main(int argc, char **argv) {
    if (argc == 3 && strcmp(argv[1], "bench") == 0) return bench(atol(argv[2]));
    test_calls();
    test_messages();
    test_threads();
    if (failures > 0) {
        fprintf(stderr, "%d checks failed\n", failures);
        return 1;
    }
    printf("every check passed\n");
    return 0;
}
