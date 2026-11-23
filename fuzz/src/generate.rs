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
        }
    }
}

#[derive(Clone, Debug)]
struct Name {
    text: String,
    kind: Kind,
    mutable: bool,
    read: bool,
    /// Being gone through by a `for each`, or matched on: it can't change for now.
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
    /// procedures, maths and converting. Lists and `for each` are shared too, but Ralph doesn't run them yet.
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
        }
    }

    fn seal(&self) -> bool {
        self.dialect == Dialect::Seal
    }

    /// Writes a line, now and then with a comment on the line before it or after its statement (D63).
    fn line(&mut self, text: &str) {
        if self.random.chance(1, 30) {
            self.plain_line("// a note");
        }
        if self.random.chance(1, 30) {
            self.plain_line(&format!("{text} // a note"));
        } else {
            self.plain_line(text);
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
        format!("\"{}\"", self.random.pick(&TEXTS).copied().unwrap_or("x"))
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
        match kind {
            Kind::Whole => {
                if simple {
                    return self.whole_literal();
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
                    _ => self.call_value(Kind::Float, level).unwrap_or_else(|| self.float_literal()),
                }
            }
            Kind::Bool => {
                if simple {
                    return if self.random.chance(1, 2) { "true".to_owned() } else { "false".to_owned() };
                }
                match self.random.below(8) {
                    // Two texts compared exactly (D64); Unio leaves comparing text open, so only Seal's dialect has it.
                    7 if self.seal() => {
                        let operator = if self.random.chance(1, 2) { "==" } else { "!=" };
                        format!(
                            "{} {operator} {}",
                            self.operand(Kind::Text, level - 1),
                            self.operand(Kind::Text, level - 1)
                        )
                    }
                    0..=2 => {
                        let operator =
                            *self.random.pick(&["==", "!=", "<", ">", "at_least", "at_most"]).unwrap_or(&"<");
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
                    0 | 1 => {
                        let right_kind =
                            *self.random.pick(&[Kind::Text, Kind::Whole, Kind::Bool]).unwrap_or(&Kind::Text);
                        format!(
                            "{} & {}",
                            self.join_operand(Kind::Text, level - 1),
                            self.join_operand(right_kind, level - 1)
                        )
                    }
                    2 => {
                        let from = if self.seal() {
                            *self.random.pick(&[Kind::Whole, Kind::Float, Kind::Bool]).unwrap_or(&Kind::Whole)
                        } else {
                            *self.random.pick(&[Kind::Whole, Kind::Bool]).unwrap_or(&Kind::Whole)
                        };
                        format!("be.Text({})", self.argument(from, level))
                    }
                    3 => self.fmt_text(),
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
            Kind::WholeList => {
                let count = self.random.below(4) + 1;
                let items: Vec<String> = (0..count).map(|_| self.value(Kind::Whole, level.saturating_sub(1))).collect();
                format!("[{}]", items.join(", "))
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

    /// A value as a call's argument: never itself a call with a call inside, since three calls in a chain are a
    /// slide (Unio D128).
    fn argument(&mut self, kind: Kind, level: usize) -> String {
        self.value(kind, level.saturating_sub(1).min(1))
    }

    /// A value as one side of maths or a comparison: anything looser is put in brackets.
    fn operand(&mut self, kind: Kind, level: usize) -> String {
        let value = self.value(kind, level);
        if value.contains(' ') && !value.starts_with('(') && !value.ends_with(')') {
            format!("({value})")
        } else {
            value
        }
    }

    /// A value as one side of `&`: maths beside `&` needs brackets (Unio D72).
    fn join_operand(&mut self, kind: Kind, level: usize) -> String {
        let value = self.value(kind, level);
        let plain = value.chars().all(|character| character.is_ascii_alphanumeric() || character == '_')
            || (value.starts_with('"') && value.ends_with('"') && value.matches('"').count() == 2);
        if plain { value } else { format!("({value})") }
    }

    /// `fmt("…")` with names that exist in its braces (D12, D28).
    fn fmt_text(&mut self) -> String {
        let mut text = String::from("fmt(\"");
        for _ in 0..=self.random.below(3) {
            text.push_str(self.random.pick(&["value ", "is ", "", "got: ", "= "]).copied().unwrap_or(""));
            let kind = *self
                .random
                .pick(&[Kind::Whole, Kind::Text, Kind::Bool, Kind::Float, Kind::WholeList, Kind::Record])
                .unwrap_or(&Kind::Whole);
            if let Some(name) = self.read_name(kind) {
                match kind {
                    Kind::WholeList => {
                        let _ = write!(text, "{{{name}.len}} ");
                    }
                    Kind::Record => {
                        let _ = write!(text, "{{{name}.hp}} ");
                    }
                    _ => {
                        let _ = write!(text, "{{{name}}} ");
                    }
                }
            }
        }
        text.push_str("\")");
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
        } else {
            format!("({call} ?? {fallback})")
        })
    }

    // ---- statements ----

    fn show(&mut self, kind: Kind, name: &str) -> String {
        match kind {
            Kind::Whole | Kind::Bool if self.seal() && self.random.chance(1, 2) => {
                format!("show_msg(\"{name}: \" & {name})")
            }
            Kind::Whole | Kind::Bool | Kind::Text | Kind::Float => format!("show_msg(fmt(\"{name}: {{{name}}}\"))"),
            Kind::WholeList | Kind::TextList | Kind::Dict => format!("show_msg(fmt(\"{name} has {{{name}.len}}\"))"),
            Kind::Record => format!("show_msg(fmt(\"{name}: {{{name}.name}} {{{name}.hp}}\"))"),
            Kind::Spot => format!("show_msg(fmt(\"{name}: {{{name}.x}} {{{name}.y}}\"))"),
            Kind::Event | Kind::Maybe => String::new(),
        }
    }

    /// Ends a block: every name it made and never read is shown, so that none is refused as unused (D2).
    fn close_scope(&mut self) {
        let Some(scope) = self.scopes.pop() else { return };
        for name in scope.iter().filter(|name| !name.read) {
            match name.kind {
                Kind::Event => self.match_event(&name.text, false),
                Kind::Maybe => {
                    let fallback = self.whole_literal();
                    self.line(&format!("show_msg(\"{0}: \" & ({0} ?? {fallback}))", name.text));
                }
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
                "hurt[amount]" => self.line("show_msg(fmt(\"hurt {amount}\"))"),
                _ => self.line("show_msg(fmt(\"{by} healed {amount}\"))"),
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
        self.line("end");
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
                    self.line(&format!("{marker}{name}: {} = {value}", kind.written()));
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
                self.line("end");
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
                self.line(&format!("{counter} = {counter} + 1"));
                self.depth -= 1;
                self.freeze(&counter, false);
                self.line("end");
            }
            13 if nested && self.seal() => {
                let item = self.fresh();
                let from = self.random.below(5);
                let range = match self.random.below(4) {
                    0 => format!("range[from: {from}, to: {}]", from + self.random.below(5)),
                    1 => format!("range[from: {from}, before: {}]", from + self.random.below(5)),
                    2 => format!(
                        "range[from: {}, to: {from}, step_by: -{}]",
                        from + self.random.below(8),
                        self.random.below(3) + 1
                    ),
                    _ => format!(
                        "range[from: {from}, to: {}, step_by: {}]",
                        self.value(Kind::Whole, 0),
                        self.value(Kind::Whole, 0)
                    ),
                };
                // One loop in six never reads its counter, and says so (D61).
                if self.random.chance(1, 6) {
                    self.line(&format!("for each unused {item} in {range}"));
                    self.loop_body(&[]);
                } else {
                    self.line(&format!("for each {item} in {range}"));
                    self.loop_body(&[(item, Kind::Whole)]);
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
                        self.line(&format!("for each {item} at {label} {position} in {list}"));
                        self.loop_body(&[(item, item_kind), (position, Kind::Whole)]);
                    } else {
                        self.line(&format!("for each {item} in {list}"));
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
                    self.line(&format!("for each {value} at key {key} in {dict}"));
                    self.loop_body(&[(value, Kind::Whole), (key, Kind::Text)]);
                    self.freeze(&dict, false);
                }
            }
            17 if self.seal() => {
                if let Some(record) = self.changeable_name(Kind::Record) {
                    let value = self.value(Kind::Whole, 1);
                    self.line(&format!("{record}.hp = {value}"));
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
                    if procedure.fails {
                        self.line(&format!("{call} otherwise"));
                        self.depth += 1;
                        self.line("case too_low");
                        self.depth += 1;
                        self.line("show_msg(\"too low\")");
                        self.depth -= 2;
                        self.line("end");
                    } else {
                        self.line(&call);
                    }
                }
            }
            // Unio refuses anything after an `if` that leaves (Unio D9), so a `break` mid-body is Seal's alone.
            20 if self.in_loop && nested && self.seal() => {
                let condition = self.value(Kind::Bool, 1);
                self.line(&format!("if {condition}"));
                self.depth += 1;
                self.line("break");
                self.depth -= 1;
                self.line("end");
            }
            _ => {
                let kind = if self.seal() {
                    *self.random.pick(&[Kind::Whole, Kind::Float]).unwrap_or(&Kind::Whole)
                } else {
                    Kind::Whole
                };
                if let Some(name) = self.changeable_name(kind) {
                    let value = self.operand(kind, 1);
                    let operator = *self.random.pick(&["+", "-", "*"]).unwrap_or(&"+");
                    self.line(&format!("{name} = {name} {operator} {value}"));
                }
            }
        }
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
        self.depth -= 1;
        self.line("end");
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
        let info = ProcedureInfo { name: name.clone(), parameters: parameters.clone(), result, fails, pauses };
        self.scopes.push(Vec::new());
        let mut header = format!("proc {name}");
        let mut names = Vec::new();
        for (index, kind) in parameters.iter().enumerate() {
            let parameter = self.fresh();
            header.push_str(if index > 0 { ", " } else { "[" });
            // Now and then a parameter after the first is never read, and says so (D61); the body can't reach it.
            if index > 0 && self.seal() && self.random.chance(1, 6) {
                let _ = write!(header, "unused {parameter}: {}", kind.written());
                continue;
            }
            let _ = write!(header, "{parameter}: {}", kind.written());
            names.push((parameter, *kind));
        }
        if !parameters.is_empty() {
            header.push(']');
        }
        if let Some(result) = result
            && (!self.seal() || self.random.chance(1, 2))
        {
            let _ = write!(header, " -> {}", result.written());
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
            self.line("end");
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
                Kind::Maybe => self.line(&format!("show_msg(\"m: \" & ({parameter} ?? 0))")),
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
        self.depth -= 1;
        self.line("end");
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
            self.out.push_str("record Hero\n    name: Text\n    hp: Whole\nend\n\nrecord Spot\n    x: Whole\n    y: Float\nend\n\none_of Event\n    idle\n    hurt[amount: Whole]\n    healed[amount: Whole, by: Text]\nend\n\n");
        }
        for _ in 0..self.random.below(4) {
            self.budget = 12;
            self.procedure();
            self.out.push('\n');
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
pub const VOCABULARY: [&str; 76] = [
    "mut",
    "unused",
    "if",
    "elif",
    "else",
    "end",
    "while",
    "break",
    "for",
    "each",
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
    "==",
    "<",
    ">",
    "+",
    "-",
    "*",
    "/",
    "&",
    "|>",
    "??",
];

/// The symbols that sit tight against their neighbours.
pub const TIGHT: [&str; 10] = ["(", ")", "[", "]", ",", ":", ".", "?", "<", ">"];

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
