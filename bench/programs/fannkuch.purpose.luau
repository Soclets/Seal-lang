-- Purpose: shuffling a list in place. A list of 16 whole numbers is reversed `rounds` times, item by item.
function run(rounds)
    local items = {}
    for i = 0, 15 do
        items[i + 1] = i
    end
    for _ = 1, rounds do
        local low = 1
        local high = 16
        while low < high do
            local swap = items[low]
            items[low] = items[high]
            items[high] = swap
            low = low + 1
            high = high - 1
        end
    end
    return items[1] * 100 + items[16]
end
