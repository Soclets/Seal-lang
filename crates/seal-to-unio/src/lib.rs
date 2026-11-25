//! The Unio translator: turns the parts Seal shares with Unio into a Unio program, for checking against Ralph (D19).
//!
//! It writes Unio's current spellings (Unio D190 to D203): `let` or `var` with every type written, `;`, each block's
//! own `end`, `eq` and `not_eq`, `else if`, and text filled in with braces holding only names, and it wraps the top
//! level in `proc.main`. The test host's doors `show_msg` and `ask` become Unio's built-ins. It's built on the
//! library's own checking, so both agree on what a script means; anything Unio doesn't have, such as records, a
//! `Dict`, `match`, `fail` or a pause, is reported as not shared.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::fmt::Write;

use seal::check::tree::{
    Block, Conversion, Expression, ExpressionKind, HandleFallback, Procedure, Program, Rounding, Statement,
    StatementKind, VariableId,
};
use seal::check::types::{Type, TypeId};
use seal::number::float_to_text;
use seal::syntax::tree::{BinaryOperator, PositionKind};
use seal::{DoorSignature, Message};

/// Why a script has no Unio twin.
#[derive(Clone, Debug, PartialEq)]
pub enum Untranslatable {
    /// The script doesn't pass Seal's own checking.
    Refused(Vec<Message>),
    /// The script uses something Seal doesn't share with Unio: what it is.
    NotShared(String),
}

struct Translator<'p> {
    program: &'p Program,
    doors: &'p [DoorSignature],
    out: String,
    depth: usize,
}

type Translated<T> = Result<T, Untranslatable>;

/// A call's name and its arguments, each with its label when it has one.
type CallParts<'e> = (String, Vec<(Option<String>, &'e Expression)>);

fn not_shared<T>(what: &str) -> Translated<T> {
    Err(Untranslatable::NotShared(what.to_owned()))
}

/// Translates a Seal script into the same Unio program.
///
/// # Errors
///
/// Gives back why not, when the script is refused or uses something Unio doesn't have.
pub fn translate(source: &str, doors: &[DoorSignature]) -> Translated<String> {
    let tree = seal::syntax::parser::parse(source).map_err(|message| Untranslatable::Refused(vec![message]))?;
    let program = seal::check::check(&tree, doors, &[], &[]).map_err(Untranslatable::Refused)?;
    let mut translator = Translator { program: &program, doors, out: String::new(), depth: 0 };
    if program.variables.iter().any(|variable| variable.captured) {
        return not_shared("a top-level name used by a procedure");
    }
    for procedure in &program.procedures {
        translator.procedure(procedure)?;
        translator.out.push('\n');
    }
    translator.line("proc.main");
    translator.depth += 1;
    translator.block(&program.top_level)?;
    translator.depth -= 1;
    translator.line("end proc.main");
    Ok(translator.out)
}

