//! Making random Seal scripts from the decisions and the standard, never from the interpreter's own source
//! (Building rule 6). A generated script is meant to pass checking, so that it reaches the VM; what it does there
//! may still stop on a failed check, a limit or a crash, all of which a host must survive.

use std::fmt::Write;

/// A small random number generator (`SplitMix64`), so every run can be repeated from its seed.
#[derive(Clone, Debug)]
pub struct Random {
    state: u64,
}

impl Random {
    #[must_use]
    pub fn new(seed: u64) -> Self {
        Random { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut mixed = self.state;
        mixed = (mixed ^ (mixed >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        mixed = (mixed ^ (mixed >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        mixed ^ (mixed >> 31)
    }

    /// A number from 0 up to, not including, `limit`.
    pub fn below(&mut self, limit: usize) -> usize {
        if limit == 0 {
            return 0;
        }
        usize::try_from(self.next() % (limit as u64)).unwrap_or(0)
    }

    /// True `chances` times in `out_of`.
    pub fn chance(&mut self, chances: usize, out_of: usize) -> bool {
        self.below(out_of) < chances
    }

    pub fn pick<'items, T>(&mut self, items: &'items [T]) -> Option<&'items T> {
        items.get(self.below(items.len()))
    }
}

/// The types the generator writes values of.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Whole,
    Float,
    Bool,
    Text,
    WholeList,
    TextList,
    /// `Dict<key: Text, value: Whole>`.
    Dict,
    /// The script's record, `Hero`.
    Record,
    /// The script's `one_of`, `Event`.
    Event,
    /// `Maybe<Whole>`.
    Maybe,
    /// The script's record of plain numbers, `Spot`.
    Spot,
    /// The script's record of lists and a dictionary, `Pack`: parts to change parts of (D62).
    Pack,
    /// `List<List<Whole>>`.
    Grid,
    /// `List<Pack>`.
    Packs,
    /// The script's record of records, `Crate` (D72).
    Crate,
    /// The script's `one_of` that holds itself, `Tree` (D72).
    Tree,
    /// `List<Spot>`: records of plain numbers in a list, whose fields loops read and change.
    Spots,
}

impl Kind {
    fn written(self) -> &'static str {
        match self {
            Kind::Whole => "Whole",
            Kind::Float => "Float",
            Kind::Bool => "Bool",
            Kind::Text => "Text",
            Kind::WholeList => "List<Whole>",
            Kind::TextList => "List<Text>",
            Kind::Dict => "Dict<key: Text, value: Whole>",
            Kind::Record => "Hero",
            Kind::Event => "Event",
            Kind::Maybe => "Maybe<Whole>",
            Kind::Spot => "Spot",
            Kind::Pack => "Pack",
            Kind::Grid => "List<List<Whole>>",
            Kind::Packs => "List<Pack>",
            Kind::Crate => "Crate",
            Kind::Tree => "Tree",
            Kind::Spots => "List<Spot>",
        }
    }
}

#[derive(Clone, Debug)]
struct Name {
    text: String,
    kind: Kind,
    mutable: bool,
    read: bool,
    /// Being gone through by a `for_each`, or matched on: it can't change for now.
    frozen: bool,
}

#[derive(Clone, Debug)]
struct ProcedureInfo {
    name: String,
    parameters: Vec<Kind>,
    result: Option<Kind>,
    fails: bool,
    /// Whether the procedure calls the pausing door `wait`. Only the host calls such a procedure.
    pauses: bool,
}

/// What the generator may use: everything, or only what Seal shares with Unio (D19), for comparing with Ralph.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dialect {
    /// Everything Seal has.
    Seal,
    /// What Unio's oracle, Ralph, has built so far of what Seal shares with Unio: numbers, text, `if`, `while`,
    /// procedures, maths and converting. Lists and `for_each` are shared too, but Ralph doesn't run them yet.
    Ralph,
}

pub struct Generator {
    random: Random,
    dialect: Dialect,
    out: String,
    scopes: Vec<Vec<Name>>,
    procedures: Vec<ProcedureInfo>,
    counter: usize,
    depth: usize,
    in_procedure: Option<ProcedureInfo>,
    in_loop: bool,
    budget: usize,
    /// Whether the script has a handler group for its failing procedures (D110).
    handlers: bool,
    /// Whether the script declares its own unit group, `coins` (D123).
    coins: bool,
}

const WORDS: [&str; 12] =
    ["hp", "score", "level", "bag", "name", "speed", "count", "total", "flag", "note", "gold", "step"];
const TEXTS: [&str; 8] = ["Mia", "goblin", "", "a b", "42", " 7", "x", "Hello, World!"];

impl Generator {
    #[must_use]
    pub fn new(seed: u64, dialect: Dialect) -> Self {
        Generator {
            random: Random::new(seed),
            dialect,
            out: String::new(),
            scopes: vec![Vec::new()],
            procedures: Vec::new(),
            counter: 0,
            depth: 0,
            in_procedure: None,
            in_loop: false,
            budget: 0,
            handlers: false,
            coins: false,
        }
    }

    fn seal(&self) -> bool {
        self.dialect == Dialect::Seal
    }

    /// Writes a line, now and then with a comment on the line before it or after its statement (D63). A comment
    /// before `end`, `elif` or `else` is indented one level in, with the block it ends (D91, Unio D145).
    fn line(&mut self, text: &str) {
        if self.random.chance(1, 30) {
            let closes =
                ["end", "elif", "else"].iter().any(|word| text == *word || text.starts_with(&format!("{word} ")));
            if closes {
                self.depth += 1;
                self.plain_line("// a note");
                self.depth -= 1;
            } else {
                self.plain_line("// a note");
            }
        }
        if self.random.chance(1, 30) {
            self.plain_line(&format!("{text} // a note"));
        } else {
            self.plain_line(text);
        }
    }

    /// Closes a block with `end`, now and then with its checked note in Seal's own dialect (D102).
    fn end(&mut self, note: &str) {
        if self.seal() && self.random.chance(1, 4) {
            self.line(&format!("end \\\\ {note}"));
        } else {
            self.line("end");
        }
    }

    fn plain_line(&mut self, text: &str) {
        for _ in 0..self.depth {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn fresh(&mut self) -> String {
        self.counter += 1;
        let word = self.random.pick(&WORDS).copied().unwrap_or("x");
        format!("{word}{}", self.counter)
    }

    fn kinds(&self) -> Vec<Kind> {
        if self.seal() {
            vec![
                Kind::Whole,
                Kind::Whole,
                Kind::Float,
                Kind::Bool,
                Kind::Text,
                Kind::WholeList,
                Kind::TextList,
                Kind::Dict,
                Kind::Record,
                Kind::Event,
                Kind::Maybe,
                Kind::Spot,
                Kind::Spot,
                Kind::Pack,
                Kind::Grid,
                Kind::Packs,
                Kind::Crate,
                Kind::Tree,
                Kind::Spots,
                Kind::Spots,
            ]
        } else {
            vec![Kind::Whole, Kind::Whole, Kind::Bool, Kind::Text]
        }
    }

    fn any_kind(&mut self) -> Kind {
        let kinds = self.kinds();
        self.random.pick(&kinds).copied().unwrap_or(Kind::Whole)
    }

    fn visible(&self) -> impl Iterator<Item = &Name> {
        self.scopes.iter().flatten()
    }

    /// A name of this kind that exists here, marked as read.
    fn read_name(&mut self, kind: Kind) -> Option<String> {
        let candidates: Vec<String> =
            self.visible().filter(|name| name.kind == kind).map(|name| name.text.clone()).collect();
        let chosen = self.random.pick(&candidates)?.clone();
        for name in self.scopes.iter_mut().flatten() {
            if name.text == chosen {
                name.read = true;
            }
        }
        Some(chosen)
    }

    fn changeable_name(&mut self, kind: Kind) -> Option<String> {
        let candidates: Vec<String> = self
            .visible()
            .filter(|name| name.kind == kind && name.mutable && !name.frozen)
            .map(|name| name.text.clone())
            .collect();
        self.random.pick(&candidates).cloned()
    }

    fn declare(&mut self, text: &str, kind: Kind, mutable: bool) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.push(Name { text: text.to_owned(), kind, mutable, read: false, frozen: false });
        }
    }

    // ---- values ----

    fn whole_literal(&mut self) -> String {
        // Ralph doesn't read the biggest and smallest `Whole`s yet, so they're left to Seal's own scripts.
        let first = if self.seal() { 0 } else { 2 };
        match first + self.random.below(12 - first) {
            0 => "9223372036854775807".to_owned(),
            1 => "-9223372036854775808".to_owned(),
            2 => "0".to_owned(),
            3 => format!("{}", self.random.below(1_000_000)),
            4 => format!("-{}", self.random.below(100)),
            _ => format!("{}", self.random.below(20)),
        }
    }

    fn float_literal(&mut self) -> String {
        const EXACT: [&str; 8] = ["0.0", "0.5", "1.0", "2.0", "0.25", "7.0", "100.0", "1.5"];
        const INEXACT: [&str; 5] = ["~0.1", "~0.2", "~3.3", "~0.7", "~1.1"];
        let literal = if self.random.chance(2, 3) {
            self.random.pick(&EXACT).copied().unwrap_or("1.0")
        } else {
            self.random.pick(&INEXACT).copied().unwrap_or("~0.1")
        };
        if self.random.chance(1, 6) { format!("-{literal}") } else { literal.to_owned() }
    }

