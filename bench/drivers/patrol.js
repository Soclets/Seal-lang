// The patrolling-scripts benchmark for JavaScript: COUNT scripts, each compiled from its own copy of the source and
// run as a generator that pauses at `yield`; then every one is resumed ROUNDS times.
{
    const count = COUNT, rounds = ROUNDS;
    const source = `
    function* patrol(guard) {
        while (true) {
            walkTo(guard, 10);
            yield 2;
            walkTo(guard, 0);
            yield 2;
        }
    }
    function step(guard) {
        walkTo(guard, 1);
    }
    return [patrol, step];
    `;
    let walks = 0;
    const walkTo = (guard, x) => { walks = walks + 1; };
    if (globalThis.gc) { globalThis.gc(); }
    const before = process.memoryUsage().heapUsed;
    let started = performance.now();
    const tasks = [], steps = [];
    for (let guard = 0; guard < count; guard++) {
        const [patrol, step] = new Function("walkTo", source)(walkTo);
        const task = patrol(guard);
        task.next();
        tasks.push(task);
        steps.push(step);
    }
    const loaded = (performance.now() - started) / 1000;
    if (globalThis.gc) { globalThis.gc(); }
    const after = process.memoryUsage().heapUsed;
    started = performance.now();
    for (let round = 0; round < rounds; round++) {
        for (let guard = 0; guard < count; guard++) {
            tasks[guard].next();
        }
    }
    const resumed = (performance.now() - started) / 1000;
    started = performance.now();
    for (let round = 0; round < rounds; round++) {
        for (let guard = 0; guard < count; guard++) {
            steps[guard](guard);
        }
    }
    const called = (performance.now() - started) / 1000;
    console.log("result " + walks);
    console.log("load_each " + (loaded / count).toFixed(9));
    console.log("resume_each " + (resumed / (count * rounds)).toFixed(9));
    console.log("call_each " + (called / (count * rounds)).toFixed(9));
    console.log("memory_each " + Math.round((after - before) / count));
}
