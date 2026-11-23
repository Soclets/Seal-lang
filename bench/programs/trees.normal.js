function build(depth) {
    if (depth === 0) {
        return { left: null, right: null };
    }
    return { left: build(depth - 1), right: build(depth - 1) };
}

function count(tree) {
    if (tree.left === null) {
        return 1;
    }
    return 1 + count(tree.left) + count(tree.right);
}

function run(maxDepth) {
    const stretch = build(maxDepth + 1);
    let total = count(stretch);
    const longLived = build(maxDepth);
    let depth = 4;
    while (depth <= maxDepth) {
        let rounds = 1;
        for (let doubling = 1; doubling <= maxDepth - depth + 4; doubling += 1) {
            rounds = rounds * 2;
        }
        let check = 0;
        for (let round = 1; round <= rounds; round += 1) {
            check = check + count(build(depth));
        }
        total = total + check;
        depth = depth + 2;
    }
    return total + count(longLived);
}
