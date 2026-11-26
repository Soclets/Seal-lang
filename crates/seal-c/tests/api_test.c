/* Seal's C API tested from C (D136, D151, D152), built and run by tests/c_api.rs under AddressSanitizer where the
 * system's compiler has it. Every check that fails prints what and where, and the program exits with 1. With the
 * argument `bench N`, it times N door calls through the API instead and prints the time each took. */
#include <pthread.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include "seal.h"

static int failures = 0;

#define CHECK(condition)                                                                                   \
    do {                                                                                                   \
        if (!(condition)) {                                                                                \
            fprintf(stderr, "api_test.c:%d: check failed: %s\n", __LINE__, #condition);                    \
            failures++;                                                                                    \
        }                                                                                                  \
    } while (0)

static const seal_limits LIMITS = {1000000, 1 << 20};

/* The host's own data: what its doors were given. */
typedef struct {
    int64_t hurt_total;
    int calls;
    char last_said[64];
} game;

static seal_status damage(void *data, const seal_arg *args, size_t count, seal_arg *result) {
    game *the_game = data;
    if (count != 2 || args[0].kind != SEAL_WHOLE || args[1].kind != SEAL_WHOLE) return SEAL_HOST_MISTAKE;
    the_game->hurt_total += args[0].as.whole * args[1].as.whole;
    the_game->calls++;
    result->kind = SEAL_NOTHING;
    return SEAL_OK;
}

/* Text in, text out: the argument is lent for the call, the result is a new value Seal takes over. */
static seal_status shout(void *data, const seal_arg *args, size_t count, seal_arg *result) {
    game *the_game = data;
    const char *text = NULL;
    size_t length = 0;
    if (count != 1 || args[0].kind != SEAL_TEXT) return SEAL_HOST_MISTAKE;
    if (seal_value_get_text(args[0].as.value, &text, &length) != SEAL_OK) return SEAL_HOST_MISTAKE;
    snprintf(the_game->last_said, sizeof the_game->last_said, "%.*s", (int)length, text);
    char loud[80];
    int written = snprintf(loud, sizeof loud, "%.*s!", (int)length, text);
    result->kind = SEAL_TEXT;
    result->as.value = seal_value_text(loud, (size_t)written);
    /* A lent value is the caller's: freeing it does nothing. */
    seal_value_free(args[0].as.value);
    return SEAL_OK;
}

/* Fails with its first failure, `too_high`, past 10. */
static seal_status pick(void *data, const seal_arg *args, size_t count, seal_arg *result) {
    (void)data;
    if (count != 1) return SEAL_HOST_MISTAKE;
    if (args[0].as.whole > 10) {
        result->as.whole = 0;
        return SEAL_FAILED;
    }
    result->kind = SEAL_WHOLE;
    result->as.whole = args[0].as.whole * 2;
    return SEAL_OK;
}

static seal_status wait_door(void *data, const seal_arg *args, size_t count, seal_arg *result) {
    (void)data;
    (void)args;
    (void)count;
    (void)result;
    return SEAL_PAUSED;
}

static seal_doors *make_doors(game *the_game) {
    seal_doors *doors = seal_doors_new();
    const char *two_wholes[] = {"Whole", "Whole"};
    const char *one_text[] = {"Text"};
    const char *one_whole[] = {"Whole"};
    const char *pick_failures[] = {"too_high"};
    CHECK(seal_doors_add(doors, "damage", two_wholes, 2, "Nothing", NULL, 0, 0, damage, the_game) == SEAL_OK);
    CHECK(seal_doors_add(doors, "shout", one_text, 1, "Text", NULL, 0, 0, shout, the_game) == SEAL_OK);
    CHECK(seal_doors_add(doors, "pick", one_whole, 1, "Whole", pick_failures, 1, 0, pick, the_game) == SEAL_OK);
    CHECK(seal_doors_add(doors, "wait", one_whole, 1, "Nothing", NULL, 0, 1, wait_door, the_game) == SEAL_OK);
    const char *labels[] = {"seconds"};
    CHECK(seal_doors_labels(doors, "wait", labels, 1) == SEAL_OK);
    return doors;
}

static const char *SCRIPT =
    "proc fight[rounds: Whole] -> Whole\n"
    "    mut total = 0\n"
    "    for_each round in range[from: 1, to: rounds]\n"
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
    seal_doors *doors = make_doors(&the_game);
    seal_script *script = NULL;
    CHECK(seal_script_load("game.seal", SCRIPT, strlen(SCRIPT), doors, LIMITS, &script) == SEAL_OK);

    seal_arg four = {SEAL_WHOLE, {.whole = 4}};
    seal_arg result;
    seal_task task = 0;
    CHECK(seal_script_call(script, "fight", NULL, &four, 1, &result, &task) == SEAL_OK);
    CHECK(result.kind == SEAL_WHOLE && result.as.whole == 10);
    CHECK(the_game.hurt_total == 30 && the_game.calls == 4);

    /* Text through a door both ways. */
    seal_arg name = {SEAL_TEXT, {.value = seal_value_text("Ann", 3)}};
    CHECK(seal_script_call(script, "greet", NULL, &name, 1, &result, &task) == SEAL_OK);
    CHECK(result.kind == SEAL_TEXT);
    const char *text = NULL;
    size_t length = 0;
    CHECK(seal_value_get_text(result.as.value, &text, &length) == SEAL_OK);
    CHECK(length == 7 && memcmp(text, "hi Ann!", 7) == 0);
    CHECK(strcmp(the_game.last_said, "hi Ann") == 0);
    /* A reader on the wrong kind says so, rather than giving something. */
    size_t count = 0;
    CHECK(seal_value_count(result.as.value, &count) == SEAL_HOST_MISTAKE);
    CHECK(seal_message_count() == 1 && seal_message_kind_of(0) == SEAL_MISTAKE);
    seal_value_free(result.as.value);
    seal_value_free(name.as.value);
    /* A freed handle is refused, never followed, and so is one made up. */
    CHECK(seal_value_get_text(name.as.value, &text, &length) == SEAL_HOST_MISTAKE);
    CHECK(seal_value_get_text((seal_value *)(uintptr_t)0xdeadbeef, &text, &length) == SEAL_HOST_MISTAKE);
    seal_arg freed = {SEAL_TEXT, {.value = name.as.value}};
    CHECK(seal_script_call(script, "greet", NULL, &freed, 1, &result, &task) == SEAL_HOST_MISTAKE);

    /* A failure on purpose: the script carries on. */
    seal_arg twelve = {SEAL_WHOLE, {.whole = 12}};
    CHECK(seal_script_call(script, "doubled", NULL, &twelve, 1, &result, &task) == SEAL_FAILED);
    CHECK(seal_message_count() == 1 && seal_message_kind_of(0) == SEAL_FAILURE);
    CHECK(seal_message_what(0) != NULL && strcmp(seal_message_what(0), "too_high") == 0);
    seal_arg three = {SEAL_WHOLE, {.whole = 3}};
    CHECK(seal_script_call(script, "doubled", NULL, &three, 1, &result, &task) == SEAL_OK);
    CHECK(result.as.whole == 6);

    /* A pause, resumed, then another, cancelled. */
    CHECK(seal_script_call(script, "patrol", NULL, NULL, 0, &result, &task) == SEAL_PAUSED);
    seal_task again = 0;
    CHECK(seal_script_resume(script, task, NULL, &result, &again) == SEAL_OK);
    CHECK(the_game.hurt_total == 31);
    CHECK(seal_script_call(script, "patrol", NULL, NULL, 0, &result, &task) == SEAL_PAUSED);
    CHECK(seal_script_cancel(script, task) == SEAL_OK);
    CHECK(seal_script_cancel(script, task) != SEAL_OK);

    /* A list, read item by item. */
    CHECK(seal_script_call(script, "names", NULL, NULL, 0, &result, &task) == SEAL_OK);
    CHECK(result.kind == SEAL_LIST && seal_value_count(result.as.value, &count) == SEAL_OK && count == 2);
    seal_arg item;
    CHECK(seal_value_item(result.as.value, 1, &item) == SEAL_OK && item.kind == SEAL_TEXT);
    CHECK(seal_value_get_text(item.as.value, &text, &length) == SEAL_OK && length == 2 && memcmp(text, "Bo", 2) == 0);
    CHECK(seal_value_item(result.as.value, 2, &item) == SEAL_HOST_MISTAKE);
    seal_value_free(result.as.value);
    CHECK(seal_script_memory_in_use(script) > 0);

    /* Built into a file, loaded from it, the same. */
    const char *names[] = {"game.seal"};
    const char *sources[] = {SCRIPT};
    const uint8_t *bytes = NULL;
    size_t size = 0;
    CHECK(seal_build(doors, names, sources, 1, &bytes, &size) == SEAL_OK && size > 0);
    seal_script *compiled = NULL;
    CHECK(seal_script_load_compiled(bytes, size, "game.seal", doors, LIMITS, &compiled) == SEAL_OK);
    CHECK(seal_script_call(compiled, "fight", NULL, &four, 1, &result, &task) == SEAL_OK && result.as.whole == 10);
    seal_script_free(compiled);
    seal_bytes_free(bytes);

    seal_script_free(script);
    CHECK(seal_script_call(script, "fight", NULL, &four, 1, &result, &task) == SEAL_HOST_MISTAKE);
    seal_doors_free(doors);
}

