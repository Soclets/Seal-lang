//! Compiling the checked tree to bytecode (D46).
//!
//! Who holds each heap value is worked out here (D15): a name made with `=` owns its value, a parameter, a loop's
//! item and a case's value borrow theirs, and a value worked out along the way is held until its statement ends.
//! The compiler emits `Retain` and `Release` so that every value is freed the moment its last holder ends.

use std::collections::HashMap;

use super::bytecode::{
    Compare, Compiled, CompiledParameter, CompiledProcedure, Crossing, CrossingCase, DoorSite, FAR, Instruction,
    JoinPiece, Layout, ProcedureEntry, Register, number_operation,
};
use crate::check::tree::{
    Block, Conversion, Expression, ExpressionKind, HandleFallback, Inside, NumberConversion, NumberHow, NumberType,
    PathStep, Procedure, Program, Rounding, Statement, StatementKind, VariableId, VariableKind,
};
use crate::check::types::{Type, TypeId};
use crate::decimal::Number;
use crate::host::Value;
use crate::message::{Message, Place};
use crate::signature::DoorSignature;
use crate::syntax::tree::{BinaryOperator, PositionKind};

/// Where a name lives while its script runs.
#[derive(Clone, Copy)]
enum Location {
    Register(Register),
    /// A top-level name that procedures use, shared by the script's tasks (D51).
    Global(u16),
    /// A `mut` record of plain numbers and `Bool`s, kept field by field in the registers from this one on. Its
    /// fields are read and changed with no heap value at all; it becomes one only where the whole record is
    /// wanted. The name still owns its own value (D15).
    InRegisters(Register),
}

struct Loop {
    /// How many held values there were when the loop's body began; a `break` frees the ones after.
    held_mark: usize,
    breaks: Vec<usize>,
}

#[derive(Default)]
struct Function {
    locations: HashMap<VariableId, Location>,
    /// The next free register.
    next: u32,
    high_water: u32,
    /// The registers holding heap values this frame must free: names it made and values not yet handed on.
    held: Vec<Register>,
    loops: Vec<Loop>,
    /// Where an `instead` puts its value, innermost `otherwise` last.
    use_targets: Vec<Register>,
    /// A name being handed over whole to the call being compiled, and the register it lives in: for now its
    /// value is the callee's, so this frame neither frees it nor lists it as held.
    moved_name: Option<VariableId>,
    moved_register: Option<Register>,
    /// The dictionary and key a statement has already looked up, and the registers saying where it was found.
    found: Option<(VariableId, VariableId, Register)>,
    /// The held registers known to hold a text.
    texts: std::collections::HashSet<Register>,
    /// Names that take over a parameter's value where the parameter is last used: each lives where its parameter did.
    takeovers: HashMap<VariableId, VariableId>,
    /// The list items whose fields are kept in registers for now, through a run of plain statements.
    views: Vec<ItemView>,
    /// Whether the statement being compiled is a plain one, which reads and changes items through their views.
    views_on: bool,
    /// Pairs of names known to hold different numbers here, as a `for each j in range[from: i + 1, …]` makes `i`
    /// and `j`: items reached with one are never the item reached with the other.
    distinct: Vec<(VariableId, VariableId)>,
}

/// How float operations are joined into one instruction: the shape, the operations in the order they run, the sides
/// in the order they're written, and the operations whose places follow the instruction.
type JoinPlan<'e> = (u8, Vec<u8>, Vec<&'e Expression>, Vec<&'e Expression>);

/// Which item of a list a view is of: the one at a name's number, or at a number written out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ViewKey {
    Name(VariableId),
    Number(i64),
}

/// A list's item, a record of plain numbers, with its fields kept in registers through a run of plain statements:
/// read into them once, changed there, and written back in one step when the run ends. The list's own value stays
/// right at every point anything else could see it, since anything else ends the run first.
#[derive(Clone, Copy, Debug)]
struct ItemView {
    list: VariableId,
    list_register: Register,
    key: ViewKey,
    zero_based: bool,
    /// The register the item's place is in.
    index: Register,
    /// The first of the registers its fields are in.
    base: Register,
    count: u32,
    /// Whether a field has been changed since it was read or written back.
    dirty: bool,
}

struct Compiler<'p> {
    program: &'p Program,
    doors: &'p [DoorSignature],
    out: Compiled,
    constant_ids: HashMap<u64, u32>,
    text_ids: HashMap<String, u32>,
    layout_ids: HashMap<Layout, u16>,
    globals: HashMap<VariableId, u16>,
    function: Function,
    place: Place,
    problem: Option<Message>,
    /// The procedures whose bodies are being written out in place of calls to them, outermost first.
    inlining: Vec<u32>,
    /// The records and `one_of`s being described for crossing to the host, outermost first (D72).
    crossing_open: Vec<Type>,
    /// The procedure being compiled, when it's not the top level.
    current: Option<u32>,
    /// Whether to compile plainly, with none of the shortcuts whose answers the fuzzer checks against this.
    plain: bool,
}

/// Compiles a checked script.
///
/// # Errors
///
/// Returns a message when the script is past one of the implementation's sizes, or the checked tree isn't what the
/// compiler expects, which is a bug in Seal.
pub fn compile(program: &Program, doors: &[DoorSignature]) -> Result<Compiled, Message> {
    compile_as(program, doors, false)
}

/// Compiles a checked script, as [`compile_as`] does, taking over the names and types the compiled script keeps
/// instead of copying them, since the checked script isn't needed after (D140).
///
/// # Errors
///
/// As [`compile`].
pub fn compile_taking(mut program: Program, doors: &[DoorSignature], plain: bool) -> Result<Compiled, Message> {
    let mut compiled = compile_parts(&program, doors, plain)?;
    compiled.failure_names = std::mem::take(&mut program.failure_names);
    compiled.types = std::mem::take(&mut program.types);
    for (procedure, checked) in compiled.procedures.iter_mut().zip(&mut program.procedures) {
        procedure.name = std::mem::take(&mut checked.name);
        for (parameter, &variable) in procedure.parameters.iter_mut().zip(&checked.parameters) {
            if let Some(variable) = program.variables.get_mut(variable.0 as usize) {
                parameter.name = std::mem::take(&mut variable.name);
            }
        }
    }
    Ok(compiled)
}

/// Compiles a checked script, plainly when `plain` is set: with no shortcut for a list's items, so that what the
/// shortcuts give can be checked against it.
///
/// # Errors
///
/// As [`compile`].
pub fn compile_as(program: &Program, doors: &[DoorSignature], plain: bool) -> Result<Compiled, Message> {
    let mut compiled = compile_parts(program, doors, plain)?;
    compiled.failure_names.clone_from(&program.failure_names);
    compiled.types = program.types.clone();
    for (procedure, checked) in compiled.procedures.iter_mut().zip(&program.procedures) {
        procedure.name.clone_from(&checked.name);
        for (parameter, &variable) in procedure.parameters.iter_mut().zip(&checked.parameters) {
            if let Some(variable) = program.variable(variable) {
                parameter.name.clone_from(&variable.name);
            }
        }
    }
    Ok(compiled)
}

/// Compiles a checked script, leaving out the names and types the compiled script keeps, which the caller copies or
/// takes from the checked script.
fn compile_parts(program: &Program, doors: &[DoorSignature], plain: bool) -> Result<Compiled, Message> {
    let mut compiler = Compiler {
        plain,
        program,
        doors,
        out: Compiled {
            built_in_failures: ["missing", "not_a_whole", "full"].map(|wanted| {
                (1u32..).zip(&program.failure_names).find(|(_, known)| *known == wanted).map_or(0, |(number, _)| number)
            }),
            // Room for a small script's instructions from the start, rather than growing to it.
            code: Vec::with_capacity(32),
            places: Vec::with_capacity(32),
            ..Compiled::default()
        },
        constant_ids: HashMap::new(),
        text_ids: HashMap::new(),
        layout_ids: HashMap::new(),
        globals: HashMap::new(),
        function: Function::default(),
        place: Place::default(),
        problem: None,
        inlining: Vec::new(),
        crossing_open: Vec::new(),
        current: None,
    };
    for (index, variable) in (0u32..).zip(&program.variables) {
        if variable.top_level && variable.captured {
            let global = u16::try_from(compiler.globals.len()).unwrap_or(u16::MAX);
            compiler.globals.insert(VariableId(index), global);
        }
    }
    if compiler.globals.len() >= usize::from(u16::MAX) {
        return Err(Message::not_built_yet(
            Place::default(),
            "more than 65,000 top-level variables used by procedures (D4) aren't",
        ));
    }
    compiler.out.global_count = u32::try_from(compiler.globals.len()).unwrap_or(u32::MAX);
    compiler.out.global_references = vec![false; compiler.globals.len()];
    for (&variable, &global) in &compiler.globals {
        let reference = program.variable(variable).is_some_and(|found| program.types.is_reference(found.type_id));
        if let Some(slot) = compiler.out.global_references.get_mut(usize::from(global)) {
            *slot = reference;
        }
    }
    compiler.top_level();
    for (number, procedure) in (0u32..).zip(&program.procedures) {
        compiler.current = Some(number);
        compiler.procedure(procedure);
    }
    match compiler.problem {
        Some(problem) => Err(problem),
        None => Ok(compiler.out),
    }
}

impl<'p> Compiler<'p> {
    // ---- small helpers ----

    fn internal(&mut self, detail: &str) {
        if self.problem.is_none() {
            self.problem = Some(Message::internal(format!("the compiler met {detail}")));
        }
    }

    fn too_big(&mut self, what: &str) {
        if self.problem.is_none() {
            self.problem = Some(Message::not_built_yet(self.place, format!("{what} aren't")));
        }
    }

    fn emit(&mut self, instruction: Instruction) -> usize {
        self.out.code.push(instruction);
        self.out.places.push(self.place);
        self.out.code.len() - 1
    }

    fn here(&self) -> usize {
        self.out.code.len()
    }

    /// Points the jump at `at` to `target`.
    fn patch(&mut self, at: usize, target: usize) {
        let distance = i64::try_from(target).unwrap_or(0) - i64::try_from(at).unwrap_or(0) - 1;
        let Ok(long) = i32::try_from(distance) else {
            self.too_big("scripts with more than two thousand million instructions");
            return;
        };
        let short = match i16::try_from(distance) {
            Ok(short) if short != FAR => short,
            _ => FAR,
        };
        let Some(instruction) = self.out.code.get_mut(at) else { return };
        let mut is_short = false;
        match instruction {
            Instruction::Jump { offset }
            | Instruction::JumpIfTrue { offset, .. }
            | Instruction::JumpIfFalse { offset, .. }
            | Instruction::JumpUnlessFailed { offset }
            | Instruction::JumpUnlessFailureIs { offset, .. } => *offset = long,
            Instruction::JumpIfEqualWhole { offset, .. }
            | Instruction::JumpIfNotEqualWhole { offset, .. }
            | Instruction::JumpIfLessWhole { offset, .. }
            | Instruction::JumpIfAtMostWhole { offset, .. }
            | Instruction::JumpIfEqualWholeSmall { offset, .. }
            | Instruction::JumpIfNotEqualWholeSmall { offset, .. }
            | Instruction::JumpIfLessWholeSmall { offset, .. }
            | Instruction::JumpIfAtMostWholeSmall { offset, .. }
            | Instruction::JumpIfGreaterWholeSmall { offset, .. }
            | Instruction::JumpIfAtLeastWholeSmall { offset, .. }
            | Instruction::JumpIfEqualFloat { offset, .. }
            | Instruction::JumpIfNotEqualFloat { offset, .. }
            | Instruction::JumpIfLessFloat { offset, .. }
            | Instruction::JumpIfAtMostFloat { offset, .. }
            | Instruction::NextItem { offset, .. }
            | Instruction::NextInRange { offset, .. }
            | Instruction::NextEntry { offset, .. } => {
                *offset = short;
                is_short = true;
            }
            _ => {}
        }
        if is_short && short == FAR {
            self.out.far_jumps.insert(u32::try_from(at).unwrap_or(u32::MAX), long);
        }
    }

    fn patch_all(&mut self, jumps: &[usize], target: usize) {
        for &jump in jumps {
            self.patch(jump, target);
        }
    }

    fn allocate_many(&mut self, count: u32) -> Register {
        let first = self.function.next;
        self.function.next = self.function.next.saturating_add(count);
        self.function.high_water = self.function.high_water.max(self.function.next);
        if self.function.next >= u32::from(u16::MAX) {
            self.too_big("procedures needing more than 65,000 registers");
            return 0;
        }
        Register::try_from(first).unwrap_or(0)
    }

    fn allocate(&mut self) -> Register {
        self.allocate_many(1)
    }

    /// A type's own kind; a number with a unit is its plain number when the script runs (D111).
    fn kind(&self, type_id: TypeId) -> &Type {
        match self.program.types.get(type_id) {
            Type::Measured { base, .. } => self.program.types.get(*base),
            other => other,
        }
    }

    /// trace: D115, D116. Which of the number types on the heap a type is: 0 for `HugeWhole`, 1 for `Decimal`, 2
    /// for `HugeDecimal`.
    fn number_kind(&self, type_id: TypeId) -> Option<u8> {
        match self.kind(type_id) {
            Type::HugeWhole => Some(0),
            Type::Decimal => Some(1),
            Type::HugeDecimal => Some(2),
            _ => None,
        }
    }

    fn number_operation(&mut self, operation: u8, type_id: TypeId, to: Register, left: Register, right: Register) {
        let kind = self.number_kind(type_id).unwrap_or(0);
        let operation = kind * number_operation::TYPE + operation;
        self.emit(Instruction::NumberOperation { to, left, right, operation });
    }

    /// trace: D115, D116, D127. Maths and comparisons on `HugeWhole`s, `Decimal`s and `HugeDecimal`s.
    fn number_binary(
        &mut self,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
        whole: &Expression,
        to: Register,
    ) {
        let type_id = left.type_id;
        // A `Whole` on the right of maths that keeps its sides in order is read as it is (Building rule 5).
        let keeps_order =
            !matches!(operator, BinaryOperator::Greater | BinaryOperator::AtLeast | BinaryOperator::Power);
        let left = self.value(left);
        let whole_right = if keeps_order { self.whole_operand(right) } else { None };
        let right = match whole_right {
            Some(register) => register,
            None => self.value(right),
        };
        self.place = whole.place;
        if whole_right.is_some() {
            let operation = match operator {
                BinaryOperator::Add => number_operation::ADD,
                BinaryOperator::Subtract => number_operation::SUBTRACT,
                BinaryOperator::Multiply => number_operation::MULTIPLY,
                BinaryOperator::Divide => number_operation::DIVIDE,
                BinaryOperator::DivFloor => number_operation::DIVIDE_FLOOR,
                BinaryOperator::DivTrunc => number_operation::DIVIDE_TRUNC,
                BinaryOperator::RemFloor => number_operation::REMAINDER_FLOOR,
                BinaryOperator::RemTrunc => number_operation::REMAINDER_TRUNC,
                BinaryOperator::Equal => number_operation::EQUAL,
                BinaryOperator::NotEqual => number_operation::NOT_EQUAL,
                BinaryOperator::Less => number_operation::LESS,
                BinaryOperator::AtMost => number_operation::AT_MOST,
                _ => return self.internal("an operator in the wrong place"),
            };
            let kind = self.number_kind(type_id).unwrap_or(0);
            let operation = kind * number_operation::TYPE + operation + number_operation::WHOLE_RIGHT;
            self.emit(Instruction::NumberOperation { to, left, right, operation });
            return;
        }
        let (operation, left, right) = match operator {
            BinaryOperator::Add => (number_operation::ADD, left, right),
            BinaryOperator::Subtract => (number_operation::SUBTRACT, left, right),
            BinaryOperator::Multiply => (number_operation::MULTIPLY, left, right),
            BinaryOperator::Divide => (number_operation::DIVIDE, left, right),
            BinaryOperator::Power => (number_operation::POWER, left, right),
            BinaryOperator::DivFloor => (number_operation::DIVIDE_FLOOR, left, right),
            BinaryOperator::DivTrunc => (number_operation::DIVIDE_TRUNC, left, right),
            BinaryOperator::RemFloor => (number_operation::REMAINDER_FLOOR, left, right),
            BinaryOperator::RemTrunc => (number_operation::REMAINDER_TRUNC, left, right),
            BinaryOperator::Equal => (number_operation::EQUAL, left, right),
            BinaryOperator::NotEqual => (number_operation::NOT_EQUAL, left, right),
            BinaryOperator::Less => (number_operation::LESS, left, right),
            BinaryOperator::AtMost => (number_operation::AT_MOST, left, right),
            BinaryOperator::Greater => (number_operation::LESS, right, left),
            BinaryOperator::AtLeast => (number_operation::AT_MOST, right, left),
            BinaryOperator::Join | BinaryOperator::And | BinaryOperator::Or => {
                return self.internal("an operator in the wrong place");
            }
        };
        self.number_operation(operation, type_id, to, left, right);
    }

