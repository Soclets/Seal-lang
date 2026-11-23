local function build(depth)
    if depth == 0 then
        return {}
    end
    return { left = build(depth - 1), right = build(depth - 1) }
end

local function count(tree)
    if tree.left == nil then
        return 1
    end
    return 1 + count(tree.left) + count(tree.right)
end

function run(maxDepth)
    local stretch = build(maxDepth + 1)
    local total = count(stretch)
    local longLived = build(maxDepth)
    local depth = 4
    while depth <= maxDepth do
        local rounds = 1
        for _ = 1, maxDepth - depth + 4 do
            rounds = rounds * 2
        end
        local check = 0
        for _ = 1, rounds do
            check = check + count(build(depth))
        end
        total = total + check
        depth = depth + 2
    end
    return total + count(longLived)
end
