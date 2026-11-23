function run(count)
    local text = ""
    for number = 0, count - 1 do
        text = text .. number .. ","
    end
    return text
end
