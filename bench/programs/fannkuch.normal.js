function run(n) {
    const perm1 = [];
    const count = [];
    for (let i = 0; i < n; i++) {
        perm1.push(i);
        count.push(0);
    }
    let maxFlips = 0;
    let checksum = 0;
    let permutations = 0;
    let r = n;
    let done = false;
    while (!done) {
        while (r !== 1) {
            count[r - 1] = r;
            r = r - 1;
        }
        const perm = perm1.slice();
        let flips = 0;
        let k = perm[0];
        while (k !== 0) {
            let low = 0;
            let high = k;
            while (low < high) {
                const swap = perm[low];
                perm[low] = perm[high];
                perm[high] = swap;
                low = low + 1;
                high = high - 1;
            }
            flips = flips + 1;
            k = perm[0];
        }
        if (flips > maxFlips) {
            maxFlips = flips;
        }
        if (permutations % 2 === 0) {
            checksum = checksum + flips;
        } else {
            checksum = checksum - flips;
        }
        permutations = permutations + 1;
        let rotating = true;
        while (rotating) {
            if (r === n) {
                done = true;
                rotating = false;
            } else {
                const first = perm1[0];
                let i = 0;
                while (i < r) {
                    perm1[i] = perm1[i + 1];
                    i = i + 1;
                }
                perm1[r] = first;
                count[r] = count[r] - 1;
                if (count[r] > 0) {
                    rotating = false;
                } else {
                    r = r + 1;
                }
            }
        }
    }
    return checksum * 100 + maxFlips;
}
