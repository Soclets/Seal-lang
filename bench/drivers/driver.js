
// The benchmark driver: checks nothing itself, only runs `run` and prints what it gave and how long it took.
{
    const argument = ARGUMENT, warmups = WARMUPS, runs = RUNS;
    const started = performance.now();
    let result = run(argument);
    const first = (performance.now() - started) / 1000;
    for (let index = 0; index < warmups; index++) {
        run(argument);
    }
    const times = [];
    for (let index = 0; index < runs; index++) {
        const before = performance.now();
        result = run(argument);
        times.push(((performance.now() - before) / 1000).toFixed(9));
    }
    if (typeof result === "number") {
        console.log("result " + result);
    } else {
        console.log("result text " + result.length + " " + result.slice(-12));
    }
    console.log("first " + first.toFixed(9));
    console.log("times " + times.join(" "));
}
