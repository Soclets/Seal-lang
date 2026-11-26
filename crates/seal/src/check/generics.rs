//! Generic procedures made concrete (D161). A generic procedure is checked once, with each type parameter a type of
//! its own; each set of types its calls fill in then gets a copy of it, an instance, with those types in place of the
//! parameters and names of its own, which is what's compiled and run. So a generic procedure runs exactly as the same
//! procedure written for those types would, with nothing looked up while it runs.

use std::collections::HashMap;

use super::tree::{
    Block, Expression, ExpressionKind, HandleFallback, Inside, PathStep, Procedure, ProcedureId, Program, Statement,
    StatementKind, VariableId,
};
use super::types::{Type, TypeId};
use crate::message::Message;

/// The most instances a script's generic procedures may have.
const MOST_INSTANCES: usize = 10_000;

/// How deep the types an instance is made for may nest, so that a generic procedure calling itself with ever bigger
/// types, as `List<?Item>` growing into `List<List<?Item>>`, stops while loading, before walking its types could
/// overrun the host's stack (Building rule 1).
const DEEPEST_TYPE: u32 = 100;

/// What each part of a checked tree is visited for.
trait Visit {
    fn expression(&mut self, _expression: &mut Expression) {}
    fn variable(&mut self, _variable: &mut VariableId) {}
    fn type_id(&mut self, _type_id: &mut TypeId) {}
}

fn block(visit: &mut dyn Visit, block_: &mut Block) {
    for statement_ in block_ {
        statement(visit, statement_);
    }
}

fn inside(visit: &mut dyn Visit, path: &mut Inside) {
    for (variable, value) in &mut path.lets {
        visit.variable(variable);
        expression(visit, value);
    }
    visit.variable(&mut path.root);
    for step in &mut path.steps {
        if let PathStep::Item { index, .. } = step {
            expression(visit, index);
        }
    }
    for type_id in &mut path.types {
        visit.type_id(type_id);
    }
    visit.variable(&mut path.part);
}

fn statement(visit: &mut dyn Visit, statement_: &mut Statement) {
    match &mut statement_.kind {
        StatementKind::Inside { path, change } => {
            inside(visit, path);
            statement(visit, change);
        }
        StatementKind::Make { variable, value }
        | StatementKind::Set { variable, value }
        | StatementKind::SetField { variable, value, .. } => {
            visit.variable(variable);
            expression(visit, value);
        }
        StatementKind::SetItem { variable, index, value, .. } => {
            visit.variable(variable);
            expression(visit, index);
            expression(visit, value);
        }
        StatementKind::SetKey { variable, key, value } => {
            visit.variable(variable);
            expression(visit, key);
            expression(visit, value);
        }
        StatementKind::Evaluate(value) | StatementKind::Use(value) | StatementKind::Return(Some(value)) => {
            expression(visit, value);
        }
        StatementKind::If { branches, otherwise } => {
            for (condition, body) in branches {
                expression(visit, condition);
                block(visit, body);
            }
            if let Some(body) = otherwise {
                block(visit, body);
            }
        }
        StatementKind::While { condition, body } => {
            expression(visit, condition);
            block(visit, body);
        }
        StatementKind::ForRange { item, position, from, end, step, body, .. } => {
            visit.variable(item);
            if let Some((position, _)) = position {
                visit.variable(position);
            }
            expression(visit, from);
            expression(visit, end);
            if let Some(step) = step {
                expression(visit, step);
            }
            block(visit, body);
        }
        StatementKind::ForEach { item, position, source, body } => {
            visit.variable(item);
            if let Some((position, _)) = position {
                visit.variable(position);
            }
            expression(visit, source);
            block(visit, body);
        }
        StatementKind::Match { subject, arms } => {
            expression(visit, subject);
            for arm in arms {
                for (_, variable) in &mut arm.values {
                    visit.variable(variable);
                }
                block(visit, &mut arm.body);
            }
        }
        StatementKind::Break | StatementKind::Return(None) | StatementKind::Fail(_) => {}
    }
}

