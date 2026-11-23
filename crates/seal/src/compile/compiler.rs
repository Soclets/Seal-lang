//! Compiling the checked tree to bytecode (D46).
//!
//! Who holds each heap value is worked out here (D15): a name made with `=` owns its value, a parameter, a loop's
//! item and a case's value borrow theirs, and a value worked out along the way is held until its statement ends.
//! The compiler emits `Retain` and `Release` so that every value is freed the moment its last holder ends.

use std::collections::HashMap;

use super::bytecode::{
    Compiled, CompiledParameter, CompiledProcedure, Crossing, CrossingCase, DoorSite, FAR, Instruction, Layout,
    Register,
};
use crate::check::tree::{
    Block, Conversion, Expression, ExpressionKind, HandleFallback, Procedure, Program, Rounding, Statement,
    StatementKind, VariableId, VariableKind,
};
use crate::check::types::{Type, TypeId};
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
    instead_targets: Vec<Register>,
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
}

/// Compiles a checked script.
///
/// # Errors
///
/// Returns a message when the script is past one of the implementation's sizes, or the checked tree isn't what the
/// compiler expects, which is a bug in Seal.
pub fn compile(program: &Program, doors: &[DoorSignature]) -> Result<Compiled, Message> {
    let mut compiler = Compiler {
        program,
        doors,
        out: Compiled {
            failure_names: program.failure_names.clone(),
            types: program.types.clone(),
            ..Compiled::default()
        },
        constant_ids: HashMap::new(),
        text_ids: HashMap::new(),
        layout_ids: HashMap::new(),
        globals: HashMap::new(),
        function: Function::default(),
        place: Place::default(),
        problem: None,
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
            "more than 65,000 top-level names used by procedures (D4) aren't",
        ));
    }
    compiler.out.global_count = u32::try_from(compiler.globals.len()).unwrap_or(u32::MAX);
    compiler.top_level();
    for procedure in &program.procedures {
        compiler.procedure(procedure);
    }
    match compiler.problem {
        Some(problem) => Err(problem),
        None => Ok(compiler.out),
    }
}

