// Purpose: making and dropping values on the heap. A tree 10 deep is made and dropped `rounds` times, never read.
function build(depth) {
    if (depth === 0) {
        return { left: null, right: null };
    }
    return { left: build(depth - 1), right: build(depth - 1) };
}

function run(rounds) {
    let made = 0;
    for (let round = 1; round <= rounds; round += 1) {
        build(10);
        made = made + 1;
    }
    return made;
}
