-- Purpose: calls. A function calls itself `depth` deep, 20,000 times over, and does nothing else.
local function down(depth)
    if depth == 0 then
        return 0
    end
    return 1 + down(depth - 1)
end

function run(depth)
    local total = 0
    for _ = 1, 20000 do
        total = total + down(depth)
    end
    return total
end
