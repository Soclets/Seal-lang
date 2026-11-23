
-- The benchmark driver: checks nothing itself, only runs `run` and prints what it gave and how long it took.
local argument, warmups, runs = ARGUMENT, WARMUPS, RUNS
local started = os.clock()
local result = run(argument)
local first = os.clock() - started
for _ = 1, warmups do
    run(argument)
end
local times = {}
for index = 1, runs do
    local before = os.clock()
    result = run(argument)
    times[index] = string.format("%.9f", os.clock() - before)
end
if type(result) == "number" then
    print("result " .. string.format("%.17g", result))
else
    print("result text " .. #result .. " " .. string.sub(result, -12))
end
print("first " .. string.format("%.9f", first))
print("times " .. table.concat(times, " "))