/// Visits an expression's parts first, then the expression itself, so a call is seen with its arguments done.
fn expression(visit: &mut dyn Visit, expression_: &mut Expression) {
    match &mut expression_.kind {
        ExpressionKind::Error
        | ExpressionKind::Whole(_)
        | ExpressionKind::Float(_)
        | ExpressionKind::Bool(_)
        | ExpressionKind::Text(_)
        | ExpressionKind::Number(_) => {}
        ExpressionKind::Variable(variable) => visit.variable(variable),
        ExpressionKind::Member { receiver: inner, .. }
        | ExpressionKind::Convert { value: inner, .. }
        | ExpressionKind::Sqrt(inner)
        | ExpressionKind::Negate(inner)
        | ExpressionKind::Not(inner)
        | ExpressionKind::ShowPlaces { value: inner, .. }
        | ExpressionKind::Repeat { value: inner, .. }
        | ExpressionKind::Try(inner)
        | ExpressionKind::MaybeCrash(inner) => expression(visit, inner),
        ExpressionKind::Land { value, target } => {
            expression(visit, value);
            visit.type_id(target);
        }
        ExpressionKind::Item { receiver: first, index: second, .. }
        | ExpressionKind::Lookup { dict: first, key: second }
        | ExpressionKind::Has { dict: first, key: second }
        | ExpressionKind::Split { text: first, by: second, .. }
        | ExpressionKind::Binary { left: first, right: second, .. }
        | ExpressionKind::MaybeOr { maybe: first, fallback: second } => {
            expression(visit, first);
            expression(visit, second);
        }
        ExpressionKind::Inside { path, change } => {
            inside(visit, path);
            expression(visit, change);
        }
        ExpressionKind::Call { arguments, types, .. } => {
            for argument in arguments {
                expression(visit, argument);
            }
            for type_id in types {
                visit.type_id(type_id);
            }
        }
        ExpressionKind::DoorCall { arguments: items, .. }
        | ExpressionKind::Maths { arguments: items, .. }
        | ExpressionKind::Join(items)
        | ExpressionKind::List(items) => {
            for item in items {
                expression(visit, item);
            }
        }
        ExpressionKind::BuildRecord { fields, .. } | ExpressionKind::BuildCase { fields, .. } => {
            for (_, value) in fields {
                expression(visit, value);
            }
        }
        ExpressionKind::Range { from, end, step, .. } => {
            expression(visit, from);
            expression(visit, end);
            if let Some(step) = step {
                expression(visit, step);
            }
        }
        ExpressionKind::Dict(pairs) => {
            for (key, value) in pairs {
                expression(visit, key);
                expression(visit, value);
            }
        }
        ExpressionKind::Handle { call, fallback } => {
            expression(visit, call);
            if let HandleFallback::Value(value) = fallback {
                expression(visit, value);
            }
        }
        ExpressionKind::Otherwise { call, cases, .. } => {
            expression(visit, call);
            for (_, body) in cases {
                block(visit, body);
            }
        }
        ExpressionKind::ListAdd { variable, value, position } => {
            visit.variable(variable);
            expression(visit, value);
            if let Some((position, _)) = position {
                expression(visit, position);
            }
        }
        ExpressionKind::ListRemove { variable, position, .. } => {
            visit.variable(variable);
            expression(visit, position);
        }
        ExpressionKind::DictRemove { variable, key } => {
            visit.variable(variable);
            expression(visit, key);
        }
    }
    visit.type_id(&mut expression_.type_id);
    visit.expression(expression_);
}

/// Puts concrete types in place of a generic procedure's type parameters, and gives each of its own names a new
/// name of the instance's own, leaving the top level's names as they are.
struct Fill<'p> {
    program: &'p mut Program,
    map: Vec<(u32, TypeId)>,
    names: HashMap<VariableId, VariableId>,
}

impl Visit for Fill<'_> {
    fn variable(&mut self, variable: &mut VariableId) {
        if let Some(&renamed) = self.names.get(variable) {
            *variable = renamed;
            return;
        }
        let Some(found) = self.program.variable(*variable).cloned() else { return };
        if found.top_level {
            return;
        }
        let mut copy = found;
        copy.type_id = self.program.types.substitute(copy.type_id, &self.map);
        let renamed = VariableId(u32::try_from(self.program.variables.len()).unwrap_or(u32::MAX));
        self.program.variables.push(copy);
        self.names.insert(*variable, renamed);
        *variable = renamed;
    }

    fn type_id(&mut self, type_id: &mut TypeId) {
        *type_id = self.program.types.substitute(*type_id, &self.map);
    }
}

/// Points each call to a generic procedure at the instance for the types it fills in, making the instance when it's
/// the first such call.
struct Point<'p> {
    program: &'p mut Program,
    instances: &'p mut HashMap<(ProcedureId, Vec<TypeId>), ProcedureId>,
    waiting: &'p mut Vec<ProcedureId>,
    problem: Option<Message>,
}

impl Visit for Point<'_> {
    fn expression(&mut self, expression: &mut Expression) {
        let ExpressionKind::Call { procedure, types, .. } = &mut expression.kind else { return };
        if types.is_empty() || self.problem.is_some() {
            return;
        }
        let filled: Vec<TypeId> = types.iter().map(|&type_id| self.program.types.settled_copy(type_id)).collect();
        if filled.iter().any(|&type_id| self.program.types.has_parameter(type_id)) {
            // A call inside a generic procedure, made concrete with its instance.
            return;
        }
        let key = (*procedure, filled.clone());
        let instance = if let Some(&instance) = self.instances.get(&key) {
            instance
        } else {
            if self.instances.len() >= MOST_INSTANCES {
                self.problem = Some(Message::not_built_yet(
                    expression.place,
                    "a generic procedure filled in with more than 10,000 sets of types isn't",
                ));
                return;
            }
            if filled.iter().any(|&type_id| self.program.types.depth(type_id) > DEEPEST_TYPE) {
                self.problem = Some(Message::not_built_yet(
                    expression.place,
                    "a generic procedure filled in with types nested more than 100 deep, as one calling itself with a list of what it was given, isn't",
                ));
                return;
            }
            let instance = make_instance(self.program, *procedure, &filled);
            self.instances.insert(key, instance);
            self.waiting.push(instance);
            instance
        };
        *procedure = instance;
        types.clear();
    }
}

