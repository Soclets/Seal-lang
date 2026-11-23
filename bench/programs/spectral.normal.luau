local function evalA(i, j)
    return 1.0 / (math.floor((i + j) * (i + j + 1) / 2) + i + 1)
end

local function times(u, size)
    local out = {}
    for i = 0, size - 1 do
        local sum = 0.0
        for j = 0, size - 1 do
            sum = sum + evalA(i, j) * u[j + 1]
        end
        out[i + 1] = sum
    end
    return out
end

local function timesTransposed(u, size)
    local out = {}
    for i = 0, size - 1 do
        local sum = 0.0
        for j = 0, size - 1 do
            sum = sum + evalA(j, i) * u[j + 1]
        end
        out[i + 1] = sum
    end
    return out
end

function run(size)
    local u = {}
    for i = 1, size do
        u[i] = 1.0
    end
    local v = u
    for _ = 1, 10 do
        v = timesTransposed(times(u, size), size)
        u = timesTransposed(times(v, size), size)
    end
    local vbv, vv = 0.0, 0.0
    for i = 1, size do
        vbv = vbv + u[i] * v[i]
        vv = vv + v[i] * v[i]
    end
    return math.sqrt(vbv / vv)
end
