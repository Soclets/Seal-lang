-- Purpose: joining text. One text grows by two characters, `count` times.
function run(count)
    local text = ""
    for _ = 1, count do
        text = text .. "ab"
    end
    return text
end
