local PI = 3.141592653589793
local SOLAR_MASS = 4.0 * PI * PI
local DAYS = 365.24

local function energy(bodies)
    local total = 0.0
    for i, body in ipairs(bodies) do
        total = total + 0.5 * body.mass * (body.vx * body.vx + body.vy * body.vy + body.vz * body.vz)
        for j, other in ipairs(bodies) do
            if j > i then
                local dx = body.x - other.x
                local dy = body.y - other.y
                local dz = body.z - other.z
                total = total - body.mass * other.mass / math.sqrt(dx * dx + dy * dy + dz * dz)
            end
        end
    end
    return total
end

local function advance(bodies, dt)
    local count = #bodies
    for i = 1, count do
        local a = bodies[i]
        for j = i + 1, count do
            local b = bodies[j]
            local dx = a.x - b.x
            local dy = a.y - b.y
            local dz = a.z - b.z
            local distanceSquared = dx * dx + dy * dy + dz * dz
            local mag = dt / (distanceSquared * math.sqrt(distanceSquared))
            a.vx = a.vx - dx * b.mass * mag
            a.vy = a.vy - dy * b.mass * mag
            a.vz = a.vz - dz * b.mass * mag
            b.vx = b.vx + dx * a.mass * mag
            b.vy = b.vy + dy * a.mass * mag
            b.vz = b.vz + dz * a.mass * mag
        end
        a.x = a.x + dt * a.vx
        a.y = a.y + dt * a.vy
        a.z = a.z + dt * a.vz
    end
end

function run(steps)
    local bodies = {
        { x = 0.0, y = 0.0, z = 0.0, vx = 0.0, vy = 0.0, vz = 0.0, mass = SOLAR_MASS },
        { x = 4.84143144246472090, y = -1.16032004402742839, z = -0.103622044471123109,
          vx = 0.00166007664274403694 * DAYS, vy = 0.00769901118419740425 * DAYS,
          vz = -0.0000690460016972063023 * DAYS, mass = 0.000954791938424326609 * SOLAR_MASS },
        { x = 8.34336671824457987, y = 4.12479856412430479, z = -0.403523417114321381,
          vx = -0.00276742510726862411 * DAYS, vy = 0.00499852801234917238 * DAYS,
          vz = 0.0000230417297573763929 * DAYS, mass = 0.000285885980666130812 * SOLAR_MASS },
        { x = 12.8943695621391310, y = -15.1111514016986312, z = -0.223307578892655734,
          vx = 0.00296460137564761618 * DAYS, vy = 0.00237847173959480950 * DAYS,
          vz = -0.0000296589568540237556 * DAYS, mass = 0.0000436624404335156298 * SOLAR_MASS },
        { x = 15.3796971148509165, y = -25.9193146099879641, z = 0.179258772950371181,
          vx = 0.00268067772490389322 * DAYS, vy = 0.00162824170038242295 * DAYS,
          vz = -0.0000951592254519715870 * DAYS, mass = 0.0000515138902046611451 * SOLAR_MASS },
    }
    local px, py, pz = 0.0, 0.0, 0.0
    for _, body in ipairs(bodies) do
        px = px + body.vx * body.mass
        py = py + body.vy * body.mass
        pz = pz + body.vz * body.mass
    end
    local sun = bodies[1]
    sun.vx = -px / SOLAR_MASS
    sun.vy = -py / SOLAR_MASS
    sun.vz = -pz / SOLAR_MASS
    for _ = 1, steps do
        advance(bodies, 0.01)
    end
    return energy(bodies)
end
