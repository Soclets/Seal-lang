//! Fuzzing through the C API (D136, Building rule 6): random calls, with handles made up, freed, of the wrong kind
//! or mixed up, arguments of any kind and bits, scripts damaged at random, doors that answer anything. Seal must
//! answer each with a status, never crash, never stop the test, and keep every script inside its limits.
#![allow(clippy::too_many_lines)] // one case per kind of call
#![allow(clippy::cast_possible_truncation)] // handles are pointer-sized; the test runs on 64-bit hosts

use std::ffi::{CString, c_char, c_void};

use seal_c::{seal_arg, seal_handle, seal_limits, seal_status};

struct Random(u64);

impl Random {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, limit: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(limit.max(1)).unwrap_or(1)).unwrap_or(0)
    }
}

/// A door that answers at random: any status, any kind, any bits, sometimes a value it makes.
unsafe extern "C" fn wild_door(
    data: *mut c_void,
    args: *const seal_arg,
    count: usize,
    result: *mut seal_arg,
) -> seal_status {
    // SAFETY: the test gives every door a pointer to its `u64`, alive for the whole test.
    let state = unsafe { &mut *data.cast::<u64>() };
    *state ^= *state << 13;
    *state ^= *state >> 7;
    *state ^= *state << 17;
    let pick = *state;
    // Read what was lent, as a door would.
    if count > 0 && !args.is_null() {
        // SAFETY: Seal gives `count` arguments, valid for the call.
        let first = unsafe { args.read() };
        let mut text: *const c_char = std::ptr::null();
        let mut length = 0usize;
        // SAFETY: the API's own functions, with room for their answers.
        let _ = unsafe {
            seal_c::seal_value_get_text(first.bits as usize as *const seal_handle, &raw mut text, &raw mut length)
        };
    }
    let kind = i32::try_from(pick % 14).unwrap_or(0);
    let bits = if pick % 5 == 0 {
        let made = CString::new("made").unwrap_or_default();
        // SAFETY: a text of its length.
        unsafe { seal_c::seal_value_text(made.as_ptr(), 4) }.addr() as u64
    } else {
        pick >> 3
    };
    // SAFETY: Seal gives room for one result.
    unsafe { result.write(seal_arg { kind, bits }) };
    i32::try_from((pick >> 8) % 7).unwrap_or(0)
}

const SCRIPTS: [&str; 5] = [
    "proc run[n: Whole] -> Whole\n    mut total = 0\n    for_each i in range[from: 1, to: n]\n        total = total + poke(i, 2)\n    end\n    return total\nend\n",
    "proc greet[name: Text] -> Text\n    return shout(\"hi {name}\")\nend\n",
    "proc nap\n    rest(1)\nend\n",
    "proc list -> List<Text>\n    return [\"a\", shout(\"b\")]\nend\n",
    "proc spin -> Whole\n    mut n = 0\n    while true\n        n = n + 1\n    end\n    return n\nend\n",
];

