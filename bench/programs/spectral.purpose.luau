-- Purpose: loops over a list of floats. A list of 1,000 floats is summed `rounds` times.
function run(rounds)
    local values = {}
    for i = 1, 1000 do
        table.insert(values, i * 0.5)
    end
    local total = 0.0
    for _ = 1, rounds do
        for _, value in ipairs(values) do
            total = total + value
        end
    end
    return total
end
