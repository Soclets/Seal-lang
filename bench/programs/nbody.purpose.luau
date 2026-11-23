-- Purpose: float maths on a record's fields. One point is moved `steps` times; nothing is called or allocated.
function run(steps)
    local point = { x = 0.0, y = 0.0, vx = 1.0, vy = 0.5 }
    for _ = 1, steps do
        point.vx = point.vx * 0.999 + 0.001
        point.vy = point.vy * 0.999 - 0.0005
        point.x = point.x + point.vx * 0.01
        point.y = point.y + point.vy * 0.01
    end
    return point.x + point.y
end
