-- Purpose: making and dropping values on the heap. A tree 10 deep is made and dropped `rounds` times, never read.
local function build(depth)
    if depth == 0 then
        return {}
    end
    return { left = build(depth - 1), right = build(depth - 1) }
end

function run(rounds)
    local made = 0
    for _ = 1, rounds do
        local _tree = build(10)
        made = made + 1
    end
    return made
end
