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
        shows(&format!("{stock}for_each count at key unused item in stock\n    show_msg(\"{{count}}\")\nend\n")),
        ["1", "2"]
    );
    let by_place = format!("{stock}for_each count at pos place in stock\n    show_msg(\"{{count}} {{place}}\")\nend\n");
    assert!(refusal(&by_place).contains("for_each count at key place in stock"), "{}", refusal(&by_place));
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
    let script = "record Spot\n    x: Whole\n    y: Float\nend\na = Spot[x: 1, y: ~0.1]\nb = Spot[x: 1, y: ~0.1]\nsame = a eq b\nstock: Dict<key: Text, value: Whole> = [\"a\": 1, \"b\": 2]\nagain: Dict<key: Text, value: Whole> = [\"b\": 2, \"a\": 1]\nalike = stock eq again\nlist = [a, b]\nquote = \"say \\\"hi\\\" \\{x}{tab}\"\nquotes = [quote]\nshow_msg(\"{same} {alike} {list} {quotes}\")\n";
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
            .contains("is a mutable variable")
    );
}

/// Two procedures that fail as the handler tests' handlers expect.
const LOADERS: &str = "proc load_level[name: Text] -> Whole ! [not_found, broken]\n    if name eq \"forest\"\n        fail not_found\n    elif name eq \"swamp\"\n        fail broken\n    else\n        return 7\n    end\nend\n\nproc load_save[slot: Text] -> Whole ! [not_found, broken]\n    if slot eq \"slot1\"\n        fail not_found\n    elif slot eq \"slot2\"\n        fail broken\n    else\n        return 2\n    end\nend\n\n";

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

// trace: D107, D128
#[test]
fn d107_d128_ranged_numbers_are_checked_where_values_land() {
    let script = "mut hp: Whole[min.clamp: 0, max.clamp: 100] = 100\nhp = hp - 150\nlow = hp\nhp = hp + 500\nrecord Stats\n    level: Whole[min.clamp: 1, max.clamp: 10]\nend\nstats = Stats[level: 50]\nproc volume[v: Whole[min.clamp: 0, max.clamp: 100]] -> Whole\n    return v\nend\nloud = volume(170)\nshow_msg(\"{low} {hp} {stats.level} {loud}\")\n";
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
    let script = "total = 2 secs + 500 ms\nseconds = total to secs\nwait_for: Float[unit: secs] = total\nspeed = 120.0 px per secs\nstep = speed * 0.5 secs\nshow_msg(\"{total} {seconds} {total to ms} {wait_for} {step}\")\n";
    assert_eq!(shows(script), ["2500 ms 2.5 2500 2.5 secs 60.0 px"]);
    assert!(refusal("x = 2 secs + 3 px\nshow_msg(\"{x}\")\n").contains("units of different groups"));
    assert!(refusal("x = 2 secs\ny = x to px\nshow_msg(\"{y}\")\n").contains("isn't measured in `px`"));
    assert!(refusal("x: Whole[unit: secs] = 1500 ms\nshow_msg(\"{x}\")\n").contains("isn't always a whole number"));
    assert!(refusal("x: Float[unit: secs] = 2.0\nshow_msg(\"{x}\")\n").contains("2.0 secs"));
}