/// A copy of a generic procedure with `filled` in place of its type parameters, added to the program.
fn make_instance(program: &mut Program, generic: ProcedureId, filled: &[TypeId]) -> ProcedureId {
    let Some(original) = program.procedure(generic).cloned() else { return generic };
    let map: Vec<(u32, TypeId)> = original
        .type_parameters
        .iter()
        .zip(filled)
        .filter_map(|(&parameter, &filled)| match program.types.get(parameter) {
            Type::Parameter(number) => Some((*number, filled)),
            _ => None,
        })
        .collect();
    let Procedure {
        name,
        labels,
        labels_any_order,
        mut defaults,
        result,
        can_fail,
        failures,
        can_pause,
        mut body,
        place,
        end_place,
        mut parameters,
        method,
        ..
    } = original;
    let mut fill = Fill { program, map, names: HashMap::new() };
    for parameter in &mut parameters {
        fill.variable(parameter);
    }
    for default in defaults.iter_mut().flatten() {
        expression(&mut fill, default);
    }
    block(&mut fill, &mut body);
    let result = fill.program.types.substitute(result, &fill.map);
    let id = ProcedureId(u32::try_from(fill.program.procedures.len()).unwrap_or(u32::MAX));
    fill.program.procedures.push(Procedure {
        name,
        type_parameters: Vec::new(),
        instance_of: Some(generic),
        method,
        parameters,
        labels,
        labels_any_order,
        defaults,
        result,
        can_fail,
        failures,
        can_pause,
        body,
        place,
        end_place,
    });
    id
}

/// Finds the calls to generic procedures whose types nothing worked out.
struct Unsettled<'p> {
    program: &'p Program,
    found: Vec<Message>,
}

impl Visit for Unsettled<'_> {
    fn expression(&mut self, expression: &mut Expression) {
        let ExpressionKind::Call { procedure, types, .. } = &expression.kind else { return };
        let Some(generic) = self.program.procedure(*procedure) else { return };
        for (&filled, &parameter) in types.iter().zip(&generic.type_parameters) {
            if !self.program.types.is_settled(filled) {
                let name = self.program.types.written(parameter);
                self.found.push(
                    Message::refused(
                        expression.place,
                        format!("this call can't work out what `{name}` stands for in `{}`", generic.name),
                    )
                    .with_why("a call works its types out from its arguments, or from where its result goes, as `top: Maybe<Whole> = first(scores)`; a type is written where nothing says it."),
                );
                return;
            }
        }
    }
}

/// trace: D161. Refuses calls to generic procedures whose types nothing works out, then makes an instance of each
/// generic procedure for each set of types it's called with, from the top level and every procedure that isn't
/// generic, and from the instances themselves.
///
/// # Errors
///
/// The refusals, or a stop when there would be too many instances.
pub fn make_instances(program: &mut Program) -> Result<(), Vec<Message>> {
    if program.procedures.iter().all(|procedure| procedure.type_parameters.is_empty()) {
        return Ok(());
    }
    // Every body is looked through for calls whose types are still open, generic ones too.
    let mut top_level = std::mem::take(&mut program.top_level);
    let mut bodies: Vec<Block> =
        program.procedures.iter_mut().map(|procedure| std::mem::take(&mut procedure.body)).collect();
    let mut unsettled = Unsettled { program, found: Vec::new() };
    block(&mut unsettled, &mut top_level);
    for body in &mut bodies {
        block(&mut unsettled, body);
    }
    let found = unsettled.found;
    for (procedure, body) in program.procedures.iter_mut().zip(bodies) {
        procedure.body = body;
    }
    if !found.is_empty() {
        program.top_level = top_level;
        return Err(found);
    }
    let mut instances = HashMap::new();
    let mut waiting: Vec<ProcedureId> = (0u32..)
        .zip(&program.procedures)
        .filter(|(_, procedure)| procedure.type_parameters.is_empty())
        .map(|(index, _)| ProcedureId(index))
        .collect();
    let mut point = Point { program, instances: &mut instances, waiting: &mut waiting, problem: None };
    block(&mut point, &mut top_level);
    point.program.top_level = top_level;
    while let Some(next) = point.waiting.pop() {
        let Some(procedure) = point.program.procedures.get_mut(next.0 as usize) else { continue };
        let mut body = std::mem::take(&mut procedure.body);
        let mut defaults = std::mem::take(&mut procedure.defaults);
        block(&mut point, &mut body);
        for default in defaults.iter_mut().flatten() {
            expression(&mut point, default);
        }
        if let Some(procedure) = point.program.procedures.get_mut(next.0 as usize) {
            procedure.body = body;
            procedure.defaults = defaults;
        }
        if point.problem.is_some() {
            break;
        }
    }
    match point.problem.take() {
        Some(problem) => Err(vec![problem]),
        None => Ok(()),
    }
}
