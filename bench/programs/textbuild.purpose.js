// Purpose: joining text. One text grows by two characters, `count` times.
function run(count) {
    let text = "";
    for (let piece = 1; piece <= count; piece += 1) {
        text = text + "ab";
    }
    return text;
}
