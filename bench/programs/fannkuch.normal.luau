function run(n)
    local perm1 = {}
    local count = {}
    for i = 0, n - 1 do
        perm1[i] = i
        count[i] = 0
    end
    local maxFlips = 0
    local checksum = 0
    local permutations = 0
    local r = n
    local done = false
    while not done do
        while r ~= 1 do
            count[r - 1] = r
            r = r - 1
        end
        local perm = {}
        for i = 0, n - 1 do
            perm[i] = perm1[i]
        end
        local flips = 0
        local k = perm[0]
        while k ~= 0 do
            local low = 0
            local high = k
            while low < high do
                local swap = perm[low]
                perm[low] = perm[high]
                perm[high] = swap
                low = low + 1
                high = high - 1
            end
            flips = flips + 1
            k = perm[0]
        end
        if flips > maxFlips then
            maxFlips = flips
        end
        if permutations % 2 == 0 then
            checksum = checksum + flips
        else
            checksum = checksum - flips
        end
        permutations = permutations + 1
        local rotating = true
        while rotating do
            if r == n then
                done = true
                rotating = false
            else
                local first = perm1[0]
                local i = 0
                while i < r do
                    perm1[i] = perm1[i + 1]
                    i = i + 1
                end
                perm1[r] = first
                count[r] = count[r] - 1
                if count[r] > 0 then
                    rotating = false
                else
                    r = r + 1
                end
            end
        end
    end
    return checksum * 100 + maxFlips
end