    fn text_literal(&mut self) -> String {
        // Seal's own dialect now and then writes text beyond plain English characters, where a grapheme is more
        // than a byte (D66), and a quote or a backslash made literal (D63).
        if self.seal() && self.random.chance(1, 6) {
            let text = self.random.pick(&["héllo", "👍🏽 ok", "e\u{301}a", "日本", "a \\\"b\\\" c", "back\\\\slash"]);
            return format!("\"{}\"", text.copied().unwrap_or("x"));
        }
        format!("\"{}\"", self.random.pick(&TEXTS).copied().unwrap_or("x"))
    }

    /// A call to one of the generic procedures or methods every script in Seal's dialect has, giving `kind`.
    fn generic_call(&mut self, kind: Kind, level: usize) -> Option<String> {
        let inner = level.saturating_sub(1);
        // trace: D164. A method of the script's records and `one_of`, on a variable of the type.
        if self.random.chance(1, 2) {
            match kind {
                Kind::Whole if self.random.chance(1, 4) => {
                    // trace: D168. A generic calling a method its requirement lists.
                    if let Some(name) = self.read_name(Kind::Record) {
                        return Some(format!("generic_power({name})"));
                    }
                }
                Kind::Whole => {
                    let (of, method) = match self.random.below(3) {
                        0 => (Kind::Record, "power"),
                        1 => (Kind::Spot, "size"),
                        _ => (Kind::Tree, "size"),
                    };
                    if let Some(name) = self.read_name(of) {
                        return Some(format!("{name}.{method}()"));
                    }
                }
                Kind::Record => {
                    if let Some(name) = self.read_name(Kind::Record) {
                        return Some(format!("{name}.renamed({})", self.value(Kind::Text, inner)));
                    }
                }
                _ => {}
            }
        }
        // trace: D166. A generic whose type parameter requires a `Number`.
        if matches!(kind, Kind::Whole | Kind::Float) && self.random.chance(1, 3) {
            return Some(format!("generic_bigger({}, {})", self.value(kind, inner), self.value(kind, inner)));
        }
        Some(match kind {
            Kind::Whole => {
                format!("generic_pick({}, {})", self.value(Kind::WholeList, inner), self.value(Kind::Whole, inner))
            }
            Kind::Text => {
                format!("generic_pick({}, {})", self.value(Kind::TextList, inner), self.value(Kind::Text, inner))
            }
            Kind::WholeList => format!("generic_pair({})", self.value(Kind::Whole, inner)),
            Kind::TextList => format!("generic_pair({})", self.value(Kind::Text, inner)),
            Kind::Maybe => format!("generic_first({})", self.value(Kind::WholeList, inner)),
            Kind::Grid => format!("generic_pairs({})", self.value(Kind::Whole, inner)),
            _ => return None,
        })
    }

