-- Purpose: a table with text keys. 1,000 keys are made once, then each is looked up and counted `rounds` times.
function run(rounds)
    local keys = {}
    local counts = {}
    for i = 1, 1000 do
        table.insert(keys, "key" .. i)
        counts["key" .. i] = 0
    end
    for _ = 1, rounds do
        for _, key in ipairs(keys) do
            counts[key] = (counts[key] or 0) + 1
        end
    end
    local kinds = 0
    for _ in pairs(counts) do
        kinds = kinds + 1
    end
    return counts["key500"] * 1000 + kinds
end