static void test_messages(void) {
    game the_game = {0};
    seal_doors *doors = make_doors(&the_game);
    seal_script *script = NULL;
    const char *bad = "x = 1\nx = 2\n";
    CHECK(seal_script_load("bad.seal", bad, strlen(bad), doors, LIMITS, &script) == SEAL_REFUSED);
    CHECK(script == NULL);
    CHECK(seal_message_count() >= 1);
    CHECK(seal_message_kind_of(0) == SEAL_REFUSAL);
    /* With a fix, the whole text shows `fix:` in place of `what:` (Unio D101); the part is there all the same. */
    CHECK(seal_message_what(0) != NULL && strstr(seal_message_what(0), "can't be given a new value") != NULL);
    CHECK(seal_message_text(0) != NULL && strstr(seal_message_text(0), "fix:") != NULL);
    CHECK(seal_message_fix(0) != NULL && strcmp(seal_message_fix(0), "mut x = 1") == 0);
    CHECK(seal_message_why(0) != NULL);
    CHECK(seal_message_file(0) != NULL && strcmp(seal_message_file(0), "bad.seal") == 0);
    CHECK(seal_message_line(0) == 1 && seal_message_column(0) == 1 && seal_message_width(0) == 1);
    CHECK(seal_message_what(99) == NULL && seal_message_line(99) == 0);
    /* A script that stops: a failed check, with its place. */
    const char *stops = "proc boom -> Whole\n    return 9223372036854775807 + 1\nend\n";
    CHECK(seal_script_load("stops.seal", stops, strlen(stops), doors, LIMITS, &script) == SEAL_OK);
    seal_arg result;
    seal_task task;
    CHECK(seal_script_call(script, "boom", NULL, NULL, 0, &result, &task) == SEAL_STOPPED);
    CHECK(seal_message_kind_of(0) == SEAL_CHECK_FAILED && seal_message_line(0) == 2);
    CHECK(seal_message_when(0) != NULL);
    /* The next call clears them, as errno is cleared. */
    CHECK(seal_script_call(script, "boom", NULL, NULL, 0, &result, &task) == SEAL_STOPPED);
    seal_script_free(script);
    CHECK(seal_message_count() == 0);
    /* Mistakes in the host's own calls. */
    CHECK(seal_script_load(NULL, bad, strlen(bad), doors, LIMITS, &script) == SEAL_HOST_MISTAKE);
    const char *nothing[] = {"Record"};
    CHECK(seal_doors_add(doors, "odd", nothing, 1, "Nothing", NULL, 0, 0, damage, &the_game) == SEAL_HOST_MISTAKE);
    CHECK(seal_doors_add(doors, "damage", nothing, 0, "Nothing", NULL, 0, 0, damage, &the_game) == SEAL_REFUSED);
    seal_doors_free(doors);
}

