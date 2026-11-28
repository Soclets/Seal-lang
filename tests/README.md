# Wax's tests

The contract Wax's building session builds to. Each test is written by hand from the decisions and the standard, and
names the decisions it checks.

## A test

Each test is a folder named after its main decision, as in `d027_records/`:

| File | What it holds |
|---|---|
| `program.wax` | the script |
| `about.txt` | one line: what it checks, with its decisions |
| `expected.out` | exactly what the test host's doors print |
| `expected.status` | the `wax` command's exit status: 0 when the script ran to its end |
| `input.txt` | optional: the lines `ask` reads, one per call |
| `random.txt` | optional: the numbers `rand` gives, one per call, in order |
| `calls.txt` | optional: calls the test host makes into the script after loading it, one per line |

A test passes when, run as

```
cd tests/NAME
wax run program.wax < input.txt
```

from inside the test's own folder (with no input when there's no `input.txt`), the output and exit status match the
expected files exactly.

## The test host

A Wax script reaches nothing except its host's doors (D11), so the `wax` command runs tests with a test host (D47).
Its doors are a convention for tests, not part of Wax, and follow D24's spelling:

| Door | What it does |
|---|---|
| `show_msg(text)` | prints `text` and a line break |
| `show_msg.same_line(text)` | prints `text` and leaves the line open, a narrowing on a door (D13) |
| `ask(question)` | prints `question`, leaves the line open, and gives back the next line of `input.txt` as `Text`; with no more input, the script ends there, quietly, with status 0 |
| `rand(from: low, to: high)` | gives the next number in `random.txt`; with none left, `low` |
| `wait(seconds: n)` | pauses the script (D49); the test host resumes it at once |

The test host gives every script a step budget of 1,000,000 and a memory cap of 64 MB (D14).

After loading a script, which runs its top level (D4), the test host makes each call in `calls.txt` in order, written
as a Wax call with plain values, as `on_hit(10)` or `heal(1, by: 5)`.

## Rules

- **Expected files come from the decisions and the standard,** never from what an implementation happens to print.
- **A building session never edits an expected file.** A test that looks wrong goes back to the design session.
- **Refusals and stops get tests once their wording is decided** (D26), as in Unio.
- **Where something isn't decided,** the implementation stops with "not designed yet", and a test for it comes only once
  it's decided.
