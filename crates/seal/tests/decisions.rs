//! Small scripts run through a host, each checking what one decision makes a script show (D79 to D105).

use std::sync::{Arc, Mutex};

use seal::{Doors, Error, Limits, Script};

const LIMITS: Limits = Limits { steps: 1_000_000, memory: 1 << 20 };

/// What a script shows through `show_msg`, or the first line of why it stopped or was refused.
fn run(source: &str) -> Result<Vec<String>, String> {
    let shown = Arc::new(Mutex::new(Vec::new()));
    let mut doors = Doors::new();
    let into = Arc::clone(&shown);
    doors
        .add("show_msg", move |text: String| into.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(text))
        .map_err(|message| message.text())?;
    match Script::load("test.seal", source, &doors, LIMITS) {
        Ok(_) => Ok(shown.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()),
        Err(Error::Refused(messages)) => Err(messages.text()),
        Err(Error::Stopped(message)) => Err(message.text()),
        Err(other) => Err(other.to_string()),
    }
}

fn shows(source: &str) -> Vec<String> {
    run(source).unwrap_or_else(|why| vec![why])
}

fn refusal(source: &str) -> String {
    run(source).err().unwrap_or_default()
}

// trace: D79
#[test]
fn d79_crash_on_a_maybe() {
    let script = "found: Maybe<Whole> = Maybe.some[value: 3]\nvalue = found ?? crash\nshow_msg(\"{value}\")\n";
    assert_eq!(shows(script), ["3"]);
    let none = "found: Maybe<Whole> = Maybe.none\nvalue = found ?? crash\nshow_msg(\"{value}\")\n";
    assert!(refusal(none).contains("this `Maybe` is `none`"), "{}", refusal(none));
    let ignored = "found: Maybe<Whole> = Maybe.none\nvalue = found ?? ignore\nshow_msg(\"{value}\")\n";
    assert!(refusal(ignored).contains("`?? ignore`"), "{}", refusal(ignored));
}

// trace: D80
#[test]
fn d80_instead_is_the_last_line_of_its_case() {
    let script = "proc pick[n: Whole] -> Whole ! [low]\n    if n < 1\n        fail low\n    else\n        return n\n    end\nend\nvalue = pick(0) otherwise\n    case low\n        instead 1\n        show_msg(\"after\")\nend\nshow_msg(\"{value}\")\n";
    assert!(refusal(script).contains("this line can never run"), "{}", refusal(script));
}

// trace: D82
#[test]
fn d82_a_written_type_leaves_out_what_the_value_shows() {
    assert_eq!(shows("board: Array[len: 3] = [all: 0]\nshow_msg(\"{board}\")\n"), ["[0, 0, 0]"]);
    assert!(refusal("bag: List[len.max: 2] = []\nshow_msg(\"{bag}\")\n").contains("leaves out what it holds"));
}

// trace: D83, D86, D105
#[test]
fn d83_d86_d105_names_and_dictionaries() {
    assert!(refusal("all = 3\nshow_msg(\"{all}\")\n").contains("a name can't be `all`"));
    let stock = "stock: Dict<key: Text, value: Whole> = [\"a\": 1, \"b\": 2]\n";
    assert_eq!(
        shows(&format!("{stock}for each count at key unused item in stock\n    show_msg(\"{{count}}\")\nend\n")),
        ["1", "2"]
    );
    let by_place = format!("{stock}for each count at pos place in stock\n    show_msg(\"{{count}} {{place}}\")\nend\n");
    assert!(refusal(&by_place).contains("for each count at key place in stock"), "{}", refusal(&by_place));
    let leaving = "proc first[event: Maybe<Whole>] -> Whole\n    match event\n        case some[value]\n            return value\n        case none\n    end\n    return 0\nend\nfound: Maybe<Whole> = Maybe.none\nn = first(found)\nshow_msg(\"{n}\")\n";
    assert!(refusal(leaving).contains("follows a `match` that can leave"), "{}", refusal(leaving));
}

