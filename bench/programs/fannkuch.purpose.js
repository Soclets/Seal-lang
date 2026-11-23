// Purpose: shuffling a list in place. A list of 16 whole numbers is reversed `rounds` times, item by item.
function run(rounds) {
    const items = [];
    for (let i = 0; i < 16; i += 1) {
        items.push(i);
    }
    for (let round = 1; round <= rounds; round += 1) {
        let low = 0;
        let high = 15;
        while (low < high) {
            const swap = items[low];
            items[low] = items[high];
            items[high] = swap;
            low = low + 1;
            high = high - 1;
        }
    }
    return items[0] * 100 + items[15];
}
