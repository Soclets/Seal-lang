function evalA(i, j) {
    return 1.0 / (Math.floor((i + j) * (i + j + 1) / 2) + i + 1);
}

function times(u, size) {
    const out = [];
    for (let i = 0; i < size; i++) {
        let sum = 0.0;
        for (let j = 0; j < size; j++) {
            sum = sum + evalA(i, j) * u[j];
        }
        out.push(sum);
    }
    return out;
}

function timesTransposed(u, size) {
    const out = [];
    for (let i = 0; i < size; i++) {
        let sum = 0.0;
        for (let j = 0; j < size; j++) {
            sum = sum + evalA(j, i) * u[j];
        }
        out.push(sum);
    }
    return out;
}

function run(size) {
    let u = [];
    for (let i = 0; i < size; i++) {
        u.push(1.0);
    }
    let v = u;
    for (let round = 0; round < 10; round++) {
        v = timesTransposed(times(u, size), size);
        u = timesTransposed(times(v, size), size);
    }
    let vbv = 0.0, vv = 0.0;
    for (let i = 0; i < size; i++) {
        vbv = vbv + u[i] * v[i];
        vv = vv + v[i] * v[i];
    }
    return Math.sqrt(vbv / vv);
}