    /// A value of `kind`, as an expression. `level` bounds how deep it nests.
    #[allow(clippy::too_many_lines)] // one case per kind of value
    fn value(&mut self, kind: Kind, level: usize) -> String {
        let simple = level == 0 || self.random.chance(1, 3);
        if simple
            && self.random.chance(2, 3)
            && let Some(name) = self.read_name(kind)
        {
            return name;
        }
        // trace: D161. Now and then a call to one of the script's generic procedures, filled in with this kind.
        if !simple
            && self.seal()
            && self.random.chance(1, 14)
            && let Some(call) = self.generic_call(kind, level)
        {
            return call;
        }
        match kind {
            Kind::Whole => {
                if simple {
                    return self.whole_literal();
                }
                if self.seal() && self.random.chance(1, 10) {
                    return self.measured(Kind::Whole);
                }
                if self.seal() && self.random.chance(1, 12) {
                    return self.big_number(Kind::Whole);
                }
                // The maths built-ins and `**` on `Whole`s (D107, D108).
                if self.seal() && self.random.chance(1, 8) {
                    let a = self.operand(Kind::Whole, level - 1);
                    // The second value is made only where it's written, so a name it reads is really read.
                    return match self.random.below(5) {
                        0 => format!("abs({a})"),
                        1 => format!("min({a}, {})", self.operand(Kind::Whole, level - 1)),
                        2 => format!("max({a}, {})", self.operand(Kind::Whole, level - 1)),
                        3 => format!("clamp({a}, low: {}, high: {})", self.random.below(5), 5 + self.random.below(20)),
                        _ => {
                            let base = if a.starts_with('-') { format!("({a})") } else { a };
                            format!("{base} ** {}", self.random.below(5))
                        }
                    };
                }
                match self.random.below(11) {
                    0..=2 => {
                        let operator = *self.random.pick(&["+", "-", "*"]).unwrap_or(&"+");
                        format!(
                            "{} {operator} {}",
                            self.operand(Kind::Whole, level - 1),
                            self.operand(Kind::Whole, level - 1)
                        )
                    }
                    3 if self.seal() => {
                        let operator = *self
                            .random
                            .pick(&["div.floor", "div.trunc", "rem.floor", "rem.trunc", "/"])
                            .unwrap_or(&"/");
                        format!(
                            "{} {operator} {}",
                            self.operand(Kind::Whole, level - 1),
                            self.operand(Kind::Whole, level - 1)
                        )
                    }
                    // A text's length, in graphemes, code points or bytes (D65, D66).
                    4 if self.seal() && self.random.chance(1, 3) => match self.read_name(Kind::Text) {
                        Some(text) => {
                            let unit = *self.random.pick(&["", "", ".egc", ".ucp", ".bytes"]).unwrap_or(&"");
                            format!("{text}.len{unit}")
                        }
                        None => self.whole_literal(),
                    },
                    4 => match self.read_name(Kind::WholeList) {
                        Some(list) => format!("{list}.len"),
                        None => self.whole_literal(),
                    },
                    5 => match self.read_name(Kind::WholeList) {
                        Some(list) => {
                            let label = if self.random.chance(1, 2) { "pos" } else { "offset" };
                            format!("{list}[{label}: {}]", self.value(Kind::Whole, level - 1))
                        }
                        None => self.whole_literal(),
                    },
                    6 => format!("(be.Whole({}) ?? {})", self.argument(Kind::Text, level), self.whole_literal()),
                    7 if self.seal() => {
                        let rounding = *self
                            .random
                            .pick(&["floor", "trunc", "ceil", "nearest", "nearest.even"])
                            .unwrap_or(&"floor");
                        format!("be.Whole.{rounding}({})", self.argument(Kind::Float, level))
                    }
                    8 if self.seal() => match self.read_name(Kind::Dict) {
                        Some(dict) => {
                            format!("({dict}[key: {}] ?? {})", self.value(Kind::Text, 0), self.whole_literal())
                        }
                        None => self.whole_literal(),
                    },
                    9 if self.seal() && self.random.chance(1, 3) => {
                        // A part of a part, read (D62).
                        if let Some(grid) = self.read_name(Kind::Grid) {
                            format!("{grid}[pos: 1].len")
                        } else if let Some(pack) = self.read_name(Kind::Pack) {
                            format!("{pack}.items.len")
                        } else if let Some(packs) = self.read_name(Kind::Packs) {
                            format!("{packs}.len")
                        } else {
                            self.whole_literal()
                        }
                    }
                    9 if self.seal() => match self.read_name(Kind::Maybe) {
                        Some(maybe) => format!("({maybe} ?? {})", self.whole_literal()),
                        None => match self.read_name(Kind::Spot) {
                            Some(spot) => format!("{spot}.x"),
                            None => self.whole_literal(),
                        },
                    },
                    _ => self.call_value(Kind::Whole, level).unwrap_or_else(|| self.whole_literal()),
                }
            }
            Kind::Float => {
                if simple {
                    return self.float_literal();
                }
                if self.seal() && self.random.chance(1, 8) {
                    return self.measured(Kind::Float);
                }
                if self.seal() && self.random.chance(1, 10) {
                    return self.big_number(Kind::Float);
                }
                match self.random.below(6) {
                    0..=2 => {
                        let operator = *self.random.pick(&["+", "-", "*", "/"]).unwrap_or(&"+");
                        format!(
                            "{} {operator} {}",
                            self.operand(Kind::Float, level - 1),
                            self.operand(Kind::Float, level - 1)
                        )
                    }
                    3 => format!("sqrt({})", self.argument(Kind::Float, level)),
                    4 => format!("be.Float({})", self.argument(Kind::Whole, level)),
                    // The angle built-ins, `lerp`, `**` and `pi` on `Float`s (D108, D109, D114).
                    5 if self.seal() && self.random.chance(1, 2) => {
                        // Each value is made only where it's written, so a name it reads is really read.
                        let which = self.random.below(8);
                        let a = if which < 6 { self.argument(Kind::Float, level) } else { String::new() };
                        let b = if matches!(which, 4 | 5) { self.argument(Kind::Float, level) } else { String::new() };
                        match which {
                            0 => format!("sin({a})"),
                            1 => format!("cos({a})"),
                            2 => format!("tan({a})"),
                            // trace: D134. On plain numbers these give a plain `Float`.
                            3 => format!("asin({a})"),
                            4 => format!("atan2(y: {a}, x: {b})"),
                            5 => format!("lerp({a}, to: {b}, by: 0.5)"),
                            6 => {
                                let base = self.operand(Kind::Float, level - 1);
                                let base = if base.starts_with('-') { format!("({base})") } else { base };
                                let power = self.random.pick(&["2.0", "0.5", "-1.0", "3.0"]).copied().unwrap_or("2.0");
                                format!("{base} ** {power}")
                            }
                            _ => "pi".to_owned(),
                        }
                    }
                    _ => self.call_value(Kind::Float, level).unwrap_or_else(|| self.float_literal()),
                }
            }
            Kind::Bool => {
                if simple {
                    return if self.random.chance(1, 2) { "true".to_owned() } else { "false".to_owned() };
                }
                match self.random.below(10) {
                    // Two bigger values compared by their contents (D95), or text read as a `Bool` (D101).
                    8 if self.seal() => {
                        let kind = *self
                            .random
                            .pick(&[
                                Kind::WholeList,
                                Kind::Record,
                                Kind::Dict,
                                Kind::Spot,
                                Kind::Tree,
                                Kind::Event,
                                Kind::Maybe,
                            ])
                            .unwrap_or(&Kind::WholeList);
                        let operator = if self.random.chance(1, 2) { "eq" } else { "not_eq" };
                        let left = self.read_name(kind).unwrap_or_else(|| self.value(kind, 1));
                        let right = self.read_name(kind).unwrap_or_else(|| self.value(kind, 1));
                        format!("{} {operator} {}", Self::bracketed(left), Self::bracketed(right))
                    }
                    9 if self.seal() => {
                        let text =
                            *self.random.pick(&["\"true\"", "\"false\"", "\"yes\"", "\"\""]).unwrap_or(&"\"true\"");
                        format!("(be.Bool({text}) ?? false)")
                    }
                    // Two texts compared exactly (D64); Unio leaves comparing text open, so only Seal's dialect has it.
                    7 if self.seal() => {
                        let operator = if self.random.chance(1, 2) { "eq" } else { "not_eq" };
                        format!(
                            "{} {operator} {}",
                            self.operand(Kind::Text, level - 1),
                            self.operand(Kind::Text, level - 1)
                        )
                    }
                    0..=2 => {
                        let operator =
                            *self.random.pick(&["eq", "not_eq", "<", ">", "at_least", "at_most"]).unwrap_or(&"<");
                        format!(
                            "{} {operator} {}",
                            self.operand(Kind::Whole, level - 1),
                            self.operand(Kind::Whole, level - 1)
                        )
                    }
                    3 if self.seal() => {
                        let operator = *self.random.pick(&["<", ">", "at_least", "at_most"]).unwrap_or(&"<");
                        format!(
                            "{} {operator} {}",
                            self.operand(Kind::Float, level - 1),
                            self.operand(Kind::Float, level - 1)
                        )
                    }
                    4 => {
                        let operator = if self.random.chance(1, 2) { "and" } else { "or" };
                        format!(
                            "({} {operator} {})",
                            self.value(Kind::Bool, level - 1),
                            self.value(Kind::Bool, level - 1)
                        )
                    }
                    5 => format!("not ({})", self.value(Kind::Bool, level - 1)),
                    _ => match self.read_name(Kind::Dict) {
                        Some(dict) if self.seal() => format!("{dict}.has(key: {})", self.value(Kind::Text, 0)),
                        _ => "true".to_owned(),
                    },
                }
            }
            Kind::Text => {
                if simple {
                    return self.text_literal();
                }
                match self.random.below(6) {
                    2 => {
                        let from = if self.seal() {
                            *self.random.pick(&[Kind::Whole, Kind::Float, Kind::Bool]).unwrap_or(&Kind::Whole)
                        } else {
                            *self.random.pick(&[Kind::Whole, Kind::Bool]).unwrap_or(&Kind::Whole)
                        };
                        format!("be.Text({})", self.argument(from, level))
                    }
                    0 | 1 | 3 => self.filled_text(),
                    4 => match self.read_name(Kind::TextList) {
                        Some(list) => format!("{list}[pos: {}]", self.value(Kind::Whole, 0)),
                        None => self.text_literal(),
                    },
                    _ => match self.read_name(Kind::Record) {
                        Some(record) if self.seal() => format!("{record}.name"),
                        _ => self.text_literal(),
                    },
                }
            }
            // A range kept as a list (D99).
            Kind::WholeList if self.seal() && self.random.chance(1, 5) => {
                let edge = if self.random.chance(1, 2) { "to" } else { "before" };
                // Now and then a range whose numbers are only known while running, which may stop there.
                if self.random.chance(1, 6) {
                    let from = self.value(Kind::Whole, 0);
                    return format!("range[from: {from}, {edge}: {}]", self.random.below(12));
                }
                let from = self.random.below(8);
                if self.random.chance(1, 3) {
                    format!("range[from: {from}, {edge}: {}, step_by: -1]", self.random.below(from + 1))
                } else {
                    let step = *self.random.pick(&["", ", step_by: 2"]).unwrap_or(&"");
                    format!("range[from: {from}, {edge}: {}{step}]", from + self.random.below(8))
                }
            }
            Kind::WholeList => {
                let count = self.random.below(4) + 1;
                let items: Vec<String> = (0..count).map(|_| self.value(Kind::Whole, level.saturating_sub(1))).collect();
                format!("[{}]", items.join(", "))
            }
            // A text split into its pieces (D65).
            Kind::TextList if self.seal() && self.random.chance(1, 4) => {
                let text = match self.read_name(Kind::Text) {
                    Some(text) => text,
                    None => "\"a b  c\"".to_owned(),
                };
                let method = if self.random.chance(1, 2) { "split" } else { "split.no_empty" };
                let by = *self.random.pick(&[" ", ",", "a"]).unwrap_or(&" ");
                format!("{text}.{method}(by: \"{by}\")")
            }
            Kind::TextList => {
                let count = self.random.below(3) + 1;
                let items: Vec<String> = (0..count).map(|_| self.value(Kind::Text, 0)).collect();
                format!("[{}]", items.join(", "))
            }
            Kind::Dict => {
                let count = self.random.below(3) + 1;
                let pairs: Vec<String> = (0..count)
                    .map(|index| format!("\"key{index}\": {}", self.value(Kind::Whole, level.saturating_sub(1))))
                    .collect();
                format!("[{}]", pairs.join(", "))
            }
            Kind::Record => {
                format!(
                    "Hero[name: {}, hp: {}]",
                    self.value(Kind::Text, level.saturating_sub(1)),
                    self.value(Kind::Whole, level.saturating_sub(1))
                )
            }
            Kind::Event => match self.random.below(3) {
                0 => "Event.idle".to_owned(),
                1 => format!("Event.hurt[amount: {}]", self.value(Kind::Whole, level.saturating_sub(1))),
                _ => format!(
                    "Event.healed[amount: {}, by: {}]",
                    self.value(Kind::Whole, level.saturating_sub(1)),
                    self.value(Kind::Text, 0)
                ),
            },
            Kind::Pack => {
                format!(
                    "Pack[items: {}, tags: {}, stock: {}]",
                    self.value(Kind::WholeList, 1),
                    self.value(Kind::TextList, 1),
                    self.value(Kind::Dict, 1)
                )
            }
            Kind::Crate => {
                format!("Crate[pack: {}, spot: {}]", self.value(Kind::Pack, 1), self.value(Kind::Spot, 1))
            }
            Kind::Tree => {
                if level == 0 || self.random.chance(1, 3) {
                    "Tree.leaf".to_owned()
                } else {
                    format!(
                        "Tree.node[left: {}, right: {}]",
                        self.value(Kind::Tree, level - 1),
                        self.value(Kind::Tree, level - 1)
                    )
                }
            }
            Kind::Spots => {
                let count = self.random.below(4) + 1;
                let items: Vec<String> = (0..count).map(|_| self.value(Kind::Spot, 1)).collect();
                format!("[{}]", items.join(", "))
            }
            Kind::Grid | Kind::Packs => {
                let inner = if kind == Kind::Grid { Kind::WholeList } else { Kind::Pack };
                let count = self.random.below(3) + 1;
                let items: Vec<String> = (0..count).map(|_| self.value(inner, 1)).collect();
                format!("[{}]", items.join(", "))
            }
            Kind::Spot => {
                format!(
                    "Spot[x: {}, y: {}]",
                    self.value(Kind::Whole, level.saturating_sub(1)),
                    self.value(Kind::Float, level.saturating_sub(1))
                )
            }
            Kind::Maybe => {
                if self.random.chance(1, 3) {
                    "Maybe.none".to_owned()
                } else {
                    format!("Maybe.some[value: {}]", self.value(Kind::Whole, level.saturating_sub(1)))
                }
            }
        }
    }

    /// A value as one side of `eq`: anything that isn't a single word gets brackets.
    /// Numbers with units, worked with and measured back into a plain number with `in` (D111 to D113, D120 to
    /// D123).
    fn measured(&mut self, kind: Kind) -> String {
        let (a, b) = (self.random.below(1000), self.random.below(1000));
        if kind == Kind::Whole {
            return match self.random.below(6) {
                0 => format!("({a} secs + {b} ms) to ms"),
                1 => format!("(-{a} mins + {b} secs) to secs"),
                2 => format!("({a} turns) to degs"),
                3 if self.coins => format!("({a} stacks - {b} bits) to bits"),
                4 => format!("({a} px * {}) to px", self.random.below(9)),
                _ => format!("({a} secs) to secs"),
            };
        }
        let (x, y) = (self.float_literal(), self.float_literal());
        match self.random.below(9) {
            0 => format!("({x} secs - {y} ms) to secs"),
            1 => format!("({x} px per secs * {y} secs) to px"),
            2 => format!("atan2(y: {x} px, x: {y} px) to degs"),
            3 => format!("({x} px.point - {y} px.point) to px"),
            4 => format!("sin({a} degs) + cos({x} turns)"),
            5 => format!("tan({x} rads)"),
            6 => format!("({a} degs * {b}) to turns"),
            7 => format!("({x} secs / {y} secs)"),
            _ => format!("({x} rads) to degs"),
        }
    }

