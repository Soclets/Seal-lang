/* The door-call benchmark on LuaJIT's C API, beside Wax's (D136, D152): a C function registered with the state, called
 * from a Lua loop `count` times, with its two numbers read the checked way, as Wax's door reads its kinds. Run as
 * `doors MODE COUNT`, where MODE is `joff` (the interpreter alone) or `jit`. Prints the best of seven runs' time per
 * call, and the total the calls added up, for checking. */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

#include <lauxlib.h>
#include <lua.h>
#include <luajit.h>
#include <lualib.h>

static int64_t total = 0;

static int damage(lua_State *state) {
    total += (int64_t)luaL_checkinteger(state, 1) * (int64_t)luaL_checkinteger(state, 2);
    return 0;
}

static double seconds(void) {
    struct timespec now;
    clock_gettime(CLOCK_MONOTONIC, &now);
    return (double)now.tv_sec + (double)now.tv_nsec / 1e9;
}

int main(int argc, char **argv) {
    if (argc != 3) return 2;
    long count = atol(argv[2]);
    lua_State *state = luaL_newstate();
    luaL_openlibs(state);
    if (strcmp(argv[1], "joff") == 0) luaJIT_setmode(state, 0, LUAJIT_MODE_ENGINE | LUAJIT_MODE_OFF);
    lua_register(state, "damage", damage);
    if (luaL_loadstring(state, "local n = ...\nfor round = 1, n do damage(round, 2) end\n") != 0) return 1;
    double best = 1e9;
    for (int run = 0; run < 7; run++) {
        total = 0;
        lua_pushvalue(state, -1);
        lua_pushnumber(state, (lua_Number)count);
        double started = seconds();
        if (lua_pcall(state, 1, 0, 0) != 0) return 1;
        double took = seconds() - started;
        if (took < best) best = took;
    }
    printf("result %lld\n", (long long)total);
    printf("door_call_each %.12f\n", best / (double)count);
    lua_close(state);
    return 0;
}