#[test]
fn d113_angles_take_units_and_a_radian_is_180_over_pi_degrees() {
    let script = "up = sin(90 degs)\nside = cos(0.5 turns)\naim = atan2(y: 3.0 px, x: 3.0 px)\nhalf = 0.5 turns\nshow_msg(\"{up} {side} {aim} {aim to degs} {half to rads}\")\n";
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
    let script = "group.unit money\n    ::cents base\n    ::dollars = 100 cents\nend\nprice = 3 dollars + 50 cents\nshow_msg(\"{price} {price to dollars}\")\n";
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
    let script = "mut gold: Decimal = 0.0\nprice: Decimal = 4.99\ngold = gold + price * 3\nhalf = 0.5 + gold\na: Decimal = 1.50\nb = a + 1.50\nsame = b eq 3.0\nten: Decimal = 10.0\nthird = ten / 3\nshow_msg(\"{gold} {half} {b} {same} {third} {third, places: 2}\")\n";
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
    assert!(refusal("x = be.Decimal(~0.1)\nshow_msg(\"{x}\")\n").contains("needs to say which `Decimal`"));
    assert!(
        refusal("x: Decimal = 2.5\ny = be.Whole(x) ?? 0\nshow_msg(\"{y}\")\n")
            .contains("needs to say how the fraction goes")
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
    let script = "mut x: HugeWhole[min.clamp: 0, max.clamp: 1000] = 5\nx = x * 1000\nbig: HugeWhole = 99999999999999999999\ny: HugeWhole[max.clamp: 7] = big\nproc cap[v: HugeWhole[min.clamp: 10]] -> HugeWhole\n    return v\nend\nz = cap(3)\nmut gold: HugeWhole[unit: cents] = 250 cents\ngold = gold + 2 dollars\nw = gold to cents\nmore = gold > 3 dollars\nshow_msg(\"{x} {y} {z} {gold} {w} {more}\")\ngroup.unit money\n    ::cents base\n    ::dollars = 100 cents\nend\n";
    assert_eq!(shows(script), ["1000 7 10 450 cents 450 true"]);
    assert!(refusal("h: HugeWhole[min: 0] = -5\nshow_msg(\"{h}\")\n").contains("below its type's `min: 0`"));
    assert!(refusal("d: Decimal[min: 0] = 5\nshow_msg(\"{d}\")\n").contains("isn't decided yet"));
}

// trace: D146, D157, D158
#[test]
fn d146_d157_d158_return_for_each_and_eq_are_the_words() {
    let script = "proc twice[n: Whole] -> Whole\n    return n * 2\nend\nmut total = 0\nfor_each n in [1, 2]\n    if n not_eq 2\n        total = total + twice(n)\n    end\nend\nsame = total eq 2\nshow_msg(\"{total} {same}\")\n";
    assert_eq!(shows(script), ["2 true"]);
    // `for` and `each` alone are names, now that `for_each` is one keyword.
    assert_eq!(shows("for = 1\neach = 2\nshow_msg(\"{for} {each}\")\n"), ["1 2"]);
    // `ret`, `incl` and `use` are plain names again.
    assert_eq!(shows("ret = 1\nincl = 2\nuse = 3\nshow_msg(\"{ret} {incl} {use}\")\n"), ["1 2 3"]);
    let old = "proc one -> Whole\n    ret 1\nend\n";
    assert!(refusal(old).contains("comes after the end of this line's statement"), "{}", refusal(old));
    assert!(!refusal("a = 1\nb = a == 1\n").is_empty());
    assert!(!refusal("a = 1\nb = a != 1\n").is_empty());
    let failing = "proc pick[n: Whole] -> Whole ! [low]\n    if n < 0\n        fail low\n    else\n        return n\n    end\nend\nv = pick(-1) otherwise\n    case low\n        instead 7\nend\nshow_msg(\"{v}\")\n";
    assert_eq!(shows(failing), ["7"]);
}

// trace: D131, Unio D42
#[test]
fn d131_a_handler_missing_a_case_after_question_marks_is_named_as_a_handler() {
    let script = "proc load[name: Text] -> Whole ! [not_found, broken]\n    if name eq \"cave\"\n        fail not_found\n    elif name eq \"ruin\"\n        fail broken\n    else\n        return 3\n    end\nend\n\ngroup.handler load_failed\n    handles [load]\n    ::start_over\n        case not_found\n            instead 1\n    end\nend\n\nlevel = load(\"cave\") ?? ::start_over\nshow_msg(\"{level}\")\n";
    assert!(refusal(script).contains("this handler has no `case` for `broken`"), "{}", refusal(script));
    let included =
        script.replace("load(\"cave\") ?? ::start_over", "load(\"cave\") otherwise\n    include ::start_over\nend");
    assert!(refusal(&included).contains("this `otherwise` has no `case` for `broken`"), "{}", refusal(&included));
}

// trace: D147
#[test]
fn d147_underscore_reads_what_the_change_changes() {
    let name = "mut score = 5\nscore = _ * _ + 1\nshow_msg(\"{score}\")\n";
    assert_eq!(shows(name), ["26"]);
    // A position is worked out once, before the value, so `bump` growing the list doesn't move it.
    let once = "mut g = [3]\nproc bump -> Whole\n    g.add(9)\n    return 1\nend\ng[pos: g.len] = bump() + _\nshow_msg(\"{g}\")\n";
    assert_eq!(shows(once), ["[4, 9]"]);
    // A key's value is read as a lookup is, so it may be missing.
    let counts = "mut d: Dict<key: Text, value: Whole> = [:]\nd[key: \"a\"] = (_ ?? 0) + 1\nd[key: \"a\"] = (_ ?? 0) + 1\nn = d[key: \"a\"] ?? 0\nshow_msg(\"{n}\")\n";
    assert_eq!(shows(counts), ["2"]);
    let bare = "mut d: Dict<key: Text, value: Whole> = [:]\nd[key: \"a\"] = _ + 1\n";
    assert!(refusal(bare).contains("this can fail with `missing`"), "{}", refusal(bare));
    // Through a way into a part of a part.
    let deep = "record P\n    hp: Whole\nend\nmut ps = [[P[hp: 1]], [P[hp: 5]]]\nps[pos: 2][pos: 1].hp = _ * 10\nv = ps[pos: 2][pos: 1].hp\nshow_msg(\"{v}\")\n";
    assert_eq!(shows(deep), ["50"]);
    let making = "y = _ + 1\nshow_msg(\"{y}\")\n";
    assert!(refusal(making).contains("`_` is the value of what an `=` changes"), "{}", refusal(making));
}

// trace: D148, D149
#[test]
fn d148_d149_measuring_is_written_to_and_maths_before_it_is_bracketed() {
    assert_eq!(shows("total = 2 secs + 500 ms\nms = total to ms\nshow_msg(\"{ms} {total to secs}\")\n"), ["2500 2.5"]);
    let bare = "a = 2 secs\nb = 500 ms\nc = a + b to ms\n";
    let refused = refusal(bare);
    assert!(refused.contains("c = (a + b) to ms"), "{refused}");
}

// trace: D17. A check failing in an instruction made of two points at its maths, as it did when they were two.
#[test]
fn d17_a_fused_instruction_points_at_its_maths() {
    let call = "proc down[n: Whole] -> Whole\n    if n < -9223372036854775806\n        return n\n    else\n        return down(n - 3)\n    end\nend\na = down(-9223372036854775800)\nshow_msg(\"{a}\")\n";
    assert!(refusal(call).contains("test.seal:5:21"), "{}", refusal(call));
    let sum = "proc add[a: Whole, b: Whole] -> Whole\n    return a + b\nend\nc = add(9223372036854775807, 1)\nshow_msg(\"{c}\")\n";
    assert!(refusal(sum).contains("test.seal:2:12"), "{}", refusal(sum));
}

// trace: D115, D107. A `HugeWhole`'s limits may be past a `Whole`'s range.
#[test]
fn d115_a_huge_whole_limit_past_a_whole() {
    let clamped =
        "mut x: HugeWhole[max.clamp: 100000000000000000000] = 5\nx = x * 100000000000000000000\nshow_msg(\"{x}\")\n";
    assert_eq!(shows(clamped), ["100000000000000000000"]);
    let checked = "mut y: HugeWhole[max: 100000000000000000000] = 1\ny = y * 10000000000\nshow_msg(\"{y}\")\ny = y * 100000000000\n";
    assert!(refusal(checked).contains("above its type's `max: 100000000000000000000`"), "{}", refusal(checked));
    let written = "z: HugeWhole[max: 100000000000000000000] = 200000000000000000000\nshow_msg(\"{z}\")\n";
    assert!(refusal(written).contains("this is above its type's `max: 100000000000000000000`"), "{}", refusal(written));
    let crossed = "w: HugeWhole[min: 100000000000000000000, max: 5] = 7\nshow_msg(\"{w}\")\n";
    assert!(refusal(crossed).contains("lowest edge is above its highest"), "{}", refusal(crossed));
    let same = "a: HugeWhole[max: 100000000000000000000] = 5\nb: HugeWhole[max: 100000000000000000000] = a\nshow_msg(\"{b}\")\n";
    assert_eq!(shows(same), ["5"]);
}

// trace: D161. Generic procedures: one copy per set of types, every value's holders counted right, and what a type
// parameter can do, or a host's call to one, left undecided.
#[test]
fn d161_generic_procedures_fill_in_their_types_at_each_call() {
    let nested = "proc wrap<?Item>[a: ?Item] -> List<?Item>\n    return [a, a]\nend\nproc twice<?Item>[a: ?Item] -> List<List<?Item>>\n    inner = wrap(a)\n    return [inner, wrap(a)]\nend\nnames = twice(\"Ann\")\nnumbers = twice(4)\nshow_msg(\"{names} {numbers}\")\n";
    assert_eq!(shows(nested), ["[[\"Ann\", \"Ann\"], [\"Ann\", \"Ann\"]] [[4, 4], [4, 4]]"]);
    let failing = "proc pick<?Item>[items: List<?Item>, spot: Whole] -> ?Item ! [missing]\n    if spot > items.len\n        fail missing\n    else\n        return items[pos: spot]\n    end\nend\nmut held = [\"a\", \"b\"]\nfirst = pick(held, 1) ?? \"none\"\nheld[pos: 1] = \"z\"\nlater = pick(held, 5) ?? \"none\"\nshow_msg(\"{first} {later} {held}\")\n";
    assert_eq!(shows(failing), ["a none [\"z\", \"b\"]"]);
    let from_result = "proc first<?Item>[items: List<?Item>] -> Maybe<?Item>\n    for_each item in items\n        return Maybe.some[value: item]\n    end\n    return Maybe.none\nend\nempty: List<Text> = []\ntop: Maybe<Text> = first(empty)\nshown = top ?? \"nobody\"\nshow_msg(shown)\n";
    assert_eq!(shows(from_result), ["nobody"]);
    let recursive = "proc count<?Item>[items: List<?Item>, n: Whole] -> Whole\n    if n eq 0\n        return items.len\n    else\n        return count(items, n - 1)\n    end\nend\nc = count([\"a\"], 3)\nshow_msg(\"{c}\")\n";
    assert_eq!(shows(recursive), ["1"]);
    for (operation, what) in [("return a + b", "`+`"), ("return a eq b", "`eq`")] {
        let script = format!(
            "proc both<?Item>[a: ?Item, b: ?Item] -> ?Item\n    {operation}\nend\nx = both(1, 2)\nshow_msg(\"{{x}}\")\n"
        );
        let refused = refusal(&script);
        assert!(refused.contains(what) && refused.contains("what a type parameter can do"), "{refused}");
    }
    let shown = "proc say<?Item>[a: ?Item]\n    show_msg(\"{a}\")\nend\nsay(1)\n";
    assert!(refusal(shown).contains("showing `?Item` in braces"), "{}", refusal(shown));
    let unused = "proc keep<?Item, ?Other>[a: ?Item] -> ?Item\n    return a\nend\nx = keep(1)\nshow_msg(\"{x}\")\n";
    assert!(refusal(unused).contains("`?Other` isn't in `keep`'s parameters or result"), "{}", refusal(unused));
    let outside = "x: ?Item = 3\nshow_msg(\"{x}\")\n";
    assert!(refusal(outside).contains("`?Item` isn't one of this procedure's type parameters"), "{}", refusal(outside));
    let twice = "proc keep<?Item, ?Item>[a: ?Item] -> ?Item\n    return a\nend\nx = keep(1)\nshow_msg(\"{x}\")\n";
    assert!(refusal(twice).contains("two type parameters are named `?Item`"), "{}", refusal(twice));
    let growing = "proc grow<?Item>[a: ?Item, n: Whole] -> Whole\n    if n eq 0\n        return 0\n    else\n        return grow([a], n - 1)\n    end\nend\ng = grow(1, 20000)\nshow_msg(\"{g}\")\n";
    assert!(refusal(growing).contains("nested more than 100 deep"), "{}", refusal(growing));
}

// trace: D161. A host's call to a generic procedure stops as not designed yet; its instances keep every value's
// holders right.
#[test]
fn d161_a_host_calls_no_generic_procedure_and_instances_keep_their_counts() {
    let source = "proc pair<?Item>[a: ?Item] -> List<?Item>\n    return [a, a]\nend\nproc names -> Whole\n    both = pair(\"Bo\")\n    return both.len\nend\n";
    let doors = Doors::new();
    let Ok(mut script) = Script::load("generic.seal", source, &doors, LIMITS) else {
        panic!("the script didn't load");
    };
    let called = script.call_values("pair", &[seal::Argument { label: None, value: seal::Value::Whole(1) }]);
    assert!(
        matches!(&called, Err(Error::Refused(messages)) if messages.text().contains("a generic procedure")),
        "{called:?}"
    );
    assert!(matches!(script.call_values("names", &[]), Ok(seal::Called::Finished(seal::Value::Whole(2)))));
    assert_eq!(script.audit(), Ok(()));
}

// trace: D164. Methods: `proc`s inside a record or `one_of`, called only with a dot, two types sharing a name.
#[test]
fn d164_methods_are_procs_inside_their_type_called_with_a_dot() {
    let script = "record Enemy\n    name: Text\n    hp: Whole\n\n    proc is_alive[enemy: Enemy] -> Bool\n        return enemy.hp > 0\n    end\n\n    proc hurt[enemy: Enemy, by: Whole] -> Enemy\n        return Enemy[name: enemy.name, hp: enemy.hp - by]\n    end\n\n    proc size[enemy: Enemy] -> Whole\n        return enemy.hp div.floor 10\n    end\nend\n\none_of Tree\n    leaf\n    node[left: Tree, right: Tree]\n\n    proc size[of: Tree] -> Whole\n        match of\n            case leaf\n                return 1\n            case node[left, right]\n                return 1 + left.size() + right.size()\n        end\n    end\nend\n\ngoblin = Enemy[name: \"Goblin\", hp: 30]\nforest = Tree.node[left: Tree.leaf, right: Tree.leaf]\na = goblin.hurt(30).is_alive()\nb = Enemy[name: \"Ghost\", hp: 50].size()\nc = forest.size()\nshow_msg(\"{a} {b} {c}\")\n";
    assert_eq!(shows(script), ["false 5 3"]);
    let plain = "record Enemy\n    hp: Whole\n\n    proc twice[enemy: Enemy] -> Whole\n        return enemy.hp * 2\n    end\nend\ng = Enemy[hp: 4]\nx = twice(g)\nshow_msg(\"{x}\")\n";
    assert!(refusal(plain).contains("there's no procedure or door named `twice`"), "{}", refusal(plain));
    let wrong_first =
        "record Enemy\n    hp: Whole\n\n    proc hp_of[x: Whole] -> Whole\n        return x\n    end\nend\n";
    assert!(refusal(wrong_first).contains("isn't a value of `Enemy`"), "{}", refusal(wrong_first));
    let twice = "record Enemy\n    hp: Whole\n\n    proc size[e: Enemy] -> Whole\n        return e.hp\n    end\n\n    proc size[e: Enemy] -> Whole\n        return e.hp\n    end\nend\n";
    assert!(refusal(twice).contains("`Enemy` has two methods named `size`"), "{}", refusal(twice));
    let late = "record Enemy\n    hp: Whole\n\n    proc size[e: Enemy] -> Whole\n        return e.hp\n    end\n    name: Text\nend\n";
    assert!(refusal(late).contains("a field comes after a method"), "{}", refusal(late));
    let source = "record Enemy\n    hp: Whole\n\n    proc size[e: Enemy] -> Whole\n        return e.hp\n    end\nend\n";
    let Ok(mut loaded) = Script::load("methods.seal", source, &Doors::new(), LIMITS) else {
        panic!("the script didn't load");
    };
    let called = loaded.call_values("Enemy.size", &[]);
    assert!(matches!(&called, Err(Error::Refused(messages)) if messages.text().contains("a method")), "{called:?}");
}

// trace: D165. A method whose first parameter is `mut` changes the mutable variable it's called on, and only that.
#[test]
fn d165_a_mut_first_parameter_changes_the_variable_it_is_called_on() {
    let kept = "record Spot\n    x: Whole\n    y: Whole\n\n    proc nudge[mut spot: Spot, by: Whole]\n        spot.x = _ + by\n        if spot.x > 10\n            return\n        else\n            spot.y = _ + 1\n        end\n    end\nend\n\nmut here = Spot[x: 1, y: 1]\nhere.nudge(5)\nhere.nudge(5)\nshow_msg(\"{here.x} {here.y}\")\nkept = here\nhere.nudge(1)\nshow_msg(\"{here.x} {kept.x}\")\n";
    assert_eq!(shows(kept), ["11 2", "12 11"]);
    let top_level = "record Bag\n    items: List<Text>\n\n    proc put[mut bag: Bag, item: Text]\n        bag.items.add(item)\n    end\nend\n\nmut bag1 = Bag[items: []]\nproc fill\n    bag1.put(\"a\")\n    bag1.put(\"b\")\nend\nfill()\nkept = bag1\nbag1.put(\"c\")\nshow_msg(\"{bag1.items} {kept.items}\")\n";
    assert_eq!(shows(top_level), ["[\"a\", \"b\", \"c\"] [\"a\", \"b\"]"]);
    let immutable = "record Spot\n    x: Whole\n\n    proc nudge[mut spot: Spot]\n        spot.x = 2\n    end\nend\nhere = Spot[x: 1]\nhere.nudge()\nshow_msg(\"{here.x}\")\n";
    assert!(refusal(immutable).contains("`mut here = Spot[x: 1]`"), "{}", refusal(immutable));
    let unchanged = "record Spot\n    x: Whole\n\n    proc nudge[mut spot: Spot]\n        show_msg(\"{spot.x}\")\n    end\nend\nmut here = Spot[x: 1]\nhere.nudge()\n";
    assert!(refusal(unchanged).contains("proc nudge[spot: Spot]"), "{}", refusal(unchanged));
    let reaching = "record Bag\n    items: List<Text>\n\n    proc put[mut bag: Bag, item: Text]\n        bag.items.add(item)\n        peek()\n    end\nend\n\nmut bag1 = Bag[items: []]\nproc peek\n    show_msg(\"{bag1.items}\")\nend\nbag1.put(\"a\")\n";
    assert!(refusal(reaching).contains("`put` reaches `bag1` itself"), "{}", refusal(reaching));
    let elsewhere = "proc bump[mut n: Whole]\n    n = n + 1\nend\nmut x = 1\nbump(x)\n";
    assert!(refusal(elsewhere).contains("a `mut` parameter other than a method's first"), "{}", refusal(elsewhere));
}