    /// The steps of `name = name op value op value …` on a big number, innermost first: each operation's code, with
    /// its type, its right side, and its place. Nothing when the value isn't such a chain, or any right side reads the name.
    fn update_steps<'e>(
        &self,
        variable: VariableId,
        value: &'e Expression,
    ) -> Option<Vec<(u8, &'e Expression, Place)>> {
        let kind = self.number_kind(value.type_id)?;
        let mut steps = Vec::new();
        let mut current = value;
        loop {
            match &current.kind {
                ExpressionKind::Variable(read) if *read == variable && !steps.is_empty() => break,
                ExpressionKind::Binary { operator, left, right }
                    if self.number_kind(current.type_id) == Some(kind) && !mentions(right, variable) =>
                {
                    let operation = match operator {
                        BinaryOperator::Add => number_operation::ADD,
                        BinaryOperator::Subtract => number_operation::SUBTRACT,
                        BinaryOperator::Multiply => number_operation::MULTIPLY,
                        BinaryOperator::Divide => number_operation::DIVIDE,
                        BinaryOperator::DivFloor => number_operation::DIVIDE_FLOOR,
                        BinaryOperator::DivTrunc => number_operation::DIVIDE_TRUNC,
                        BinaryOperator::RemFloor => number_operation::REMAINDER_FLOOR,
                        BinaryOperator::RemTrunc => number_operation::REMAINDER_TRUNC,
                        _ => return None,
                    };
                    steps.push((kind * number_operation::TYPE + operation, right.as_ref(), current.place));
                    current = left;
                }
                _ => return None,
            }
        }
        steps.reverse();
        Some(steps)
    }

    /// trace: Building rule 5. A `Whole` standing exactly for a big number on the right of its maths: a `Whole`
    /// converted to the number's type, or a whole number written out. Gives the register holding the plain `Whole`;
    /// nothing for anything else, or when compiling plainly.
    fn whole_operand(&mut self, expression: &Expression) -> Option<Register> {
        if self.plain {
            return None;
        }
        match &expression.kind {
            ExpressionKind::Convert {
                conversion: Conversion::Number(NumberConversion { from: NumberType::Whole, how: NumberHow::Exact, .. }),
                value,
            } => Some(self.value(value)),
            ExpressionKind::Number(number) => {
                let whole = match number {
                    Number::Huge(value) => value.to_i64(),
                    Number::Decimal(value) if value.exponent == 0 && !(value.negative && value.coefficient == 0) => {
                        i64::try_from(value.coefficient).ok().map(|whole| if value.negative { -whole } else { whole })
                    }
                    Number::HugeDecimal(value) if value.exponent == 0 => value.coefficient.to_i64(),
                    _ => None,
                }?;
                let register = self.allocate();
                self.load_whole(register, whole);
                Some(register)
            }
            _ => None,
        }
    }

    fn is_reference(&self, type_id: TypeId) -> bool {
        self.program.types.is_reference(type_id)
    }

    fn constant(&mut self, bits: u64) -> u32 {
        if let Some(&id) = self.constant_ids.get(&bits) {
            return id;
        }
        let id = u32::try_from(self.out.constants.len()).unwrap_or(u32::MAX);
        self.out.constants.push(bits);
        self.constant_ids.insert(bits, id);
        id
    }

    fn text(&mut self, text: &str) -> u32 {
        if let Some(&id) = self.text_ids.get(text) {
            return id;
        }
        let id = u32::try_from(self.out.texts.len()).unwrap_or(u32::MAX);
        self.out.texts.push(text.to_owned());
        self.text_ids.insert(text.to_owned(), id);
        id
    }

    fn layout(&mut self, layout: Layout) -> u16 {
        if let Some(&id) = self.layout_ids.get(&layout) {
            return id;
        }
        let Ok(id) = u16::try_from(self.out.layouts.len()) else {
            self.too_big("scripts with more than 65,000 kinds of record and case");
            return 0;
        };
        self.out.layouts.push(layout.clone());
        self.layout_ids.insert(layout, id);
        id
    }

    fn fields_layout(&mut self, case: u32, fields: &[(String, TypeId)]) -> u16 {
        let references = fields.iter().map(|(_, field)| self.is_reference(*field)).collect();
        self.layout(Layout { case, references })
    }

    /// The fields of a record, or of one case of a `one_of` or a `Maybe`.
    fn fields_of(&self, type_id: TypeId, case: u32) -> Vec<(String, TypeId)> {
        match self.kind(type_id) {
            Type::Record(record) => {
                self.program.types.records.get(*record as usize).map(|record| record.fields.clone()).unwrap_or_default()
            }
            Type::OneOf(one_of) => self
                .program
                .types
                .one_ofs
                .get(*one_of as usize)
                .and_then(|one_of| one_of.cases.get(case as usize))
                .map(|case| case.fields.clone())
                .unwrap_or_default(),
            Type::Maybe(inner) if case == 1 => vec![("value".to_owned(), *inner)],
            _ => Vec::new(),
        }
    }

    /// How a value of a type crosses between the script and its host.
    fn crossing(&mut self, type_id: TypeId) -> Crossing {
        // A type that holds itself (D72) is met again while it's being described: it's marked there, not gone into.
        let kind = self.kind(type_id).clone();
        if matches!(kind, Type::Record(_) | Type::OneOf(_)) {
            if self.crossing_open.contains(&kind) {
                return Crossing::Itself { name: self.program.types.describe(type_id) };
            }
            self.crossing_open.push(kind.clone());
            let crossing = self.crossing_of(type_id);
            self.crossing_open.pop();
            return crossing;
        }
        self.crossing_of(type_id)
    }

    fn crossing_of(&mut self, type_id: TypeId) -> Crossing {
        match self.kind(type_id).clone() {
            Type::Whole => Crossing::Whole,
            Type::Float => Crossing::Float,
            Type::Bool => Crossing::Bool,
            Type::Text => Crossing::Text,
            Type::HugeWhole => Crossing::HugeWhole,
            Type::Decimal => Crossing::Decimal,
            Type::HugeDecimal => Crossing::HugeDecimal,
            Type::List { item, .. } | Type::Array { item, .. } => Crossing::List(Box::new(self.crossing(item))),
            Type::Dict { key, value } => Crossing::Dict(Box::new(self.crossing(key)), Box::new(self.crossing(value))),
            Type::Maybe(inner) => {
                let layout = self.fields_layout(1, &[("value".to_owned(), inner)]);
                Crossing::Maybe { inner: Box::new(self.crossing(inner)), layout }
            }
            Type::Record(record) => {
                let fields = self.fields_of(type_id, 0);
                let layout = self.fields_layout(0, &fields);
                let name = self
                    .program
                    .types
                    .records
                    .get(record as usize)
                    .map(|record| record.name.clone())
                    .unwrap_or_default();
                let fields =
                    fields.iter().map(|(field, field_type)| (field.clone(), self.crossing(*field_type))).collect();
                Crossing::Record { name, layout, fields }
            }
            Type::OneOf(one_of) => {
                let declared = self.program.types.one_ofs.get(one_of as usize).cloned();
                let mut cases = Vec::new();
                for (number, case) in (0u32..).zip(declared.iter().flat_map(|one_of| &one_of.cases)) {
                    let layout = self.fields_layout(number, &case.fields);
                    let fields = case
                        .fields
                        .iter()
                        .map(|(field, field_type)| (field.clone(), self.crossing(*field_type)))
                        .collect();
                    cases.push(CrossingCase { name: case.name.clone(), layout, fields });
                }
                Crossing::OneOf { name: declared.map(|one_of| one_of.name).unwrap_or_default(), cases }
            }
            Type::Nothing
            | Type::Error
            | Type::Variable(_)
            | Type::Parameter(_)
            | Type::Ranged { .. }
            | Type::Measured { .. } => Crossing::Nothing,
        }
    }

    // ---- who holds what ----

    fn hold(&mut self, register: Register) {
        self.function.held.push(register);
    }

    /// Notes that a register holds a text, so that freeing it can take the short way.
    fn hold_text(&mut self, register: Register) {
        self.function.texts.insert(register);
        self.hold(register);
    }

    fn release_instruction(&self, register: Register) -> Instruction {
        if self.function.texts.contains(&register) {
            Instruction::ReleaseText { register }
        } else {
            Instruction::Release { register }
        }
    }

    /// Stops counting a register as held: its value has been handed on.
    fn hand_on(&mut self, register: Register) {
        if let Some(position) = self.function.held.iter().rposition(|&held| held == register) {
            self.function.held.remove(position);
            self.function.texts.remove(&register);
        }
    }

    /// Frees everything held since `mark`, and forgets it.
    fn release_from(&mut self, mark: usize) {
        while self.function.held.len() > mark {
            if let Some(register) = self.function.held.pop() {
                let release = self.release_instruction(register);
                self.emit(release);
                self.function.texts.remove(&register);
            }
        }
    }

    /// Frees everything held since `mark` on a way out, such as `break` or `return`, leaving the list as it is for
    /// the code that follows on other ways.
    fn release_on_exit(&mut self, mark: usize, except: Option<Register>) {
        let held: Vec<Register> = self.function.held.get(mark..).unwrap_or(&[]).to_vec();
        for register in held.into_iter().rev() {
            if Some(register) != except && Some(register) != self.function.moved_register {
                let release = self.release_instruction(register);
                self.emit(release);
            }
        }
    }

    /// Records, for a call that can pause, what the frame holds there (D54).
    fn pause_map(&mut self) {
        let pc = u32::try_from(self.here()).unwrap_or(u32::MAX);
        let moved = self.function.moved_register;
        let held = self.function.held.iter().copied().filter(|register| Some(*register) != moved).collect();
        self.out.pause_maps.insert(pc, held);
    }

    // ---- procedures ----

    fn start_function(&mut self, first_free: u32) {
        // The last function's tables and lists are emptied and reused, keeping their room (D140).
        let mut last = std::mem::take(&mut self.function);
        last.locations.clear();
        last.held.clear();
        last.loops.clear();
        last.use_targets.clear();
        last.texts.clear();
        last.takeovers.clear();
        last.views.clear();
        last.distinct.clear();
        self.function = Function {
            next: first_free,
            high_water: first_free,
            locations: last.locations,
            held: last.held,
            loops: last.loops,
            use_targets: last.use_targets,
            texts: last.texts,
            takeovers: last.takeovers,
            views: last.views,
            distinct: last.distinct,
            ..Function::default()
        };
        for (&variable, &global) in &self.globals {
            self.function.locations.insert(variable, Location::Global(global));
        }
    }

    fn top_level(&mut self) {
        self.start_function(1);
        self.out.top_level_entry = u32::try_from(self.here()).unwrap_or(u32::MAX);
        let program = self.program;
        self.block(&program.top_level);
        self.emit(Instruction::ReturnNothing);
        self.out.top_level_frame_size = self.function.high_water + 1;
    }

    /// Checks, or pulls back, the number in `register` against a ranged type's limits, when it has any (D107, D128).
    fn check_limits(&mut self, target: TypeId, register: Register) {
        let Some(limits) = self.program.types.limits(target) else { return };
        let site = u16::try_from(self.out.limits.len()).unwrap_or(u16::MAX);
        if site == u16::MAX {
            return self.too_big("ranged numbers' checks");
        }
        self.out.limits.push(limits);
        self.emit(Instruction::CheckLimits { register, site });
    }

    /// A default written out plainly, as the host's value, for a call from the host that leaves it out (D42, D104).
    fn constant_value(&self, expression: &Expression) -> Option<Value> {
        Some(match &expression.kind {
            ExpressionKind::Whole(value) => Value::Whole(*value),
            ExpressionKind::Float(value) => Value::Float(*value),
            ExpressionKind::Bool(value) => Value::Bool(*value),
            ExpressionKind::Text(value) => Value::Text(value.clone()),
            ExpressionKind::List(items) => {
                Value::List(items.iter().map(|item| self.constant_value(item)).collect::<Option<Vec<_>>>()?)
            }
            ExpressionKind::Repeat { value, count } => {
                let value = self.constant_value(value)?;
                Value::List(vec![value; usize::try_from(*count).ok()?])
            }
            ExpressionKind::Dict(pairs) => Value::Dict(
                pairs
                    .iter()
                    .map(|(key, value)| Some((self.constant_value(key)?, self.constant_value(value)?)))
                    .collect::<Option<Vec<_>>>()?,
            ),
            ExpressionKind::BuildRecord { fields, .. } => {
                let mut ordered: Vec<&(u32, Expression)> = fields.iter().collect();
                ordered.sort_by_key(|(index, _)| *index);
                Value::Record(ordered.iter().map(|(_, value)| self.constant_value(value)).collect::<Option<Vec<_>>>()?)
            }
            ExpressionKind::BuildCase { case, fields } => {
                let mut ordered: Vec<&(u32, Expression)> = fields.iter().collect();
                ordered.sort_by_key(|(index, _)| *index);
                let values = ordered.iter().map(|(_, value)| self.constant_value(value)).collect::<Option<Vec<_>>>()?;
                if matches!(self.kind(expression.type_id), Type::Maybe(_)) {
                    Value::Maybe(values.into_iter().next().map(Box::new))
                } else {
                    Value::Case { case: *case, fields: values }
                }
            }
            _ => return None,
        })
    }

    fn procedure(&mut self, procedure: &Procedure) {
        // trace: D161. A generic procedure runs only as the instances its calls make, so it's kept as a name, with
        // nothing to run.
        if !procedure.type_parameters.is_empty() {
            self.start_function(1);
            self.place = procedure.place;
            let entry = u32::try_from(self.here()).unwrap_or(u32::MAX);
            self.emit(Instruction::Unreachable);
            self.out.entries.push(ProcedureEntry { entry, frame_size: 1 });
            self.out.procedures.push(CompiledProcedure {
                name: String::new(),
                entry,
                frame_size: 1,
                parameters: Vec::new(),
                result: Crossing::Nothing,
                result_type: crate::check::types::NOTHING,
                failures: Vec::new(),
                can_pause: false,
                holds_itself: false,
                generic: true,
                method: false,
            });
            return;
        }
        let parameter_count = u32::try_from(procedure.parameters.len()).unwrap_or(u32::MAX);
        self.start_function(parameter_count.saturating_add(1));
        self.place = procedure.place;
        let entry = u32::try_from(self.here()).unwrap_or(u32::MAX);
        let mut parameters = Vec::new();
        for (position, &parameter) in (0u32..).zip(&procedure.parameters) {
            let register = Register::try_from(position + 1).unwrap_or(0);
            self.function.locations.insert(parameter, Location::Register(register));
            // A procedure takes its arguments over, so it frees them when it's done (D15). A `mut` first parameter
            // stands for the caller's variable, so its value is left for the caller to take back (D165).
            let lent = position == 0 && self.changes_first(procedure);
            if !lent && self.program.variable(parameter).is_some_and(|variable| self.is_reference(variable.type_id)) {
                self.hold(register);
            }
            let Some(variable) = self.program.variable(parameter) else { continue };
            // trace: D128. A ranged parameter is checked as the procedure starts, for the script's calls and the
            // host's alike.
            self.check_limits(variable.type_id, register);
            let default = procedure
                .defaults
                .get(position as usize)
                .cloned()
                .flatten()
                .and_then(|default| self.constant_value(&default));
            parameters.push(CompiledParameter {
                // The name comes from the checked script once compiling is done.
                name: String::new(),
                label: procedure.labels.get(position as usize).cloned().flatten(),
                crossing: self.crossing(variable.type_id),
                type_id: variable.type_id,
                default,
            });
        }
        // `mut copy = parameter` where the parameter is never used again takes the parameter's value over, with
        // nothing copied and no count taken.
        for (position, statement) in procedure.body.iter().enumerate() {
            if let StatementKind::Make { variable, value } = &statement.kind
                && let ExpressionKind::Variable(parameter) = &value.kind
                && procedure.parameters.contains(parameter)
                && !(self.changes_first(procedure) && procedure.parameters.first() == Some(parameter))
                && self.is_reference(value.type_id)
                && self.keeps_in_registers(*variable).is_none()
                && !self.globals.contains_key(variable)
                && !block_uses(procedure.body.get(position + 1..).unwrap_or(&[]), *parameter)
            {
                self.function.takeovers.insert(*variable, *parameter);
            }
        }
        self.block(&procedure.body);
        self.place = procedure.end_place;
        if matches!(self.kind(procedure.result), Type::Nothing) {
            if !procedure
                .body
                .last()
                .is_some_and(|last| matches!(last.kind, StatementKind::Return(_) | StatementKind::Fail(_)))
            {
                self.release_on_exit(0, None);
            }
            self.emit(Instruction::ReturnNothing);
        } else {
            // Checking made sure every way through ends in a `return` or a `fail`.
            self.emit(Instruction::Unreachable);
        }
        let failures =
            procedure.failures.iter().map(|&failure| self.program.failure_name(failure).to_owned()).collect();
        let result = self.crossing(procedure.result);
        let holds_itself = result.holds_itself()
            || parameters.iter().any(|parameter: &CompiledParameter| parameter.crossing.holds_itself());
        self.out.entries.push(ProcedureEntry { entry, frame_size: self.function.high_water + 1 });
        self.out.procedures.push(CompiledProcedure {
            // The name comes from the checked script once compiling is done.
            name: String::new(),
            entry,
            frame_size: self.function.high_water + 1,
            parameters,
            result,
            result_type: procedure.result,
            failures,
            can_pause: procedure.can_pause,
            holds_itself,
            generic: false,
            method: procedure.method,
        });
    }

    // ---- statements ----

    fn block(&mut self, block: &Block) {
        let held_mark = self.function.held.len();
        let register_mark = self.function.next;
        for statement in block {
            // Items' fields stay in registers through a run of plain statements; anything else ends the run.
            let plain = !self.plain && self.is_plain_statement(statement);
            if !plain || self.changes_a_view_key(statement) {
                self.flush_views();
            }
            self.function.views_on = plain;
            self.statement(statement);
            self.function.views_on = false;
            // The registers of the views still in use stay taken.
            self.function.next = self.function.next.max(self.views_top());
        }
        self.flush_views();
        // A block that ends by leaving has already freed what it holds on its way out.
        if block.last().is_some_and(|last| {
            matches!(last.kind, StatementKind::Return(_) | StatementKind::Break | StatementKind::Fail(_))
        }) {
            self.function.held.truncate(held_mark);
        } else {
            self.release_from(held_mark);
        }
        self.function.next = register_mark;
    }

    fn location(&mut self, variable: VariableId) -> Location {
        if let Some(&location) = self.function.locations.get(&variable) {
            location
        } else {
            self.internal("a name with no place to live");
            Location::Register(0)
        }
    }

    fn variable_type(&self, variable: VariableId) -> TypeId {
        self.program.variable(variable).map_or(crate::check::types::ERROR, |variable| variable.type_id)
    }

    /// Runs `change` on the register holding a name's value, so the value can be changed in place. A top-level
    /// name shared with procedures is taken out for the change and put back after it.
    fn change_in_place(&mut self, variable: VariableId, change: impl FnOnce(&mut Self, Register)) {
        match self.location(variable) {
            Location::Register(register) => change(self, register),
            Location::Global(global) => {
                let taken = self.allocate();
                self.emit(Instruction::GetGlobal { to: taken, global });
                change(self, taken);
                self.emit(Instruction::SetGlobal { global, from: taken });
            }
            Location::InRegisters(_) => self.internal("a change inside a record kept in registers"),
        }
    }

    // ---- float operations joined into one instruction ----

    /// A float `+`, `-`, `*` or `/` that can be joined with others into one instruction: its number, and its two
    /// sides. One with a number written out on a side has an instruction of its own, and isn't joined.
    fn joinable<'e>(&self, expression: &'e Expression) -> Option<(u8, &'e Expression, &'e Expression)> {
        let ExpressionKind::Binary { operator, left, right } = &expression.kind else { return None };
        if !matches!(self.kind(left.type_id), Type::Float)
            || matches!(left.kind, ExpressionKind::Float(_))
            || matches!(right.kind, ExpressionKind::Float(_))
            || fold(expression).is_some()
        {
            return None;
        }
        let code = match operator {
            BinaryOperator::Add => 0,
            BinaryOperator::Subtract => 1,
            BinaryOperator::Multiply => 2,
            BinaryOperator::Divide => 3,
            _ => return None,
        };
        Some((code, left, right))
    }

    /// Whether a value is already in a register, so that reading it runs nothing: a name, or a field kept in
    /// registers. Such a value can be read after an operation that might fail without changing what the script does.
    fn already_in_a_register(&self, expression: &Expression) -> bool {
        match &expression.kind {
            ExpressionKind::Variable(name) => {
                matches!(self.function.locations.get(name), Some(Location::Register(_)))
            }
            ExpressionKind::Member { receiver, .. } => match &receiver.kind {
                ExpressionKind::Variable(record) => {
                    matches!(self.function.locations.get(record), Some(Location::InRegisters(_)))
                }
                ExpressionKind::Item { receiver: list, index, zero_based } if self.function.views_on => {
                    let (ExpressionKind::Variable(list), key) = (
                        &list.kind,
                        match &index.kind {
                            ExpressionKind::Variable(key) => Some(ViewKey::Name(*key)),
                            ExpressionKind::Whole(number) => Some(ViewKey::Number(*number)),
                            _ => None,
                        },
                    ) else {
                        return false;
                    };
                    key.is_some_and(|key| {
                        self.function
                            .views
                            .iter()
                            .any(|view| view.list == *list && view.key == key && view.zero_based == *zero_based)
                    })
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Joins two or three float operations into one instruction, when `whole` is the last of such a group. The
    /// operations run in the order they would on their own, and each is checked and reported as on its own; a
    /// side read after one of them has run must already be in a register, so that nothing else changes order.
    fn fused_floats(&mut self, whole: &Expression, to: Register) -> bool {
        // Every register a joined instruction names fits in a byte.
        if self.function.next >= 240 || to >= 240 {
            return false;
        }
        let Some((last, left, right)) = self.joinable(whole) else { return false };
        let inside_left = self.joinable(left);
        let inside_right = self.joinable(right);
        let mut plan: Option<JoinPlan> = None;
        // a·((b·c)·d)
        if plan.is_none()
            && let Some((second, inner, d)) = inside_right
            && let Some((first, b, c)) = self.joinable(inner)
            && self.already_in_a_register(d)
        {
            plan = Some((5, vec![first, second, last], vec![left, b, c, d], vec![inner, right]));
        }
        // ((a·b)·c)·d
        if plan.is_none()
            && let Some((second, inner, c)) = inside_left
            && let Some((first, a, b)) = self.joinable(inner)
            && self.already_in_a_register(c)
            && self.already_in_a_register(right)
        {
            plan = Some((2, vec![first, second, last], vec![a, b, c, right], vec![inner, left]));
        }
        // (a·b)·(c·d)
        if plan.is_none()
            && let Some((first, a, b)) = inside_left
            && let Some((second, c, d)) = inside_right
            && self.already_in_a_register(c)
            && self.already_in_a_register(d)
        {
            plan = Some((4, vec![first, second, last], vec![a, b, c, d], vec![left, right]));
        }
        // (a·(b·c))·d
        if plan.is_none()
            && let Some((second, a, inner)) = inside_left
            && let Some((first, b, c)) = self.joinable(inner)
            && self.already_in_a_register(right)
        {
            plan = Some((3, vec![first, second, last], vec![a, b, c, right], vec![inner, left]));
        }
        // a·(b·(c·d))
        if plan.is_none()
            && let Some((second, b, inner)) = inside_right
            && let Some((first, c, d)) = self.joinable(inner)
        {
            plan = Some((6, vec![first, second, last], vec![left, b, c, d], vec![inner, right]));
        }
        // a·(b·c)
        if plan.is_none()
            && let Some((first, b, c)) = inside_right
        {
            plan = Some((1, vec![first, last], vec![left, b, c], vec![right]));
        }
        // (a·b)·c
        if plan.is_none()
            && let Some((first, a, b)) = inside_left
            && self.already_in_a_register(right)
        {
            plan = Some((0, vec![first, last], vec![a, b, right], vec![left]));
        }
        let Some((shape, operations, sides, earlier)) = plan else { return false };
        // A side already in a register is read where it is. Any other is worked out into a register set aside before
        // anything runs, so that every register named stays within a byte however much working out a side takes.
        let set_aside = self.allocate_many(u32::try_from(sides.len()).unwrap_or(4));
        let registers: Vec<Register> = sides
            .iter()
            .zip(set_aside..)
            .map(|(side, slot)| {
                if self.already_in_a_register(side) {
                    self.value(side)
                } else {
                    self.value_into(side, slot);
                    slot
                }
            })
            .collect();
        let byte = |register: Option<&Register>| register.and_then(|register| u8::try_from(*register).ok());
        let (Some(a), Some(b), Some(c), Some(to)) =
            (byte(registers.first()), byte(registers.get(1)), byte(registers.get(2)), u8::try_from(to).ok())
        else {
            self.internal("a joined float instruction naming a register past a byte");
            return true;
        };
        let d = byte(registers.get(3)).unwrap_or(a);
        let ops = operations.iter().rev().fold(0u8, |ops, code| (ops << 2) | code);
        self.place = whole.place;
        self.emit(Instruction::FloatFused { shape, ops, to, a, b, c, d });
        for operation in earlier {
            self.place = operation.place;
            self.emit(Instruction::PlaceOnly);
        }
        self.place = whole.place;
        true
    }

    // ---- items' fields kept in registers ----

    /// The name a range starts just past, when it counts up from `name + n` with `n` at least 1 and the name can't
    /// change: every number the range gives is bigger than the name's.
    fn counts_up_from_past(&self, from: &Expression, step: Option<&Expression>) -> Option<VariableId> {
        if !step.is_none_or(|step| matches!(step.kind, ExpressionKind::Whole(by) if by >= 1)) {
            return None;
        }
        let ExpressionKind::Binary { operator: BinaryOperator::Add, left, right } = &from.kind else { return None };
        let (ExpressionKind::Variable(name), ExpressionKind::Whole(more)) = (&left.kind, &right.kind) else {
            return None;
        };
        let fixed = self.program.variable(*name).is_some_and(|variable| !variable.mutable);
        (fixed && *more >= 1).then_some(*name)
    }

    /// The parts of `list[pos: k]` or `list[offset: k]` a view needs, when it can have one: the list is a name of
    /// this procedure's own, its items are records of plain numbers, and the place is a name or a number written
    /// out.
    fn view_target(
        &mut self,
        list: &Expression,
        index: &Expression,
        zero_based: bool,
    ) -> Option<(VariableId, Register, ViewKey, u32)> {
        let ExpressionKind::Variable(name) = &list.kind else { return None };
        let Some(&Location::Register(list_register)) = self.function.locations.get(name) else { return None };
        let item = match self.kind(list.type_id) {
            Type::List { item, .. } | Type::Array { item, .. } => *item,
            _ => return None,
        };
        if !matches!(self.kind(item), Type::Record(_)) {
            return None;
        }
        let fields = self.fields_of(item, 0);
        if fields.is_empty() || fields.len() > 32 || fields.iter().any(|(_, field)| self.is_reference(*field)) {
            return None;
        }
        let key = match &index.kind {
            ExpressionKind::Variable(key)
                if matches!(self.function.locations.get(key), Some(Location::Register(_))) =>
            {
                ViewKey::Name(*key)
            }
            ExpressionKind::Whole(number) => ViewKey::Number(*number),
            _ => return None,
        };
        let _ = zero_based;
        Some((*name, list_register, key, u32::try_from(fields.len()).unwrap_or(0)))
    }

    /// Whether two places in the same list could be the same item.
    fn may_be_the_same_item(&self, first: (ViewKey, bool), second: (ViewKey, bool)) -> bool {
        match (first, second) {
            ((ViewKey::Number(a), a_zero), (ViewKey::Number(b), b_zero)) => {
                a.saturating_sub(i64::from(!a_zero)) == b.saturating_sub(i64::from(!b_zero))
            }
            ((ViewKey::Name(a), a_zero), (ViewKey::Name(b), b_zero)) => {
                a == b
                    || a_zero != b_zero
                    || !self.function.distinct.iter().any(|&(x, y)| (x == a && y == b) || (x == b && y == a))
            }
            _ => true,
        }
    }

    /// The view of an item, made now when there isn't one: its fields are read into registers of their own. A
    /// view of what could be the same item with a change not yet written back is written back first.
    fn view(&mut self, list: VariableId, list_register: Register, key: ViewKey, zero_based: bool, count: u32) -> usize {
        if let Some(found) = self
            .function
            .views
            .iter()
            .position(|view| view.list == list && view.key == key && view.zero_based == zero_based)
        {
            return found;
        }
        for position in 0..self.function.views.len() {
            let Some(other) = self.function.views.get(position).copied() else { continue };
            if other.list == list
                && other.dirty
                && self.may_be_the_same_item((other.key, other.zero_based), (key, zero_based))
            {
                self.write_back(position);
            }
        }
        let index = match key {
            ViewKey::Name(name) => {
                if let Some(&Location::Register(register)) = self.function.locations.get(&name) {
                    register
                } else {
                    self.internal("a view's place with nowhere to live");
                    0
                }
            }
            ViewKey::Number(number) => {
                let register = self.allocate();
                self.load_whole(register, number);
                register
            }
        };
        let base = self.allocate_many(count);
        self.emit(if zero_based {
            Instruction::UnpackItemAtOffset { base, list: list_register, index }
        } else {
            Instruction::UnpackItemAtPosition { base, list: list_register, index }
        });
        self.function.views.push(ItemView { list, list_register, key, zero_based, index, base, count, dirty: false });
        self.function.views.len() - 1
    }

    /// Writes a view's changed fields back into its item, in place when the list alone holds the item.
    fn write_back(&mut self, position: usize) {
        let Some(view) = self.function.views.get_mut(position) else { return };
        if !view.dirty {
            return;
        }
        view.dirty = false;
        let (list, index, base, zero_based) = (view.list_register, view.index, view.base, view.zero_based);
        self.emit(if zero_based {
            Instruction::PackIntoItemAtOffset { list, index, base }
        } else {
            Instruction::PackIntoItemAtPosition { list, index, base }
        });
    }

    /// Ends the run of plain statements: every changed view is written back, and none is kept.
    fn flush_views(&mut self) {
        for position in 0..self.function.views.len() {
            self.write_back(position);
        }
        self.function.views.clear();
    }

    /// The first register past the ones the views in use keep.
    fn views_top(&self) -> u32 {
        self.function
            .views
            .iter()
            .map(|view| (u32::from(view.base) + view.count).max(u32::from(view.index) + 1))
            .max()
            .unwrap_or(0)
    }

    /// The register a field of a list's item is kept in, when `expression` reads one in a run of plain statements.
    fn view_field(&mut self, expression: &Expression) -> Option<Register> {
        if !self.function.views_on {
            return None;
        }
        let ExpressionKind::Member { receiver, name } = &expression.kind else { return None };
        let ExpressionKind::Item { receiver: list, index, zero_based } = &receiver.kind else { return None };
        let (list_name, list_register, key, count) = self.view_target(list, index, *zero_based)?;
        let field = self.field_index(receiver.type_id, name);
        // A bad place is reported where the plain way reports it: at the item reached.
        self.place = receiver.place;
        let view = self.view(list_name, list_register, key, *zero_based, count);
        self.function.views.get(view).map(|view| view.base + field)
    }

    /// Whether working out a value runs nothing but plain maths on numbers this procedure holds in registers, and
    /// fields of list items that can have views: no call, no jump, nothing that could see a list or change a name.
    fn is_plain_value(&mut self, expression: &Expression, hidden: &[VariableId]) -> bool {
        match &expression.kind {
            ExpressionKind::Whole(_) | ExpressionKind::Float(_) | ExpressionKind::Bool(_) => true,
            ExpressionKind::Variable(name) => {
                !self.is_reference(expression.type_id)
                    && (hidden.contains(name)
                        || matches!(self.function.locations.get(name), Some(Location::Register(_))))
            }
            ExpressionKind::Binary { operator, left, right } => {
                !matches!(operator, BinaryOperator::And | BinaryOperator::Or | BinaryOperator::Join)
                    && !matches!(self.kind(left.type_id), Type::Text)
                    && self.is_plain_value(left, hidden)
                    && self.is_plain_value(right, hidden)
            }
            ExpressionKind::Negate(inner) | ExpressionKind::Not(inner) | ExpressionKind::Sqrt(inner) => {
                self.is_plain_value(inner, hidden)
            }
            ExpressionKind::Convert { conversion: Conversion::WholeToFloat | Conversion::FloatToWhole(_), value } => {
                self.is_plain_value(value, hidden)
            }
            ExpressionKind::Member { receiver, .. } => match &receiver.kind {
                ExpressionKind::Variable(record) => {
                    matches!(self.function.locations.get(record), Some(Location::InRegisters(_)))
                }
                ExpressionKind::Item { receiver: list, index, zero_based } => {
                    !self.is_reference(expression.type_id) && self.view_target(list, index, *zero_based).is_some()
                }
                _ => false,
            },
            _ => false,
        }
    }

    /// Whether a statement is a plain one: it gives a number to a name of this procedure's own, or changes a field
    /// of a list's item that can have a view, working out nothing but plain values.
    fn is_plain_statement(&mut self, statement: &Statement) -> bool {
        match &statement.kind {
            StatementKind::Make { variable, value } | StatementKind::Set { variable, value } => {
                let made = matches!(statement.kind, StatementKind::Make { .. });
                !self.is_reference(value.type_id)
                    && !self.globals.contains_key(variable)
                    && !self.function.takeovers.contains_key(variable)
                    && (made || matches!(self.function.locations.get(variable), Some(Location::Register(_))))
                    && self.is_plain_value(value, &[])
            }
            StatementKind::Inside { path, change } => {
                let ([PathStep::Item { index, zero_based }], StatementKind::SetField { value, .. }) =
                    (path.steps.as_slice(), &change.kind)
                else {
                    return false;
                };
                let root = Expression {
                    kind: ExpressionKind::Variable(path.root),
                    type_id: path.types.first().copied().unwrap_or(value.type_id),
                    place: statement.place,
                };
                if self.is_reference(value.type_id) || self.view_target(&root, index, *zero_based).is_none() {
                    return false;
                }
                let mut hidden = Vec::new();
                for (name, worked_out) in &path.lets {
                    if !self.is_plain_value(worked_out, &hidden) {
                        return false;
                    }
                    hidden.push(*name);
                }
                self.is_plain_value(value, &hidden)
            }
            _ => false,
        }
    }

    /// Whether a statement gives a new number to a name some view's item is reached with.
    fn changes_a_view_key(&self, statement: &Statement) -> bool {
        let StatementKind::Set { variable, .. } = &statement.kind else { return false };
        self.function.views.iter().any(|view| view.key == ViewKey::Name(*variable))
    }

    /// Changes a field of a list's item in the registers its view keeps; the item itself is written back when the
    /// run of plain statements ends.
    fn change_through_view(&mut self, path: &Inside, change: &Statement) {
        let ([PathStep::Item { index, zero_based }], StatementKind::SetField { variable, field, value }) =
            (path.steps.as_slice(), &change.kind)
        else {
            self.internal("a change through a view that isn't a field of a list's item");
            return;
        };
        let root = Expression {
            kind: ExpressionKind::Variable(path.root),
            type_id: path.types.first().copied().unwrap_or(value.type_id),
            place: change.place,
        };
        let Some((list, list_register, key, count)) = self.view_target(&root, index, *zero_based) else {
            self.internal("a change through a view of something that can't have one");
            return;
        };
        let record_type = self.variable_type(*variable);
        let field = self.field_index(record_type, field);
        let place = self.place;
        // With the item's view already there, plain maths works its new value out straight into the field's
        // register. Otherwise the value comes first and the view after, in the order the plain way reaches them, so
        // that a bad place is found in the same order either way.
        let already = self
            .function
            .views
            .iter()
            .find(|view| view.list == list && view.key == key && view.zero_based == *zero_based)
            .map(|view| view.base + field);
        let mut worked_out_in_place = false;
        for (position, (hidden, worked_out)) in path.lets.iter().enumerate() {
            let final_one = position + 1 == path.lets.len();
            let is_the_value = matches!(&value.kind, ExpressionKind::Variable(read) if read == hidden);
            match already {
                Some(target)
                    if final_one && is_the_value && matches!(worked_out.kind, ExpressionKind::Binary { .. }) =>
                {
                    // Plain maths reads what it needs before its one instruction writes.
                    self.value_into(worked_out, target);
                    worked_out_in_place = true;
                }
                _ => {
                    let register = self.held_value(worked_out);
                    self.function.locations.insert(*hidden, Location::Register(register));
                }
            }
        }
        if !worked_out_in_place {
            let from = self.value(value);
            self.place = place;
            let view = self.view(list, list_register, key, *zero_based, count);
            let Some(target) = self.function.views.get(view).map(|view| view.base + field) else { return };
            if from != target {
                self.emit(Instruction::Move { to: target, from });
            }
        }
        // Any other view of what could be the same item is out of date now.
        self.drop_views_that_may_be(list, key, *zero_based);
        if let Some(view) = self
            .function
            .views
            .iter_mut()
            .find(|view| view.list == list && view.key == key && view.zero_based == *zero_based)
        {
            view.dirty = true;
        }
    }

    /// Drops every other view of what could be the same item as the one at `key`, writing back any with a change
    /// of its own first.
    fn drop_views_that_may_be(&mut self, list: VariableId, key: ViewKey, zero_based: bool) {
        let mut position = 0;
        while position < self.function.views.len() {
            let Some(other) = self.function.views.get(position).copied() else { break };
            let itself = other.list == list && other.key == key && other.zero_based == zero_based;
            if !itself
                && other.list == list
                && self.may_be_the_same_item((other.key, other.zero_based), (key, zero_based))
            {
                self.write_back(position);
                self.function.views.remove(position);
            } else {
                position += 1;
            }
        }
    }

    /// Takes the way into a part of a name, then runs `change` on the part (D62). What the change needs was worked
    /// out under hidden names first, so nothing runs between reaching the part and changing it.
    fn inside(&mut self, path: &Inside, change: impl FnOnce(&mut Self)) {
        let place = self.place;
        for (hidden, value) in &path.lets {
            let register = self.held_value(value);
            self.function.locations.insert(*hidden, Location::Register(register));
        }
        let places: Vec<Option<(Register, bool)>> = path
            .steps
            .iter()
            .map(|step| match step {
                PathStep::Item { index, zero_based } => Some((self.value(index), *zero_based)),
                PathStep::Field(_) => None,
            })
            .collect();
        let fields: Vec<Option<u16>> = path
            .steps
            .iter()
            .zip(&path.types)
            .map(|(step, &inside)| match step {
                PathStep::Field(name) => Some(self.field_index(inside, name)),
                PathStep::Item { .. } => None,
            })
            .collect();
        let part = path.part;
        self.change_in_place(path.root, |compiler, root| {
            compiler.place = place;
            compiler.emit(Instruction::Own { register: root });
            let mut reached = root;
            for (item, field) in places.iter().zip(&fields) {
                let to = compiler.allocate();
                match (item, field) {
                    (Some((index, true)), _) => {
                        compiler.emit(Instruction::OwnItemAtOffset { to, list: reached, index: *index });
                    }
                    (Some((index, false)), _) => {
                        compiler.emit(Instruction::OwnItemAtPosition { to, list: reached, index: *index });
                    }
                    (None, Some(field)) => {
                        compiler.emit(Instruction::OwnField { to, record: reached, field: *field });
                    }
                    (None, None) => compiler.internal("a step into a part that's neither a field nor a place"),
                }
                reached = to;
            }
            // The part is read where it sits: its holder is the value it's a part of.
            compiler.function.locations.insert(part, Location::Register(reached));
            change(compiler);
        });
    }

    /// How many registers a name's record is kept in, when it's kept that way: a `mut` record local to its
    /// procedure, whose fields are all plain numbers and `Bool`s.
    fn keeps_in_registers(&self, variable: VariableId) -> Option<u32> {
        let found = self.program.variable(variable)?;
        if !found.mutable || found.kind != VariableKind::Named || self.globals.contains_key(&variable) {
            return None;
        }
        let Type::Record(_) = self.kind(found.type_id) else { return None };
        let fields = self.fields_of(found.type_id, 0);
        let plain =
            !fields.is_empty() && fields.len() <= 32 && fields.iter().all(|(_, field)| !self.is_reference(*field));
        plain.then(|| u32::try_from(fields.len()).unwrap_or(0))
    }

    /// The register one field of a record kept in registers lives in, when `expression` reads such a field.
    fn field_register(&mut self, expression: &Expression) -> Option<Register> {
        let ExpressionKind::Member { receiver, name } = &expression.kind else { return None };
        let ExpressionKind::Variable(variable) = &receiver.kind else { return None };
        let Location::InRegisters(base) = self.location(*variable) else { return None };
        let record_type = self.variable_type(*variable);
        Some(base + self.field_index(record_type, name))
    }

    /// Puts a record's fields into the registers from `base` on. `fresh` says the registers hold nothing yet, so
    /// the fields can be worked out straight into them.
    fn unpack_into(&mut self, value: &Expression, base: Register, fresh: bool) {
        match &value.kind {
            ExpressionKind::BuildRecord { fields, .. } if fresh => {
                for (field, field_value) in fields {
                    self.value_into(field_value, base + Register::try_from(*field).unwrap_or(0));
                }
            }
            ExpressionKind::BuildRecord { fields, .. } => {
                // The new fields may read the old ones, so they're all worked out before any is put in place.
                let mut copies = Vec::new();
                for (field, field_value) in fields {
                    let copy = self.allocate();
                    self.value_into(field_value, copy);
                    copies.push((*field, copy));
                }
                for (field, copy) in copies {
                    self.emit(Instruction::Move { to: base + Register::try_from(field).unwrap_or(0), from: copy });
                }
            }
            ExpressionKind::Variable(other) if matches!(self.location(*other), Location::InRegisters(_)) => {
                if let Location::InRegisters(from) = self.location(*other)
                    && from != base
                {
                    let count = self.fields_of(value.type_id, 0).len();
                    for offset in (0..).take(count) {
                        self.emit(Instruction::Move { to: base + offset, from: from + offset });
                    }
                }
            }
            _ => {
                let record = self.reading(value);
                self.place = value.place;
                self.emit(Instruction::UnpackRecord { base, record });
            }
        }
    }

    fn field_index(&mut self, type_id: TypeId, field: &str) -> u16 {
        let fields = self.fields_of(type_id, 0);
        if let Some(index) = fields.iter().position(|(name, _)| name == field) {
            u16::try_from(index).unwrap_or(0)
        } else {
            self.internal("a field its record doesn't have");
            0
        }
    }

    #[allow(clippy::too_many_lines)] // one case per kind of statement
    fn statement(&mut self, statement: &Statement) {
        self.place = statement.place;
        let held_mark = self.function.held.len();
        let register_mark = self.function.next;
        match &statement.kind {
            // In a run of plain statements, a field of a list's item is changed in the registers its view keeps.
            StatementKind::Inside { path, change } if self.function.views_on => {
                self.change_through_view(path, change);
            }
            // A plain field of a record at a place in a list, as `bodies[pos: i].vx = v`, is changed in one step.
            StatementKind::Inside { path, change }
                if !self.plain
                    && matches!(path.steps.as_slice(), [PathStep::Item { .. }])
                    && matches!(&change.kind, StatementKind::SetField { value, .. } if !self.is_reference(value.type_id)) =>
            {
                let ([PathStep::Item { index, zero_based }], StatementKind::SetField { variable, field, value }) =
                    (path.steps.as_slice(), &change.kind)
                else {
                    return;
                };
                let record_type = self.variable_type(*variable);
                let Ok(field) = u8::try_from(self.field_index(record_type, field)) else {
                    self.inside(path, |compiler| compiler.statement(change));
                    return;
                };
                for (hidden, hoisted) in &path.lets {
                    let register = self.held_value(hoisted);
                    self.function.locations.insert(*hidden, Location::Register(register));
                }
                let index = self.value(index);
                let value = self.value(value);
                let zero_based = *zero_based;
                self.place = statement.place;
                self.change_in_place(path.root, |compiler, list| {
                    compiler.emit(if zero_based {
                        Instruction::SetItemFieldAtOffset { list, index, field, value }
                    } else {
                        Instruction::SetItemFieldAtPosition { list, index, field, value }
                    });
                });
            }
            // trace: D62
            StatementKind::Inside { path, change } => self.inside(path, |compiler| compiler.statement(change)),
            StatementKind::Make { variable, value } => {
                let reference = self.is_reference(value.type_id);
                if let Some(&parameter) = self.function.takeovers.get(variable) {
                    let location = self.location(parameter);
                    self.function.locations.insert(*variable, location);
                    return;
                }
                if let Some(count) = self.keeps_in_registers(*variable) {
                    let base = self.allocate_many(count);
                    let temporaries = self.function.next;
                    self.unpack_into(value, base, true);
                    self.release_from(held_mark);
                    self.function.next = temporaries;
                    self.function.locations.insert(*variable, Location::InRegisters(base));
                    return;
                }
                if let Some(&global) = self.globals.get(variable) {
                    let register = self.held_value(value);
                    self.hand_on(register);
                    self.emit(Instruction::SetGlobal { global, from: register });
                    self.release_from(held_mark);
                    self.function.next = register_mark;
                } else {
                    let register = self.allocate();
                    let temporaries = self.function.next;
                    self.held_into(value, register);
                    self.release_from(held_mark);
                    self.function.next = temporaries;
                    self.function.locations.insert(*variable, Location::Register(register));
                    if matches!(self.kind(value.type_id), Type::Text) {
                        self.hold_text(register);
                    } else if reference {
                        self.hold(register);
                    }
                }
                return;
            }
            StatementKind::Set { variable, value } => self.set(*variable, value),
            StatementKind::SetField { variable, field, value } => {
                let record_type = self.variable_type(*variable);
                let field = self.field_index(record_type, field);
                if let Location::InRegisters(base) = self.location(*variable) {
                    self.value_into(value, base + field);
                    self.release_from(held_mark);
                    self.function.next = register_mark;
                    return;
                }
                let reference = self.is_reference(value.type_id);
                let value = self.held_value(value);
                self.hand_on(value);
                self.place = statement.place;
                self.change_in_place(*variable, |compiler, record| {
                    compiler.emit(if reference {
                        Instruction::SetReferenceField { record, field, value }
                    } else {
                        Instruction::SetField { record, field, value }
                    });
                });
            }
            StatementKind::SetItem { variable, index, zero_based, value } => {
                let index = self.value(index);
                if let ExpressionKind::Variable(record) = &value.kind
                    && let Location::InRegisters(base) = self.location(*record)
                {
                    // The record's fields go straight into the item, in place when the list alone holds it.
                    self.place = statement.place;
                    let zero_based = *zero_based;
                    self.change_in_place(*variable, |compiler, list| {
                        compiler.emit(if zero_based {
                            Instruction::PackIntoItemAtOffset { list, index, base }
                        } else {
                            Instruction::PackIntoItemAtPosition { list, index, base }
                        });
                    });
                    self.release_from(held_mark);
                    self.function.next = register_mark;
                    return;
                }
                let value = self.held_value(value);
                self.hand_on(value);
                self.place = statement.place;
                let zero_based = *zero_based;
                self.change_in_place(*variable, |compiler, list| {
                    compiler.emit(if zero_based {
                        Instruction::SetItemAtOffset { list, index, value }
                    } else {
                        Instruction::SetItemAtPosition { list, index, value }
                    });
                });
            }
            StatementKind::SetKey { variable, key, value }
                if matches!(self.location(*variable), Location::Register(_))
                    && matches!(&key.kind, ExpressionKind::Variable(read) if matches!(self.location(*read), Location::Register(_)))
                    && matches!(&key.kind, ExpressionKind::Variable(read) if lookups_and_other_uses(value, *variable, *read) == (1, 0)) =>
            {
                // `dict[key: k] = … dict[key: k] …` looks the key up once, for the read and the write together.
                let (Location::Register(dict), ExpressionKind::Variable(key_variable)) =
                    (self.location(*variable), &key.kind)
                else {
                    return;
                };
                let Location::Register(key_register) = self.location(*key_variable) else { return };
                let found = self.allocate_many(2);
                self.emit(Instruction::FindKey { to: found, dict, key: key_register });
                self.function.found = Some((*variable, *key_variable, found));
                // The new value is worked out straight into the register after the one saying where the key is.
                self.held_into(value, found + 1);
                self.function.found = None;
                self.place = statement.place;
                self.emit(Instruction::SetFound { dict, found, key: key_register });
            }
            StatementKind::SetKey { variable, key, value } => {
                let key = self.value(key);
                let value = self.held_value(value);
                self.hand_on(value);
                self.place = statement.place;
                self.change_in_place(*variable, |compiler, dict| {
                    compiler.emit(Instruction::SetKey { dict, key, value });
                });
            }
            StatementKind::Evaluate(expression) => {
                self.value(expression);
            }
            StatementKind::If { branches, otherwise } if self.guard_return(branches) => {
                // The guard and its `return` were one instruction; what's left is the rest of the `if`.
                if let Some(body) = otherwise {
                    self.block(body);
                }
            }
            StatementKind::If { branches, otherwise } => {
                let mut ends = Vec::new();
                let count = branches.len();
                for (position, (condition, body)) in branches.iter().enumerate() {
                    let skips = self.condition(condition, false);
                    self.block(body);
                    let is_last = position + 1 == count && otherwise.is_none();
                    if !is_last {
                        ends.push(self.emit(Instruction::Jump { offset: 0 }));
                    }
                    let here = self.here();
                    self.patch_all(&skips, here);
                }
                if let Some(body) = otherwise {
                    self.block(body);
                }
                let here = self.here();
                self.patch_all(&ends, here);
            }
            StatementKind::While { condition, body } => {
                // The check sits at the bottom, so each pass costs one jump.
                let enter = self.emit(Instruction::Jump { offset: 0 });
                let top = self.here();
                self.function.loops.push(Loop { held_mark: self.function.held.len(), breaks: Vec::new() });
                self.block(body);
                let check = self.here();
                self.patch(enter, check);
                self.place = condition.place;
                let again = self.condition(condition, true);
                self.patch_all(&again, top);
                let finished = self.function.loops.pop();
                let here = self.here();
                self.patch_all(&finished.map(|finished| finished.breaks).unwrap_or_default(), here);
            }
            StatementKind::ForRange { item, position, from, end, inclusive, step, body } => {
                let base = self.allocate_many(3);
                self.value_into(from, base);
                self.value_into(end, base + 1);
                match step {
                    Some(step) => {
                        self.value_into(step, base + 2);
                    }
                    None => {
                        self.emit(Instruction::LoadSmall { to: base + 2, value: 1 });
                    }
                }
                self.place = statement.place;
                self.emit(if *inclusive {
                    Instruction::PrepareRangeTo { base }
                } else {
                    Instruction::PrepareRangeBefore { base }
                });
                let item_register = self.allocate();
                self.function.locations.insert(*item, Location::Register(item_register));
                let counter = position.map(|(variable, zero_based)| {
                    let counter = self.allocate();
                    self.emit(Instruction::LoadSmall { to: counter, value: if zero_based { -1 } else { 0 } });
                    self.function.locations.insert(variable, Location::Register(counter));
                    counter
                });
                let enter = self.emit(Instruction::Jump { offset: 0 });
                let top = self.here();
                if let Some(counter) = counter {
                    self.emit(Instruction::AddWholeSmall { to: counter, left: counter, right: 1 });
                }
                self.function.loops.push(Loop { held_mark: self.function.held.len(), breaks: Vec::new() });
                // A range counting up from past a name that can't change never reaches that name's number.
                let apart = self.counts_up_from_past(from, step.as_deref());
                if let Some(below) = apart {
                    self.function.distinct.push((below, *item));
                }
                self.block(body);
                if apart.is_some() {
                    self.function.distinct.pop();
                }
                let next = self.emit(Instruction::NextInRange { to: item_register, base, offset: 0 });
                self.patch(enter, next);
                self.patch(next, top);
                let finished = self.function.loops.pop();
                let here = self.here();
                self.patch_all(&finished.map(|finished| finished.breaks).unwrap_or_default(), here);
            }
            StatementKind::ForEach { item, position, source, body } => self.for_each(*item, *position, source, body),
            StatementKind::Match { subject, arms } => {
                let subject_type = subject.type_id;
                let subject_register = self.value(subject);
                let number = if self.is_reference(subject_type) {
                    let number = self.allocate();
                    self.emit(Instruction::CaseNumber { to: number, from: subject_register });
                    number
                } else {
                    subject_register
                };
                let mut ends = Vec::new();
                let count = arms.len();
                for (position, arm) in arms.iter().enumerate() {
                    let is_last = position + 1 == count;
                    let skip = if is_last {
                        None
                    } else {
                        let case = i16::try_from(arm.case).unwrap_or(i16::MAX);
                        Some(self.emit(Instruction::JumpIfNotEqualWholeSmall { left: number, right: case, offset: 0 }))
                    };
                    let registers = self.function.next;
                    for &(field, variable) in &arm.values {
                        let register = self.allocate();
                        self.emit(Instruction::GetCaseValue {
                            to: register,
                            case: subject_register,
                            field: u16::try_from(field).unwrap_or(0),
                        });
                        self.function.locations.insert(variable, Location::Register(register));
                    }
                    self.block(&arm.body);
                    self.function.next = registers;
                    if !is_last {
                        ends.push(self.emit(Instruction::Jump { offset: 0 }));
                    }
                    if let Some(skip) = skip {
                        let here = self.here();
                        self.patch(skip, here);
                    }
                }
                let here = self.here();
                self.patch_all(&ends, here);
            }
            StatementKind::Break => {
                let Some(mark) = self.function.loops.last().map(|inner| inner.held_mark) else {
                    self.internal("a `break` outside a loop");
                    return;
                };
                self.release_on_exit(mark, None);
                let jump = self.emit(Instruction::Jump { offset: 0 });
                if let Some(inner) = self.function.loops.last_mut() {
                    inner.breaks.push(jump);
                }
                return;
            }
            StatementKind::Return(None) => {
                self.release_on_exit(0, None);
                self.emit(Instruction::ReturnNothing);
                return;
            }
            StatementKind::Return(Some(value)) => {
                let reference = self.is_reference(value.type_id);
                // A plain value an instruction works out is worked out straight into the first register, where the
                // value a procedure gives back goes, so it returns with nothing copied: `ReturnNothing` leaves that
                // register as it is.
                if !reference && !matches!(value.kind, ExpressionKind::Variable(_)) {
                    // `return a + b` with two `Whole`s, neither written as a number, adds and returns in one
                    // instruction.
                    if let ExpressionKind::Binary { operator: BinaryOperator::Add, left, right, .. } = &value.kind
                        && matches!(self.kind(left.type_id), Type::Whole)
                        && self.number_kind(left.type_id).is_none()
                        && !matches!(left.kind, ExpressionKind::Whole(_))
                        && !matches!(right.kind, ExpressionKind::Whole(_))
                        && fold(value).is_none()
                    {
                        let (left, right) = (self.value(left), self.value(right));
                        self.release_on_exit(0, None);
                        self.place = statement.place;
                        let index = u32::try_from(self.here()).unwrap_or(u32::MAX);
                        self.out.fused_places.push((index, value.place));
                        self.emit(Instruction::ReturnAddWhole { left, right });
                        self.function.held.truncate(held_mark);
                        return;
                    }
                    self.value_into(value, 0);
                    self.release_on_exit(0, None);
                    self.place = statement.place;
                    self.emit(Instruction::ReturnNothing);
                    self.function.held.truncate(held_mark);
                    return;
                }
                let register = self.value(value);
                if reference && !self.function.held.contains(&register) {
                    // A borrowed value, such as a parameter, gets a holder of its own before it leaves.
                    self.emit(Instruction::Retain { register });
                    self.release_on_exit(0, None);
                } else {
                    self.release_on_exit(0, reference.then_some(register));
                }
                self.place = statement.place;
                self.emit(Instruction::Return { from: register });
                self.function.held.truncate(held_mark);
                return;
            }
            StatementKind::Fail(failure) => {
                self.release_on_exit(0, None);
                self.emit(Instruction::Fail { failure: failure.0 });
                return;
            }
            StatementKind::Use(value) => {
                let Some(&target) = self.function.use_targets.last() else {
                    self.internal("an `instead` outside an `otherwise`");
                    return;
                };
                self.held_into(value, target);
            }
        }
        self.release_from(held_mark);
        self.function.next = register_mark;
    }

    /// `name = value` for a name that exists (D2).
    fn set(&mut self, variable: VariableId, value: &Expression) {
        let reference = self.is_reference(value.type_id);
        // `text = text & more` joins onto the text in place, as long as nothing else it joins reads the name.
        if let ExpressionKind::Join(parts) = &value.kind
            && let Some((first, rest)) = parts.split_first()
            && matches!(first.kind, ExpressionKind::Variable(first) if first == variable)
            && !rest.iter().any(|part| mentions(part, variable))
        {
            let pieces = self.pieces(rest);
            self.place = value.place;
            let site = self.join_site(&pieces);
            self.change_in_place(variable, |compiler, text| {
                compiler.emit(Instruction::AppendPieces { to: text, site });
            });
            return;
        }
        // trace: D15, Building rule 5. `name = name op value op value …` on a big number changes the name's number
        // where it is, one operation at a time, when nothing else holds it, as long as no value reads the name.
        if !self.plain
            && let Some(steps) = self.update_steps(variable, value)
            && matches!(self.location(variable), Location::Register(_) | Location::Global(_))
        {
            self.change_in_place(variable, |compiler, target| {
                // Each step points at its own maths, as it would were it worked out on its own (D17).
                for (operation, right, place) in steps {
                    let (right, whole) = match compiler.whole_operand(right) {
                        Some(register) => (register, number_operation::WHOLE_RIGHT),
                        None => (compiler.value(right), 0),
                    };
                    compiler.place = place;
                    compiler.emit(Instruction::NumberUpdate { target, right, operation: operation + whole });
                }
            });
            return;
        }
        // `name = procedure(name)` hands the name's value over to the call and takes the call's value back, so
        // nothing is copied on the way in or out.
        if reference
            && let Location::Register(register) = self.location(variable)
            && self.function.held.contains(&register)
            && let ExpressionKind::Call { procedure, arguments, .. } = &value.kind
            && self.inlined_body(procedure.0).is_none()
            && arguments
                .iter()
                .filter(|argument| matches!(argument.kind, ExpressionKind::Variable(read) if read == variable))
                .count()
                == 1
            && arguments
                .iter()
                .filter(|argument| !matches!(argument.kind, ExpressionKind::Variable(_)))
                .all(|argument| !expression_uses(argument, variable))
        {
            self.function.moved_name = Some(variable);
            self.function.moved_register = Some(register);
            let result = self.allocate();
            self.value_into(value, result);
            self.function.moved_name = None;
            self.function.moved_register = None;
            self.emit(Instruction::Move { to: register, from: result });
            return;
        }
        match self.location(variable) {
            Location::InRegisters(base) => self.unpack_into(value, base, false),
            Location::Register(register) => {
                if reference {
                    let new = self.held_value(value);
                    self.hand_on(new);
                    self.emit(Instruction::Release { register });
                    self.emit(Instruction::Move { to: register, from: new });
                } else {
                    self.value_into(value, register);
                }
            }
            Location::Global(global) => {
                if reference {
                    let new = self.held_value(value);
                    self.hand_on(new);
                    let old = self.allocate();
                    self.emit(Instruction::GetGlobal { to: old, global });
                    self.emit(Instruction::SetGlobal { global, from: new });
                    self.emit(Instruction::Release { register: old });
                } else {
                    let register = self.value(value);
                    self.emit(Instruction::SetGlobal { global, from: register });
                }
            }
        }
    }

    /// Works out the pieces a text is joined from. A piece written in the script as text stays a constant.
    fn pieces(&mut self, parts: &[Expression]) -> Vec<Piece> {
        parts
            .iter()
            .map(|part| match &part.kind {
                ExpressionKind::Text(text) => Piece::Constant(self.text(text)),
                ExpressionKind::ShowPlaces { value, places } => {
                    let decimal = self.number_kind(value.type_id).is_some();
                    Piece::Places(self.value(value), *places, decimal)
                }
                _ => Piece::Register(self.value(part), part.type_id),
            })
            .collect()
    }

    /// Records the pieces of one join, giving the join's number.
    fn join_site(&mut self, pieces: &[Piece]) -> u32 {
        let site = u32::try_from(self.out.joins.len()).unwrap_or(u32::MAX);
        let pieces = pieces
            .iter()
            .map(|piece| match *piece {
                Piece::Constant(text) => JoinPiece::Constant(text),
                Piece::Places(register, places, false) => JoinPiece::Places(register, places),
                Piece::Places(register, places, true) => JoinPiece::NumberPlaces(register, places),
                // A number with a unit shows with its unit (D100, D112).
                Piece::Register(register, piece_type) if self.program.types.measured(piece_type).is_some() => {
                    JoinPiece::Value(register, self.program.types.resolve(piece_type).0)
                }
                Piece::Register(register, piece_type) => match self.kind(piece_type) {
                    Type::Whole => JoinPiece::Whole(register),
                    Type::Bool => JoinPiece::Bool(register),
                    Type::Float => JoinPiece::Float(register),
                    Type::Text => JoinPiece::Text(register),
                    _ => JoinPiece::Value(register, self.program.types.resolve(piece_type).0),
                },
            })
            .collect();
        self.out.joins.push(pieces);
        site
    }

    /// trace: D33, Unio D171, D173, D174
    fn for_each(
        &mut self,
        item: VariableId,
        position: Option<(VariableId, PositionKind)>,
        source: &Expression,
        body: &Block,
    ) {
        let is_dict = matches!(self.kind(source.type_id), Type::Dict { .. });
        // The source and how far through it the loop is, side by side.
        let base = self.allocate_many(2);
        if matches!(self.kind(source.type_id), Type::Text) {
            // trace: D66. A text is gone through by graphemes: they're listed once, and the loop goes through the list.
            let text = self.value(source);
            self.place = source.place;
            self.emit(Instruction::Graphemes { to: base, from: text });
            self.hold(base);
        } else if self.value_into(source, base) {
            self.hold(base);
        }
        self.emit(Instruction::LoadSmall { to: base + 1, value: 0 });
        // The item, and its key when it's a dictionary's, side by side.
        let item_register = self.allocate_many(2);
        self.function.locations.insert(item, Location::Register(item_register));
        let mut offset_counter = None;
        match position {
            Some((variable, PositionKind::Key)) => {
                self.function.locations.insert(variable, Location::Register(item_register + 1));
            }
            // How many items are done is the position of the one in hand, counting from 1.
            Some((variable, PositionKind::Pos)) => {
                self.function.locations.insert(variable, Location::Register(base + 1));
            }
            Some((variable, PositionKind::Offset)) => {
                let counter = self.allocate();
                self.function.locations.insert(variable, Location::Register(counter));
                offset_counter = Some(counter);
            }
            None => {}
        }
        let enter = self.emit(Instruction::Jump { offset: 0 });
        let top = self.here();
        if let Some(counter) = offset_counter {
            self.emit(Instruction::AddWholeSmall { to: counter, left: base + 1, right: -1 });
        }
        self.function.loops.push(Loop { held_mark: self.function.held.len(), breaks: Vec::new() });
        self.block(body);
        self.place = source.place;
        let next = self.emit(if is_dict {
            Instruction::NextEntry { to: item_register, base, offset: 0 }
        } else {
            Instruction::NextItem { to: item_register, base, offset: 0 }
        });
        self.patch(enter, next);
        self.patch(next, top);
        let finished = self.function.loops.pop();
        let here = self.here();
        self.patch_all(&finished.map(|finished| finished.breaks).unwrap_or_default(), here);
    }

    // ---- conditions ----

    /// Emits a check of `condition` that jumps when it comes out as `jump_when`, and carries on otherwise. Gives
    /// the jumps to point at their target.
    fn condition(&mut self, condition: &Expression, jump_when: bool) -> Vec<usize> {
        match &condition.kind {
            ExpressionKind::Bool(value) => {
                if *value == jump_when {
                    vec![self.emit(Instruction::Jump { offset: 0 })]
                } else {
                    Vec::new()
                }
            }
            ExpressionKind::Not(inner) => self.condition(inner, !jump_when),
            ExpressionKind::Binary { operator: BinaryOperator::And, left, right } => {
                if jump_when {
                    let skip = self.condition(left, false);
                    let jumps = self.condition(right, true);
                    let here = self.here();
                    self.patch_all(&skip, here);
                    jumps
                } else {
                    let mut jumps = self.condition(left, false);
                    jumps.extend(self.condition(right, false));
                    jumps
                }
            }
            ExpressionKind::Binary { operator: BinaryOperator::Or, left, right } => {
                if jump_when {
                    let mut jumps = self.condition(left, true);
                    jumps.extend(self.condition(right, true));
                    jumps
                } else {
                    let skip = self.condition(left, true);
                    let jumps = self.condition(right, false);
                    let here = self.here();
                    self.patch_all(&skip, here);
                    jumps
                }
            }
            // Two texts, or two bigger values, are compared while both are still held, so they take the plain way
            // below (D64, D95).
            ExpressionKind::Binary { operator, left, right }
                if operator.is_comparison() && !self.is_reference(left.type_id) =>
            {
                let held_mark = self.function.held.len();
                let register_mark = self.function.next;
                let instruction = self.comparison_jump(*operator, left, right, jump_when);
                // What the two sides held is freed before the jump; the numbers compared are already in hand.
                self.release_from(held_mark);
                self.place = condition.place;
                let jump = self.emit(instruction);
                self.function.next = register_mark;
                vec![jump]
            }
            _ => {
                let held_mark = self.function.held.len();
                let register_mark = self.function.next;
                let register = self.value(condition);
                self.release_from(held_mark);
                self.function.next = register_mark;
                let instruction = if jump_when {
                    Instruction::JumpIfTrue { register, offset: 0 }
                } else {
                    Instruction::JumpIfFalse { register, offset: 0 }
                };
                vec![self.emit(instruction)]
            }
        }
    }

    /// The one instruction that compares and jumps, for `left operator right` coming out as `jump_when`.
    fn comparison_jump(
        &mut self,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
        jump_when: bool,
    ) -> Instruction {
        use BinaryOperator::{AtLeast, AtMost, Equal, Greater, Less, NotEqual};
        // Jumping when a comparison is false is jumping when its opposite is true.
        let operator = if jump_when {
            operator
        } else {
            match operator {
                Equal => NotEqual,
                NotEqual => Equal,
                Less => AtLeast,
                AtLeast => Less,
                Greater => AtMost,
                _ => Greater,
            }
        };
        let offset = 0;
        if matches!(self.kind(left.type_id), Type::Float) {
            let left = self.value(left);
            let right = self.value(right);
            return match operator {
                Equal => Instruction::JumpIfEqualFloat { left, right, offset },
                NotEqual => Instruction::JumpIfNotEqualFloat { left, right, offset },
                Less => Instruction::JumpIfLessFloat { left, right, offset },
                AtMost => Instruction::JumpIfAtMostFloat { left, right, offset },
                Greater => Instruction::JumpIfLessFloat { left: right, right: left, offset },
                _ => Instruction::JumpIfAtMostFloat { left: right, right: left, offset },
            };
        }
        if let Some(small) = small_whole(right) {
            let left = self.value(left);
            let right = small;
            return match operator {
                Equal => Instruction::JumpIfEqualWholeSmall { left, right, offset },
                NotEqual => Instruction::JumpIfNotEqualWholeSmall { left, right, offset },
                Less => Instruction::JumpIfLessWholeSmall { left, right, offset },
                AtMost => Instruction::JumpIfAtMostWholeSmall { left, right, offset },
                Greater => Instruction::JumpIfGreaterWholeSmall { left, right, offset },
                _ => Instruction::JumpIfAtLeastWholeSmall { left, right, offset },
            };
        }
        let left = self.value(left);
        let right = self.value(right);
        match operator {
            Equal => Instruction::JumpIfEqualWhole { left, right, offset },
            NotEqual => Instruction::JumpIfNotEqualWhole { left, right, offset },
            Less => Instruction::JumpIfLessWhole { left, right, offset },
            AtMost => Instruction::JumpIfAtMostWhole { left, right, offset },
            Greater => Instruction::JumpIfLessWhole { left: right, right: left, offset },
            _ => Instruction::JumpIfAtMostWhole { left: right, right: left, offset },
        }
    }

    // ---- values ----

    /// Works out a value, giving the register it's in. A name's value is read where the name lives; anything
    /// worked out lands in a new register, and is held until its statement ends when it's a heap value.
    fn value(&mut self, expression: &Expression) -> Register {
        if let ExpressionKind::Variable(variable) = &expression.kind
            && let Location::Register(register) = self.location(*variable)
        {
            return register;
        }
        if let Some(register) = self.field_register(expression) {
            return register;
        }
        if let Some(register) = self.view_field(expression) {
            return register;
        }
        let register = self.allocate();
        if self.value_into(expression, register) {
            self.hold(register);
        }
        register
    }

    /// Works out a value into a new register, making sure the register has a holder's count of its own.
    fn held_value(&mut self, expression: &Expression) -> Register {
        let register = self.allocate();
        self.held_into(expression, register);
        if self.is_reference(expression.type_id) {
            self.hold(register);
        }
        register
    }

    /// Works out a value into `target` with a holder's count of its own, which the caller takes charge of.
    fn held_into(&mut self, expression: &Expression, target: Register) {
        if !self.value_into(expression, target) && self.is_reference(expression.type_id) {
            self.emit(Instruction::Retain { register: target });
        }
    }

    fn load_whole(&mut self, to: Register, value: i64) {
        if let Ok(small) = i32::try_from(value) {
            self.emit(Instruction::LoadSmall { to, value: small })
        } else {
            let constant = self.constant(value.cast_unsigned());
            self.emit(Instruction::LoadConstant { to, constant })
        };
    }

    /// Works out a value into `target`. Gives whether `target` now has a holder's count of its own, which the
    /// caller must take charge of; a borrowed value, such as a name's or a field's, has none.
    #[allow(clippy::too_many_lines)] // one case per kind of expression
    fn value_into(&mut self, expression: &Expression, target: Register) -> bool {
        self.place = expression.place;
        let reference = self.is_reference(expression.type_id);
        let to = target;
        match &expression.kind {
            ExpressionKind::Error => {
                self.internal("a refused expression");
                false
            }
            ExpressionKind::Whole(value) => {
                self.load_whole(to, *value);
                false
            }
            ExpressionKind::Float(value) => {
                let constant = self.constant(value.to_bits());
                self.emit(Instruction::LoadConstant { to, constant });
                false
            }
            ExpressionKind::Bool(value) => {
                self.emit(Instruction::LoadSmall { to, value: i32::from(*value) });
                false
            }
            ExpressionKind::Text(text) => {
                let text = self.text(text);
                self.emit(Instruction::LoadText { to, text });
                false
            }
            ExpressionKind::Variable(variable) => match self.location(*variable) {
                Location::Register(from) => {
                    if from != to {
                        self.emit(Instruction::Move { to, from });
                    }
                    false
                }
                Location::InRegisters(base) => {
                    // The whole record is wanted, so it's made into a heap value here.
                    let fields = self.fields_of(expression.type_id, 0);
                    let layout = self.fields_layout(0, &fields);
                    self.emit(Instruction::NewRecord { to, base, layout });
                    true
                }
                Location::Global(global) => {
                    self.emit(Instruction::GetGlobal { to, global });
                    if reference {
                        // Another task, or a procedure called meanwhile, may change the name; a holder's count of
                        // its own keeps this value as it is.
                        self.emit(Instruction::Retain { register: to });
                    }
                    reference
                }
            },
            ExpressionKind::Member { receiver, name } => {
                if let Some(from) = self.field_register(expression) {
                    if from != to {
                        self.emit(Instruction::Move { to, from });
                    }
                    return false;
                }
                if let Some(from) = self.view_field(expression) {
                    if from != to {
                        self.emit(Instruction::Move { to, from });
                    }
                    return false;
                }
                // A plain field of a record at a place in a list, as `bodies[pos: i].vx`, is read in one step.
                if let ExpressionKind::Item { receiver: list, index, zero_based } = &receiver.kind
                    && !self.plain
                    && matches!(self.kind(receiver.type_id), Type::Record(_))
                    && !matches!(self.kind(list.type_id), Type::Text)
                    && !self.is_reference(expression.type_id)
                    && let Ok(field) = u8::try_from(self.field_index(receiver.type_id, name))
                {
                    let index = self.value(index);
                    let list = self.reading(list);
                    self.place = expression.place;
                    self.emit(if *zero_based {
                        Instruction::ItemFieldAtOffset { to, list, index, field }
                    } else {
                        Instruction::ItemFieldAtPosition { to, list, index, field }
                    });
                    return false;
                }
                let receiver_type = receiver.type_id;
                let from = self.reading(receiver);
                self.place = expression.place;
                if matches!(self.kind(receiver_type), Type::Record(_)) {
                    let field = self.field_index(receiver_type, name);
                    self.emit(Instruction::GetField { to, record: from, field });
                } else if matches!(self.kind(receiver_type), Type::Text) {
                    // trace: D65, D66
                    let unit = match name.as_str() {
                        "len.ucp" => 1,
                        "len.bytes" => 2,
                        _ => 0,
                    };
                    self.emit(Instruction::TextLength { to, from, unit });
                } else {
                    self.emit(Instruction::Length { to, from });
                }
                false
            }
            // trace: D66. A text's grapheme at a place is a new text.
            ExpressionKind::Item { receiver, index, zero_based }
                if matches!(self.kind(receiver.type_id), Type::Text) =>
            {
                let index = self.value(index);
                let text = self.reading(receiver);
                self.place = expression.place;
                self.emit(if *zero_based {
                    Instruction::GraphemeAtOffset { to, text, index }
                } else {
                    Instruction::GraphemeAtPosition { to, text, index }
                });
                true
            }
            // trace: D62
            ExpressionKind::Inside { path, change } => {
                self.inside(path, |compiler| {
                    compiler.value_into(change, to);
                });
                false
            }
            // trace: D65
            ExpressionKind::Split { text, by, keep_empty } => {
                let by = self.value(by);
                let text = self.reading(text);
                self.place = expression.place;
                self.emit(if *keep_empty {
                    Instruction::SplitText { to, text, by }
                } else {
                    Instruction::SplitTextNoEmpty { to, text, by }
                });
                true
            }
            ExpressionKind::Item { receiver, index, zero_based } => {
                let index = self.value(index);
                let list = self.reading(receiver);
                self.place = expression.place;
                self.emit(if *zero_based {
                    Instruction::ItemAtOffset { to, list, index }
                } else {
                    Instruction::ItemAtPosition { to, list, index }
                });
                false
            }
            ExpressionKind::Lookup { dict, key }
                if self.function.found.is_some_and(|(found_dict, found_key, _)| {
                    matches!(dict.kind, ExpressionKind::Variable(read) if read == found_dict)
                        && matches!(key.kind, ExpressionKind::Variable(read) if read == found_key)
                }) =>
            {
                let Some((found_dict, _, found)) = self.function.found else { return false };
                if let Location::Register(dict) = self.location(found_dict) {
                    self.emit(Instruction::GetFound { to, dict, found });
                }
                false
            }
            ExpressionKind::Lookup { dict, key } => {
                let key = self.value(key);
                let dict = self.reading(dict);
                self.place = expression.place;
                self.emit(Instruction::GetKey { to, dict, key });
                false
            }
            ExpressionKind::Has { dict, key } => {
                let key = self.value(key);
                let dict = self.reading(dict);
                self.place = expression.place;
                self.emit(Instruction::HasKey { to, dict, key });
                false
            }
            ExpressionKind::Call { procedure, arguments, .. } if self.inlined_body(procedure.0).is_some() => {
                // A small procedure that only gives back one value is written out here, with no call at all.
                let program = self.program;
                let Some(callee) = program.procedure(*procedure) else { return false };
                let Some(body) = self.inlined_body(procedure.0) else { return false };
                let registers: Vec<Register> = arguments.iter().map(|argument| self.value(argument)).collect();
                // The body's last step writes its result; if that would land on an argument still being read, the
                // result goes through a register of its own.
                let target = if registers.contains(&to) { self.allocate() } else { to };
                let mut before = Vec::new();
                for (&parameter, &register) in callee.parameters.iter().zip(&registers) {
                    before.push((parameter, self.function.locations.insert(parameter, Location::Register(register))));
                }
                self.inlining.push(procedure.0);
                let held = self.value_into(body, target);
                self.inlining.pop();
                for (parameter, earlier) in before {
                    match earlier {
                        Some(location) => self.function.locations.insert(parameter, location),
                        None => self.function.locations.remove(&parameter),
                    };
                }
                if target != to {
                    self.emit(Instruction::Move { to, from: target });
                }
                held
            }
            ExpressionKind::Call { procedure, arguments, .. } => {
                let can_pause = self.program.procedure(*procedure).is_some_and(|procedure| procedure.can_pause);
                // A call whose value goes to the newest register, holding nothing to free, is made from that register,
                // so its value lands there with no move after: the callee writes its first register only as it
                // returns. Only for a call that can neither fail nor pause, with a plain value.
                let in_place =
                    self.program.procedure(*procedure).is_some_and(|callee| !callee.can_fail && !callee.can_pause)
                        && !reference
                        && !matches!(self.kind(expression.type_id), Type::Nothing)
                        && to != 0
                        && u32::from(to) + 1 == self.function.next
                        && !self.function.held.contains(&to);
                if let Some((number, from, add, maths)) = self.call_adding(*procedure, arguments) {
                    // The argument's maths and the call, as one instruction (see `call_adding`).
                    let base = if in_place {
                        self.allocate_many(1);
                        to
                    } else {
                        self.allocate_many(2)
                    };
                    self.place = expression.place;
                    let index = u32::try_from(self.here()).unwrap_or(u32::MAX);
                    self.out.fused_places.push((index, maths));
                    self.emit(Instruction::CallAdding { base, procedure: number, from, add });
                    if can_pause {
                        self.pause_map();
                    }
                    if !matches!(self.kind(expression.type_id), Type::Nothing) && base != to {
                        self.emit(Instruction::Move { to, from: base });
                    }
                    return reference;
                }
                let base = if in_place {
                    self.arguments_from(Some(to), arguments, false, self.function.moved_name)
                } else {
                    self.arguments(arguments, false, self.function.moved_name)
                };
                self.place = expression.place;
                self.emit(Instruction::Call { base, procedure: procedure.0 });
                if can_pause {
                    self.pause_map();
                }
                // trace: D165. A method that changes its value leaves it where its first argument went, and the
                // variable it was called on takes it back.
                if let Some(callee) = self.program.procedure(*procedure)
                    && self.changes_first(callee)
                    && let Some(ExpressionKind::Variable(lent)) = arguments.first().map(|argument| &argument.kind)
                {
                    self.take_back(*lent, base.saturating_add(1));
                }
                if !matches!(self.kind(expression.type_id), Type::Nothing) && base != to {
                    self.emit(Instruction::Move { to, from: base });
                }
                reference
            }
            ExpressionKind::DoorCall { door, arguments } => {
                let can_pause = self.doors.get(*door as usize).is_some_and(|door| door.can_pause);
                let first_argument = u32::try_from(self.out.door_arguments.len()).unwrap_or(u32::MAX);
                for argument in arguments {
                    let crossing = self.crossing(argument.type_id);
                    self.out.door_arguments.push(crossing);
                }
                let argument_count = u32::try_from(arguments.len()).unwrap_or(u32::MAX);
                let plain = self
                    .out
                    .door_arguments
                    .get(first_argument as usize..)
                    .unwrap_or(&[])
                    .iter()
                    .all(|crossing| !crossing.is_reference());
                let result = self.crossing(expression.type_id);
                let base = self.arguments(arguments, true, None);
                let site = u32::try_from(self.out.door_sites.len()).unwrap_or(u32::MAX);
                self.out.door_sites.push(DoorSite { door: *door, plain, first_argument, argument_count, result });
                self.place = expression.place;
                self.emit(Instruction::CallDoor { base, site });
                if can_pause {
                    self.pause_map();
                }
                if !matches!(self.kind(expression.type_id), Type::Nothing) && base != to {
                    self.emit(Instruction::Move { to, from: base });
                }
                reference
            }
            ExpressionKind::Convert { conversion, value } => {
                let value_type = value.type_id;
                let from = self.value(value);
                self.place = expression.place;
                match conversion {
                    Conversion::ToText => {
                        let site = self.join_site(&[Piece::Register(from, value_type)]);
                        self.emit(Instruction::JoinPieces { to, site });
                        true
                    }
                    Conversion::WholeToFloat => {
                        self.emit(Instruction::WholeToFloat { to, from });
                        false
                    }
                    // trace: D120, D122. Into another unit: an exact whole multiple stays a `Whole` multiplication,
                    // checked for overflow; anything else is scaled once, rounded once.
                    Conversion::Scale { size, from_whole, to_float } => {
                        if *from_whole && !*to_float && size.is_whole() && i64::try_from(size.numerator).is_ok() {
                            let factor = self.allocate();
                            self.load_whole(factor, i64::try_from(size.numerator).unwrap_or(1));
                            self.place = expression.place;
                            self.emit(Instruction::MultiplyWhole { to, left: from, right: factor });
                        } else {
                            let site = u16::try_from(self.out.scales.len()).unwrap_or(u16::MAX);
                            if site == u16::MAX {
                                self.too_big("unit conversions");
                            }
                            self.out.scales.push((*size, *from_whole));
                            self.emit(Instruction::Scale { to, from, site });
                        }
                        false
                    }
                    Conversion::TextToBool => {
                        let failure = (1u16..)
                            .zip(&self.program.failure_names)
                            .find(|(_, name)| *name == "not_a_bool")
                            .map_or(0, |(number, _)| number);
                        self.emit(Instruction::TextToBool { to, from, failure });
                        false
                    }
                    Conversion::TextToWhole => {
                        self.emit(Instruction::TextToWhole { to, from });
                        false
                    }
                    // trace: D73
                    Conversion::ListToLimit(limit) => {
                        let limit_register = self.allocate();
                        self.load_whole(limit_register, *limit);
                        self.place = expression.place;
                        self.emit(Instruction::FitList { to, from, limit: limit_register });
                        true
                    }
                    Conversion::FloatToWhole(rounding) => {
                        let rounding = match rounding {
                            Rounding::Floor => 0,
                            Rounding::Trunc => 1,
                            Rounding::Ceil => 2,
                            Rounding::Nearest => 3,
                            Rounding::NearestEven => 4,
                        };
                        self.emit(Instruction::FloatToWhole { to, from, rounding });
                        false
                    }
                    // trace: D115, D116, D126
                    Conversion::Number(conversion) => {
                        let failure = conversion.failure().map_or(0, |name| {
                            (1u16..)
                                .zip(&self.program.failure_names)
                                .find(|(_, known)| *known == name)
                                .map_or(0, |(number, _)| number)
                        });
                        let site = u16::try_from(self.out.number_conversions.len()).unwrap_or(u16::MAX);
                        if site == u16::MAX {
                            self.too_big("number conversions");
                        }
                        self.out.number_conversions.push((*conversion, failure));
                        self.emit(Instruction::ConvertNumber { to, from, site });
                        self.is_reference(expression.type_id)
                    }
                }
            }
            ExpressionKind::Number(number) => {
                let site = u16::try_from(self.out.numbers.len()).unwrap_or(u16::MAX);
                if site == u16::MAX {
                    self.too_big("numbers written out");
                }
                self.out.numbers.push(number.clone());
                self.emit(Instruction::LoadNumber { to, site });
                // Made when the script loads and kept as long as it lives, so it's borrowed, as a text is.
                false
            }
            ExpressionKind::Sqrt(value) if self.number_kind(expression.type_id).is_some() => {
                let from = self.value(value);
                self.place = expression.place;
                self.number_operation(number_operation::SQRT, expression.type_id, to, from, from);
                true
            }
            ExpressionKind::Negate(value) if self.number_kind(expression.type_id).is_some() => {
                let from = self.value(value);
                self.place = expression.place;
                self.number_operation(number_operation::NEGATE, expression.type_id, to, from, from);
                true
            }
            ExpressionKind::Maths { function, arguments, .. } if self.number_kind(expression.type_id).is_some() => {
                use crate::check::tree::MathsFunction as Function;
                let operation = match function {
                    Function::Abs => number_operation::ABS,
                    Function::Min => number_operation::MIN,
                    Function::Max => number_operation::MAX,
                    Function::Clamp => number_operation::CLAMP,
                    _ => number_operation::LERP,
                };
                let count = u32::try_from(arguments.len()).unwrap_or(1);
                let base = self.allocate_many(count);
                for (offset, argument) in (0u16..).zip(arguments) {
                    if self.value_into(argument, base.wrapping_add(offset)) {
                        self.hold(base.wrapping_add(offset));
                    }
                }
                self.place = expression.place;
                self.number_operation(operation, expression.type_id, to, base, base.wrapping_add(1));
                true
            }
            ExpressionKind::Sqrt(value) => {
                let from = self.value(value);
                self.place = expression.place;
                self.emit(Instruction::SquareRoot { to, from });
                false
            }
            ExpressionKind::Negate(value) => {
                let from = self.value(value);
                self.place = expression.place;
                self.emit(if matches!(self.kind(expression.type_id), Type::Float) {
                    Instruction::NegateFloat { to, from }
                } else {
                    Instruction::NegateWhole { to, from }
                });
                false
            }
            ExpressionKind::Not(value) => {
                let from = self.value(value);
                self.emit(Instruction::Not { to, from });
                false
            }
            ExpressionKind::Binary { operator, left, right } => {
                self.binary(*operator, left, right, expression, to);
                // Maths on `HugeWhole`s and decimals makes a new number, which is the caller's to let go of.
                self.number_kind(expression.type_id).is_some()
            }
            ExpressionKind::ShowPlaces { value, places } => {
                let decimal = self.number_kind(value.type_id).is_some();
                let from = self.value(value);
                self.place = expression.place;
                let site = self.join_site(&[Piece::Places(from, *places, decimal)]);
                self.emit(Instruction::JoinPieces { to, site });
                true
            }
            ExpressionKind::Join(parts) => {
                if let [only] = parts.as_slice()
                    && matches!(self.kind(only.type_id), Type::Text)
                {
                    return self.value_into(only, to);
                }
                // The pieces are worked out first, so that a piece that reads `to`'s old value still can.
                let pieces = self.pieces(parts);
                self.place = expression.place;
                let site = self.join_site(&pieces);
                self.emit(Instruction::JoinPieces { to, site });
                true
            }
            ExpressionKind::BuildRecord { fields, .. } => {
                let declared = self.fields_of(expression.type_id, 0);
                let layout = self.fields_layout(0, &declared);
                let base = self.build_fields(fields, declared.len());
                self.place = expression.place;
                self.emit(Instruction::NewRecord { to, base, layout });
                true
            }
            ExpressionKind::BuildCase { case, fields } => {
                if !reference {
                    // A `one_of` whose cases carry nothing is just its case's number.
                    self.emit(Instruction::LoadSmall { to, value: i32::try_from(*case).unwrap_or(0) });
                    return false;
                }
                let declared = self.fields_of(expression.type_id, *case);
                if declared.is_empty() {
                    self.emit(Instruction::LoadPlainCase { to, case: *case });
                    return false;
                }
                let layout = self.fields_layout(*case, &declared);
                let base = self.build_fields(fields, declared.len());
                self.place = expression.place;
                self.emit(Instruction::NewCase { to, base, layout });
                true
            }
            ExpressionKind::List(items) => {
                let Ok(count) = u16::try_from(items.len()) else {
                    self.too_big("lists written out with more than 65,000 items");
                    return false;
                };
                let references = items.first().is_some_and(|item| self.is_reference(item.type_id));
                let base = self.allocate_many(u32::from(count));
                for (item, slot) in items.iter().zip(base..) {
                    self.held_into(item, slot);
                    if references {
                        self.hold(slot);
                    }
                }
                for slot in (base..).take(items.len()) {
                    self.hand_on(slot);
                }
                self.place = expression.place;
                self.emit(if references {
                    Instruction::NewReferenceList { to, base, count }
                } else {
                    Instruction::NewList { to, base, count }
                });
                true
            }
            ExpressionKind::Repeat { value, count } => {
                let references = self.is_reference(value.type_id);
                let value = self.value(value);
                let count_register = self.allocate();
                self.load_whole(count_register, *count);
                self.place = expression.place;
                self.emit(if references {
                    Instruction::RepeatReferenceList { to, value, count: count_register }
                } else {
                    Instruction::RepeatList { to, value, count: count_register }
                });
                true
            }
            ExpressionKind::Dict(pairs) => {
                let (text_keys, reference_values) = match self.kind(expression.type_id) {
                    Type::Dict { key, value } => (matches!(self.kind(*key), Type::Text), self.is_reference(*value)),
                    _ => (false, false),
                };
                // Built in a register of its own, held while its pairs are worked out, then handed to `to`.
                let dict = self.allocate();
                self.emit(Instruction::NewDict {
                    to: dict,
                    shape: u16::from(text_keys) | (u16::from(reference_values) << 1),
                });
                self.hold(dict);
                for (key, value) in pairs {
                    let key = self.value(key);
                    let value = self.held_value(value);
                    self.hand_on(value);
                    self.place = expression.place;
                    self.emit(Instruction::SetKey { dict, key, value });
                }
                self.hand_on(dict);
                self.emit(Instruction::Move { to, from: dict });
                true
            }
            ExpressionKind::Try(inner) => {
                let held = self.value_into(inner, to);
                self.place = expression.place;
                let fine = self.emit(Instruction::JumpUnlessFailed { offset: 0 });
                self.release_on_exit(0, None);
                self.emit(Instruction::PassFailureUp);
                let here = self.here();
                self.patch(fine, here);
                held
            }
            ExpressionKind::Handle { call, fallback: HandleFallback::Value(fallback) }
                if !reference
                    && matches!(call.kind, ExpressionKind::Lookup { .. })
                    && matches!(
                        fallback.kind,
                        ExpressionKind::Whole(_) | ExpressionKind::Float(_) | ExpressionKind::Bool(_)
                    ) =>
            {
                // A plain fallback goes in first, and the key's value replaces it when the key is there: no failure
                // is raised only to be dealt with at once.
                let ExpressionKind::Lookup { dict, key } = &call.kind else { return false };
                let already_found = self.function.found.filter(|(found_dict, found_key, _)| {
                    matches!(dict.kind, ExpressionKind::Variable(read) if read == *found_dict)
                        && matches!(key.kind, ExpressionKind::Variable(read) if read == *found_key)
                });
                if let Some((found_dict, _, found)) = already_found
                    && let Location::Register(dict) = self.location(found_dict)
                {
                    self.value_into(fallback, to);
                    self.emit(Instruction::GetFoundIfAny { to, dict, found });
                    return false;
                }
                let key = self.value(key);
                let dict = self.reading(dict);
                // The dictionary and key are in hand before the fallback lands, in case `to` is one of them.
                let result = if to == key || to == dict { self.allocate() } else { to };
                self.value_into(fallback, result);
                self.place = expression.place;
                self.emit(Instruction::GetKeyIfAny { to: result, dict, key });
                if result != to {
                    self.emit(Instruction::Move { to, from: result });
                }
                false
            }
            ExpressionKind::Handle { call, fallback } => {
                let held = self.value_into(call, to);
                self.place = expression.place;
                let fine = self.emit(Instruction::JumpUnlessFailed { offset: 0 });
                match fallback {
                    HandleFallback::Value(value) => {
                        self.emit(Instruction::ClearFailure);
                        let held_mark = self.function.held.len();
                        let register_mark = self.function.next;
                        self.held_into(value, to);
                        self.release_from(held_mark);
                        self.function.next = register_mark;
                    }
                    HandleFallback::Ignore => {
                        self.emit(Instruction::ClearFailure);
                    }
                    HandleFallback::Crash => {
                        self.emit(Instruction::Crash);
                    }
                }
                self.merge_after_handling(fine, to, held, reference)
            }
            ExpressionKind::Otherwise { call, cases, .. } => {
                let held = self.value_into(call, to);
                self.place = expression.place;
                let fine = self.emit(Instruction::JumpUnlessFailed { offset: 0 });
                let mut ends = Vec::new();
                self.function.use_targets.push(to);
                for (failure, body) in cases {
                    let next = self.emit(Instruction::JumpUnlessFailureIs { failure: failure.0, offset: 0 });
                    self.emit(Instruction::ClearFailure);
                    self.block(body);
                    ends.push(self.emit(Instruction::Jump { offset: 0 }));
                    let here = self.here();
                    self.patch(next, here);
                }
                self.function.use_targets.pop();
                // Checking made sure the cases cover every failure the call can give.
                self.emit(Instruction::Unreachable);
                let here = self.here();
                self.patch_all(&ends, here);
                self.merge_after_handling(fine, to, held, reference)
            }
            // trace: D99. A range kept as a list is built at once, with its memory counted first.
            ExpressionKind::Range { from, end, inclusive, step } => {
                let base = self.allocate_many(3);
                self.value_into(from, base);
                self.value_into(end, base.wrapping_add(1));
                match step {
                    Some(step) => {
                        self.value_into(step, base.wrapping_add(2));
                    }
                    None => self.load_whole(base.wrapping_add(2), 1),
                }
                self.place = expression.place;
                self.emit(Instruction::MakeRange { to, base, inclusive: *inclusive });
                true
            }
            ExpressionKind::Maths { function, arguments, unit } => {
                use crate::check::tree::MathsFunction as Function;
                let float = matches!(self.kind(expression.type_id), Type::Float);
                let number = match function {
                    Function::Abs => u8::from(float),
                    Function::Min => 2 + u8::from(float),
                    Function::Max => 4 + u8::from(float),
                    Function::Clamp => 6 + u8::from(float),
                    // `degs` and `turns` have functions of their own, which work on the exact fraction of a turn (D113).
                    Function::Sin => [8, 15, 18][usize::from(*unit).min(2)],
                    Function::Cos => [9, 16, 19][usize::from(*unit).min(2)],
                    Function::Tan => [10, 17, 20][usize::from(*unit).min(2)],
                    Function::Asin => 11,
                    Function::Acos => 12,
                    Function::Atan2 => 13,
                    Function::Lerp => 14,
                };
                let count = u32::try_from(arguments.len()).unwrap_or(1);
                let base = self.allocate_many(count);
                for (offset, argument) in (0u16..).zip(arguments) {
                    self.value_into(argument, base.wrapping_add(offset));
                }
                self.place = expression.place;
                self.emit(Instruction::Maths { to, base, function: number });
                false
            }
            // trace: D107, D128. The value is checked, or pulled back, as it lands.
            ExpressionKind::Land { value, target } => {
                let mut held = self.value_into(value, to);
                // A `HugeWhole` pulled back to its edge is a new number in place of the old, which only an owner
                // can let go of (D115).
                if !held && self.program.types.limits(*target).is_some_and(|limits| limits.huge) {
                    self.emit(Instruction::Retain { register: to });
                    held = true;
                }
                self.check_limits(*target, to);
                held
            }
            ExpressionKind::MaybeCrash(maybe) => {
                let maybe = self.value(maybe);
                let number = self.allocate();
                self.place = expression.place;
                self.emit(Instruction::CaseNumber { to: number, from: maybe });
                self.emit(Instruction::CrashIfNone { number });
                self.emit(Instruction::GetCaseValue { to, case: maybe, field: 0 });
                if reference {
                    self.emit(Instruction::Retain { register: to });
                }
                reference
            }
            ExpressionKind::MaybeOr { maybe, fallback } => {
                let maybe = self.value(maybe);
                let number = self.allocate();
                self.place = expression.place;
                self.emit(Instruction::CaseNumber { to: number, from: maybe });
                let none = self.emit(Instruction::JumpIfEqualWholeSmall { left: number, right: 0, offset: 0 });
                self.emit(Instruction::GetCaseValue { to, case: maybe, field: 0 });
                if reference {
                    self.emit(Instruction::Retain { register: to });
                }
                let done = self.emit(Instruction::Jump { offset: 0 });
                let here = self.here();
                self.patch(none, here);
                let held_mark = self.function.held.len();
                let register_mark = self.function.next;
                self.held_into(fallback, to);
                self.release_from(held_mark);
                self.function.next = register_mark;
                let here = self.here();
                self.patch(done, here);
                reference
            }
            ExpressionKind::ListAdd { variable, value, position } => {
                let list_type = self.variable_type(*variable);
                let limit = match self.kind(list_type) {
                    Type::List { max, .. } => *max,
                    _ => None,
                };
                let item_reference = self.is_reference(value.type_id);
                let index = position.as_ref().map(|(index, zero_based)| (self.value(index), *zero_based));
                let place = expression.place;
                match limit {
                    None => {
                        let value = self.held_value(value);
                        self.hand_on(value);
                        self.place = place;
                        self.change_in_place(*variable, |compiler, list| {
                            compiler.emit(match index {
                                None => Instruction::AddItem { list, value },
                                Some((index, true)) => Instruction::InsertItemAtOffset { list, index, value },
                                Some((index, false)) => Instruction::InsertItemAtPosition { list, index, value },
                            });
                        });
                    }
                    Some(limit) => {
                        // The item stays held here until the list is known to have room for it (Unio D168).
                        let value = self.value(value);
                        let limit_register = self.allocate();
                        self.load_whole(limit_register, limit);
                        self.place = place;
                        self.change_in_place(*variable, |compiler, list| {
                            compiler.emit(Instruction::CheckRoom { list, limit: limit_register });
                            let full = compiler.emit(Instruction::JumpUnlessFailed { offset: 0 });
                            let skip = compiler.emit(Instruction::Jump { offset: 0 });
                            let here = compiler.here();
                            compiler.patch(full, here);
                            if item_reference {
                                compiler.emit(Instruction::Retain { register: value });
                            }
                            compiler.emit(match index {
                                None => Instruction::AddItem { list, value },
                                Some((index, true)) => Instruction::InsertItemAtOffset { list, index, value },
                                Some((index, false)) => Instruction::InsertItemAtPosition { list, index, value },
                            });
                            let here = compiler.here();
                            compiler.patch(skip, here);
                        });
                    }
                }
                false
            }
            ExpressionKind::ListRemove { variable, position, zero_based } => {
                let index = self.value(position);
                self.place = expression.place;
                let zero_based = *zero_based;
                self.change_in_place(*variable, |compiler, list| {
                    compiler.emit(if zero_based {
                        Instruction::RemoveItemAtOffset { list, index }
                    } else {
                        Instruction::RemoveItemAtPosition { list, index }
                    });
                });
                false
            }
            ExpressionKind::DictRemove { variable, key } => {
                let key = self.value(key);
                self.place = expression.place;
                self.change_in_place(*variable, |compiler, dict| {
                    compiler.emit(Instruction::RemoveKey { dict, key });
                });
                false
            }
        }
    }

    /// After a call's failure was handled: both ways, the fine one and the handled one, must leave `to` with a
    /// holder's count of its own when it's a heap value.
    fn merge_after_handling(&mut self, fine: usize, to: Register, held: bool, reference: bool) -> bool {
        if reference && !held {
            // The fine way gave a borrowed value, as a dictionary's lookup does; it gets a count like the fallback's.
            let done = self.emit(Instruction::Jump { offset: 0 });
            let here = self.here();
            self.patch(fine, here);
            self.emit(Instruction::Retain { register: to });
            let here = self.here();
            self.patch(done, here);
        } else {
            let here = self.here();
            self.patch(fine, here);
        }
        reference
    }

    /// The register a value can be read from for one instruction. A top-level name's own value is read where it
    /// is, with no count taken, since nothing else runs before the instruction that reads it.
    fn reading(&mut self, expression: &Expression) -> Register {
        if let ExpressionKind::Variable(variable) = &expression.kind
            && let Location::Global(global) = self.location(*variable)
        {
            let to = self.allocate();
            self.emit(Instruction::GetGlobal { to, global });
            return to;
        }
        self.value(expression)
    }

    /// Works out a record's or a case's values and lines them up in declared order, ready to build from.
    fn build_fields(&mut self, fields: &[(u32, Expression)], declared: usize) -> Register {
        let count = u32::try_from(declared).unwrap_or(u32::MAX);
        let in_order = fields.iter().zip(0u32..).all(|((field, _), position)| *field == position);
        if in_order {
            let base = self.allocate_many(count);
            for (slot, (_, value)) in (base..).zip(fields) {
                self.held_into(value, slot);
                if self.is_reference(value.type_id) {
                    self.hold(slot);
                }
            }
            for slot in (base..).take(fields.len()) {
                self.hand_on(slot);
            }
            return base;
        }
        // Written in another order than declared: worked out as written, then moved into place.
        let worked: Vec<(u32, Register)> =
            fields.iter().map(|(field, value)| (*field, self.held_value(value))).collect();
        let base = self.allocate_many(count);
        for (field, register) in worked {
            self.hand_on(register);
            self.emit(Instruction::Move { to: base + Register::try_from(field).unwrap_or(0), from: register });
        }
        base
    }

    /// The one value a procedure gives back, when the procedure is small enough to be written out in place of a
    /// call to it: its whole body is `return value`, it can't fail or pause, and it isn't already being written
    /// out, as a procedure that calls itself would be.
    /// trace: D165. Whether a procedure is a method whose `mut` first parameter stands for the variable it's called on.
    fn changes_first(&self, procedure: &Procedure) -> bool {
        procedure.method
            && procedure
                .parameters
                .first()
                .and_then(|&first| self.program.variable(first))
                .is_some_and(|first| first.mutable)
    }

    /// trace: D165. Gives a variable lent to a method the value the method left in `from`, freeing the one it had.
    fn take_back(&mut self, variable: VariableId, from: Register) {
        match self.location(variable) {
            Location::Register(register) => {
                if self.is_reference(self.variable_type(variable)) {
                    self.emit(Instruction::Release { register });
                }
                self.emit(Instruction::Move { to: register, from });
            }
            Location::Global(global) => {
                let old = self.allocate();
                self.emit(Instruction::GetGlobal { to: old, global });
                self.emit(Instruction::SetGlobal { global, from });
                if self.is_reference(self.variable_type(variable)) {
                    self.emit(Instruction::Release { register: old });
                }
            }
            Location::InRegisters(base) => {
                // A record kept field by field in registers is unpacked from the value given back, then freed.
                self.emit(Instruction::UnpackRecord { base, record: from });
                self.emit(Instruction::Release { register: from });
            }
        }
    }

    fn inlined_body(&self, procedure: u32) -> Option<&'p Expression> {
        let program: &'p Program = self.program;
        let found = program.procedures.get(procedure as usize)?;
        if found.can_fail
            || found.can_pause
            || self.current == Some(procedure)
            || self.inlining.contains(&procedure)
            || self.changes_first(found)
        {
            return None;
        }
        // A ranged parameter is checked as the procedure starts (D128), which a call written out in place skips.
        if found.parameters.iter().any(|&parameter| {
            program.variable(parameter).is_some_and(|variable| program.types.limits(variable.type_id).is_some())
        }) {
            return None;
        }
        let [Statement { kind: StatementKind::Return(Some(body)), .. }] = found.body.as_slice() else { return None };
        (size(body) <= 48).then_some(body)
    }

    /// `if name < 2` and the like, a `Whole` against a small number, with a body that's only `return` of a name's plain
    /// value, nothing held to free: emitted as one `ReturnIfWholeSmall`, and true. False, emitting nothing, for any
    /// other `if`.
    fn guard_return(&mut self, branches: &[(Expression, Block)]) -> bool {
        use BinaryOperator::{AtLeast, AtMost, Equal, Greater, Less, NotEqual};
        let [(condition, body)] = branches else { return false };
        let [Statement { kind: StatementKind::Return(Some(value)), .. }] = body.as_slice() else { return false };
        let ExpressionKind::Variable(variable) = value.kind else { return false };
        let ExpressionKind::Binary { operator, left, right, .. } = &condition.kind else { return false };
        let compare = match operator {
            Equal => Compare::Equal,
            NotEqual => Compare::NotEqual,
            Less => Compare::Less,
            AtMost => Compare::AtMost,
            Greater => Compare::Greater,
            AtLeast => Compare::AtLeast,
            _ => return false,
        };
        let Some(small) = small_whole(right) else { return false };
        let plain_whole = matches!(self.kind(left.type_id), Type::Whole) && self.number_kind(left.type_id).is_none();
        if !plain_whole
            || self.is_reference(value.type_id)
            || !self.function.held.is_empty()
            || self.function.moved_register.is_some()
            || !matches!(self.function.locations.get(&variable), Some(Location::Register(_)))
            || !matches!(left.kind, ExpressionKind::Variable(compared)
                if matches!(self.function.locations.get(&compared), Some(Location::Register(_))))
        {
            return false;
        }
        // Both are names in registers, so reading them emits nothing, and the instruction keeps the place the
        // comparison's jump had, where a step that runs out is shown.
        let left = self.value(left);
        let from = self.value(value);
        self.emit(Instruction::ReturnIfWholeSmall { from, left, right: small, compare });
        true
    }

    /// A call's one argument written as a `Whole` plus or minus a small number, as `fib(n - 1)`: the procedure's
    /// number, the register the `Whole` is in, worked out here, the small number, and the maths' place. Nothing
    /// when the call isn't of that shape, so it's made as usual.
    fn call_adding(
        &mut self,
        procedure: crate::check::tree::ProcedureId,
        arguments: &[Expression],
    ) -> Option<(u16, Register, i8, Place)> {
        let [argument] = arguments else { return None };
        let number = u16::try_from(procedure.0).ok()?;
        let ExpressionKind::Binary { operator, left, right, .. } = &argument.kind else { return None };
        let plain_whole = matches!(self.kind(left.type_id), Type::Whole) && self.number_kind(left.type_id).is_none();
        if !plain_whole || fold(argument).is_some() {
            return None;
        }
        let small = small_whole(right)?;
        let add = match operator {
            BinaryOperator::Add => i8::try_from(small).ok()?,
            BinaryOperator::Subtract => i8::try_from(small.checked_neg()?).ok()?,
            _ => return None,
        };
        let from = self.value(left);
        Some((number, from, add, argument.place))
    }

    /// Lines a call's arguments up after the register its result comes back in, giving that register.
    ///
    /// A procedure takes its arguments over: each heap value it's given has a holder's count of its own, which the
    /// procedure frees when it's done (D15). `moved` is a name handed over whole, with no count taken, because its
    /// statement gives it a new value straight after. A door only reads its arguments, so they're lent to it.
    fn arguments(&mut self, arguments: &[Expression], to_door: bool, moved: Option<VariableId>) -> Register {
        self.arguments_from(None, arguments, to_door, moved)
    }

    /// The arguments, worked out into the registers after `base`, which is the newest register when it's given, and a
    /// new one when it isn't.
    fn arguments_from(
        &mut self,
        base: Option<Register>,
        arguments: &[Expression],
        to_door: bool,
        moved: Option<VariableId>,
    ) -> Register {
        let count = u32::try_from(arguments.len()).unwrap_or(u32::MAX);
        let held_before = self.function.held.len();
        let base = match base {
            Some(base) => {
                self.allocate_many(count);
                base
            }
            None => self.allocate_many(count + 1),
        };
        let mut given = Vec::new();
        for (slot, argument) in (base + 1..).zip(arguments) {
            let is_moved = matches!(&argument.kind, ExpressionKind::Variable(variable) if Some(*variable) == moved);
            if to_door || is_moved || !self.is_reference(argument.type_id) {
                if self.value_into(argument, slot) {
                    self.hold(slot);
                }
            } else {
                self.held_into(argument, slot);
                // Held here until the call is made, in case working out a later argument leaves the procedure.
                self.hold(slot);
                given.push(slot);
            }
        }
        if !to_door {
            for slot in given {
                self.hand_on(slot);
            }
            // The callee's frame starts at `base`, so what working the arguments out held is freed before the call.
            self.release_from(held_before);
        }
        base
    }

    /// Maths with a number written in the script on one side, as one instruction that reads it from the
    /// constants. Gives nothing when there's no such instruction for it.
    fn constant_maths(
        &mut self,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
        is_float: bool,
        to: Register,
    ) -> Option<Instruction> {
        use BinaryOperator::{Add, DivFloor, Divide, Multiply, RemFloor, Subtract};
        let constant = |compiler: &mut Self, bits: u64| u16::try_from(compiler.constant(bits)).ok();
        if is_float {
            let number = |expression: &Expression| match expression.kind {
                ExpressionKind::Float(value) => Some(value),
                _ => None,
            };
            return match (operator, number(left), number(right)) {
                (Add, None, Some(value)) => {
                    let constant = constant(self, value.to_bits())?;
                    Some(Instruction::AddFloatConstant { to, left: self.value(left), constant })
                }
                (Add, Some(value), None) => {
                    let constant = constant(self, value.to_bits())?;
                    Some(Instruction::AddFloatConstant { to, left: self.value(right), constant })
                }
                // `x - c` is `x + (-c)`, bit for bit.
                (Subtract, None, Some(value)) => {
                    let constant = constant(self, (-value).to_bits())?;
                    Some(Instruction::AddFloatConstant { to, left: self.value(left), constant })
                }
                (Multiply, None, Some(value)) => {
                    let constant = constant(self, value.to_bits())?;
                    Some(Instruction::MultiplyFloatConstant { to, left: self.value(left), constant })
                }
                (Multiply, Some(value), None) => {
                    let constant = constant(self, value.to_bits())?;
                    Some(Instruction::MultiplyFloatConstant { to, left: self.value(right), constant })
                }
                (Divide, None, Some(value)) if value != 0.0 => {
                    let constant = constant(self, value.to_bits())?;
                    Some(Instruction::DivideFloatConstant { to, left: self.value(left), constant })
                }
                (Divide, Some(value), None) => {
                    let constant = constant(self, value.to_bits())?;
                    Some(Instruction::DivideConstantByFloat { to, constant, right: self.value(right) })
                }
                _ => None,
            };
        }
        let number = |expression: &Expression| match expression.kind {
            ExpressionKind::Whole(value) => Some(value),
            _ => None,
        };
        match (operator, number(left), number(right)) {
            (Add, None, Some(value)) => {
                let constant = constant(self, value.cast_unsigned())?;
                Some(Instruction::AddWholeConstant { to, left: self.value(left), constant })
            }
            (Add, Some(value), None) => {
                let constant = constant(self, value.cast_unsigned())?;
                Some(Instruction::AddWholeConstant { to, left: self.value(right), constant })
            }
            (Multiply, None, Some(value)) => {
                let constant = constant(self, value.cast_unsigned())?;
                Some(Instruction::MultiplyWholeConstant { to, left: self.value(left), constant })
            }
            (Multiply, Some(value), None) => {
                let constant = constant(self, value.cast_unsigned())?;
                Some(Instruction::MultiplyWholeConstant { to, left: self.value(right), constant })
            }
            // Dividing by a plain number that's neither 0 nor -1 has no check left that can fail (D17).
            // A power of two divides by shifting, and leaves what the low bits hold.
            (DivFloor, None, Some(value)) if value > 1 && value.count_ones() == 1 => {
                let shift = u16::try_from(value.trailing_zeros()).ok()?;
                Some(Instruction::DivideFloorByPowerOfTwo { to, left: self.value(left), shift })
            }
            (RemFloor, None, Some(value)) if value > 1 && value.count_ones() == 1 => {
                let bits = u16::try_from(value.trailing_zeros()).ok()?;
                Some(Instruction::RemainderFloorByPowerOfTwo { to, left: self.value(left), bits })
            }
            (DivFloor, None, Some(value)) if value != 0 && value != -1 => {
                let constant = constant(self, value.cast_unsigned())?;
                Some(Instruction::DivideFloorConstant { to, left: self.value(left), constant })
            }
            (RemFloor, None, Some(value)) if value != 0 && value != -1 => {
                let constant = constant(self, value.cast_unsigned())?;
                Some(Instruction::RemainderFloorConstant { to, left: self.value(left), constant })
            }
            _ => None,
        }
    }

    /// trace: D95. Two bigger values are equal when their contents are.
    fn compare_contents(
        &mut self,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
        whole: &Expression,
        to: Register,
    ) {
        let compared = self.program.types.resolve(left.type_id).0;
        let (left, right) = (self.value(left), self.value(right));
        let pair = self.allocate_many(2);
        self.emit(Instruction::Move { to: pair, from: left });
        self.emit(Instruction::Move { to: pair.wrapping_add(1), from: right });
        let site = u16::try_from(self.out.compared_types.len()).unwrap_or(u16::MAX);
        if site == u16::MAX {
            return self.too_big("comparisons of bigger values");
        }
        self.out.compared_types.push(compared);
        self.place = whole.place;
        self.emit(Instruction::EqualValues { to, left: pair, site });
        if operator == BinaryOperator::NotEqual {
            self.emit(Instruction::Not { to, from: to });
        }
    }

    fn binary(
        &mut self,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
        whole: &Expression,
        to: Register,
    ) {
        if let Some(folded) = fold(whole) {
            match folded {
                Folded::Whole(value) => self.load_whole(to, value),
                Folded::Float(value) => {
                    let constant = self.constant(value.to_bits());
                    self.emit(Instruction::LoadConstant { to, constant });
                }
                Folded::Bool(value) => {
                    self.emit(Instruction::LoadSmall { to, value: i32::from(value) });
                }
            }
            return;
        }
        if matches!(operator, BinaryOperator::And | BinaryOperator::Or) {
            let skips = self.condition(whole, false);
            self.emit(Instruction::LoadSmall { to, value: 1 });
            let done = self.emit(Instruction::Jump { offset: 0 });
            let here = self.here();
            self.patch_all(&skips, here);
            self.emit(Instruction::LoadSmall { to, value: 0 });
            let here = self.here();
            self.patch(done, here);
            return;
        }
        if self.number_kind(left.type_id).is_some() {
            return self.number_binary(operator, left, right, whole, to);
        }
        let is_float = matches!(self.kind(left.type_id), Type::Float);
        if !is_float
            && matches!(operator, BinaryOperator::Add | BinaryOperator::Subtract)
            && let Some(small) = small_whole(right)
            && let Some(small) = if operator == BinaryOperator::Add { Some(small) } else { small.checked_neg() }
        {
            let left = self.value(left);
            self.place = whole.place;
            self.emit(Instruction::AddWholeSmall { to, left, right: small });
            return;
        }
        if let Some(instruction) = self.constant_maths(operator, left, right, is_float, to) {
            self.place = whole.place;
            self.emit(instruction);
            return;
        }
        if is_float && !self.plain && self.fused_floats(whole, to) {
            return;
        }
        let is_text = matches!(self.kind(left.type_id), Type::Text);
        if !is_text && self.is_reference(left.type_id) {
            return self.compare_contents(operator, left, right, whole, to);
        }
        let left = self.value(left);
        let right = self.value(right);
        self.place = whole.place;
        // trace: D64
        if is_text {
            match operator {
                BinaryOperator::Equal => self.emit(Instruction::EqualText { to, left, right }),
                BinaryOperator::NotEqual => self.emit(Instruction::NotEqualText { to, left, right }),
                _ => return self.internal("an operator that doesn't work on text"),
            };
            return;
        }
        let instruction = match (operator, is_float) {
            (BinaryOperator::Add, false) => Instruction::AddWhole { to, left, right },
            (BinaryOperator::Subtract, false) => Instruction::SubtractWhole { to, left, right },
            (BinaryOperator::Multiply, false) => Instruction::MultiplyWhole { to, left, right },
            (BinaryOperator::Divide, false) => Instruction::DivideWhole { to, left, right },
            (BinaryOperator::Add, true) => Instruction::AddFloat { to, left, right },
            (BinaryOperator::Subtract, true) => Instruction::SubtractFloat { to, left, right },
            (BinaryOperator::Multiply, true) => Instruction::MultiplyFloat { to, left, right },
            (BinaryOperator::Divide, true) => Instruction::DivideFloat { to, left, right },
            (BinaryOperator::Power, false) => Instruction::PowerWhole { to, left, right },
            (BinaryOperator::Power, true) => Instruction::PowerFloat { to, left, right },
            (BinaryOperator::DivFloor, _) => Instruction::DivideFloor { to, left, right },
            (BinaryOperator::DivTrunc, _) => Instruction::DivideTrunc { to, left, right },
            (BinaryOperator::RemFloor, _) => Instruction::RemainderFloor { to, left, right },
            (BinaryOperator::RemTrunc, _) => Instruction::RemainderTrunc { to, left, right },
            (BinaryOperator::Equal, false) => Instruction::EqualWhole { to, left, right },
            (BinaryOperator::NotEqual, false) => Instruction::NotEqualWhole { to, left, right },
            (BinaryOperator::Less, false) => Instruction::LessWhole { to, left, right },
            (BinaryOperator::AtMost, false) => Instruction::AtMostWhole { to, left, right },
            (BinaryOperator::Greater, false) => Instruction::LessWhole { to, left: right, right: left },
            (BinaryOperator::AtLeast, false) => Instruction::AtMostWhole { to, left: right, right: left },
            (BinaryOperator::Equal, true) => Instruction::EqualFloat { to, left, right },
            (BinaryOperator::NotEqual, true) => Instruction::NotEqualFloat { to, left, right },
            (BinaryOperator::Less, true) => Instruction::LessFloat { to, left, right },
            (BinaryOperator::AtMost, true) => Instruction::AtMostFloat { to, left, right },
            (BinaryOperator::Greater, true) => Instruction::LessFloat { to, left: right, right: left },
            (BinaryOperator::AtLeast, true) => Instruction::AtMostFloat { to, left: right, right: left },
            (BinaryOperator::Join | BinaryOperator::And | BinaryOperator::Or, _) => {
                self.internal("an operator in the wrong place");
                return;
            }
        };
        self.emit(instruction);
    }
}

/// A piece of a text being joined: a value in a register, or one of the script's own texts.
#[derive(Clone, Copy)]
enum Piece {
    Register(Register, TypeId),
    Constant(u32),
    /// A `Float` shown to so many digits after its point (D94), or a decimal when it says so (D125).
    Places(Register, u8, bool),
}

/// How many times an expression looks `key` up in `dict`, and how many other times it uses `dict` at all.
fn lookups_and_other_uses(expression: &Expression, dict: VariableId, key: VariableId) -> (usize, usize) {
    match &expression.kind {
        ExpressionKind::Lookup { dict: read, key: read_key }
            if matches!(read.kind, ExpressionKind::Variable(read) if read == dict)
                && matches!(read_key.kind, ExpressionKind::Variable(read) if read == key) =>
        {
            (1, 0)
        }
        ExpressionKind::Binary { left, right, .. } => {
            let (left_lookups, left_uses) = lookups_and_other_uses(left, dict, key);
            let (right_lookups, right_uses) = lookups_and_other_uses(right, dict, key);
            (left_lookups + right_lookups, left_uses + right_uses)
        }
        ExpressionKind::Handle { call, fallback: HandleFallback::Value(fallback) } => {
            let (lookups, uses) = lookups_and_other_uses(call, dict, key);
            (lookups, uses + usize::from(expression_uses(fallback, dict)))
        }
        ExpressionKind::Negate(inner) | ExpressionKind::Convert { value: inner, .. } => {
            lookups_and_other_uses(inner, dict, key)
        }
        _ => (0, usize::from(expression_uses(expression, dict))),
    }
}

/// Whether an expression reads or changes a name, anywhere inside it.
fn expression_uses(expression: &Expression, variable: VariableId) -> bool {
    let uses = |inner: &Expression| expression_uses(inner, variable);
    match &expression.kind {
        ExpressionKind::Inside { path, change } => inside_uses(path, variable) || uses(change),
        ExpressionKind::Variable(read) => *read == variable,
        ExpressionKind::Error
        | ExpressionKind::Whole(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Bool(_)
        | ExpressionKind::Number(_)
        | ExpressionKind::Text(_) => false,
        ExpressionKind::Member { receiver: inner, .. }
        | ExpressionKind::Convert { value: inner, .. }
        | ExpressionKind::Sqrt(inner)
        | ExpressionKind::MaybeCrash(inner)
        | ExpressionKind::Land { value: inner, .. }
        | ExpressionKind::ShowPlaces { value: inner, .. }
        | ExpressionKind::Negate(inner)
        | ExpressionKind::Not(inner)
        | ExpressionKind::Repeat { value: inner, .. }
        | ExpressionKind::Try(inner) => uses(inner),
        ExpressionKind::Item { receiver: first, index: second, .. }
        | ExpressionKind::Lookup { dict: first, key: second }
        | ExpressionKind::Has { dict: first, key: second }
        | ExpressionKind::Split { text: first, by: second, .. }
        | ExpressionKind::Binary { left: first, right: second, .. }
        | ExpressionKind::MaybeOr { maybe: first, fallback: second } => uses(first) || uses(second),
        ExpressionKind::Call { arguments: items, .. }
        | ExpressionKind::Maths { arguments: items, .. }
        | ExpressionKind::DoorCall { arguments: items, .. }
        | ExpressionKind::Join(items)
        | ExpressionKind::List(items) => items.iter().any(uses),
        ExpressionKind::BuildRecord { fields, .. } | ExpressionKind::BuildCase { fields, .. } => {
            fields.iter().any(|(_, value)| uses(value))
        }
        ExpressionKind::Dict(pairs) => pairs.iter().any(|(key, value)| uses(key) || uses(value)),
        ExpressionKind::Range { from, end, step, .. } => {
            uses(from) || uses(end) || step.as_ref().is_some_and(|step| uses(step))
        }
        ExpressionKind::Handle { call, fallback } => {
            uses(call) || matches!(fallback, HandleFallback::Value(value) if uses(value))
        }
        ExpressionKind::Otherwise { call, cases, .. } => {
            uses(call) || cases.iter().any(|(_, body)| block_uses(body, variable))
        }
        ExpressionKind::ListAdd { variable: changed, value, position } => {
            *changed == variable || uses(value) || position.as_ref().is_some_and(|(position, _)| uses(position))
        }
        ExpressionKind::ListRemove { variable: changed, position: inner, .. }
        | ExpressionKind::DictRemove { variable: changed, key: inner } => *changed == variable || uses(inner),
    }
}

/// Whether a way into a part reads or changes a name (D62).
fn inside_uses(path: &Inside, variable: VariableId) -> bool {
    path.root == variable
        || path.lets.iter().any(|(_, value)| expression_uses(value, variable))
        || path.steps.iter().any(|step| match step {
            PathStep::Item { index, .. } => expression_uses(index, variable),
            PathStep::Field(_) => false,
        })
}

/// Whether any statement of a block reads or changes a name.
fn block_uses(block: &[Statement], variable: VariableId) -> bool {
    let uses = |expression: &Expression| expression_uses(expression, variable);
    block.iter().any(|statement| match &statement.kind {
        StatementKind::Inside { path, change } => {
            inside_uses(path, variable) || block_uses(std::slice::from_ref(&**change), variable)
        }
        StatementKind::Make { value, .. } | StatementKind::Evaluate(value) | StatementKind::Use(value) => uses(value),
        StatementKind::Set { variable: changed, value } | StatementKind::SetField { variable: changed, value, .. } => {
            *changed == variable || uses(value)
        }
        StatementKind::SetItem { variable: changed, index: inner, value, .. }
        | StatementKind::SetKey { variable: changed, key: inner, value } => {
            *changed == variable || uses(inner) || uses(value)
        }
        StatementKind::If { branches, otherwise } => {
            branches.iter().any(|(condition, body)| uses(condition) || block_uses(body, variable))
                || otherwise.as_ref().is_some_and(|body| block_uses(body, variable))
        }
        StatementKind::While { condition, body } => uses(condition) || block_uses(body, variable),
        StatementKind::ForRange { from, end, step, body, .. } => {
            uses(from) || uses(end) || step.as_deref().is_some_and(uses) || block_uses(body, variable)
        }
        StatementKind::ForEach { source, body, .. } => uses(source) || block_uses(body, variable),
        StatementKind::Match { subject, arms } => {
            uses(subject) || arms.iter().any(|arm| block_uses(&arm.body, variable))
        }
        StatementKind::Return(value) => value.as_ref().is_some_and(uses),
        StatementKind::Break | StatementKind::Fail(_) => false,
    })
}

/// How many values an expression is made of, roughly: how much code writing it out costs.
fn size(expression: &Expression) -> usize {
    let mut total = 1;
    let mut add = |inner: &Expression| total += size(inner);
    match &expression.kind {
        ExpressionKind::Member { receiver: inner, .. }
        | ExpressionKind::Convert { value: inner, .. }
        | ExpressionKind::Sqrt(inner)
        | ExpressionKind::MaybeCrash(inner)
        | ExpressionKind::Land { value: inner, .. }
        | ExpressionKind::ShowPlaces { value: inner, .. }
        | ExpressionKind::Negate(inner)
        | ExpressionKind::Not(inner) => add(inner),
        ExpressionKind::Item { receiver: first, index: second, .. }
        | ExpressionKind::Binary { left: first, right: second, .. } => {
            add(first);
            add(second);
        }
        ExpressionKind::Call { arguments, .. } | ExpressionKind::Join(arguments) => {
            for argument in arguments {
                add(argument);
            }
        }
        ExpressionKind::Whole(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Bool(_)
        | ExpressionKind::Text(_)
        | ExpressionKind::Variable(_) => {}
        // Anything else is taken as too big to write out in place.
        _ => return 1000,
    }
    total
}

/// A plain whole number small enough to sit inside an instruction.
fn small_whole(expression: &Expression) -> Option<i16> {
    match expression.kind {
        ExpressionKind::Whole(value) => i16::try_from(value).ok().filter(|small| *small != FAR),
        _ => None,
    }
}

/// Whether an expression reads a name.
fn mentions(expression: &Expression, variable: VariableId) -> bool {
    let mut found = false;
    let mut look = |inner: &Expression| found |= mentions(inner, variable);
    match &expression.kind {
        ExpressionKind::Variable(read) => return *read == variable,
        ExpressionKind::Error
        | ExpressionKind::Whole(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Bool(_)
        | ExpressionKind::Number(_)
        | ExpressionKind::Text(_) => {}
        ExpressionKind::Member { receiver: inner, .. }
        | ExpressionKind::Convert { value: inner, .. }
        | ExpressionKind::Sqrt(inner)
        | ExpressionKind::MaybeCrash(inner)
        | ExpressionKind::Land { value: inner, .. }
        | ExpressionKind::ShowPlaces { value: inner, .. }
        | ExpressionKind::Negate(inner)
        | ExpressionKind::Not(inner)
        | ExpressionKind::Repeat { value: inner, .. }
        | ExpressionKind::Try(inner) => look(inner),
        ExpressionKind::Item { receiver: first, index: second, .. }
        | ExpressionKind::Lookup { dict: first, key: second }
        | ExpressionKind::Has { dict: first, key: second }
        | ExpressionKind::Split { text: first, by: second, .. }
        | ExpressionKind::Binary { left: first, right: second, .. }
        | ExpressionKind::MaybeOr { maybe: first, fallback: second } => {
            look(first);
            look(second);
        }
        ExpressionKind::Join(items) | ExpressionKind::List(items) => {
            for item in items {
                look(item);
            }
        }
        ExpressionKind::BuildRecord { fields, .. } | ExpressionKind::BuildCase { fields, .. } => {
            for (_, value) in fields {
                look(value);
            }
        }
        // Anything that calls or changes is taken to read the name, to stay on the safe side.
        _ => return true,
    }
    found
}

enum Folded {
    Whole(i64),
    Float(f64),
    Bool(bool),
}

/// Works out maths on plain numbers at load time, where no check can fail (D17). Anything that would fail a
/// check is left to fail while running, at its line.
fn fold(expression: &Expression) -> Option<Folded> {
    match &expression.kind {
        ExpressionKind::Whole(value) => Some(Folded::Whole(*value)),
        ExpressionKind::Float(value) => Some(Folded::Float(*value)),
        ExpressionKind::Bool(value) => Some(Folded::Bool(*value)),
        ExpressionKind::Binary { operator, left, right } => match (fold(left)?, fold(right)?) {
            (Folded::Whole(left), Folded::Whole(right)) => Some(match operator {
                BinaryOperator::Add => Folded::Whole(left.checked_add(right)?),
                BinaryOperator::Subtract => Folded::Whole(left.checked_sub(right)?),
                BinaryOperator::Multiply => Folded::Whole(left.checked_mul(right)?),
                BinaryOperator::Equal => Folded::Bool(left == right),
                BinaryOperator::NotEqual => Folded::Bool(left != right),
                BinaryOperator::Less => Folded::Bool(left < right),
                BinaryOperator::Greater => Folded::Bool(left > right),
                BinaryOperator::AtLeast => Folded::Bool(left >= right),
                BinaryOperator::AtMost => Folded::Bool(left <= right),
                _ => return None,
            }),
            (Folded::Float(left), Folded::Float(right)) => {
                let result = match operator {
                    BinaryOperator::Add => left + right,
                    BinaryOperator::Subtract => left - right,
                    BinaryOperator::Multiply => left * right,
                    BinaryOperator::Less => return Some(Folded::Bool(left < right)),
                    BinaryOperator::Greater => return Some(Folded::Bool(left > right)),
                    BinaryOperator::AtLeast => return Some(Folded::Bool(left >= right)),
                    BinaryOperator::AtMost => return Some(Folded::Bool(left <= right)),
                    _ => return None,
                };
                result.is_finite().then_some(Folded::Float(result))
            }
            _ => None,
        },
        _ => None,
    }
}

/// Whether a name is made by `for_each` or `match`, which only borrow what they name.
#[must_use]
pub fn borrows(kind: VariableKind) -> bool {
    !matches!(kind, VariableKind::Named)
}