// trace: D87
#[test]
fn d87_arguments_go_in_their_declared_order() {
    let heal = "proc heal[hp, amount, times : Whole] -> Whole\n    labels [amount as by, times as times]\n    return hp + amount * times\nend\n";
    let wrong = format!("{heal}value = heal(10, times: 2, by: 5)\nshow_msg(\"{{value}}\")\n");
    assert!(refusal(&wrong).contains("value = heal(10, by: 5, times: 2)"), "{}", refusal(&wrong));
    let any = heal.replace("labels [", "labels.any_order [");
    assert_eq!(shows(&format!("{any}value = heal(10, times: 2, by: 5)\nshow_msg(\"{{value}}\")\n")), ["20"]);
}

// trace: D89, D90, D93, D96
#[test]
fn d89_to_d96_refusals() {
    assert!(
        refusal("words = \"a b\".split(by: \"\")\nshow_msg(\"{words}\")\n").contains("splits a text by an empty text")
    );
    let running = "proc pieces[by: Text] -> List<Text>\n    return \"a b\".split(by: by)\nend\nwords = pieces(\"\")\nshow_msg(\"{words}\")\n";
    assert!(refusal(running).contains("check failed"), "{}", refusal(running));
    let partway = "record Shop\n    stock: List<Text>\nend\nmut shops: Dict<key: Text, value: Shop> = [\"north\": Shop[stock: []]]\nshops[key: \"north\"].stock.add(\"Key\")\nshow_msg(\"{shops}\")\n";
    assert!(refusal(partway).contains("through a `Dict`'s key"), "{}", refusal(partway));
    assert!(refusal("one_of Nothing\nend\n").contains("has no cases"));
    assert!(refusal("a = \"x\" < \"y\"\nshow_msg(\"{a}\")\n").contains("aren't put in order"));
}

// trace: D95, D100
#[test]
fn d95_d100_bigger_values_compare_and_show_as_written() {
    let script = "record Spot\n    x: Whole\n    y: Float\nend\na = Spot[x: 1, y: ~0.1]\nb = Spot[x: 1, y: ~0.1]\nsame = a == b\nstock: Dict<key: Text, value: Whole> = [\"a\": 1, \"b\": 2]\nagain: Dict<key: Text, value: Whole> = [\"b\": 2, \"a\": 1]\nalike = stock == again\nlist = [a, b]\nquote = \"say \\\"hi\\\" \\{x}{tab}\"\nquotes = [quote]\nshow_msg(\"{same} {alike} {list} {quotes}\")\n";
    assert_eq!(shows(script), ["true true [Spot[x: 1, y: ~0.1], Spot[x: 1, y: ~0.1]] [\"say \\\"hi\\\" \\{x}{tab}\"]"]);
}

// trace: D99
#[test]
fn d99_a_range_is_a_list() {
    assert_eq!(
        shows("rounds = range[from: 10, before: 0, step_by: -3]\nshow_msg(\"{rounds} {rounds.len}\")\n"),
        ["[10, 7, 4, 1] 4"]
    );
    assert!(
        refusal("huge = range[from: 1, to: 9223372036854775807]\nshow_msg(\"{huge.len}\")\n")
            .contains("ran out of memory")
    );
}

// trace: D101
#[test]
fn d101_be_bool_reads_true_or_false() {
    assert_eq!(
        shows("a = be.Bool(\"true\") ?? false\nb = be.Bool(\"yes\") ?? false\nshow_msg(\"{a} {b}\")\n"),
        ["true false"]
    );
    assert!(refusal("a = be.Bool(1)\nshow_msg(\"{a}\")\n").contains("no conversion from"));
}

