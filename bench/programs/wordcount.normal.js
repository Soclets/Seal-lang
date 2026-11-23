function run(words) {
    const counts = new Map();
    let seed = 42;
    for (let made = 0; made < words; made++) {
        seed = (seed * 1664525 + 1013904223) % 4294967296;
        const word = "word" + (Math.floor(seed / 65536) % 5000);
        counts.set(word, (counts.get(word) ?? 0) + 1);
    }
    let most = 0;
    for (const count of counts.values()) {
        if (count > most) {
            most = count;
        }
    }
    return counts.size * 1000000 + most;
}