impl Compiler<'_> {
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

    fn kind(&self, type_id: TypeId) -> &Type {
        self.program.types.get(type_id)
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
        match self.kind(type_id).clone() {
            Type::Whole => Crossing::Whole,
            Type::Float => Crossing::Float,
            Type::Bool => Crossing::Bool,
            Type::Text => Crossing::Text,
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
            Type::Nothing | Type::Error | Type::Variable(_) => Crossing::Nothing,
        }
    }

    // ---- who holds what ----

    fn hold(&mut self, register: Register) {
        self.function.held.push(register);
    }

    /// Stops counting a register as held: its value has been handed on.
    fn hand_on(&mut self, register: Register) {
        if let Some(position) = self.function.held.iter().rposition(|&held| held == register) {
            self.function.held.remove(position);
        }
    }

    /// Frees everything held since `mark`, and forgets it.
    fn release_from(&mut self, mark: usize) {
        while self.function.held.len() > mark {
            if let Some(register) = self.function.held.pop() {
                self.emit(Instruction::Release { register });
            }
        }
    }

    /// Frees everything held since `mark` on a way out, such as `break` or `return`, leaving the list as it is for
    /// the code that follows on other ways.
    fn release_on_exit(&mut self, mark: usize, except: Option<Register>) {
        let held: Vec<Register> = self.function.held.get(mark..).unwrap_or(&[]).to_vec();
        for register in held.into_iter().rev() {
            if Some(register) != except {
                self.emit(Instruction::Release { register });
            }
        }
    }

    /// Records, for a call that can pause, what the frame holds there (D54).
    fn pause_map(&mut self) {
        let pc = u32::try_from(self.here()).unwrap_or(u32::MAX);
        self.out.pause_maps.insert(pc, self.function.held.clone());
    }

    // ---- procedures ----

    fn start_function(&mut self, first_free: u32) {
        self.function = Function { next: first_free, high_water: first_free, ..Function::default() };
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

    fn procedure(&mut self, procedure: &Procedure) {
        let parameter_count = u32::try_from(procedure.parameters.len()).unwrap_or(u32::MAX);
        self.start_function(parameter_count.saturating_add(1));
        self.place = procedure.place;
        let entry = u32::try_from(self.here()).unwrap_or(u32::MAX);
        let mut parameters = Vec::new();
        for (position, &parameter) in (0u32..).zip(&procedure.parameters) {
            let register = Register::try_from(position + 1).unwrap_or(0);
            self.function.locations.insert(parameter, Location::Register(register));
            let Some(variable) = self.program.variable(parameter) else { continue };
            let default =
                procedure.defaults.get(position as usize).cloned().flatten().and_then(|default| match default.kind {
                    ExpressionKind::Whole(value) => Some(Value::Whole(value)),
                    ExpressionKind::Float(value) => Some(Value::Float(value)),
                    ExpressionKind::Bool(value) => Some(Value::Bool(value)),
                    ExpressionKind::Text(value) => Some(Value::Text(value)),
                    _ => None,
                });
            parameters.push(CompiledParameter {
                name: variable.name.clone(),
                label: procedure.labels.get(position as usize).cloned().flatten(),
                crossing: self.crossing(variable.type_id),
                written: self.program.types.written(variable.type_id),
                default,
            });
        }
        self.block(&procedure.body);
        self.place = procedure.end_place;
        if matches!(self.kind(procedure.result), Type::Nothing) {
            self.emit(Instruction::ReturnNothing);
        } else {
            // Checking made sure every way through ends in a `return` or a `fail`.
            self.emit(Instruction::Unreachable);
        }
        let failures =
            procedure.failures.iter().map(|&failure| self.program.failure_name(failure).to_owned()).collect();
        let result = self.crossing(procedure.result);
        self.out.procedures.push(CompiledProcedure {
            name: procedure.name.clone(),
            entry,
            frame_size: self.function.high_water + 1,
            parameters,
            result,
            result_written: self.program.types.written(procedure.result),
            failures,
            can_pause: procedure.can_pause,
        });
    }

    // ---- statements ----

    fn block(&mut self, block: &Block) {
        let held_mark = self.function.held.len();
        let register_mark = self.function.next;
        for statement in block {
            self.statement(statement);
        }
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
            StatementKind::Make { variable, value } => {
                let reference = self.is_reference(value.type_id);
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
                    if reference {
                        self.hold(register);
                    }
                }
                return;
            }
            StatementKind::Set { variable, value } => self.set(*variable, value),
            StatementKind::SetField { variable, field, value } => {
                let record_type = self.variable_type(*variable);
                let field = self.field_index(record_type, field);
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
                self.block(body);
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
            StatementKind::Instead(value) => {
                let Some(&target) = self.function.instead_targets.last() else {
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
            let parts: Vec<(Register, TypeId)> = rest.iter().map(|part| (self.value(part), part.type_id)).collect();
            self.place = value.place;
            self.change_in_place(variable, |compiler, text| {
                for (part, part_type) in parts {
                    compiler.append(text, part, part_type);
                }
            });
            return;
        }
        match self.location(variable) {
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

    fn append(&mut self, text: Register, part: Register, part_type: TypeId) {
        let instruction = match self.kind(part_type) {
            Type::Whole => Instruction::AppendWhole { to: text, from: part },
            Type::Bool => Instruction::AppendBool { to: text, from: part },
            Type::Float => Instruction::AppendFloat { to: text, from: part },
            _ => Instruction::Append { to: text, from: part },
        };
        self.emit(instruction);
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
        if self.value_into(source, base) {
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
            ExpressionKind::Binary { operator, left, right } if operator.is_comparison() => {
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
                let receiver_type = receiver.type_id;
                let from = self.reading(receiver);
                self.place = expression.place;
                if matches!(self.kind(receiver_type), Type::Record(_)) {
                    let field = self.field_index(receiver_type, name);
                    self.emit(Instruction::GetField { to, record: from, field });
                } else {
                    self.emit(Instruction::Length { to, from });
                }
                false
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
            ExpressionKind::Call { procedure, arguments } => {
                let can_pause = self.program.procedure(*procedure).is_some_and(|procedure| procedure.can_pause);
                let base = self.arguments(arguments);
                self.place = expression.place;
                self.emit(Instruction::Call { base, procedure: procedure.0 });
                if can_pause {
                    self.pause_map();
                }
                if !matches!(self.kind(expression.type_id), Type::Nothing) && base != to {
                    self.emit(Instruction::Move { to, from: base });
                }
                reference
            }
            ExpressionKind::DoorCall { door, arguments } => {
                let can_pause = self.doors.get(*door as usize).is_some_and(|door| door.can_pause);
                let crossings: Vec<Crossing> =
                    arguments.iter().map(|argument| self.crossing(argument.type_id)).collect();
                let result = self.crossing(expression.type_id);
                let base = self.arguments(arguments);
                let site = u32::try_from(self.out.door_sites.len()).unwrap_or(u32::MAX);
                self.out.door_sites.push(DoorSite { door: *door, arguments: crossings, result });
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
                        self.emit(Instruction::NewText { to });
                        self.append(to, from, value_type);
                        true
                    }
                    Conversion::WholeToFloat => {
                        self.emit(Instruction::WholeToFloat { to, from });
                        false
                    }
                    Conversion::TextToWhole => {
                        self.emit(Instruction::TextToWhole { to, from });
                        false
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
                }
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
                false
            }
            ExpressionKind::Join(parts) => {
                if let [only] = parts.as_slice()
                    && matches!(self.kind(only.type_id), Type::Text)
                {
                    return self.value_into(only, to);
                }
                // The pieces are worked out first, so that a piece that reads `to`'s old value still can.
                let pieces: Vec<(Register, TypeId)> =
                    parts.iter().map(|part| (self.value(part), part.type_id)).collect();
                self.place = expression.place;
                let built = if pieces.iter().any(|(piece, _)| *piece == to) { self.allocate() } else { to };
                self.emit(Instruction::NewText { to: built });
                for (piece, piece_type) in pieces {
                    self.append(built, piece, piece_type);
                }
                if built != to {
                    self.emit(Instruction::Move { to, from: built });
                }
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
            ExpressionKind::Otherwise { call, cases } => {
                let held = self.value_into(call, to);
                self.place = expression.place;
                let fine = self.emit(Instruction::JumpUnlessFailed { offset: 0 });
                let mut ends = Vec::new();
                self.function.instead_targets.push(to);
                for (failure, body) in cases {
                    let next = self.emit(Instruction::JumpUnlessFailureIs { failure: failure.0, offset: 0 });
                    self.emit(Instruction::ClearFailure);
                    self.block(body);
                    ends.push(self.emit(Instruction::Jump { offset: 0 }));
                    let here = self.here();
                    self.patch(next, here);
                }
                self.function.instead_targets.pop();
                // Checking made sure the cases cover every failure the call can give.
                self.emit(Instruction::Unreachable);
                let here = self.here();
                self.patch_all(&ends, here);
                self.merge_after_handling(fine, to, held, reference)
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

    /// Lines a call's arguments up after the register its result comes back in, giving that register.
    fn arguments(&mut self, arguments: &[Expression]) -> Register {
        let count = u32::try_from(arguments.len()).unwrap_or(u32::MAX);
        let held_before = self.function.held.len();
        let base = self.allocate_many(count + 1);
        for (slot, argument) in (base + 1..).zip(arguments) {
            if self.value_into(argument, slot) {
                self.hold(slot);
            }
        }
        // The callee's frame starts at `base`, so nothing this frame still holds may sit above the arguments.
        let last = u32::from(base) + count;
        let above = self.function.held.get(held_before..).unwrap_or(&[]).iter().any(|&held| u32::from(held) > last);
        if !above {
            return base;
        }
        let moved = self.allocate_many(count + 1);
        for offset in 1..=Register::try_from(count).unwrap_or(0) {
            self.emit(Instruction::Move { to: moved + offset, from: base + offset });
            if let Some(held) = self.function.held.iter_mut().rev().find(|held| **held == base + offset) {
                *held = moved + offset;
            }
        }
        moved
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
        let left = self.value(left);
        let right = self.value(right);
        self.place = whole.place;
        let instruction = match (operator, is_float) {
            (BinaryOperator::Add, false) => Instruction::AddWhole { to, left, right },
            (BinaryOperator::Subtract, false) => Instruction::SubtractWhole { to, left, right },
            (BinaryOperator::Multiply, false) => Instruction::MultiplyWhole { to, left, right },
            (BinaryOperator::Divide, false) => Instruction::DivideWhole { to, left, right },
            (BinaryOperator::Add, true) => Instruction::AddFloat { to, left, right },
            (BinaryOperator::Subtract, true) => Instruction::SubtractFloat { to, left, right },
            (BinaryOperator::Multiply, true) => Instruction::MultiplyFloat { to, left, right },
            (BinaryOperator::Divide, true) => Instruction::DivideFloat { to, left, right },
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
        | ExpressionKind::Text(_) => {}
        ExpressionKind::Member { receiver: inner, .. }
        | ExpressionKind::Convert { value: inner, .. }
        | ExpressionKind::Sqrt(inner)
        | ExpressionKind::Negate(inner)
        | ExpressionKind::Not(inner)
        | ExpressionKind::Repeat { value: inner, .. }
        | ExpressionKind::Try(inner) => look(inner),
        ExpressionKind::Item { receiver: first, index: second, .. }
        | ExpressionKind::Lookup { dict: first, key: second }
        | ExpressionKind::Has { dict: first, key: second }
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

/// Whether a name is made by `for each` or `match`, which only borrow what they name.
#[must_use]
pub fn borrows(kind: VariableKind) -> bool {
    !matches!(kind, VariableKind::Named)
}
