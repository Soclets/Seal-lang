// Purpose: calls. A function calls itself `depth` deep, 20,000 times over, and does nothing else.
function down(depth) {
    if (depth === 0) {
        return 0;
    }
    return 1 + down(depth - 1);
}

function run(depth) {
    let total = 0;
    for (let round = 1; round <= 20000; round += 1) {
        total = total + down(depth);
    }
    return total;
}
