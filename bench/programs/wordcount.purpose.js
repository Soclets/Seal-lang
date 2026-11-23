// Purpose: a Map with text keys. 1,000 keys are made once, then each is looked up and counted `rounds` times.
function run(rounds) {
    const keys = [];
    const counts = new Map();
    for (let i = 1; i <= 1000; i += 1) {
        keys.push("key" + i);
        counts.set("key" + i, 0);
    }
    for (let round = 1; round <= rounds; round += 1) {
        for (const key of keys) {
            counts.set(key, (counts.get(key) ?? 0) + 1);
        }
    }
    return counts.get("key500") * 1000 + counts.size;
}
