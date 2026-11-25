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
    let script = "proc pick[n: Whole] -> Whole ! [low]\n    if n < 1\n        fail low\n    else\n        ret n\n    end\nend\nvalue = pick(0) otherwise\n    case low\n        use 1\n        show_msg(\"after\")\nend\nshow_msg(\"{value}\")\n";
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
        shows(&format!("{stock}for_each count at key unused item in stock\n    show_msg(\"{{count}}\")\nend\n")),
        ["1", "2"]
    );
    let by_place = format!("{stock}for_each count at pos place in stock\n    show_msg(\"{{count}} {{place}}\")\nend\n");
    assert!(refusal(&by_place).contains("for_each count at key place in stock"), "{}", refusal(&by_place));
    let leaving = "proc first[event: Maybe<Whole>] -> Whole\n    match event\n        case some[value]\n            ret value\n        case none\n    end\n    ret 0\nend\nfound: Maybe<Whole> = Maybe.none\nn = first(found)\nshow_msg(\"{n}\")\n";
    assert!(refusal(leaving).contains("follows a `match` that can leave"), "{}", refusal(leaving));
}

// trace: D87
#[test]
fn d87_arguments_go_in_their_declared_order() {
    let heal = "proc heal[hp, amount, times : Whole] -> Whole\n    labels [amount as by, times as times]\n    ret hp + amount * times\nend\n";
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
    let running = "proc pieces[by: Text] -> List<Text>\n    ret \"a b\".split(by: by)\nend\nwords = pieces(\"\")\nshow_msg(\"{words}\")\n";
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
const LOADERS: &str = "proc load_level[name: Text] -> Whole ! [not_found, broken]\n    if name == \"forest\"\n        fail not_found\n    elif name == \"swamp\"\n        fail broken\n    else\n        ret 7\n    end\nend\n\nproc load_save[slot: Text] -> Whole ! [not_found, broken]\n    if slot == \"slot1\"\n        fail not_found\n    elif slot == \"slot2\"\n        fail broken\n    else\n        ret 2\n    end\nend\n\n";

const GROUP: &str = "group.handler level_failed\n    handles [load_level, load_save]\n    ::start_over\n        case not_found\n            use 1\n        case broken\n            use 1\n    end \\\\ ::start_over\n    ::start_at[level: Whole]\n        labels [level as level]\n        case not_found\n            use level\n        case broken\n            use level\n    end\n    ::give_up\n        case not_found\n            ret\n        case broken\n            ret\n    end\nend \\\\ group.handler level_failed\n\n";

// trace: D97, D98, D110, Unio D42, D43, D44
#[test]
fn d110_handlers_are_used_as_if_their_cases_were_written_at_the_call() {
    let script = format!(
        "{LOADERS}{GROUP}proc try_both\n    lost = load_level(\"swamp\") ?? ::*give_up\n    show_msg(\"never {{lost}}\")\nend\n\na = load_level(\"forest\") ?? ::start_over\nb = load_save(\"slot1\") ?? level_failed::start_at(level: 3)\nc = load_level(\"cave\") otherwise\n    incl ::start_over\nend\nd = load_level(\"swamp\") otherwise\n    incl ::start_at(level: 9)\nend\nshow_msg(\"{{a}} {{b}} {{c}} {{d}}\")\ntry_both()\nshow_msg(\"after\")\n"
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
    let twice = with("a = load_level(\"x\") otherwise\n    incl ::start_over\n    case broken\n        use 2\nend");
    assert!(refusal(&twice).contains("handled twice"), "{}", refusal(&twice));
    let second = GROUP.replace("group.handler level_failed", "group.handler save_failed");
    let both = format!("{LOADERS}{GROUP}{second}a = load_level(\"x\") ?? ::start_over\nshow_msg(\"{{a}}\")\n");
    assert!(refusal(&both).contains("could be"), "{}", refusal(&both));
    assert!(refusal("fam.handler level_failed\nend\n").contains("`group.handler`"));
}

// trace: D107, D128
#[test]
fn d107_d128_ranged_numbers_are_checked_where_values_land() {
    let script = "mut hp: Whole[min.clamp: 0, max.clamp: 100] = 100\nhp = hp - 150\nlow = hp\nhp = hp + 500\nrecord Stats\n    level: Whole[min.clamp: 1, max.clamp: 10]\nend\nstats = Stats[level: 50]\nproc volume[v: Whole[min.clamp: 0, max.clamp: 100]] -> Whole\n    ret v\nend\nloud = volume(170)\nshow_msg(\"{low} {hp} {stats.level} {loud}\")\n";
    assert_eq!(shows(script), ["0 100 10 100"]);
    let hard = "mut gold: Whole[min: 0] = 10\nproc spend[amount: Whole]\n    gold = gold - amount\nend\nspend(20)\nshow_msg(\"{gold}\")\n";
    assert!(refusal(hard).contains("this value is below its type's `min: 0`"), "{}", refusal(hard));
    assert!(refusal("gold: Whole[min: 0] = -5\nshow_msg(\"{gold}\")\n").contains("below its type's `min: 0`"));
    let argument = "proc volume[v: Whole[min: 0, max: 100]]\n    show_msg(\"{v}\")\nend\nvolume(150)\n";
    assert!(refusal(argument).contains("above its type's `max: 100`"), "{}", refusal(argument));
    let floats = "speed: Float[min.clamp: 0.0, max.clamp: 12.5] = 20.0\nshow_msg(\"{speed}\")\n";
    assert_eq!(shows(floats), ["12.5"]);
}

// trace: D107, D108, D109, D114
#[test]
fn d108_d109_d114_maths() {
    let script = "x = 3\nsquare = x ** 2\narea = 2 * x ** 2\nroot = 2.0 ** 0.5\nangle = atan2(y: 1.0, x: 1.0)\nwave = sin(1.0)\nfar = abs(-7)\nlow = min(3.5, 2.5)\nhigh = max(2, 9)\nheld = clamp(150, low: 0, high: 100)\npart = lerp(0.0, to: 10.0, by: 0.25)\nshow_msg(\"{square} {area} {root} {pi} {angle} {wave} {far} {low} {high} {held} {part}\")\n";
    assert_eq!(
        shows(script),
        ["9 18 ~1.4142135623730951 ~3.141592653589793 ~0.7853981633974483 ~0.8414709848078965 7 2.5 9 100 2.5"]
    );
    assert!(refusal("big = 2 ** 70\nshow_msg(\"{big}\")\n").contains("too big for a `Whole`"));
    assert!(refusal("half = 2 ** -1\nshow_msg(\"{half}\")\n").contains("negative power"));
    assert!(refusal("x = asin(2.0)\nshow_msg(\"{x}\")\n").contains("from -1 to 1"));
    assert!(refusal("pi = 3.0\nshow_msg(\"{pi}\")\n").contains("one of Seal's own words"));
    assert!(refusal("mixed = min(1, 2.0)\nshow_msg(\"{mixed}\")\n").contains("one type"));
    assert!(refusal("x = newln\nshow_msg(x)\n").contains("{newln}"));
}

#[test]
fn d112_d120_d122_units_convert_where_exact_and_measure_with_in() {
    let script = "total = 2 secs + 500 ms\nseconds = total in secs\nwait_for: Float[unit: secs] = total\nspeed = 120.0 px per secs\nstep = speed * 0.5 secs\nshow_msg(\"{total} {seconds} {total in ms} {wait_for} {step}\")\n";
    assert_eq!(shows(script), ["2500 ms 2.5 2500 2.5 secs 60.0 px"]);
    assert!(refusal("x = 2 secs + 3 px\nshow_msg(\"{x}\")\n").contains("units of different groups"));
    assert!(refusal("x = 2 secs\ny = x in px\nshow_msg(\"{y}\")\n").contains("isn't measured in `px`"));
    assert!(refusal("x: Whole[unit: secs] = 1500 ms\nshow_msg(\"{x}\")\n").contains("isn't always a whole number"));
    assert!(refusal("x: Float[unit: secs] = 2.0\nshow_msg(\"{x}\")\n").contains("2.0 secs"));
}

#[test]
fn d113_angles_take_units_and_a_radian_is_180_over_pi_degrees() {
    let script = "up = sin(90 degs)\nside = cos(0.5 turns)\naim = atan2(y: 3.0 px, x: 3.0 px)\nhalf = 0.5 turns\nshow_msg(\"{up} {side} {aim} {aim in degs} {half in rads}\")\n";
    assert_eq!(shows(script), ["1.0 -1.0 ~0.7853981633974483 rads 45.0 ~3.141592653589793"]);
    assert!(refusal("x = 2 degs + 1 rads\nshow_msg(\"{x}\")\n").contains("isn't always a whole number"));
    assert!(refusal("x = sin(2 secs)\nshow_msg(\"{x}\")\n").contains("angle"));
}

#[test]
fn d121_points_and_spans() {
    let script = "mut x: Float[unit: px.point] = 100.0 px.point\nx = x + 5.0 px\ngap = x - 40.0 px.point\nshow_msg(\"{x} {gap}\")\n";
    assert_eq!(shows(script), ["105.0 px.point 65.0 px"]);
    assert!(refusal("a = 1.0 px.point\nb = a + a\nshow_msg(\"{b}\")\n").contains("point"));
}

#[test]
fn d123_a_script_declares_its_own_units() {
    let script = "group.unit money\n    ::cents base\n    ::dollars = 100 cents\nend\nprice = 3 dollars + 50 cents\nshow_msg(\"{price} {price in dollars}\")\n";
    assert_eq!(shows(script), ["350 cents 3.5"]);
    let inexact = "group.unit length\n    ::m base\n    ::ft = ~0.3048 m\nend\nlong: Whole[unit: m] = 10 ft\nshow_msg(\"{long}\")\n";
    assert!(refusal(inexact).contains("isn't always a whole number"));
    let two_bases = "group.unit money\n    ::cents base\n    ::dollars base\nend\nx = 1 cents\nshow_msg(\"{x}\")\n";
    assert!(refusal(two_bases).contains("exactly one"));
}

// trace: D111, D123
#[test]
fn d111_a_host_gives_its_doors_and_groups_units() {
    let run = |source: &str| -> Result<Vec<String>, String> {
        let shown = Arc::new(Mutex::new(Vec::new()));
        let mut doors = Doors::new();
        let into = Arc::clone(&shown);
        let push = move |text: String| into.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(text);
        let text = |message: seal::Message| message.text();
        doors.add("show_msg", push.clone()).map_err(text)?;
        doors.add_unit_group("money", &[("cents", "base"), ("dollars", "100 cents")]).map_err(text)?;
        doors
            .add("wait", move |seconds: f64| push(format!("waited {seconds}")))
            .and_then(|door| door.units([Some("secs")]))
            .map_err(text)?;
        doors.add("now", || 1.5).and_then(|door| door.result_unit("secs")).map_err(text)?;
        doors.add("pay", |cents: i64| cents).and_then(|door| door.units([Some("cents")])).map_err(text)?;
        match Script::load("test.seal", source, &doors, LIMITS) {
            Ok(_) => Ok(shown.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone()),
            Err(Error::Refused(messages)) => Err(messages.text()),
            Err(other) => Err(other.to_string()),
        }
    };
    assert_eq!(
        run("wait(500 ms)\nstarted = now()\nshow_msg(\"{started}\")\npaid = pay(2 dollars)\nshow_msg(\"{paid}\")\n"),
        Ok(vec!["waited 0.5".to_owned(), "1.5 secs".to_owned(), "200".to_owned()])
    );
    let plain = run("wait(2)\n").err().unwrap_or_default();
    assert!(plain.contains("wait(2.0 secs)"), "{plain}");
    assert!(run("wait(2.0 px)\n").err().unwrap_or_default().contains("px"));
    let mut doors = Doors::new();
    assert!(doors.add_unit_group("time", &[("ticks", "base")]).is_err());
    assert!(doors.add_unit_group("money", &[("cents", "base"), ("dollars", "100 euros")]).is_err());
    assert!(doors.add("say", |_text: String| ()).and_then(|door| door.units([Some("secs")])).is_err());
}

#[test]
fn d133_d134_angles_from_plain_numbers_stay_plain() {
    let script = "a = atan2(y: 0.0, x: 0.0)\nb = atan2(y: 1.0, x: 1.0) - 0.5\nc = asin(1.0)\nd = atan2(y: -0.0, x: -1.0)\ne = atan2(y: 0.0 px, x: 0.0 px)\nshow_msg(\"{a} {b} {c} {d} {e}\")\n";
    assert_eq!(shows(script), ["0.0 ~0.2853981633974483 ~1.5707963267948966 ~3.141592653589793 0.0 rads"]);
}

#[test]
fn d115_huge_whole_never_overflows() {
    let script = "mut cookies: HugeWhole = 0\nclicks = 9223372036854775807\ncookies = cookies + clicks\ncookies = cookies * clicks\nbest = be.Whole(cookies) ?? 0\nsmall: HugeWhole = 42\nback = be.Whole(small) ?? 0\nbig: HugeWhole = 2\npower = big ** 100\nread = be.HugeWhole(\"123456789012345678901234567890\") ?? 0\nwritten: HugeWhole = 100000000000000000000000\nshow_msg(\"{cookies} {best} {back} {power} {read} {written}\")\n";
    assert_eq!(
        shows(script),
        [
            "85070591730234615847396907784232501249 0 42 1267650600228229401496703205376 123456789012345678901234567890 100000000000000000000000"
        ]
    );
    assert!(refusal("x = 100000000000000000000000\nshow_msg(\"{x}\")\n").contains("too big for a `Whole`"));
    assert!(refusal("x: HugeWhole = 2\ny = x + 0.5\nshow_msg(\"{y}\")\n").contains("can't mix"));
    assert!(refusal("x: HugeWhole = 7\ny = x / 2\nshow_msg(\"{y}\")\n").contains("leaves a remainder"));
}

#[test]
fn d116_d119_d124_d125_decimals_are_exact_and_keep_their_places() {
    let script = "mut gold: Decimal = 0.0\nprice: Decimal = 4.99\ngold = gold + price * 3\nhalf = 0.5 + gold\na: Decimal = 1.50\nb = a + 1.50\nsame = b == 3.0\nten: Decimal = 10.0\nthird = ten / 3\nshow_msg(\"{gold} {half} {b} {same} {third} {third, places: 2}\")\n";
    assert_eq!(shows(script), ["14.97 15.47 3.00 true 3.333333333333333333333333333333333 3.33"]);
    assert!(refusal("x: Decimal = ~0.1\nshow_msg(\"{x}\")\n").contains("takes no `~`"));
    assert!(refusal("x: Decimal = 1.5\ny: Float = 2.0\nz = x + y\nshow_msg(\"{z}\")\n").contains("can't mix"));
    let exact = "x: HugeDecimal = 0.1\ny = x + 0.2\nz = x * x * x\nshow_msg(\"{y} {z}\")\n";
    assert_eq!(shows(exact), ["0.3 0.001"]);
    assert!(
        run("x: HugeDecimal = 1.0\ny = x / 3\nshow_msg(\"{y}\")\n")
            .err()
            .unwrap_or_default()
            .contains("no exact decimal answer")
    );
}

#[test]
fn d126_converting_between_number_types() {
    let script = "price: Decimal = 4.99\nf = be.Whole.floor(price) ?? 0\nn = be.Whole.nearest(price) ?? 0\nfloat = be.Float(price) ?? 0.0\nshown = be.Decimal.shown(~0.1)\nnearest = be.Decimal.nearest(~0.1)\nread = be.Decimal(\" 12.50 \") ?? 0.0\nbad = be.Decimal(\"twelve\") ?? 0.0\nhuge: HugeDecimal = 1.25\nback = be.Decimal(huge) ?? 0.0\nwhole = be.HugeWhole.ceil(huge)\nshow_msg(\"{f} {n} {float} {shown} {nearest} {read} {bad} {back} {whole}\")\n";
    assert_eq!(shows(script), ["4 5 ~4.99 0.1 0.1000000000000000055511151231257827 12.50 0.0 1.25 2"]);
    assert!(refusal("x = be.Decimal(~0.1)\nshow_msg(\"{x}\")\n").contains("says which `Decimal`"));
    assert!(
        refusal("x: Decimal = 2.5\ny = be.Whole(x) ?? 0\nshow_msg(\"{y}\")\n").contains("says how the fraction goes")
    );
}

#[test]
fn d127_operations_on_the_new_number_types() {
    let script = "pot: Decimal = 10.00\nshare = pot div.floor 3\nleft = pot rem.floor 3\nr: Decimal = 2.25\nroot = sqrt(r)\nsquare = r ** 2\nsize = abs(-r)\nlow = min(r, 1.5)\nmid = lerp(r, to: 4.25, by: 0.5)\nh: HugeDecimal = 2.25\nhroot = sqrt(h)\nshow_msg(\"{share} {left} {root} {square} {size} {low} {mid} {hroot}\")\n";
    assert_eq!(shows(script), ["3 1.00 1.5 5.0625 2.25 1.5 3.250 1.5"]);
    assert!(
        run("h: HugeDecimal = 2.0\nr = sqrt(h)\nshow_msg(\"{r}\")\n")
            .err()
            .unwrap_or_default()
            .contains("no exact decimal answer")
    );
    assert!(refusal("x: HugeWhole = 4\ny = sqrt(x)\nshow_msg(\"{y}\")\n").contains("`sqrt` takes"));
}

// trace: D107, D115, D120
#[test]
fn d115_a_huge_whole_takes_units_and_limits() {
    let script = "mut x: HugeWhole[min.clamp: 0, max.clamp: 1000] = 5\nx = x * 1000\nbig: HugeWhole = 99999999999999999999\ny: HugeWhole[max.clamp: 7] = big\nproc cap[v: HugeWhole[min.clamp: 10]] -> HugeWhole\n    ret v\nend\nz = cap(3)\nmut gold: HugeWhole[unit: cents] = 250 cents\ngold = gold + 2 dollars\nw = gold in cents\nmore = gold > 3 dollars\nshow_msg(\"{x} {y} {z} {gold} {w} {more}\")\ngroup.unit money\n    ::cents base\n    ::dollars = 100 cents\nend\n";
    assert_eq!(shows(script), ["1000 7 10 450 cents 450 true"]);
    assert!(refusal("h: HugeWhole[min: 0] = -5\nshow_msg(\"{h}\")\n").contains("below its type's `min: 0`"));
    assert!(refusal("d: Decimal[min: 0] = 5\nshow_msg(\"{d}\")\n").contains("isn't decided yet"));
}
