-- The patrolling-scripts benchmark for Luau: COUNT scripts, each loaded from its own copy of the source and run as
-- a coroutine that pauses at `wait`; then every one is resumed ROUNDS times. Loading is timed twice: with the text
-- compiled once and every script made from that, and with every script compiled from the text again.
local count, rounds = COUNT, ROUNDS
local source = [[
local walk_to, wait = ...
local function patrol(guard)
    while true do
        walk_to(guard, 10)
        wait(2)
        walk_to(guard, 0)
        wait(2)
    end
end
local function step(guard)
    walk_to(guard, 1)
end
return patrol, step
]]
local walks = 0
local function walk_to(guard, x)
    walks = walks + 1
end
-- Loading from the same text, compiled once: each script is a fresh pair of functions made by the one chunk.
local started = os.clock()
local chunk = loadstring(source)
local shared = {}
for guard = 1, count do
    local patrol = chunk(walk_to, coroutine.yield)
    local task = coroutine.create(patrol)
    coroutine.resume(task, guard)
    shared[guard] = task
end
local loaded_shared = os.clock() - started
shared = nil
walks = 0
collectgarbage("collect")
local before = collectgarbage("count")
started = os.clock()
local tasks, steps = {}, {}
for guard = 1, count do
    local patrol, step = loadstring(source)(walk_to, coroutine.yield)
    local task = coroutine.create(patrol)
    coroutine.resume(task, guard)
    tasks[guard] = task
    steps[guard] = step
end
local loaded = os.clock() - started
collectgarbage("collect")
local after = collectgarbage("count")
started = os.clock()
for _ = 1, rounds do
    for guard = 1, count do
        coroutine.resume(tasks[guard])
    end
end
local resumed = os.clock() - started
started = os.clock()
for _ = 1, rounds do
    for guard = 1, count do
        steps[guard](guard)
    end
end
local called = os.clock() - started
print("result " .. walks)
print(string.format("load_each %.9f", loaded / count))
print(string.format("load_shared_each %.9f", loaded_shared / count))
-- Loading each from bytecode, as `luajit -b` writes it, where the engine can: LuaJIT's `string.dump`. Luau can't.
if string.dump then
    local dumped = {}
    for guard = 1, count do
        dumped[guard] = string.dump(loadstring("-- guard " .. guard .. "\n" .. source))
    end
    local started_compiled = os.clock()
    for guard = 1, count do
        local patrol = loadstring(dumped[guard])(walk_to, coroutine.yield)
    end
    print(string.format("load_compiled_each %.9f", (os.clock() - started_compiled) / count))
end
print(string.format("resume_each %.9f", resumed / (count * rounds)))
print(string.format("call_each %.9f", called / (count * rounds)))
print(string.format("memory_each %.0f", (after - before) * 1024 / count))