// trace: D103, D104
#[test]
fn d103_d104_settings_and_defaults_from_plain_values() {
    let script = "max_items = 2\nstarting = [\"Gem\"]\none_of Color\n    red\n    blue\nend\nproc spawn[kind: Text, color: Color, loot: List<Text>]\n    labels [color as color, loot as loot]\n    defaults [color: Color.red, loot: starting]\n    show_msg(\"{kind} {color} {loot}\")\nend\nbag: List<Text>[len.max: max_items] = [\"a\"]\nspawn(\"orc\")\nshow_msg(\"{bag}\")\n";
    assert_eq!(shows(script), ["orc Color.red [\"Gem\"]", "[\"a\"]"]);
    assert!(
        refusal("mut limit = 2\nbag: List<Text>[len.max: limit] = []\nshow_msg(\"{bag}\")\n")
            .contains("made with `mut`")
    );
}

/// Two procedures that fail as the handler tests' handlers expect.
const LOADERS: &str = "proc load_level[name: Text] -> Whole ! [not_found, broken]\n    if name == \"forest\"\n        fail not_found\n    elif name == \"swamp\"\n        fail broken\n    else\n        return 7\n    end\nend\n\nproc load_save[slot: Text] -> Whole ! [not_found, broken]\n    if slot == \"slot1\"\n        fail not_found\n    elif slot == \"slot2\"\n        fail broken\n    else\n        return 2\n    end\nend\n\n";

const GROUP: &str = "group.handler level_failed\n    handles [load_level, load_save]\n    ::start_over\n        case not_found\n            instead 1\n        case broken\n            instead 1\n    end \\\\ ::start_over\n    ::start_at[level: Whole]\n        labels [level as level]\n        case not_found\n            instead level\n        case broken\n            instead level\n    end\n    ::give_up\n        case not_found\n            return\n        case broken\n            return\n    end\nend \\\\ group.handler level_failed\n\n";

// trace: D97, D98, D110, Unio D42, D43, D44
#[test]
fn d110_handlers_are_used_as_if_their_cases_were_written_at_the_call() {
    let script = format!(
        "{LOADERS}{GROUP}proc try_both\n    lost = load_level(\"swamp\") ?? ::*give_up\n    show_msg(\"never {{lost}}\")\nend\n\na = load_level(\"forest\") ?? ::start_over\nb = load_save(\"slot1\") ?? level_failed::start_at(level: 3)\nc = load_level(\"cave\") otherwise\n    include ::start_over\nend\nd = load_level(\"swamp\") otherwise\n    include ::start_at(level: 9)\nend\nshow_msg(\"{{a}} {{b}} {{c}} {{d}}\")\ntry_both()\nshow_msg(\"after\")\n"
    );
    assert_eq!(shows(&script), ["1 3 7 9", "after"]);
}

// trace: D98, D110, Unio D42
#[test]
fn d110_handler_refusals() {
    let with = |line: &str| format!("{LOADERS}{GROUP}{line}\nshow_msg(\"{{a}}\")\n");
    assert!(refusal(&with("a = load_level(\"x\") ?? ::give_up")).contains("::*give_up"));
    assert!(refusal(&with("a = load_level(\"x\") ?? ::*start_over")).contains("takes no `*`"));
    assert!(refusal(&with("a = load_level(\"x\") ?? ::begin_again")).contains("has `::begin_again`"));
    assert!(
        refusal(&with("a = load_level(\"x\") ?? level_failed::begin_again")).contains("has no handler `::begin_again`")
    );
    let one_line = refusal(&with("a = load_level(\"x\") otherwise level_failed::start_over"));
    assert!(one_line.contains("a = load_level(\"x\") ?? level_failed::start_over"), "{one_line}");
    let twice =
        with("a = load_level(\"x\") otherwise\n    include ::start_over\n    case broken\n        instead 2\nend");
    assert!(refusal(&twice).contains("handled twice"), "{}", refusal(&twice));
    let second = GROUP.replace("group.handler level_failed", "group.handler save_failed");
    let both = format!("{LOADERS}{GROUP}{second}a = load_level(\"x\") ?? ::start_over\nshow_msg(\"{{a}}\")\n");
    assert!(refusal(&both).contains("could be"), "{}", refusal(&both));
    assert!(refusal("fam.handler level_failed\nend\n").contains("`group.handler`"));
}
