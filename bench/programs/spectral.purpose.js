// Purpose: loops over a list of floats. A list of 1,000 floats is summed `rounds` times.
function run(rounds) {
    const values = [];
    for (let i = 1; i <= 1000; i += 1) {
        values.push(i * 0.5);
    }
    let total = 0.0;
    for (let round = 1; round <= rounds; round += 1) {
        for (const value of values) {
            total = total + value;
        }
    }
    return total;
}
