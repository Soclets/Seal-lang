//! A loaded script: what a host holds and calls into (D4, D48).

use std::fmt;
use std::panic::{AssertUnwindSafe, catch_unwind};

use super::convert::{Argument, Arguments, FromSeal, IntoSeal};
use std::sync::Arc;

use super::doors::{Doors, Loaded};
use super::value::Value;
use crate::check::check;
use crate::compile::bytecode::{Compiled, CompiledProcedure, Crossing};
use crate::compile::compiler::compile;
use crate::message::{Message, MessageKind, Messages, Place};
use crate::signature::HostType;
use crate::syntax::parser::parse;
use crate::vm::crossing::{CrossingError, describe, plain_slot, plain_value, to_slot, to_value};
use crate::vm::heap::Heap;
use crate::vm::machine::{Machine, Outcome, Stop, Task, run};

/// What a host allows a script (D14): how many steps each run between pauses may take (D53), and how many bytes
/// the whole script may hold, all its tasks together.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub steps: u64,
    pub memory: usize,
}

/// What can go wrong with a script. None of these harms the host (D14, D48).
#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    /// The script was refused at load time, or a call into it was refused: it names a procedure the script doesn't
    /// have, or gives or expects the wrong types (D17, D48). A refused call leaves the script as it was.
    Refused(Messages),
    /// The script stopped: a check failed, a limit was reached, or `?? crash` met a failure (D14, D17). All its
    /// tasks are over, and it can't be called again (D54).
    Stopped(Message),
    /// The procedure the host called failed on purpose, with this failure (D44). The script carries on.
    Failed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Refused(messages) => write!(formatter, "{messages}"),
            Error::Stopped(message) => write!(formatter, "{message}"),
            Error::Failed(failure) => writeln!(formatter, "failed: the procedure failed with `{failure}`"),
        }
    }
}

impl std::error::Error for Error {}

/// A task: one call into a script that's paused at a door (D49, D51).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TaskId {
    index: u32,
    generation: u32,
}

/// How a call into a script came back.
#[derive(Clone, Debug, PartialEq)]
pub enum Called<R> {
    /// The procedure ran to its end, giving this.
    Finished(R),
    /// A door paused the script (D49). The host resumes the task, or cancels it, whenever it likes.
    Paused(TaskId),
}

struct Running {
    task: Task,
    /// The procedure the host called, by its place among the script's procedures.
    procedure: usize,
}

#[derive(Default)]
struct TaskSlot {
    generation: u32,
    running: Option<Running>,
}

/// A loaded script (D4). Each one is fully separate, with its own names, memory, limits and doors (D49), and can
/// move between threads.
pub struct Script {
    name: String,
    /// The script's text, for the lines its messages show, and what it compiled to. Both are shared with any
    /// other script loaded from the same text with the same doors.
    source: Arc<str>,
    compiled: Arc<Compiled>,
    doors: Doors,
    machine: Machine,
    tasks: Vec<TaskSlot>,
    limits: Limits,
    /// Why the script stopped, once it has.
    stopped: Option<Message>,
    /// A finished task's room, kept for the next call to run in.
    spare_task: Option<Task>,
    /// The numbers of the built-in failures `missing`, `not_a_whole` and `full`.
    failures: [u32; 3],
}

impl fmt::Debug for Script {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Script")
            .field("name", &self.name)
            .field("stopped", &self.stopped.is_some())
            .finish_non_exhaustive()
    }
}

fn refusal(file: &str, what: impl Into<String>) -> Error {
    let mut message = Message::new(MessageKind::Refused, Place::default(), what);
    file.clone_into(&mut message.file);
    Error::Refused(Messages::one(message))
}