    /// `HugeWhole`s, `Decimal`s and `HugeDecimal`s, read from text and worked with, then converted back to a
    /// `Whole` or a `Float` (D115, D116, D119, D125 to D127).
    fn big_number(&mut self, kind: Kind) -> String {
        const WHOLES: [&str; 8] = [
            "0",
            "7",
            "-12",
            "123456789012345678901234567890",
            "-99999999999999999999",
            "9223372036854775808",
            "seven",
            "1_000",
        ];
        const DECIMALS: [&str; 10] = [
            "4.99",
            "0.1",
            "-0.001",
            "10.00",
            "0",
            "2.25",
            "123456789.123456789",
            "1.",
            "x",
            "99999999999999999999999999999999999.5",
        ];
        let pick = |random: &mut Random, items: &[&'static str]| random.pick(items).copied().unwrap_or("0");
        let huge = |random: &mut Random| format!("(be.HugeWhole(\"{}\") ?? 0)", pick(random, &WHOLES));
        let decimal = |random: &mut Random| format!("(be.Decimal(\"{}\") ?? 0.0)", pick(random, &DECIMALS));
        let exact = |random: &mut Random| format!("(be.HugeDecimal(\"{}\") ?? 0.0)", pick(random, &DECIMALS));
        let (h1, h2) = (huge(&mut self.random), huge(&mut self.random));
        let (d1, d2) = (decimal(&mut self.random), decimal(&mut self.random));
        let (e1, e2) = (exact(&mut self.random), exact(&mut self.random));
        let power = self.random.below(40);
        if kind == Kind::Whole {
            return match self.random.below(7) {
                0 => format!("(be.Whole({h1} * {h2} + 5) ?? 0)"),
                1 => format!("(be.Whole({h1} div.floor {h2}) ?? 0)"),
                2 => format!("(be.Whole.floor({d1} * {d2}) ?? 0)"),
                3 => format!("(be.Whole.nearest(sqrt({e1} * {e1})) ?? 0)"),
                4 => format!("(be.Whole.trunc({e1} rem.trunc {e2}) ?? 0)"),
                5 => format!("(be.Whole(({h1} * 1 secs + 500 ms) to ms) ?? 0)"),
                _ => format!("(be.Whole({h1} ** {power} rem.floor 1000) ?? 0)"),
            };
        }
        match self.random.below(7) {
            0 => format!("(be.Float({d1} / {d2} + 0.5) ?? 0.0)"),
            1 => format!("(be.Float(lerp({d1}, to: {d2}, by: 0.25)) ?? 0.0)"),
            2 => format!("(be.Float(abs({e1} - {e2}) + min({e1}, 2.5)) ?? 0.0)"),
            3 => format!("(be.Float(be.Decimal.shown({})) ?? 0.0)", self.float_literal()),
            4 => format!("(be.Float(sqrt({d1}) ** {}) ?? 0.0)", power % 5),
            5 => format!("(be.Float({e1} * {e2} - {h1}) ?? 0.0)"),
            _ => format!("(be.Float(clamp({d1}, low: 0.0, high: 9.99)) ?? 0.0)"),
        }
    }

    fn bracketed(value: String) -> String {
        if value.chars().all(|character| character.is_ascii_alphanumeric() || character == '_') {
            value
        } else {
            format!("({value})")
        }
    }

    /// A value as a call's argument: never itself a call with a call inside, since three calls in a chain are a
    /// slide (Unio D128).
    fn argument(&mut self, kind: Kind, level: usize) -> String {
        self.value(kind, level.saturating_sub(1).min(1))
    }

    /// A value as one side of maths or a comparison: anything looser is put in brackets.
    fn operand(&mut self, kind: Kind, level: usize) -> String {
        let value = self.value(kind, level);
        let measured = value.contains(" to ");
        if value.contains(' ') && (measured || (!value.starts_with('(') && !value.ends_with(')'))) {
            format!("({value})")
        } else {
            value
        }
    }

    /// Text with names that exist filled into its braces (D75, D77): in Seal's own dialect a name may be followed by
    /// its fields and facts, or be any value at all, shown as it's written (D28, D100); Unio's braces hold only a
    /// plain number, `Bool` or text's name (Unio D200, D202).
    fn filled_text(&mut self) -> String {
        let mut text = String::from("\"");
        for _ in 0..=self.random.below(3) {
            text.push_str(self.random.pick(&["value ", "is ", "", "got: ", "= ", "\\{x} "]).copied().unwrap_or(""));
            let kinds: &[Kind] = if self.seal() {
                &[
                    Kind::Whole,
                    Kind::Text,
                    Kind::Bool,
                    Kind::Float,
                    Kind::WholeList,
                    Kind::Record,
                    Kind::Dict,
                    Kind::Spot,
                    Kind::Tree,
                    Kind::Event,
                ]
            } else {
                &[Kind::Whole, Kind::Text, Kind::Bool]
            };
            let kind = *self.random.pick(kinds).unwrap_or(&Kind::Whole);
            if let Some(name) = self.read_name(kind) {
                match kind {
                    Kind::WholeList if self.random.chance(1, 2) => {
                        let _ = write!(text, "{{{name}.len}} ");
                    }
                    Kind::Record if self.random.chance(1, 2) => {
                        let _ = write!(text, "{{{name}.hp}} ");
                    }
                    Kind::Float if self.random.chance(1, 3) => {
                        let _ = write!(text, "{{{name}, places: {}}} ", self.random.below(4));
                    }
                    _ => {
                        let _ = write!(text, "{{{name}}} ");
                    }
                }
            }
            if self.seal() && self.random.chance(1, 8) {
                text.push_str(self.random.pick(&["{newln}", "{tab}"]).copied().unwrap_or(""));
            }
        }
        text.push('"');
        text
    }

    /// A call to one of the script's procedures that gives `kind`, with its failure dealt with.
    fn call_value(&mut self, kind: Kind, level: usize) -> Option<String> {
        let candidates: Vec<ProcedureInfo> = self
            .procedures
            .iter()
            .filter(|procedure| procedure.result == Some(kind) && !procedure.pauses)
            .cloned()
            .collect();
        let procedure = self.random.pick(&candidates)?.clone();
        // A procedure never calls itself here, so every generated script ends.
        if self.in_procedure.as_ref().is_some_and(|inside| inside.name == procedure.name) {
            return None;
        }
        let arguments: Vec<String> = procedure
            .parameters
            .iter()
            .map(|&parameter| self.value(parameter, level.saturating_sub(1).min(1)))
            .collect();
        let call = format!("{}({})", procedure.name, arguments.join(", "));
        if !procedure.fails {
            return Some(call);
        }
        let fallback = self.value(kind, 0);
        let inside_failing = self.in_procedure.as_ref().is_some_and(|inside| inside.fails);
        Some(if inside_failing && self.random.chance(1, 3) {
            format!("{call}?")
        } else if self.handlers && kind == Kind::Whole && self.random.chance(1, 2) {
            // trace: D98, D110. A handler on one line, short or with its group.
            let handler = *self
                .random
                .pick(&["::zero", "low_failed::zero", "::with(3)", "low_failed::with(-1)"])
                .unwrap_or(&"::zero");
            format!("({call} ?? {handler})")
        } else {
            format!("({call} ?? {fallback})")
        })
    }

    // ---- statements ----

    fn show(&mut self, kind: Kind, name: &str) -> String {
        match kind {
            // trace: D100. Every value shows as it's written in Seal.
            _ if self.seal() && self.random.chance(1, 2) => format!("show_msg(\"{name}: {{{name}}}\")"),
            Kind::WholeList | Kind::TextList | Kind::Dict | Kind::Grid | Kind::Packs | Kind::Spots => {
                format!("show_msg(\"{name} has {{{name}.len}}\")")
            }
            Kind::Pack => {
                format!("show_msg(\"{name}: {{{name}.items.len}} {{{name}.tags.len}} {{{name}.stock.len}}\")")
            }
            Kind::Crate => format!("show_msg(\"{name}: {{{name}.pack.items.len}} {{{name}.spot.x}}\")"),
            Kind::Record => format!("show_msg(\"{name}: {{{name}.name}} {{{name}.hp}}\")"),
            Kind::Spot => format!("show_msg(\"{name}: {{{name}.x}} {{{name}.y}}\")"),
            Kind::Whole | Kind::Bool | Kind::Text | Kind::Float | Kind::Tree | Kind::Event | Kind::Maybe => {
                format!("show_msg(\"{name}: {{{name}}}\")")
            }
        }
    }

    /// Loops over a `mut` list of `Spot`s that read and change their fields, through loop names and written-out
    /// places that may or may not be the same item, with every spot shown afterwards. The compiler keeps such
    /// fields in registers through a run of plain statements; this is what checks it against the plain way.
    fn spots_loops(&mut self) {
        let Some(spots) = self.changeable_name(Kind::Spots) else { return };
        let other = self.read_name(Kind::Spots).filter(|other| *other != spots);
        let (outer, inner) = (self.fresh(), self.fresh());
        // The inner loop starts past the outer one's place, or anywhere: the first can never reach the same item.
        // A range that starts past its end is a failed check, so the one starting past the outer place stops
        // `before:` one past the list's length, which is empty when there's nothing left.
        let range = match self.random.below(3) {
            0 => format!("from {outer} + 1 before {spots}.len + 1"),
            1 => format!("from 1 to {spots}.len"),
            _ => format!("from {outer} to {spots}.len"),
        };
        // trace: D172. Counting is `for`.
        self.line(&format!("for {outer} from 1 to {spots}.len"));
        self.depth += 1;
        self.line(&format!("for {inner} {range}"));
        self.depth += 1;
        let place = |generator: &mut Generator| match generator.random.below(8) {
            0..=2 => format!("pos: {outer}"),
            3..=5 => format!("pos: {inner}"),
            6 => "pos: 1".to_owned(),
            _ => "offset: 0".to_owned(),
        };
        // Both loop names are read, as D61 asks, by reaching items with them.
        self.line(&format!("{spots}[pos: {inner}].x = {spots}[pos: {outer}].x + 1"));
        let mut kept: Vec<(String, bool)> = Vec::new();
        for _ in 0..self.random.below(6) + 2 {
            let whole = self.random.chance(1, 2);
            let field = if whole { "x" } else { "y" };
            let read = |generator: &mut Generator, kept: &[(String, bool)]| {
                let named: Vec<&(String, bool)> = kept.iter().filter(|(_, is_whole)| *is_whole == whole).collect();
                match generator.random.below(5) {
                    0 if !named.is_empty() => {
                        generator.random.pick(&named).map_or_else(String::new, |(name, _)| name.clone())
                    }
                    1 => match &other {
                        Some(other) => format!("{other}[pos: 1].{field}"),
                        None => format!("{spots}[{}].{field}", place(generator)),
                    },
                    2 => {
                        if whole {
                            "2".to_owned()
                        } else {
                            "0.5".to_owned()
                        }
                    }
                    _ => format!("{spots}[{}].{field}", place(generator)),
                }
            };
            let operator = if whole {
                *self.random.pick(&["+", "-"]).unwrap_or(&"+")
            } else {
                *self.random.pick(&["+", "-", "*"]).unwrap_or(&"+")
            };
            let value = format!("{} {operator} {}", read(self, &kept), read(self, &kept));
            if self.random.below(4) == 0 {
                let name = self.fresh();
                self.line(&format!("{name} = {value}"));
                kept.push((name, whole));
            } else {
                let target = place(self);
                self.line(&format!("{spots}[{target}].{field} = {value}"));
            }
        }
        // Every name made is read, so that none is refused as never used.
        for (name, _) in &kept {
            self.line(&format!("show_msg(\"kept {{{name}}}\")"));
        }
        self.depth -= 1;
        self.end("for_each");
        self.depth -= 1;
        self.end("for_each");
        let spot = self.fresh();
        self.line(&format!("for_each {spot} in {spots}"));
        self.depth += 1;
        self.line(&format!("show_msg(\"{{{spot}.x}} {{{spot}.y}}\")"));
        self.depth -= 1;
        self.end("for_each");
    }

    /// A change that reaches through fields and places of a `mut` name, or a method at the end of such a way (D62).
    fn nested_change(&mut self) {
        let place = |generator: &mut Generator| {
            if generator.random.chance(1, 2) {
                format!("pos: {}", generator.random.below(3) + 1)
            } else {
                format!("offset: {}", generator.random.below(3))
            }
        };
        // A record inside a record, changed deep down (D72).
        if self.random.chance(1, 4)
            && let Some(held) = self.changeable_name(Kind::Crate)
        {
            let line = match self.random.below(5) {
                // trace: D147. `_` is the part's own value, read through the way the change takes.
                0 if self.random.chance(1, 2) => format!("{held}.spot.x = _ + ({})", self.value(Kind::Whole, 1)),
                0 => format!("{held}.spot.x = {}", self.value(Kind::Whole, 1)),
                1 => format!("{held}.pack.items.add({})", self.value(Kind::Whole, 1)),
                2 if self.random.chance(1, 2) => format!(
                    "{held}.pack.stock[key: {}] = (_ ?? 0) - ({})",
                    self.value(Kind::Text, 0),
                    self.value(Kind::Whole, 1)
                ),
                2 => format!("{held}.pack.stock[key: {}] = {}", self.value(Kind::Text, 0), self.value(Kind::Whole, 1)),
                3 => format!("{held}.pack.tags[{}] = {}", place(self), self.value(Kind::Text, 1)),
                _ => format!("{held}.pack = {}", self.value(Kind::Pack, 1)),
            };
            self.line(&line);
            return;
        }
        let pack = self.changeable_name(Kind::Pack);
        let grid = self.changeable_name(Kind::Grid);
        let packs = self.changeable_name(Kind::Packs);
        // The way to a pack: a name that's one, or an item of a list of them.
        let pack_way = match (pack, packs) {
            (Some(pack), Some(packs)) if self.random.chance(1, 2) => {
                let _ = pack;
                Some(format!("{packs}[{}]", place(self)))
            }
            (Some(pack), _) => Some(pack),
            (None, Some(packs)) => Some(format!("{packs}[{}]", place(self))),
            (None, None) => None,
        };
        let line = match (pack_way, grid) {
            (Some(way), grid) if grid.is_none() || self.random.chance(2, 3) => match self.random.below(8) {
                0 => format!("{way}.items.add({})", self.value(Kind::Whole, 1)),
                1 => format!("{way}.items[{}] = {}", place(self), self.value(Kind::Whole, 1)),
                2 => format!("{way}.items.remove({})", place(self)),
                3 => format!("{way}.tags.add({})", self.value(Kind::Text, 1)),
                4 => format!("{way}.tags[{}] = {}", place(self), self.value(Kind::Text, 1)),
                5 => format!("{way}.stock[key: {}] = {}", self.value(Kind::Text, 0), self.value(Kind::Whole, 1)),
                6 => format!("{way}.stock.remove(key: {}) ?? ignore", self.value(Kind::Text, 0)),
                _ => format!("{way}.items.add({}, {})", self.value(Kind::Whole, 1), place(self)),
            },
            (_, Some(grid)) => match self.random.below(4) {
                0 => format!("{grid}[{}].add({})", place(self), self.value(Kind::Whole, 1)),
                1 => format!("{grid}[{}][{}] = {}", place(self), place(self), self.value(Kind::Whole, 1)),
                2 => format!("{grid}[{}].remove({})", place(self), place(self)),
                _ => format!("{grid}[{}] = {}", place(self), self.value(Kind::WholeList, 1)),
            },
            _ => return,
        };
        self.line(&line);
    }

    /// Ends a block: every name it made and never read is shown, so that none is refused as unused (D2).
    fn close_scope(&mut self) {
        let Some(scope) = self.scopes.pop() else { return };
        // trace: D135. A `mut` name is changed somewhere; here, at the latest.
        for name in scope.iter().filter(|name| name.mutable && !name.frozen) {
            self.line(&format!("{0} = {0}", name.text));
        }
        for name in scope.iter().filter(|name| !name.read) {
            match name.kind {
                Kind::Event => self.match_event(&name.text, false),
                Kind::Maybe => self.line(&format!("show_msg(\"{0}: {{{0}}}\")", name.text)),
                kind => {
                    let shown = self.show(kind, &name.text);
                    self.line(&shown);
                }
            }
        }
    }

    fn block(&mut self, statements: usize) {
        self.scopes.push(Vec::new());
        self.depth += 1;
        for _ in 0..statements {
            self.statement();
        }
        self.close_scope();
        self.depth -= 1;
    }

    fn match_event(&mut self, name: &str, with_bodies: bool) {
        self.line(&format!("match {name}"));
        self.depth += 1;
        for case in ["idle", "hurt[amount]", "healed[amount, by]"] {
            self.line(&format!("case {case}"));
            self.depth += 1;
            match case {
                "idle" => self.line("show_msg(\"idle\")"),
                "hurt[amount]" => self.line("show_msg(\"hurt {amount}\")"),
                _ => self.line("show_msg(\"{by} healed {amount}\")"),
            }
            self.depth -= 1;
            if with_bodies && self.budget > 0 {
                // The case's values are names its body can read (D37).
                let mut taken = Vec::new();
                if case != "idle" {
                    taken.push(Name {
                        text: "amount".to_owned(),
                        kind: Kind::Whole,
                        mutable: false,
                        read: true,
                        frozen: false,
                    });
                }
                if case.contains("by") {
                    taken.push(Name {
                        text: "by".to_owned(),
                        kind: Kind::Text,
                        mutable: false,
                        read: true,
                        frozen: false,
                    });
                }
                self.scopes.push(taken);
                self.block(1);
                self.scopes.pop();
            }
        }
        self.depth -= 1;
        self.end("match");
    }

    #[allow(clippy::too_many_lines)] // one case per kind of statement
    fn statement(&mut self) {
        if self.budget == 0 {
            return;
        }
        self.budget -= 1;
        let nested = self.depth < 4 && self.budget > 2;
        match self.random.below(22) {
            0..=4 => {
                let kind = self.any_kind();
                let name = self.fresh();
                let value = self.value(kind, 2);
                let mutable = self.random.chance(1, 2);
                let typed = self.random.chance(1, 4) || matches!(kind, Kind::Dict | Kind::Maybe);
                let marker = if mutable { "mut " } else { "" };
                if typed {
                    // trace: D107. Now and then a ranged number, which checks or pulls back every value landing in it.
                    let written = match kind {
                        Kind::Whole if self.seal() && self.random.chance(1, 3) => {
                            let edges = [
                                "[min.clamp: -50, max.clamp: 50]",
                                "[min: -1000000]",
                                "[max.clamp: 7]",
                                "[min: 0, max.clamp: 100]",
                            ];
                            format!("Whole{}", self.random.pick(&edges).copied().unwrap_or(""))
                        }
                        Kind::Float if self.seal() && self.random.chance(1, 3) => {
                            "Float[min.clamp: -2.5, max.clamp: 2.5]".to_owned()
                        }
                        _ => kind.written().to_owned(),
                    };
                    self.line(&format!("{marker}{name}: {written} = {value}"));
                } else {
                    self.line(&format!("{marker}{name} = {value}"));
                }
                self.declare(&name, kind, mutable);
            }
            5..=7 => {
                let kind = self.any_kind();
                if let Some(name) = self.changeable_name(kind) {
                    let value = self.value(kind, 2);
                    self.line(&format!("{name} = {value}"));
                }
            }
            8 | 9 => {
                let last = if self.seal() { Kind::Float } else { Kind::Whole };
                let kind = *self.random.pick(&[Kind::Whole, Kind::Text, Kind::Bool, last]).unwrap_or(&Kind::Whole);
                let value = self.value(kind, 2);
                let name = self.fresh();
                self.line(&format!("{name} = {value}"));
                let shown = self.show(kind, &name);
                self.line(&shown);
            }
            10 | 11 if nested => {
                let condition = self.value(Kind::Bool, 2);
                self.line(&format!("if {condition}"));
                self.block(2);
                if self.random.chance(1, 3) {
                    let condition = self.value(Kind::Bool, 1);
                    self.line(&format!("elif {condition}"));
                    self.block(1);
                }
                if self.random.chance(1, 2) {
                    self.line("else");
                    self.block(1);
                }
                self.end("if");
            }
            12 if nested => {
                let counter = self.fresh();
                let limit = self.random.below(6);
                self.line(&format!("mut {counter} = 0"));
                self.declare(&counter, Kind::Whole, true);
                self.line(&format!("while {counter} < {limit}"));
                // The counter can't be picked for changing inside the loop, so every loop ends.
                self.freeze(&counter, true);
                let was_in_loop = std::mem::replace(&mut self.in_loop, true);
                self.block(2);
                self.in_loop = was_in_loop;
                self.depth += 1;
                if self.random.chance(1, 2) {
                    self.line(&format!("{counter} = {counter} + 1"));
                } else {
                    // trace: D147
                    self.line(&format!("{counter} = _ + 1"));
                }
                self.depth -= 1;
                self.freeze(&counter, false);
                self.end("while");
            }
            13 if nested && self.seal() => {
                let item = self.fresh();
                let from = self.random.below(5);
                let range = match self.random.below(4) {
                    0 => format!("from {from} to {}", from + self.random.below(5)),
                    1 => format!("from {from} before {}", from + self.random.below(5)),
                    2 => {
                        format!("from {} to {from} step_by -{}", from + self.random.below(8), self.random.below(3) + 1)
                    }
                    _ => {
                        format!("from {from} to {} step_by {}", self.value(Kind::Whole, 0), self.value(Kind::Whole, 0))
                    }
                };
                // trace: D172. Counting is `for`; one loop in six never reads its counter, and says so (D61).
                if self.random.chance(1, 6) {
                    self.line(&format!("for unused {item} {range}"));
                    self.loop_body(&[]);
                } else {
                    self.line(&format!("for {item} {range}"));
                    self.loop_body(&[(item, Kind::Whole)]);
                }
            }
            // Going through a text, one grapheme at a time (D66).
            14 if nested && self.seal() && self.random.chance(1, 5) => {
                if let Some(text) = self.read_name(Kind::Text) {
                    let item = self.fresh();
                    self.freeze(&text, true);
                    if self.random.chance(1, 2) {
                        let position = self.fresh();
                        self.line(&format!("for_each {item} at pos {position} in {text}"));
                        self.loop_body(&[(item, Kind::Text), (position, Kind::Whole)]);
                    } else {
                        self.line(&format!("for_each {item} in {text}"));
                        self.loop_body(&[(item, Kind::Text)]);
                    }
                    self.freeze(&text, false);
                }
            }
            14 if nested && self.seal() => {
                let text_list = self.random.chance(1, 3);
                let list_kind = if text_list { Kind::TextList } else { Kind::WholeList };
                if let Some(list) = self.read_name(list_kind) {
                    let item = self.fresh();
                    let item_kind = if text_list { Kind::Text } else { Kind::Whole };
                    self.freeze(&list, true);
                    if self.random.chance(1, 2) {
                        let position = self.fresh();
                        let label = if self.random.chance(1, 2) { "pos" } else { "offset" };
                        self.line(&format!("for_each {item} at {label} {position} in {list}"));
                        self.loop_body(&[(item, item_kind), (position, Kind::Whole)]);
                    } else {
                        self.line(&format!("for_each {item} in {list}"));
                        self.loop_body(&[(item, item_kind)]);
                    }
                    self.freeze(&list, false);
                }
            }
            15 if self.seal() => {
                let text_list = self.random.chance(1, 3);
                let list_kind = if text_list { Kind::TextList } else { Kind::WholeList };
                if let Some(list) = self.changeable_name(list_kind) {
                    let item_kind = if text_list { Kind::Text } else { Kind::Whole };
                    let place = self.value(Kind::Whole, 0);
                    match self.random.below(5) {
                        0 => self.line(&format!("{list}.remove(pos: {place})")),
                        1 => {
                            let item = self.value(item_kind, 1);
                            self.line(&format!("{list}[pos: {place}] = {item}"));
                        }
                        2 => {
                            let item = self.value(item_kind, 1);
                            self.line(&format!("{list}.add({item}, pos: {place})"));
                        }
                        _ => {
                            let item = self.value(item_kind, 1);
                            self.line(&format!("{list}.add({item})"));
                        }
                    }
                }
            }
            16 if self.seal() => {
                if let Some(dict) = self.changeable_name(Kind::Dict) {
                    let key = self.value(Kind::Text, 0);
                    if self.random.chance(1, 3) {
                        self.line(&format!("{dict}.remove(key: {key}) ?? ignore"));
                    } else {
                        let value = self.value(Kind::Whole, 1);
                        self.line(&format!("{dict}[key: {key}] = {value}"));
                    }
                } else if let Some(dict) = self.read_name(Kind::Dict) {
                    let value = self.fresh();
                    let key = self.fresh();
                    self.freeze(&dict, true);
                    self.line(&format!("for_each {value} at key {key} in {dict}"));
                    self.loop_body(&[(value, Kind::Whole), (key, Kind::Text)]);
                    self.freeze(&dict, false);
                }
            }
            // Loops that read and change the fields of a list's records, through places that may be the same item.
            17 if self.seal() && nested && self.random.chance(1, 3) => self.spots_loops(),
            // A change to a part of a part, through fields and places (D62).
            17 if self.seal() && self.random.chance(1, 2) => self.nested_change(),
            17 if self.seal() => {
                if let Some(record) = self.changeable_name(Kind::Record) {
                    let value = self.value(Kind::Whole, 1);
                    if self.random.chance(1, 4) {
                        // trace: D165. A method that changes the mutable variable it's called on.
                        self.line(&format!("{record}.heal({value})"));
                    } else if self.random.chance(1, 3) {
                        // trace: D147
                        self.line(&format!("{record}.hp = _ * 2 - ({value})"));
                    } else {
                        self.line(&format!("{record}.hp = {value}"));
                    }
                } else if let Some(spot) = self.changeable_name(Kind::Spot) {
                    if self.random.chance(1, 2) {
                        let value = self.value(Kind::Whole, 2);
                        self.line(&format!("{spot}.x = {value}"));
                    } else {
                        let value = self.value(Kind::Float, 2);
                        self.line(&format!("{spot}.y = {value}"));
                    }
                } else if self.in_procedure.as_ref().is_some_and(|inside| inside.pauses) {
                    let seconds = self.value(Kind::Whole, 1);
                    self.line(&format!("wait(seconds: {seconds})"));
                }
            }
            18 if self.seal() && nested && !self.visible().any(|name| name.text == "amount") => {
                if let Some(event) = self.read_name(Kind::Event) {
                    self.freeze(&event, true);
                    self.match_event(&event, true);
                    self.freeze(&event, false);
                }
            }
            19 => {
                let candidates: Vec<ProcedureInfo> = self
                    .procedures
                    .iter()
                    .filter(|procedure| procedure.result.is_none() && !procedure.pauses)
                    .cloned()
                    .collect();
                if let Some(procedure) = self.random.pick(&candidates).cloned()
                    && self.in_procedure.as_ref().is_none_or(|inside| inside.name != procedure.name)
                {
                    let arguments: Vec<String> =
                        procedure.parameters.iter().map(|&parameter| self.value(parameter, 1)).collect();
                    let call = format!("{}({})", procedure.name, arguments.join(", "));
                    if procedure.fails && self.handlers && self.random.chance(1, 2) {
                        // trace: D98, D110, Unio D43. A handler on one line, or brought into a block.
                        if self.random.chance(1, 2) {
                            self.line(&format!("{call} ?? ::note"));
                        } else {
                            self.line(&format!("{call} otherwise"));
                            self.depth += 1;
                            self.line("include low_failed::note");
                            self.depth -= 1;
                            self.end("otherwise");
                        }
                    } else if procedure.fails {
                        self.line(&format!("{call} otherwise"));
                        self.depth += 1;
                        self.line("case too_low");
                        self.depth += 1;
                        self.line("show_msg(\"too low\")");
                        self.depth -= 2;
                        self.end("otherwise");
                    } else {
                        self.line(&call);
                    }
                }
            }
            // trace: D107, D115. A `HugeWhole` whose limits are past a `Whole`'s range.
            20 if self.seal() && self.random.chance(1, 3) => self.ranged_huge(),
            _ => {
                let kind = if self.seal() {
                    *self.random.pick(&[Kind::Whole, Kind::Float]).unwrap_or(&Kind::Whole)
                } else {
                    Kind::Whole
                };
                if let Some(name) = self.changeable_name(kind) {
                    let value = self.operand(kind, 1);
                    let operator = *self.random.pick(&["+", "-", "*"]).unwrap_or(&"+");
                    // trace: D147. The name itself, or `_` standing for it.
                    let current = if self.random.chance(1, 2) { "_".to_owned() } else { name.clone() };
                    self.line(&format!("{name} = {current} {operator} {value}"));
                }
            }
        }
    }

    /// A ranged `HugeWhole` with edges past a `Whole`'s range, made from text, changed with `_`, and shown: it's
    /// checked or pulled back as it lands (D107, D115, D147).
    fn ranged_huge(&mut self) {
        const EDGES: [&str; 3] = [
            "[max.clamp: 100000000000000000000]",
            "[min.clamp: -99999999999999999999, max.clamp: 99999999999999999999]",
            "[min: -100000000000000000000, max.clamp: 100000000000000000000]",
        ];
        const STARTS: [&str; 5] = ["7", "-3", "9223372036854775808", "123456789012345678901234567890", "x"];
        let name = self.fresh();
        let edges = self.random.pick(&EDGES).copied().unwrap_or("");
        let start = self.random.pick(&STARTS).copied().unwrap_or("0");
        let factor = self.random.pick(&["2", "-1000", "9223372036854775807", "0"]).copied().unwrap_or("2");
        self.line(&format!("mut {name}: HugeWhole{edges} = be.HugeWhole(\"{start}\") ?? 0"));
        self.line(&format!("{name} = _ * {factor}"));
        self.line(&format!("show_msg(\"{name}: {{{name}}}\")"));
    }

    fn freeze(&mut self, text: &str, frozen: bool) {
        for name in self.scopes.iter_mut().flatten() {
            if name.text == text {
                name.frozen = frozen;
            }
        }
    }

    /// A loop's body, with the names its line makes, each of which the body reads.
    fn loop_body(&mut self, made: &[(String, Kind)]) {
        self.scopes.push(Vec::new());
        self.depth += 1;
        for (name, kind) in made {
            self.declare(name, *kind, false);
        }
        let was_in_loop = std::mem::replace(&mut self.in_loop, true);
        for _ in 0..self.random.below(3) {
            self.statement();
        }
        self.in_loop = was_in_loop;
        self.close_scope();
        // Nothing follows an `if` that can leave (D68, Unio D9), so a `break` comes last in its loop's body.
        if self.random.chance(1, 6) {
            let condition = self.value(Kind::Bool, 1);
            self.line(&format!("if {condition}"));
            self.depth += 1;
            self.line("break");
            self.depth -= 1;
            self.end("if");
        }
        self.depth -= 1;
        self.end("for_each");
    }

    #[allow(clippy::too_many_lines)] // a procedure's first line, its body and its `return`, written in order
    fn procedure(&mut self) {
        let name = format!(
            "{}_{}",
            self.random.pick(&["work", "pick", "mix", "check"]).copied().unwrap_or("work"),
            self.procedures.len()
        );
        // One procedure in six takes nothing, and so has no brackets (D60).
        let parameter_count = if self.random.chance(1, 6) { 0 } else { self.random.below(3) + 1 };
        let scalar =
            [Kind::Whole, Kind::Whole, if self.seal() { Kind::Float } else { Kind::Whole }, Kind::Bool, Kind::Text];
        let parameters: Vec<Kind> = (0..parameter_count)
            .map(|_| {
                if self.random.chance(1, 5) {
                    self.any_kind()
                } else {
                    *self.random.pick(&scalar).unwrap_or(&Kind::Whole)
                }
            })
            .collect();
        let result = if self.random.chance(2, 3) {
            Some(
                *self
                    .random
                    .pick(&[Kind::Whole, if self.seal() { Kind::Float } else { Kind::Whole }, Kind::Text])
                    .unwrap_or(&Kind::Whole),
            )
        } else {
            None
        };
        let fails = self.seal() && self.random.chance(1, 3);
        // A procedure that pauses takes one `Whole`, which is what the fuzzer's host calls every procedure with.
        let pauses = self.seal() && self.random.chance(1, 4);
        let parameters = if pauses { vec![Kind::Whole] } else { parameters };
        // A procedure that fails on purpose does so when its first parameter says so, so it takes one.
        let fails = fails && !parameters.is_empty();
        let info = ProcedureInfo { name: name.clone(), parameters: parameters.clone(), result, fails, pauses };
        self.scopes.push(Vec::new());
        let mut header = format!("proc {name}");
        let mut names = Vec::new();
        for (index, kind) in parameters.iter().enumerate() {
            let parameter = self.fresh();
            header.push_str(if index > 0 { ", " } else { "[" });
            // trace: D88. Now and then a parameter shares the next one's type, written once for the group.
            let grouped = self.seal() && parameters.get(index + 1) == Some(kind) && self.random.chance(1, 3);
            let typed = if grouped { String::new() } else { format!(": {}", kind.written()) };
            // Now and then a parameter after the first is never read, and says so (D61); the body can't reach it.
            if index > 0 && self.seal() && self.random.chance(1, 6) {
                let _ = write!(header, "unused {parameter}{typed}");
                continue;
            }
            let _ = write!(header, "{parameter}{typed}");
            names.push((parameter, *kind));
        }
        if !parameters.is_empty() {
            header.push(']');
        }
        // A group's type comes after the group, with a space on each side of its `:` (D88).
        header = respace_groups(&header);
        // trace: D81, D84. The header writes what the procedure gives back and how it can fail.
        if let Some(result) = result {
            let _ = write!(header, " -> {}", result.written());
        }
        if fails {
            header.push_str(" ! [too_low]");
        }
        // Now and then a procedure says what it's for (D63).
        if self.random.chance(1, 6) {
            self.plain_line("/// Does some work.");
        }
        self.plain_line(&header);
        // The top level's names stay out of reach: a procedure may run before they're made (not designed yet).
        let outer = std::mem::replace(&mut self.scopes, vec![Vec::new()]);
        for (parameter, kind) in &names {
            self.declare(parameter, *kind, false);
        }
        self.in_procedure = Some(info.clone());
        self.depth += 1;
        if fails {
            let Some((guard, _)) = names.first() else { return };
            let guard = guard.clone();
            let guard_kind = names.first().map_or(Kind::Whole, |(_, kind)| *kind);
            let condition = match guard_kind {
                Kind::Whole => format!("{guard} < 3"),
                Kind::Float => format!("{guard} < 1.0"),
                Kind::Bool => guard.clone(),
                _ => self.value(Kind::Bool, 1),
            };
            if matches!(guard_kind, Kind::Whole | Kind::Float | Kind::Bool) {
                self.mark_read(&guard);
            }
            self.line(&format!("if {condition}"));
            self.depth += 1;
            self.line("fail too_low");
            self.depth -= 1;
            // Nothing follows an `if` that can leave (D68, Unio D9): the rest of the body is the way that doesn't.
            self.line("else");
            self.depth += 1;
        }
        for _ in 0..self.random.below(4) {
            self.statement();
        }
        // Every parameter not marked `unused` is read, since one that's never read is refused (D61).
        let unread: Vec<(String, Kind)> = self
            .scopes
            .first()
            .map(|scope| scope.iter().filter(|name| !name.read).map(|name| (name.text.clone(), name.kind)).collect())
            .unwrap_or_default();
        for (parameter, kind) in unread {
            if !names.iter().any(|(name, _)| *name == parameter) {
                continue;
            }
            self.mark_read(&parameter);
            match kind {
                Kind::Event => self.match_event(&parameter, false),
                Kind::Maybe => self.line(&format!("show_msg(\"m: {{{parameter}}}\")")),
                kind => {
                    let shown = self.show(kind, &parameter);
                    self.line(&shown);
                }
            }
        }
        let returned = result.map(|kind| self.value(kind, 2));
        self.depth -= 1;
        // Names the body made are shown before the `return`, inside the body.
        self.depth += 1;
        self.close_scope();
        if let Some(returned) = returned {
            self.line(&format!("return {returned}"));
        }
        if fails {
            self.depth -= 1;
            self.end("if");
        }
        self.depth -= 1;
        self.end(&format!("proc {name}"));
        self.scopes = outer;
        self.scopes.pop();
        self.in_procedure = None;
        self.procedures.push(info);
    }

    fn mark_read(&mut self, text: &str) {
        for name in self.scopes.iter_mut().flatten() {
            if name.text == text {
                name.read = true;
            }
        }
    }

    /// A whole script.
    #[must_use]
    pub fn script(mut self) -> String {
        if self.seal() {
            self.out.push_str("requirement Powered\n    proc power[of: ?Self] -> Whole\nend\n\nrecord Hero\n    meets [Powered]\n    name: Text\n    hp: Whole\n\n    proc power[of_hero: Hero] -> Whole\n        return of_hero.hp div.floor 2 + 1\n    end\n\n    proc renamed[of_hero: Hero, new_name: Text] -> Hero\n        return Hero[name: new_name, hp: of_hero.hp]\n    end\n\n    proc heal[mut of_hero: Hero, by: Whole]\n        of_hero.hp = _ + by\n    end\nend\n\nrecord Spot\n    x: Whole\n    y: Float\n\n    proc size[of_spot: Spot] -> Whole\n        return of_spot.x\n    end\nend\n\nrecord Pack\n    items: List<Whole>\n    tags: List<Text>\n    stock: Dict<key: Text, value: Whole>\nend\n\nrecord Crate\n    pack: Pack\n    spot: Spot\nend\n\none_of Tree\n    leaf\n    node[left: Tree, right: Tree]\n\n    proc size[of_tree: Tree] -> Whole\n        match of_tree\n            case leaf\n                return 1\n            case node[left, right]\n                return 1 + left.size() + right.size()\n        end\n    end\nend\n\nproc tree_size[of: Tree] -> Whole\n    match of\n        case leaf\n            return 1\n        case node[left, right]\n            return 1 + tree_size(left) + tree_size(right)\n    end\nend\n\none_of Event\n    idle\n    hurt[amount: Whole]\n    healed[amount: Whole, by: Text]\nend\n\n");
            // trace: D161. Generic procedures, one calling another, filled in with whatever their calls give.
            self.out.push_str("proc generic_pick<?Item>[items: List<?Item>, fallback: ?Item] -> ?Item\n    for_each item in items\n        return item\n    end\n    return fallback\nend\n\nproc generic_pair<?Item>[value: ?Item] -> List<?Item>\n    return [value, value]\nend\n\nproc generic_first<?Item>[items: List<?Item>] -> Maybe<?Item>\n    for_each item in items\n        return Maybe.some[value: item]\n    end\n    return Maybe.none\nend\n\nproc generic_pairs<?Item>[value: ?Item] -> List<List<?Item>>\n    return generic_pair(generic_pair(value))\nend\n\nproc generic_bigger<?Item: Number>[a: ?Item, b: ?Item] -> ?Item\n    if a > b\n        return a\n    else\n        return b\n    end\nend\n\nproc generic_power<?Thing: Powered>[thing: ?Thing] -> Whole\n    return thing.power()\nend\n\n");
        }
        // trace: D123. Now and then a unit group of the script's own.
        if self.seal() && self.random.chance(1, 3) {
            self.out.push_str("group.unit coins\n    ::bits base\n    ::stacks = 12 bits\nend\n\n");
            self.coins = true;
        }
        for _ in 0..self.random.below(4) {
            self.budget = 12;
            self.procedure();
            self.out.push('\n');
        }
        // A handler group for the failing procedures, which their callers use from here on (D110).
        let failing: Vec<String> = self
            .procedures
            .iter()
            .filter(|procedure| procedure.fails)
            .map(|procedure| procedure.name.clone())
            .collect();
        if self.seal() && !failing.is_empty() {
            let _ = write!(
                self.out,
                "group.handler low_failed\n    handles [{}]\n    ::zero\n        case too_low\n            instead 0\n    end\n    ::with[value: Whole]\n        case too_low\n            instead value\n    end \\\\ ::with\n    ::note\n        case too_low\n            show_msg(\"too low\")\n    end\nend\n\n",
                failing.join(", ")
            );
            self.handlers = true;
        }
        self.budget = 10 + self.random.below(30);
        while self.budget > 0 {
            self.statement();
        }
        self.close_scope();
        self.out
    }
}

/// The words and symbols of Seal, for scripts made of nothing but them.
pub const VOCABULARY: [&str; 82] = [
    "mut",
    "unused",
    "if",
    "elif",
    "else",
    "end",
    "while",
    "break",
    "for_each",
    "for",
    "from",
    "step_by",
    "in",
    "at",
    "proc",
    "return",
    "fail",
    "record",
    "one_of",
    "match",
    "case",
    "otherwise",
    "instead",
    "ignore",
    "crash",
    "labels",
    "defaults",
    "as",
    "fam",
    "true",
    "false",
    "and",
    "or",
    "not",
    "at_least",
    "at_most",
    "be",
    "div",
    "rem",
    "fmt",
    "range",
    "sqrt",
    "pos",
    "offset",
    "key",
    "all",
    "len",
    "show_msg",
    "ask",
    "rand",
    "wait",
    "Whole",
    "Float",
    "Bool",
    "Text",
    "List",
    "Maybe",
    "x",
    "y",
    "hp",
    "0",
    "1",
    "42",
    "2.5",
    "~0.1",
    "\"hi\"",
    "\"{x}\"",
    "=",
    "eq",
    "<",
    ">",
    "+",
    "-",
    "*",
    "/",
    "&",
    "|>",
    "??",
    "\\\\",
    "\"{newln}\"",
    "\"{x, places: 2}\"",
    "labels",
];

/// The symbols that sit tight against their neighbours.
pub const TIGHT: [&str; 10] = ["(", ")", "[", "]", ",", ":", ".", "?", "<", ">"];

/// A header written with grouped names, `[a, b: Whole]`, put right as D88 has it: `[a, b : Whole]`, a group's `:`
/// spaced, and a single name's not.
fn respace_groups(header: &str) -> String {
    let Some((name, rest)) = header.split_once('[') else { return header.to_owned() };
    let Some(inside) = rest.strip_suffix(']') else { return header.to_owned() };
    let mut written = Vec::new();
    let mut group: Vec<&str> = Vec::new();
    for part in inside.split(", ") {
        match part.split_once(": ") {
            Some((last, kind)) if group.is_empty() => written.push(format!("{last}: {kind}")),
            Some((last, kind)) => {
                group.push(last);
                written.push(format!("{} : {kind}", group.join(", ")));
                group.clear();
            }
            None => group.push(part),
        }
    }
    format!("{name}[{}]", written.join(", "))
}

/// A script that's only random words and symbols: almost always refused, and never allowed to do worse.
#[must_use]
pub fn soup(random: &mut Random) -> String {
    let mut out = String::new();
    let lines = random.below(12) + 1;
    for _ in 0..lines {
        for _ in 0..random.below(4) {
            out.push_str("    ");
        }
        let words = random.below(8) + 1;
        for index in 0..words {
            if random.chance(1, 4) {
                out.push_str(random.pick(&TIGHT).copied().unwrap_or("("));
            } else {
                if index > 0 && random.chance(5, 6) {
                    out.push(' ');
                }
                out.push_str(random.pick(&VOCABULARY).copied().unwrap_or("x"));
            }
        }
        out.push('\n');
    }
    out
}

/// A copy of a script with something changed at random: a line dropped, doubled, swapped or re-indented, a word
/// replaced, or a stray character put in.
#[must_use]
pub fn mutate(script: &str, random: &mut Random) -> String {
    let mut lines: Vec<String> = script.lines().map(str::to_owned).collect();
    if lines.is_empty() {
        return soup(random);
    }
    for _ in 0..=random.below(3) {
        let at = random.below(lines.len());
        match random.below(9) {
            0 => {
                lines.remove(at);
                if lines.is_empty() {
                    lines.push(String::new());
                }
            }
            1 => {
                let copy = lines.get(at).cloned().unwrap_or_default();
                lines.insert(at, copy);
            }
            2 => {
                let other = random.below(lines.len());
                lines.swap(at, other);
            }
            3 => {
                if let Some(line) = lines.get_mut(at) {
                    *line = format!("    {line}");
                }
            }
            4 => {
                if let Some(line) = lines.get_mut(at) {
                    *line = line.trim_start().to_owned();
                }
            }
            5 | 6 => {
                if let Some(line) = lines.get_mut(at) {
                    let mut words: Vec<String> = line.split(' ').map(str::to_owned).collect();
                    let which = random.below(words.len());
                    let replacement = if random.chance(1, 3) { random.pick(&TIGHT) } else { random.pick(&VOCABULARY) }
                        .copied()
                        .unwrap_or("x");
                    if let Some(word) = words.get_mut(which)
                        && !word.is_empty()
                    {
                        replacement.clone_into(word);
                    }
                    *line = words.join(" ");
                }
            }
            7 => {
                if let Some(line) = lines.get_mut(at) {
                    let characters: Vec<char> = line.chars().collect();
                    let cut = random.below(characters.len() + 1);
                    *line = characters.iter().take(cut).collect();
                }
            }
            _ => {
                if let Some(line) = lines.get_mut(at) {
                    let stray = *random
                        .pick(&['\t', '"', '{', '}', '\\', ';', '$', '\u{e9}', '\r', '~', '!', '#'])
                        .unwrap_or(&';');
                    let characters: Vec<char> = line.chars().collect();
                    let place = random.below(characters.len() + 1);
                    let mut changed: String = characters.iter().take(place).collect();
                    changed.push(stray);
                    changed.extend(characters.iter().skip(place));
                    *line = changed;
                }
            }
        }
    }
    let mut out = lines.join("\n");
    out.push('\n');
    out
}