impl Translator<'_> {
    fn line(&mut self, text: &str) {
        for _ in 0..self.depth {
            self.out.push_str("    ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn written_type(&self, type_id: TypeId) -> Translated<String> {
        Ok(match self.program.types.get(type_id) {
            Type::Whole => "Whole".to_owned(),
            Type::Float => "Float".to_owned(),
            Type::Bool => "Bool".to_owned(),
            Type::Text => "Text".to_owned(),
            Type::List { item, max } => {
                let item = self.written_type(*item)?;
                match max {
                    Some(max) => format!("List<{item}>[len.max: {max}]"),
                    None => format!("List<{item}>"),
                }
            }
            Type::Array { item, length } => format!("Array<{}>[len: {length}]", self.written_type(*item)?),
            Type::Dict { .. } => return not_shared("a `Dict`"),
            Type::Record(_) => return not_shared("a record"),
            Type::OneOf(_) => return not_shared("a `one_of`"),
            Type::Maybe(_) => return not_shared("a `Maybe`"),
            Type::Ranged { .. } => return not_shared("a ranged number"),
            Type::Measured { .. } => return not_shared("a number with a unit"),
            Type::HugeWhole | Type::Decimal | Type::HugeDecimal => return not_shared("the new number types"),
            Type::Nothing | Type::Error | Type::Variable(_) => return not_shared("a value with no type to write"),
        })
    }

    /// A name, as Unio writes it: a plain word with no mark (Unio D192).
    fn name(&self, variable: VariableId) -> String {
        self.program.variable(variable).map_or("unknown", |variable| variable.name.as_str()).to_owned()
    }

    fn procedure(&mut self, procedure: &Procedure) -> Translated<()> {
        if procedure.name.contains('.') {
            return not_shared("a procedure with a narrowing");
        }
        if procedure.can_fail {
            return not_shared("a procedure that fails on purpose");
        }
        // A procedure that takes nothing has no brackets in Unio either (Unio D4, Seal's D60).
        let mut header = format!("proc {}", procedure.name);
        for (position, &parameter) in procedure.parameters.iter().enumerate() {
            let type_id = self.program.variable(parameter).map_or(procedure.result, |variable| variable.type_id);
            header.push_str(if position > 0 { ", " } else { "[" });
            let _ = write!(header, "{}: {}", self.name(parameter), self.written_type(type_id)?);
        }
        if !procedure.parameters.is_empty() {
            header.push(']');
        }
        // trace: Unio D190, D199. `-> Type` gives the result, and an opening line ends with nothing.
        if !matches!(self.program.types.get(procedure.result), Type::Nothing) {
            let _ = write!(header, " -> {}", self.written_type(procedure.result)?);
        }
        self.line(&header);
        self.depth += 1;
        let labels: Vec<String> = procedure
            .parameters
            .iter()
            .zip(&procedure.labels)
            .filter_map(|(&parameter, label)| {
                label.as_ref().map(|label| format!("{} as {label}", self.name(parameter)))
            })
            .collect();
        // trace: Unio D195. The `labels` line puts its pairs in `[ ]`.
        if !labels.is_empty() {
            self.line(&format!("labels [{}];", labels.join(", ")));
        }
        self.block(&procedure.body)?;
        self.depth -= 1;
        self.line(&format!("end proc {}", procedure.name));
        Ok(())
    }

    fn block(&mut self, block: &Block) -> Translated<()> {
        for statement in block {
            self.statement(statement)?;
        }
        Ok(())
    }

    fn body(&mut self, block: &Block) -> Translated<()> {
        self.depth += 1;
        self.block(block)?;
        self.depth -= 1;
        Ok(())
    }

    fn statement(&mut self, statement: &Statement) -> Translated<()> {
        match &statement.kind {
            StatementKind::Inside { .. } => return not_shared("a change to a part of a part"),
            StatementKind::Make { variable, value } => {
                let Some(made) = self.program.variable(*variable) else { return not_shared("a name that isn't there") };
                let word = if made.mutable { "var" } else { "let" };
                let text = format!(
                    "{word} {}: {} = {};",
                    self.name(*variable),
                    self.written_type(made.type_id)?,
                    self.value(value)?
                );
                self.line(&text);
            }
            StatementKind::Set { variable, value } => {
                let text = format!("{} = {};", self.name(*variable), self.value(value)?);
                self.line(&text);
            }
            StatementKind::SetItem { variable, index, zero_based, value } => {
                let label = if *zero_based { "offset" } else { "pos" };
                let text =
                    format!("{}[{label}: {}] = {};", self.name(*variable), self.value(index)?, self.value(value)?);
                self.line(&text);
            }
            StatementKind::Evaluate(expression) => {
                let text = format!("{};", self.value(expression)?);
                self.line(&text);
            }
            StatementKind::If { branches, otherwise } => {
                for (position, (condition, body)) in branches.iter().enumerate() {
                    let word = if position == 0 { "if" } else { "else if" };
                    let text = format!("{word} {}", self.value(condition)?);
                    self.line(&text);
                    self.body(body)?;
                }
                if let Some(body) = otherwise {
                    self.line("else");
                    self.body(body)?;
                }
                self.line("end if");
            }
            StatementKind::While { condition, body } => {
                let text = format!("while {}", self.value(condition)?);
                self.line(&text);
                self.body(body)?;
                self.line("end while");
            }
            StatementKind::ForRange { item, position, from, end, inclusive, step, body } => {
                if position.is_some() {
                    return not_shared("a position while going through a range");
                }
                let mut range = format!(
                    "range[from: {}, {}: {}",
                    self.value(from)?,
                    if *inclusive { "to" } else { "before" },
                    self.value(end)?
                );
                if let Some(step) = step {
                    let _ = write!(range, ", step_by: {}", self.value(step)?);
                }
                let text = format!("for each {} in {range}]", self.name(*item));
                self.line(&text);
                self.body(body)?;
                self.line("end for");
            }
            StatementKind::ForEach { item, position, source, body } => {
                let at = match position {
                    Some((variable, PositionKind::Pos)) => format!(" at pos {}", self.name(*variable)),
                    Some((variable, PositionKind::Offset)) => format!(" at offset {}", self.name(*variable)),
                    Some((_, PositionKind::Key)) => return not_shared("going through a `Dict`"),
                    None => String::new(),
                };
                if matches!(self.program.types.get(source.type_id), Type::Text) {
                    return not_shared("going through a text");
                }
                let text = format!("for each {}{at} in {}", self.name(*item), self.value(source)?);
                self.line(&text);
                self.body(body)?;
                self.line("end for");
            }
            StatementKind::Break => self.line("break;"),
            StatementKind::Return(None) => self.line("return;"),
            StatementKind::Return(Some(value)) => {
                let text = format!("return {};", self.value(value)?);
                self.line(&text);
            }
            StatementKind::SetField { .. } => return not_shared("a record"),
            StatementKind::SetKey { .. } => return not_shared("a `Dict`"),
            StatementKind::Match { .. } => return not_shared("`match`"),
            StatementKind::Fail(_) => return not_shared("`fail`"),
            StatementKind::Use(_) => return not_shared("`otherwise`"),
        }
        Ok(())
    }

    /// A call's name and its arguments, when an expression is a call of some kind.
    fn as_call<'e>(&self, expression: &'e Expression) -> Translated<Option<CallParts<'e>>> {
        Ok(match &expression.kind {
            ExpressionKind::Call { procedure, arguments } => {
                let Some(found) = self.program.procedure(*procedure) else {
                    return not_shared("a procedure that isn't there");
                };
                // A procedure whose parameters have no labels has an empty list of them.
                let labelled = (0..)
                    .zip(arguments)
                    .map(|(position, argument)| (found.labels.get(position).cloned().flatten(), argument))
                    .collect();
                Some((found.name.clone(), labelled))
            }
            ExpressionKind::DoorCall { door, arguments } => {
                let name = self.doors.get(*door as usize).map_or("", |door| door.name.as_str());
                if !matches!(name, "show_msg" | "show_msg.same_line" | "ask") {
                    return not_shared("a door Unio has no built-in for");
                }
                Some((name.to_owned(), arguments.iter().map(|argument| (None, argument)).collect()))
            }
            ExpressionKind::Convert { conversion: Conversion::ListToLimit(_), .. } => {
                return not_shared("a list moved into a smaller `len.max`");
            }
            ExpressionKind::Convert { conversion, value } => {
                let name = match conversion {
                    Conversion::ListToLimit(_) => "",
                    Conversion::ToText => "be.Text",
                    Conversion::WholeToFloat => "be.Float",
                    Conversion::TextToWhole => "be.Whole",
                    Conversion::TextToBool => return not_shared("`be.Bool`"),
                    Conversion::Scale { .. } => return not_shared("a unit's conversion"),
                    Conversion::Number(_) => return not_shared("the new number types"),
                    Conversion::FloatToWhole(Rounding::Floor) => "be.Whole.floor",
                    Conversion::FloatToWhole(Rounding::Trunc) => "be.Whole.trunc",
                    Conversion::FloatToWhole(Rounding::Ceil) => "be.Whole.ceil",
                    Conversion::FloatToWhole(Rounding::Nearest) => "be.Whole.nearest",
                    Conversion::FloatToWhole(Rounding::NearestEven) => "be.Whole.nearest.even",
                };
                Some((name.to_owned(), vec![(None, value.as_ref())]))
            }
            ExpressionKind::Sqrt(value) => Some(("sqrt".to_owned(), vec![(None, value.as_ref())])),
            _ => None,
        })
    }

    fn arguments(&self, arguments: &[(Option<String>, &Expression)]) -> Translated<String> {
        let mut written = Vec::new();
        for (label, argument) in arguments {
            let value = self.value(argument)?;
            written.push(match label {
                Some(label) => format!("{label}: {value}"),
                None => value,
            });
        }
        Ok(written.join(", "))
    }

    /// A call, written as calls when it's one or two deep and as a slide when it's three or more (Unio D128).
    fn call(&self, expression: &Expression) -> Translated<Option<String>> {
        let Some((name, arguments)) = self.as_call(expression)? else { return Ok(None) };
        // The chain of calls, outermost first, each the first argument of the next.
        let mut chain = vec![(name, arguments)];
        while let Some((_, arguments)) = chain.last() {
            let Some((None, first)) = arguments.first() else { break };
            match self.as_call(first)? {
                Some(inner) => chain.push(inner),
                None => break,
            }
        }
        if chain.len() < 3 {
            let Some((name, arguments)) = chain.first() else { return Ok(None) };
            return Ok(Some(format!("{name}({})", self.arguments(arguments)?)));
        }
        // A slide starts with the innermost call's one value, or with that call whole when it takes more.
        let Some((innermost_name, innermost_arguments)) = chain.pop() else { return Ok(None) };
        let mut slide = match innermost_arguments.as_slice() {
            [(None, only)] => format!("{} |> {innermost_name}", self.operand(only)?),
            _ => format!("{innermost_name}({})", self.arguments(&innermost_arguments)?),
        };
        for (name, arguments) in chain.iter().rev() {
            let rest = arguments.get(1..).unwrap_or(&[]);
            if rest.is_empty() {
                let _ = write!(slide, " |> {name}");
            } else {
                let _ = write!(slide, " |> {name}({})", self.arguments(rest)?);
            }
        }
        Ok(Some(slide))
    }

    /// A value as one side of an operator: anything that isn't a single thing gets brackets.
    fn operand(&self, expression: &Expression) -> Translated<String> {
        let single = matches!(
            expression.kind,
            ExpressionKind::Whole(_)
                | ExpressionKind::Float(_)
                | ExpressionKind::Bool(_)
                | ExpressionKind::Text(_)
                | ExpressionKind::Variable(_)
                | ExpressionKind::Member { .. }
                | ExpressionKind::Item { .. }
                | ExpressionKind::List(_)
        );
        let written = self.value(expression)?;
        let is_slide = written.contains(" |> ");
        let is_plain_call = self.as_call(expression)?.is_some() && !is_slide;
        Ok(if single || is_plain_call { written } else { format!("({written})") })
    }

    #[allow(clippy::too_many_lines)] // one case per kind of expression
    fn value(&self, expression: &Expression) -> Translated<String> {
        if let Some(call) = self.call(expression)? {
            return Ok(call);
        }
        Ok(match &expression.kind {
            ExpressionKind::Number(_) => return not_shared("the new number types"),
            ExpressionKind::Whole(value) => value.to_string(),
            ExpressionKind::Float(value) => {
                // A `Float`'s own text is how Unio writes the literal, `~` included (Unio D155, D180).
                let shown = float_to_text(*value);
                match shown.strip_prefix("~-") {
                    Some(rest) => format!("-~{rest}"),
                    None => shown,
                }
            }
            ExpressionKind::Bool(value) => value.to_string(),
            ExpressionKind::Text(text) => filled(&[Piece::Text(text.clone())])?,
            ExpressionKind::Variable(variable) => self.name(*variable),
            ExpressionKind::Member { receiver, name } => {
                if name != "len" {
                    return not_shared("a record");
                }
                // Unio's text names its length's unit and is reached by code points; Seal's counts graphemes (D66).
                if matches!(self.program.types.get(receiver.type_id), Type::Text) {
                    return not_shared("a text's length");
                }
                format!("{}.len", self.operand(receiver)?)
            }
            ExpressionKind::Item { receiver, .. } if matches!(self.program.types.get(receiver.type_id), Type::Text) => {
                return not_shared("reaching into a text");
            }
            ExpressionKind::Split { .. } => return not_shared("splitting a text"),
            // Unio changes one step inside a name (Unio D80); Seal's D62 reaches further.
            ExpressionKind::Inside { .. } => return not_shared("a change to a part of a part"),
            ExpressionKind::Item { receiver, index, zero_based } => {
                format!(
                    "{}[{}: {}]",
                    self.operand(receiver)?,
                    if *zero_based { "offset" } else { "pos" },
                    self.value(index)?
                )
            }
            ExpressionKind::Negate(value) => format!("-{}", self.operand(value)?),
            ExpressionKind::Not(value) => format!("not {}", self.operand(value)?),
            ExpressionKind::Binary { operator, left, right } => {
                // Unio leaves comparing two texts open (its chapter 5); Seal's D64 decides it for Seal alone.
                if !matches!(self.program.types.get(left.type_id), Type::Whole | Type::Float | Type::Bool) {
                    return not_shared("comparing two values that aren't numbers or `Bool`s");
                }
                if *operator == BinaryOperator::Power {
                    return not_shared("`**`");
                }
                // trace: Unio D190. `eq` and `not_eq` compare.
                let operator = match operator {
                    BinaryOperator::Equal => "eq",
                    BinaryOperator::NotEqual => "not_eq",
                    other => other.text(),
                };
                format!("{} {operator} {}", self.operand(left)?, self.operand(right)?)
            }
            ExpressionKind::Join(parts) => {
                // trace: Unio D200, D201, D202. Text is joined by filling it in, and braces hold only a name.
                let mut pieces: Vec<Piece> = Vec::new();
                for part in parts {
                    match &part.kind {
                        ExpressionKind::Text(text) => pieces.push(Piece::Text(text.clone())),
                        ExpressionKind::Variable(variable)
                            if matches!(
                                self.program.types.get(part.type_id),
                                Type::Whole | Type::Float | Type::Bool | Type::Text
                            ) =>
                        {
                            pieces.push(Piece::Name(self.name(*variable)));
                        }
                        _ => return not_shared("a value in braces that isn't a plain name"),
                    }
                }
                filled(&pieces)?
            }
            ExpressionKind::List(items) => {
                let items: Translated<Vec<String>> = items.iter().map(|item| self.value(item)).collect();
                format!("[{}]", items?.join(", "))
            }
            ExpressionKind::Repeat { value, .. } => format!("[all: {}]", self.value(value)?),
            ExpressionKind::Handle { call, fallback: HandleFallback::Value(fallback) } => {
                format!("{} ?? {}", self.value(call)?, self.operand(fallback)?)
            }
            ExpressionKind::Handle { call, fallback: HandleFallback::Ignore } => {
                format!("{} ?? ignore", self.value(call)?)
            }
            ExpressionKind::Handle { call, fallback: HandleFallback::Crash } => {
                format!("{} ?? crash", self.value(call)?)
            }
            ExpressionKind::ListAdd { variable, value, position } => match position {
                Some((position, zero_based)) => format!(
                    "{}.add({}, {}: {})",
                    self.name(*variable),
                    self.value(value)?,
                    if *zero_based { "offset" } else { "pos" },
                    self.value(position)?
                ),
                None => format!("{}.add({})", self.name(*variable), self.value(value)?),
            },
            ExpressionKind::ListRemove { variable, position, zero_based } => {
                format!(
                    "{}.remove({}: {})",
                    self.name(*variable),
                    if *zero_based { "offset" } else { "pos" },
                    self.value(position)?
                )
            }
            ExpressionKind::Lookup { .. }
            | ExpressionKind::Has { .. }
            | ExpressionKind::Dict(_)
            | ExpressionKind::DictRemove { .. } => {
                return not_shared("a `Dict`");
            }
            ExpressionKind::BuildRecord { .. } => return not_shared("a record"),
            ExpressionKind::BuildCase { .. } | ExpressionKind::MaybeOr { .. } | ExpressionKind::MaybeCrash(_) => {
                return not_shared("a `one_of` or a `Maybe`");
            }
            ExpressionKind::ShowPlaces { .. } => return not_shared("`places:` in braces"),
            ExpressionKind::Range { .. } => return not_shared("a `range` kept as a list"),
            ExpressionKind::Maths { .. } => return not_shared("Seal's own maths built-ins"),
            ExpressionKind::Land { .. } => return not_shared("a ranged number"),
            ExpressionKind::Try(_) | ExpressionKind::Otherwise { .. } => {
                return not_shared("a procedure that fails on purpose");
            }
            ExpressionKind::Call { .. }
            | ExpressionKind::DoorCall { .. }
            | ExpressionKind::Convert { .. }
            | ExpressionKind::Sqrt(_)
            | ExpressionKind::Error => return not_shared("something the translator can't write"),
        })
    }
}