/// Whether a Rust type's Seal type is the one a script's value has.
fn fits(host: &HostType, crossing: &Crossing) -> bool {
    match (host, crossing) {
        (HostType::Nothing, Crossing::Nothing)
        | (HostType::Whole, Crossing::Whole)
        | (HostType::Float, Crossing::Float)
        | (HostType::Bool, Crossing::Bool)
        | (HostType::Text, Crossing::Text) => true,
        (HostType::List(host), Crossing::List(crossing)) => fits(host, crossing),
        (HostType::Dict(host_key, host_value), Crossing::Dict(key, value)) => {
            fits(host_key, key) && fits(host_value, value)
        }
        (HostType::Maybe(host), Crossing::Maybe { inner, .. }) => fits(host, inner),
        (HostType::Record { name: host_name, fields: host_fields }, Crossing::Record { name, fields, .. }) => {
            host_name == name
                && host_fields.len() == fields.len()
                && host_fields.iter().zip(fields).all(|((host_field, host_type), (field, crossing))| {
                    host_field == field && fits(host_type, crossing)
                })
        }
        (HostType::OneOf { name: host_name, cases: host_cases }, Crossing::OneOf { name, cases }) => {
            host_name == name
                && host_cases.len() == cases.len()
                && host_cases.iter().zip(cases).all(|((host_case, host_fields), case)| {
                    *host_case == case.name
                        && host_fields.len() == case.fields.len()
                        && host_fields.iter().zip(&case.fields).all(|((host_field, host_type), (field, crossing))| {
                            host_field == field && fits(host_type, crossing)
                        })
                })
        }
        _ => false,
    }
}

impl Script {
    /// Loads a script: reads it, checks it, compiles it, and runs its top level once (D4, D17, D48).
    ///
    /// `name` is the file's name, as `goblin.seal`, for messages. The script gets these doors and limits, and
    /// nothing else.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Refused`] with every message when the script doesn't pass checking, and
    /// [`Error::Stopped`] when its top level stops.
    pub fn load(name: &str, source: &str, doors: &Doors, limits: Limits) -> Result<Script, Error> {
        // The same text loaded again with the same doors has already been checked and compiled.
        let prepared = match doors.kept(source) {
            Some(loaded) => Ok(Ok(loaded)),
            None => catch_unwind(AssertUnwindSafe(|| Self::prepare(name, source, doors))).map(|prepared| {
                prepared.map(|compiled| {
                    let loaded = Loaded { source: source.into(), compiled: Arc::new(compiled) };
                    doors.keep(loaded.clone());
                    loaded
                })
            }),
        };
        let Loaded { source: shared_source, compiled } = match prepared {
            Ok(Ok(loaded)) => loaded,
            Ok(Err(mut messages)) => {
                for message in &mut messages {
                    message.locate(name, source);
                }
                return Err(Error::Refused(Messages { list: messages }));
            }
            Err(_) => {
                let mut message = Message::internal("loading a script went wrong");
                name.clone_into(&mut message.file);
                return Err(Error::Stopped(message));
            }
        };
        let missing_failure = |wanted: &str| {
            (1u32..).zip(&compiled.failure_names).find(|(_, known)| *known == wanted).map_or(0, |(number, _)| number)
        };
        let failures = [missing_failure("missing"), missing_failure("not_a_whole"), missing_failure("full")];
        let mut script = Script {
            name: name.to_owned(),
            source: shared_source,
            compiled,
            doors: doors.clone(),
            machine: Machine { heap: Heap::new(limits.memory), ..Machine::default() },
            tasks: Vec::new(),
            limits,
            stopped: None,
            spare_task: None,
            failures,
        };
        script.start()?;
        Ok(script)
    }

