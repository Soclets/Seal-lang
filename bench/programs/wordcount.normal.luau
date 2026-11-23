function run(words)
    local counts = {}
    local seed = 42
    for _ = 1, words do
        seed = (seed * 1664525 + 1013904223) % 4294967296
        local word = "word" .. (math.floor(seed / 65536) % 5000)
        counts[word] = (counts[word] or 0) + 1
    end
    local most = 0
    local kinds = 0
    for _, count in pairs(counts) do
        kinds = kinds + 1
        if count > most then
            most = count
        end
    end
    return kinds * 1000000 + most
end