// trace: D136, Building rule 6
#[test]
fn d136_random_calls_through_the_c_api_never_harm_the_host() {
    let mut random = Random(0x9e37_79b9_7f4a_7c15);
    let mut door_state = 0x2545_f491_4f6c_dd1d_u64;
    let limits = seal_limits { steps: 20_000, memory: 256 * 1024 };
    let mut doors: Vec<*mut seal_handle> = Vec::new();
    let mut scripts: Vec<*mut seal_handle> = Vec::new();
    let mut values: Vec<u64> = Vec::new();
    let mut tasks: Vec<u64> = Vec::new();
    let types = [
        "Whole",
        "Float",
        "Bool",
        "Text",
        "List<Text>",
        "Maybe<Whole>",
        "Dict<key: Text, value: Whole>",
        "Nothing",
        "Record",
        "List<",
        "",
    ];
    for _ in 0..20_000 {
        // Now and then a handle made up, or one of another kind.
        let any_handle = |random: &mut Random, list: &[*mut seal_handle], others: &[u64]| -> *mut seal_handle {
            match random.below(6) {
                0 => std::ptr::without_provenance_mut(random.next() as usize),
                1 => others
                    .get(random.below(others.len()))
                    .map_or(std::ptr::null_mut(), |&other| std::ptr::without_provenance_mut(other as usize)),
                _ => list.get(random.below(list.len())).copied().unwrap_or(std::ptr::null_mut()),
            }
        };
        match random.below(12) {
            0 => doors.push(seal_c::seal_doors_new()),
            1 => {
                let target = any_handle(&mut random, &doors, &values);
                let names = ["poke", "shout", "rest", "poke.hard", "Bad", "fmt"];
                let name = CString::new(names[random.below(names.len())]).unwrap_or_default();
                let count = random.below(3);
                let written: Vec<CString> =
                    (0..count).map(|_| CString::new(types[random.below(types.len())]).unwrap_or_default()).collect();
                let pointers: Vec<*const c_char> = written.iter().map(|text| text.as_ptr()).collect();
                let result = CString::new(types[random.below(types.len())]).unwrap_or_default();
                let failure = CString::new("oops").unwrap_or_default();
                let failures = [failure.as_ptr()];
                // SAFETY: every pointer is to a live string or array of the count given.
                let _ = unsafe {
                    seal_c::seal_doors_add(
                        target,
                        name.as_ptr(),
                        pointers.as_ptr(),
                        count,
                        result.as_ptr(),
                        failures.as_ptr(),
                        random.below(2),
                        i32::from(random.below(2) == 0),
                        Some(wild_door),
                        (&raw mut door_state).cast(),
                    )
                };
            }
            2 | 3 => {
                let target = any_handle(&mut random, &doors, &values);
                let mut source = SCRIPTS[random.below(SCRIPTS.len())].as_bytes().to_vec();
                if random.below(3) == 0 {
                    let at = random.below(source.len());
                    if let Some(byte) = source.get_mut(at) {
                        *byte = u8::try_from(random.below(128)).unwrap_or(b' ');
                    }
                }
                let name = CString::new("fuzz.seal").unwrap_or_default();
                let mut script: *mut seal_handle = std::ptr::null_mut();
                // SAFETY: the name is a live string, the source live bytes of its length, and room for the handle.
                let status = unsafe {
                    seal_c::seal_script_load(
                        name.as_ptr(),
                        source.as_ptr().cast(),
                        source.len(),
                        target,
                        limits,
                        &raw mut script,
                    )
                };
                if status == seal_c::SEAL_OK {
                    scripts.push(script);
                }
            }
            4..=6 => {
                let target = any_handle(&mut random, &scripts, &values);
                let procedures = ["run", "greet", "nap", "list", "spin", "missing"];
                let name = CString::new(procedures[random.below(procedures.len())]).unwrap_or_default();
                let count = random.below(3);
                let arguments: Vec<seal_arg> = (0..count)
                    .map(|_| {
                        let kind = i32::try_from(random.below(15)).unwrap_or(0);
                        let bits = if kind == 4 && !values.is_empty() {
                            values[random.below(values.len())]
                        } else {
                            random.next() % 50
                        };
                        seal_arg { kind, bits }
                    })
                    .collect();
                let mut result = seal_arg { kind: 0, bits: 0 };
                let mut task = 0u64;
                // SAFETY: a live string, `count` arguments, and room for the answers.
                let status = unsafe {
                    seal_c::seal_script_call(
                        target,
                        name.as_ptr(),
                        std::ptr::null(),
                        arguments.as_ptr(),
                        count,
                        &raw mut result,
                        &raw mut task,
                    )
                };
                if status == seal_c::SEAL_PAUSED {
                    tasks.push(task);
                }
                if status == seal_c::SEAL_OK && result.kind >= 4 {
                    values.push(result.bits);
                }
                if status == seal_c::SEAL_OK || status == seal_c::SEAL_PAUSED {
                    assert!(seal_c::seal_script_memory_in_use(target) <= limits.memory, "a script past its memory cap");
                }
            }
            7 => {
                let target = any_handle(&mut random, &scripts, &values);
                let task = tasks.get(random.below(tasks.len())).copied().unwrap_or_else(|| random.next());
                let mut result = seal_arg { kind: 0, bits: 0 };
                let mut again = 0u64;
                if random.below(2) == 0 {
                    // SAFETY: no value, and room for the answers.
                    let _ = unsafe {
                        seal_c::seal_script_resume(target, task, std::ptr::null(), &raw mut result, &raw mut again)
                    };
                } else {
                    let _ = seal_c::seal_script_cancel(target, task);
                }
            }
            8 => {
                let text = CString::new("some text").unwrap_or_default();
                // SAFETY: a live text of its length.
                values.push(unsafe { seal_c::seal_value_text(text.as_ptr(), 9) }.addr() as u64);
            }
            9 => {
                let value = values.get(random.below(values.len())).copied().unwrap_or_else(|| random.next());
                let handle = std::ptr::without_provenance(value as usize);
                let mut text: *const c_char = std::ptr::null();
                let mut length = 0usize;
                let mut count = 0usize;
                let mut item = seal_arg { kind: 0, bits: 0 };
                // SAFETY: room for each answer.
                unsafe {
                    let _ = seal_c::seal_value_get_text(handle, &raw mut text, &raw mut length);
                    let _ = seal_c::seal_value_count(handle, &raw mut count);
                    let _ = seal_c::seal_value_item(handle, random.below(3), &raw mut item);
                }
                if !text.is_null() {
                    // SAFETY: a text Seal gave, valid until the value is freed.
                    let bytes = unsafe { std::slice::from_raw_parts(text.cast::<u8>(), length) };
                    assert!(std::str::from_utf8(bytes).is_ok());
                }
            }
            10 => match random.below(3) {
                0 => seal_c::seal_value_free(std::ptr::without_provenance_mut(values.pop().unwrap_or(0) as usize)),
                1 => seal_c::seal_script_free(scripts.pop().unwrap_or(std::ptr::null_mut())),
                _ => seal_c::seal_doors_free(doors.pop().unwrap_or(std::ptr::null_mut())),
            },
            _ => {
                for index in 0..=seal_c::seal_message_count() {
                    let _ = seal_c::seal_message_kind_of(index);
                    let text = seal_c::seal_message_text(index);
                    let _ = (seal_c::seal_message_what(index), seal_c::seal_message_line(index));
                    if !text.is_null() {
                        // SAFETY: a NUL-ended text Seal gave, valid until the next call.
                        assert!(unsafe { std::ffi::CStr::from_ptr(text) }.to_str().is_ok());
                    }
                }
            }
        }
    }
}