    /// Checks a script against a host's doors without running any of it (D17).
    ///
    /// # Errors
    ///
    /// Returns [`Error::Refused`] with every message when the script doesn't pass checking.
    pub fn check(name: &str, source: &str, doors: &Doors) -> Result<(), Error> {
        match catch_unwind(AssertUnwindSafe(|| Self::prepare(name, source, doors))) {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(mut messages)) => {
                for message in &mut messages {
                    message.locate(name, source);
                }
                Err(Error::Refused(Messages { list: messages }))
            }
            Err(_) => {
                let mut message = Message::internal("checking a script went wrong");
                name.clone_into(&mut message.file);
                Err(Error::Stopped(message))
            }
        }
    }

    fn prepare(name: &str, source: &str, doors: &Doors) -> Result<Compiled, Vec<Message>> {
        let _ = name;
        let tree = parse(source).map_err(|message| vec![message])?;
        let signatures = doors.signatures();
        let program = check(&tree, &signatures)?;
        compile(&program, &signatures).map_err(|message| vec![message])
    }

    /// Sets the script's memory up and runs its top level (D4).
    fn start(&mut self) -> Result<(), Error> {
        let globals = self.compiled.global_count as usize;
        let mut set_up = || -> Result<(), crate::vm::heap::HeapError> {
            self.machine.heap.charge(globals.saturating_mul(crate::vm::heap::SLOT_COST))?;
            self.machine.globals = vec![0; globals];
            for text in &self.compiled.texts {
                let handle = self.machine.heap.new_text(text, text.len())?;
                self.machine.texts.push(handle);
            }
            Ok(())
        };
        if let Err(error) = set_up() {
            return Err(self.stop(Stop::from(error), 0));
        }
        let mut task = Task::default();
        if let Err(error) =
            task.start(&mut self.machine.heap, self.compiled.top_level_entry, self.compiled.top_level_frame_size)
        {
            return Err(self.stop(Stop::from(error), 0));
        }
        let outcome = self.run(&mut task);
        match outcome {
            Ok(Outcome::Finished) => {
                task.finish(&mut self.machine.heap);
                Ok(())
            }
            Ok(Outcome::Paused | Outcome::Failed(_)) => Err(self
                .stop(Stop::Internal("the top level paused or failed, which checking rules out".to_owned()), task.pc)),
            Err((stop, pc)) => Err(self.stop(stop, pc)),
        }
    }

    /// Runs a task until it finishes, pauses or stops, with a fresh step budget (D53).
    fn run(&mut self, task: &mut Task) -> Result<Outcome, (Stop, usize)> {
        let (machine, compiled, doors) = (&mut self.machine, &self.compiled, &self.doors);
        let (steps, failures) = (self.limits.steps, self.failures);
        // A bug in Seal must never take the host down with it (D14): it's caught and told like any other stop.
        match catch_unwind(AssertUnwindSafe(|| run(machine, task, compiled, doors, steps, failures))) {
            Ok(outcome) => outcome,
            Err(_) => Err((Stop::Internal("the VM went wrong while running".to_owned()), task.pc)),
        }
    }

    /// Stops the whole script: every task is over, its memory is freed, and the host is told why (D14, D54).
    fn stop(&mut self, stop: Stop, pc: usize) -> Error {
        let place = self.compiled.places.get(pc.saturating_sub(1)).copied().unwrap_or_default();
        let mut message = match stop {
            Stop::CheckFailed { what, when, why } => {
                let mut message = Message::new(MessageKind::CheckFailed, place, what);
                message.when = when;
                message.why = why;
                message
            }
            Stop::OutOfSteps => Message::new(MessageKind::LimitReached, place, "the script ran out of steps")
                .with_when(format!("its host allows {} steps for each run between pauses", self.limits.steps))
                .with_why("a script that runs too long without pausing is stopped, so that it can't freeze its host."),
            Stop::OutOfMemory { needed, in_use, cap } => {
                Message::new(MessageKind::LimitReached, place, "the script ran out of memory")
                    .with_when(format!("it needed {needed} more bytes, with {in_use} in use; its host allows {cap}"))
                    .with_why("a script that would hold more memory than its host allows is stopped.")
            }
            Stop::Crashed { failure } => {
                let name = self.compiled.failure_name(failure).to_owned();
                // trace: Unio D152
                Message::new(MessageKind::Failed, place, format!("this failed with `{name}`"))
                    .with_why("`?? crash` stops the script when the call before it fails.")
            }
            Stop::NotDesigned(what) => Message::not_designed_yet(place, what),
            Stop::HostMistake(detail) => {
                let mut message =
                    Message::new(MessageKind::Internal, place, format!("the host's own code went wrong: {detail}"));
                message.why =
                    Some("this is a mistake in the program that runs this script, not in the script.".to_owned());
                message
            }
            Stop::Internal(detail) => {
                let mut message = Message::internal(detail);
                message.place = place;
                message
            }
        };
        message.locate(&self.name, &self.source);
        self.stopped = Some(message.clone());
        self.tasks.clear();
        self.machine = Machine::default();
        Error::Stopped(message)
    }

    fn check_running(&self) -> Result<(), Error> {
        match &self.stopped {
            Some(message) => Err(Error::Stopped(message.clone())),
            None => Ok(()),
        }
    }

    fn procedure(&self, name: &str) -> Result<(usize, &CompiledProcedure), Error> {
        self.compiled
            .procedures
            .iter()
            .enumerate()
            .find(|(_, procedure)| procedure.name == name)
            .ok_or_else(|| refusal(&self.name, format!("`{}` has no procedure named `{name}`", self.name)))
    }

    /// Matches a call's arguments to the procedure's parameters, by position and label (D41, D42).
    fn bind(&self, procedure: &CompiledProcedure, arguments: &[Argument]) -> Result<Vec<Value>, Error> {
        let name = &procedure.name;
        let count = procedure.parameters.len();
        let mut bound: Vec<Option<Value>> = vec![None; count];
        let mut next = 0usize;
        for argument in arguments {
            let position = if let Some(label) = &argument.label {
                procedure
                    .parameters
                    .iter()
                    .position(|parameter| parameter.label.as_deref() == Some(label.as_str()))
                    .ok_or_else(|| refusal(&self.name, format!("`{name}` has no label `{label}:`")))?
            } else {
                let found = (next..count).find(|&candidate| {
                    procedure.parameters.get(candidate).is_some_and(|parameter| parameter.label.is_none())
                });
                found.ok_or_else(|| {
                    refusal(&self.name, format!("`{name}` doesn't take this many unlabelled arguments"))
                })?
            };
            if argument.label.is_none() {
                next = position + 1;
            }
            match bound.get_mut(position) {
                Some(slot) if slot.is_none() => *slot = Some(argument.value.clone()),
                _ => return Err(refusal(&self.name, format!("an argument of `{name}` is given twice"))),
            }
        }
        let mut values = Vec::with_capacity(count);
        for (given, parameter) in bound.into_iter().zip(&procedure.parameters) {
            if let Some(value) = given.or_else(|| parameter.default.clone()) {
                values.push(value);
            } else {
                let what = match &parameter.label {
                    Some(label) => format!("this call to `{name}` leaves out `{label}:`"),
                    None => format!("this call to `{name}` leaves out `{}`", parameter.name),
                };
                return Err(refusal(&self.name, what));
            }
        }
        Ok(values)
    }

    /// Calls one of the script's procedures by name, with values (D4). The call is a task (D51).
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] when the call doesn't fit the procedure, [`Error::Failed`] when the procedure fails on
    /// purpose, and [`Error::Stopped`] when the script stops.
    pub fn call_values(&mut self, procedure: &str, arguments: &[Argument]) -> Result<Called<Value>, Error> {
        self.check_running()?;
        // The compiled script is shared, so holding it here leaves the script itself free to change.
        let compiled = Arc::clone(&self.compiled);
        let Some((index, found)) = compiled.procedures.iter().enumerate().find(|(_, found)| found.name == procedure)
        else {
            return Err(refusal(&self.name, format!("`{}` has no procedure named `{procedure}`", self.name)));
        };
        // trace: D72. A type that holds itself can't cross between a host and a script yet.
        if let Some(parameter) = found.parameters.iter().find(|parameter| parameter.crossing.holds_itself()) {
            return Err(self.not_built(format!(
                "passing a `{}`, a type that holds itself, from a host to a script (D72), which calling `{procedure}` needs, isn't",
                parameter.written
            )));
        }
        if found.result.holds_itself() {
            return Err(self.not_built(format!(
                "passing {}, a type that holds itself, from a script to its host (D72), which calling `{procedure}` needs, isn't",
                describe(&found.result)
            )));
        }
        // The usual call gives every argument in order with no labels; anything else is matched up by label.
        let plain = arguments.len() == found.parameters.len()
            && arguments
                .iter()
                .zip(&found.parameters)
                .all(|(argument, parameter)| argument.label.is_none() && parameter.label.is_none());
        let bound = if plain { None } else { Some(self.bind(found, arguments)?) };
        let (entry, frame_size) = (found.entry, found.frame_size);
        // A call runs in the room the last finished one left, when there is one.
        let mut task = self.spare_task.take().unwrap_or_default();
        if let Err(error) = task.start(&mut self.machine.heap, entry, frame_size) {
            return Err(self.stop(Stop::from(error), entry as usize));
        }
        // The arguments go in as copies (D15), in the registers after the one the result comes back in.
        for (position, parameter) in found.parameters.iter().enumerate() {
            let value = match &bound {
                Some(values) => values.get(position),
                None => arguments.get(position).map(|argument| &argument.value),
            };
            let Some(value) = value else { continue };
            let crossing = &parameter.crossing;
            let slot = match plain_slot(value, crossing) {
                Some(slot) => Ok(slot),
                None => to_slot(value, crossing, &mut self.machine.heap, &self.compiled.layouts),
            };
            match slot {
                Ok(slot) => {
                    if let Some(register) = task.stack.get_mut(position + 1) {
                        *register = slot;
                    }
                }
                Err(error) => {
                    for (earlier, parameter) in found.parameters.iter().enumerate().take(position) {
                        if parameter.crossing.is_reference() {
                            let slot = task.stack.get(earlier + 1).copied().unwrap_or(0);
                            self.machine.heap.release(slot, &self.compiled.layouts);
                        }
                    }
                    task.finish(&mut self.machine.heap);
                    return Err(match error {
                        CrossingError::Heap(error) => self.stop(Stop::from(error), entry as usize),
                        CrossingError::WrongType { found, .. } => refusal(
                            &self.name,
                            format!(
                                "argument {} of `{procedure}` is a `{}`, but the call gives {found}",
                                position + 1,
                                parameter.written
                            ),
                        ),
                    });
                }
            }
        }
        self.drive(Running { task, procedure: index }, None)
    }

    /// A refusal for something decided that isn't built yet.
    fn not_built(&self, what: String) -> Error {
        let mut message = Message::not_built_yet(Place::default(), what);
        self.name.clone_into(&mut message.file);
        Error::Refused(Messages::one(message))
    }

    /// Runs a task on, and deals with how it ends. `reuse` is the task's place when it was paused before.
    fn drive(&mut self, mut running: Running, reuse: Option<TaskId>) -> Result<Called<Value>, Error> {
        let outcome = self.run(&mut running.task);
        self.settle(outcome, running, reuse)
    }

    /// Deals with how a run of a task ended.
    fn settle(
        &mut self,
        outcome: Result<Outcome, (Stop, usize)>,
        mut running: Running,
        reuse: Option<TaskId>,
    ) -> Result<Called<Value>, Error> {
        let Some(procedure) = self.compiled.procedures.get(running.procedure) else {
            return Err(self.stop(Stop::Internal("a task lost its procedure".to_owned()), 0));
        };
        match outcome {
            Ok(Outcome::Paused) => {
                let id = if let Some(id) = reuse {
                    id
                } else {
                    let free = self.tasks.iter().position(|slot| slot.running.is_none());
                    let index = free.unwrap_or(self.tasks.len());
                    if free.is_none() {
                        self.tasks.push(TaskSlot::default());
                    }
                    let generation = self.tasks.get(index).map_or(0, |slot| slot.generation);
                    TaskId { index: u32::try_from(index).unwrap_or(u32::MAX), generation }
                };
                if let Some(slot) = self.tasks.get_mut(id.index as usize) {
                    slot.running = Some(running);
                }
                Ok(Called::Paused(id))
            }
            Ok(Outcome::Finished) => {
                let slot = running.task.stack.first().copied().unwrap_or(0);
                // A plain number, a `Bool` or nothing is read straight from its bits; anything else is copied out.
                let value = if let Some(value) = plain_value(slot, &procedure.result) {
                    Ok(value)
                } else {
                    let value = to_value(slot, &procedure.result, &self.machine.heap, &self.compiled.layouts);
                    if procedure.result.is_reference() {
                        self.machine.heap.release(slot, &self.compiled.layouts);
                    }
                    value
                };
                self.end_task(&mut running, reuse);
                match value {
                    Ok(value) => Ok(Called::Finished(value)),
                    Err(error) => Err(self.stop(Stop::from(error), 0)),
                }
            }
            Ok(Outcome::Failed(failure)) => {
                self.end_task(&mut running, reuse);
                Err(Error::Failed(self.compiled.failure_name(failure).to_owned()))
            }
            Err((stop, pc)) => Err(self.stop(stop, pc)),
        }
    }

    /// Ends a task that ran to its end or failed. The procedure took its arguments over and freed them itself (D15),
    /// so only the stack is left to give back.
    fn end_task(&mut self, running: &mut Running, reuse: Option<TaskId>) {
        running.task.finish(&mut self.machine.heap);
        // Its room is kept for the next call, unless one is kept already.
        if self.spare_task.is_none() {
            self.spare_task = Some(std::mem::take(&mut running.task));
        }
        if let Some(id) = reuse {
            self.retire(id);
        }
    }

    /// Frees a task's place, so its `TaskId` can't be used again.
    fn retire(&mut self, id: TaskId) {
        if let Some(slot) = self.tasks.get_mut(id.index as usize) {
            slot.running = None;
            slot.generation = slot.generation.wrapping_add(1);
        }
    }

    fn take_task(&mut self, id: TaskId) -> Result<Running, Error> {
        self.check_running()?;
        let taken = self
            .tasks
            .get_mut(id.index as usize)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.running.take());
        match taken {
            Some(running) => Ok(running),
            None => Err(refusal(
                &self.name,
                "this task isn't paused: it has finished, or was cancelled, or belongs to another script",
            )),
        }
    }

    /// Resumes a paused task with the value its door gives (D49). The run gets a fresh step budget (D53).
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] when the task isn't paused or the value isn't the door's type; otherwise as
    /// [`Script::call_values`].
    pub fn resume_with_value(&mut self, task: TaskId, value: &Value) -> Result<Called<Value>, Error> {
        self.check_running()?;
        // The usual resume gives a plain value and the task pauses again: it's run where it sits, nothing moved.
        let in_place = {
            let Script { tasks, machine, compiled, doors, limits, failures, .. } = self;
            let slot = tasks.get_mut(task.index as usize).filter(|slot| slot.generation == task.generation);
            let running = slot.and_then(|slot| slot.running.as_mut());
            let plain = running.and_then(|running| {
                let waiting = running.task.waiting?;
                let site = compiled.door_sites.get(waiting.site as usize)?;
                let slot = plain_slot(value, &site.result)?;
                *running.task.stack.get_mut(waiting.slot)? = slot;
                running.task.waiting = None;
                Some(running)
            });
            plain.map(|running| {
                let (steps, failures) = (limits.steps, *failures);
                let doors: &Doors = doors;
                let pc = running.task.pc;
                catch_unwind(AssertUnwindSafe(|| run(machine, &mut running.task, compiled, doors, steps, failures)))
                    .unwrap_or_else(|_| Err((Stop::Internal("the VM went wrong while running".to_owned()), pc)))
            })
        };
        match in_place {
            Some(Ok(Outcome::Paused)) => return Ok(Called::Paused(task)),
            Some(outcome) => {
                let running = self.take_task(task)?;
                return self.settle(outcome, running, Some(task));
            }
            None => {}
        }
        let mut running = self.take_task(task)?;
        let Some(waiting) = running.task.waiting else {
            return Err(
                self.stop(Stop::Internal("a paused task that isn't waiting on a door".to_owned()), running.task.pc)
            );
        };
        let Some(site) = self.compiled.door_sites.get(waiting.site as usize) else {
            return Err(self.stop(Stop::Internal("a paused task whose door is gone".to_owned()), running.task.pc));
        };
        match to_slot(value, &site.result, &mut self.machine.heap, &self.compiled.layouts) {
            Ok(slot) => {
                if let Some(register) = running.task.stack.get_mut(waiting.slot) {
                    *register = slot;
                }
            }
            Err(CrossingError::Heap(error)) => return Err(self.stop(Stop::from(error), running.task.pc)),
            Err(CrossingError::WrongType { wanted, found }) => {
                // The task stays paused, as it was.
                if let Some(slot) = self.tasks.get_mut(task.index as usize) {
                    slot.running = Some(running);
                }
                return Err(refusal(
                    &self.name,
                    format!("the door this task waits on gives {wanted}, but it was resumed with {found}"),
                ));
            }
        }
        running.task.waiting = None;
        self.drive(running, Some(task))
    }

    /// Cancels a paused task: it ends right at its pause, and the script's other tasks carry on (D54).
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] when the task isn't paused, and [`Error::Stopped`] when the script has stopped.
    pub fn cancel(&mut self, task: TaskId) -> Result<(), Error> {
        let mut running = self.take_task(task)?;
        // What each of the task's frames holds at its pause is freed, the arguments the host gave among it (D15).
        for slot in running.task.held_slots(&self.compiled) {
            self.machine.heap.release(slot, &self.compiled.layouts);
        }
        running.task.finish(&mut self.machine.heap);
        self.retire(task);
        Ok(())
    }

    fn check_result<R: FromSeal>(&self, procedure: usize) -> Result<(), Error> {
        let Some(found) = self.compiled.procedures.get(procedure) else { return Ok(()) };
        if fits(&R::host_type(), &found.result) {
            return Ok(());
        }
        Err(refusal(
            &self.name,
            format!("`{}` gives {}, which isn't the type the host's call expects", found.name, describe(&found.result)),
        ))
    }

    fn typed<R: FromSeal>(&mut self, called: Called<Value>) -> Result<Called<R>, Error> {
        match called {
            Called::Paused(task) => Ok(Called::Paused(task)),
            Called::Finished(value) => match R::from_seal(value) {
                Some(result) => Ok(Called::Finished(result)),
                None => {
                    Err(self.stop(Stop::Internal("a result that isn't the type its procedure gives".to_owned()), 0))
                }
            },
        }
    }

    /// Calls one of the script's procedures by name (D4, D48): `script.call::<()>("on_hit", (10,))`.
    ///
    /// The call is checked by name and types before anything runs, the result's type included (D5, D6). Arguments
    /// are a tuple; one with a label is written [`crate::labelled`]`("by", 5)`, and one with a default can be left
    /// out (D42).
    ///
    /// # Errors
    ///
    /// [`Error::Refused`] when the call doesn't fit the procedure, [`Error::Failed`] when the procedure fails on
    /// purpose, and [`Error::Stopped`] when the script stops.
    pub fn call<R: FromSeal>(&mut self, procedure: &str, arguments: impl Arguments) -> Result<Called<R>, Error> {
        self.check_running()?;
        let (index, _) = self.procedure(procedure)?;
        self.check_result::<R>(index)?;
        let called = arguments.with_arguments(|arguments| self.call_values(procedure, arguments))?;
        self.typed(called)
    }

    /// Resumes a paused task with the value its door gives (D49): `script.resume::<()>(task, ())` for a door that
    /// gives nothing.
    ///
    /// # Errors
    ///
    /// As [`Script::resume_with_value`].
    pub fn resume<R: FromSeal>(&mut self, task: TaskId, value: impl IntoSeal) -> Result<Called<R>, Error> {
        self.check_running()?;
        let procedure = self
            .tasks
            .get(task.index as usize)
            .filter(|slot| slot.generation == task.generation)
            .and_then(|slot| slot.running.as_ref())
            .map(|running| running.procedure);
        if let Some(procedure) = procedure {
            self.check_result::<R>(procedure)?;
        }
        let called = self.resume_with_value(task, &value.into_seal())?;
        self.typed(called)
    }

    /// Whether a procedure can pause, as worked out from its body (D52). Nothing when there's no such procedure.
    #[must_use]
    pub fn can_pause(&self, procedure: &str) -> Option<bool> {
        self.compiled.procedures.iter().find(|found| found.name == procedure).map(|found| found.can_pause)
    }

    /// The names of the script's procedures, the ones a host can call (D4).
    #[must_use]
    pub fn procedures(&self) -> Vec<&str> {
        self.compiled.procedures.iter().map(|procedure| procedure.name.as_str()).collect()
    }

    /// How many bytes of its memory cap the script holds now, in Seal's own sizes (D14, D50).
    #[must_use]
    pub fn memory_in_use(&self) -> usize {
        self.machine.heap.in_use()
    }

    /// Checks the script's own bookkeeping: that every heap value's count of holders is exactly who holds it
    /// (D15). It's for tests and the fuzzer; a difference is a bug in Seal, never in a script.
    ///
    /// # Errors
    ///
    /// Says which value's count is off.
    pub fn audit(&self) -> Result<(), String> {
        if self.stopped.is_some() {
            return Ok(());
        }
        let mut roots: Vec<u64> = self.machine.texts.clone();
        for (slot, reference) in self.machine.globals.iter().zip(&self.compiled.global_references) {
            if *reference {
                roots.push(*slot);
            }
        }
        for task in self.tasks.iter().filter_map(|slot| slot.running.as_ref()) {
            roots.extend(task.task.held_slots(&self.compiled));
        }
        self.machine.heap.audit(&roots, &self.compiled.layouts)
    }

    /// The most bytes of its memory cap the script has held at once, in Seal's own sizes (D14, D50).
    #[must_use]
    pub fn most_memory_used(&self) -> usize {
        self.machine.heap.most()
    }

    /// Why the script stopped, when it has (D14).
    #[must_use]
    pub fn stopped(&self) -> Option<&Message> {
        self.stopped.as_ref()
    }

    /// The file name the script was loaded under.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}
