const PI = 3.141592653589793;
const SOLAR_MASS = 4.0 * PI * PI;
const DAYS = 365.24;

function energy(bodies) {
    let total = 0.0;
    for (let i = 0; i < bodies.length; i++) {
        const body = bodies[i];
        total = total + 0.5 * body.mass * (body.vx * body.vx + body.vy * body.vy + body.vz * body.vz);
        for (let j = 0; j < bodies.length; j++) {
            if (j > i) {
                const other = bodies[j];
                const dx = body.x - other.x;
                const dy = body.y - other.y;
                const dz = body.z - other.z;
                total = total - body.mass * other.mass / Math.sqrt(dx * dx + dy * dy + dz * dz);
            }
        }
    }
    return total;
}

function advance(bodies, dt) {
    const count = bodies.length;
    for (let i = 0; i < count; i++) {
        const a = bodies[i];
        for (let j = i + 1; j < count; j++) {
            const b = bodies[j];
            const dx = a.x - b.x;
            const dy = a.y - b.y;
            const dz = a.z - b.z;
            const distanceSquared = dx * dx + dy * dy + dz * dz;
            const mag = dt / (distanceSquared * Math.sqrt(distanceSquared));
            a.vx = a.vx - dx * b.mass * mag;
            a.vy = a.vy - dy * b.mass * mag;
            a.vz = a.vz - dz * b.mass * mag;
            b.vx = b.vx + dx * a.mass * mag;
            b.vy = b.vy + dy * a.mass * mag;
            b.vz = b.vz + dz * a.mass * mag;
        }
        a.x = a.x + dt * a.vx;
        a.y = a.y + dt * a.vy;
        a.z = a.z + dt * a.vz;
    }
}

function run(steps) {
    const bodies = [
        { x: 0.0, y: 0.0, z: 0.0, vx: 0.0, vy: 0.0, vz: 0.0, mass: SOLAR_MASS },
        { x: 4.84143144246472090, y: -1.16032004402742839, z: -0.103622044471123109,
          vx: 0.00166007664274403694 * DAYS, vy: 0.00769901118419740425 * DAYS,
          vz: -0.0000690460016972063023 * DAYS, mass: 0.000954791938424326609 * SOLAR_MASS },
        { x: 8.34336671824457987, y: 4.12479856412430479, z: -0.403523417114321381,
          vx: -0.00276742510726862411 * DAYS, vy: 0.00499852801234917238 * DAYS,
          vz: 0.0000230417297573763929 * DAYS, mass: 0.000285885980666130812 * SOLAR_MASS },
        { x: 12.8943695621391310, y: -15.1111514016986312, z: -0.223307578892655734,
          vx: 0.00296460137564761618 * DAYS, vy: 0.00237847173959480950 * DAYS,
          vz: -0.0000296589568540237556 * DAYS, mass: 0.0000436624404335156298 * SOLAR_MASS },
        { x: 15.3796971148509165, y: -25.9193146099879641, z: 0.179258772950371181,
          vx: 0.00268067772490389322 * DAYS, vy: 0.00162824170038242295 * DAYS,
          vz: -0.0000951592254519715870 * DAYS, mass: 0.0000515138902046611451 * SOLAR_MASS },
    ];
    let px = 0.0, py = 0.0, pz = 0.0;
    for (const body of bodies) {
        px = px + body.vx * body.mass;
        py = py + body.vy * body.mass;
        pz = pz + body.vz * body.mass;
    }
    const sun = bodies[0];
    sun.vx = -px / SOLAR_MASS;
    sun.vy = -py / SOLAR_MASS;
    sun.vz = -pz / SOLAR_MASS;
    for (let step = 0; step < steps; step++) {
        advance(bodies, 0.01);
    }
    return energy(bodies);
}
