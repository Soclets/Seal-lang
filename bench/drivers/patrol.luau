-- The patrolling-scripts benchmark for Luau: COUNT scripts, each loaded from its own copy of the source and run as
-- a coroutine that pauses at `wait`; then every one is resumed ROUNDS times.
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
collectgarbage("collect")
local before = collectgarbage("count")
local started = os.clock()
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
print(string.format("resume_each %.9f", resumed / (count * rounds)))
print(string.format("call_each %.9f", called / (count * rounds)))
print(string.format("memory_each %.0f", (after - before) * 1024 / count))