/* Each thread loads and refuses scripts of its own; its messages are its own. */
static void *thread_main(void *argument) {
    long number = (long)argument;
    game the_game = {0};
    seal_doors *doors = make_doors(&the_game);
    for (int round = 0; round < 200; round++) {
        char bad[64];
        int length = snprintf(bad, sizeof bad, "x%ld = 1\nx%ld = 2\n", number, number);
        seal_script *script = NULL;
        CHECK(seal_script_load("thread.seal", bad, (size_t)length, doors, LIMITS, &script) == SEAL_REFUSED);
        char wanted[32];
        snprintf(wanted, sizeof wanted, "`x%ld`", number);
        CHECK(seal_message_count() >= 1 && strstr(seal_message_what(0), wanted) != NULL);
    }
    seal_doors_free(doors);
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
    seal_doors *doors = make_doors(&the_game);
    const char *source = "proc run[n: Whole]\n    for_each round in range[from: 1, to: n]\n        damage(round, 2)\n    end\nend\n";
    seal_script *script = NULL;
    seal_limits limits = {UINT64_MAX, 1 << 24};
    if (seal_script_load("bench.seal", source, strlen(source), doors, limits, &script) != SEAL_OK) return 1;
    seal_arg n = {SEAL_WHOLE, {.whole = count}};
    seal_arg result;
    seal_task task;
    double best = 1e9;
    for (int run = 0; run < 7; run++) {
        the_game.hurt_total = 0;
        double started = seconds();
        if (seal_script_call(script, "run", NULL, &n, 1, &result, &task) != SEAL_OK) return 1;
        double took = seconds() - started;
        if (took < best) best = took;
    }
    printf("result %lld\n", (long long)the_game.hurt_total);
    printf("door_call_each %.12f\n", best / (double)count);
    seal_script_free(script);
    seal_doors_free(doors);
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