/// One piece of a text being filled in: text as it is, or a name in braces.
enum Piece {
    Text(String),
    Name(String),
}

/// A text filled in from its pieces, as Unio writes it: in quotes, with a `\\` before a quote, a backslash or a brace
/// (Unio D30, D200). A line break or a tab has no way to be written in Unio yet, so text holding one isn't shared.
fn filled(pieces: &[Piece]) -> Translated<String> {
    let mut written = String::from("\"");
    for piece in pieces {
        match piece {
            Piece::Name(name) => {
                written.push('{');
                written.push_str(name);
                written.push('}');
            }
            Piece::Text(text) => {
                for character in text.chars() {
                    match character {
                        '"' | '\\' | '{' => {
                            written.push('\\');
                            written.push(character);
                        }
                        '\n' | '\t' => return not_shared("a line break or a tab in text"),
                        other => written.push(other),
                    }
                }
            }
        }
    }
    written.push('"');
    Ok(written)
}

/// What a script sees of the test host's doors (tests/README.md), which is what the translator checks against.
#[must_use]
pub fn test_host_doors() -> Vec<DoorSignature> {
    use seal::{DoorParameter, HostType};
    let door =
        |name: &str, parameters: Vec<(Option<&str>, HostType)>, result: HostType, can_pause: bool| DoorSignature {
            name: name.to_owned(),
            parameters: parameters
                .into_iter()
                .map(|(label, host_type)| DoorParameter { label: label.map(str::to_owned), host_type })
                .collect(),
            result,
            failures: Vec::new(),
            can_pause,
        };
    vec![
        door("show_msg", vec![(None, HostType::Text)], HostType::Nothing, false),
        door("show_msg.same_line", vec![(None, HostType::Text)], HostType::Nothing, false),
        door("ask", vec![(None, HostType::Text)], HostType::Text, false),
        door("rand", vec![(Some("from"), HostType::Whole), (Some("to"), HostType::Whole)], HostType::Whole, false),
        door("wait", vec![(Some("seconds"), HostType::Whole)], HostType::Nothing, true),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // trace: D19
    #[test]
    fn d19_a_script_becomes_the_same_unio_program() {
        let source = "mut tries = 0\nwhile tries != 3\n    tries = tries + 1\nend\nshow_msg(\"Tries: {tries}\")\n";
        let unio = translate(source, &test_host_doors());
        assert_eq!(
            unio,
            Ok("proc.main\n    var tries: Whole = 0;\n    while tries not_eq 3\n        tries = tries + 1;\n    end while\n    show_msg(\"Tries: {tries}\");\nend proc.main\n"
                .to_owned())
        );
    }

    // trace: D19, Unio D128
    #[test]
    fn d19_three_calls_in_a_chain_become_a_slide() {
        let source = "points = 2.5\npoints |> sqrt |> be.Text |> show_msg\n";
        let unio = translate(source, &test_host_doors()).unwrap_or_default();
        assert!(unio.contains("points |> sqrt |> be.Text |> show_msg;"), "{unio}");
    }

    #[test]
    fn d19_what_unio_lacks_is_not_shared() {
        let source = "mut stock: Dict<key: Text, value: Whole> = [\"sword\": 1]\nstock[key: \"axe\"] = 2\nshow_msg(\"{stock.len}\")\n";
        assert!(matches!(translate(source, &test_host_doors()), Err(Untranslatable::NotShared(_))));
    }
}
