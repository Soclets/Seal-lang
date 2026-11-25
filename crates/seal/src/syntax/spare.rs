//! The lists a script's syntax tree is made of, emptied once the tree is done with and kept for the next script read
//! on the same thread, so reading one doesn't make and free them all again (D140).
//!
//! Only the usual lists are kept: a statement's block, a call's parts, a procedure's name and parameters, and the
//! script's items. Any other list in the tree is freed as usual, so a kind of list left out here costs only its reuse.

use std::cell::RefCell;

use super::tree::{
    Entry, Expression, ExpressionKind, Item, Parameter, Procedure, Script, Statement, StatementKind, Word,
};

/// The most lists of each kind kept on a thread.
const MOST_KEPT: usize = 256;

/// The most room a list may have and still be kept, so a big script's lists aren't held on to.
const MOST_ROOM: usize = 64;

/// Emptied lists of each kind, as lists of things borrowed from no script yet.
#[derive(Default)]
pub(crate) struct SpareLists {
    items: Vec<Vec<Item<'static>>>,
    statements: Vec<Vec<Statement<'static>>>,
    entries: Vec<Vec<Entry<'static>>>,
    words: Vec<Vec<Word<'static>>>,
    parameters: Vec<Vec<Parameter<'static>>>,
}

thread_local! {
    static SPARE_LISTS: RefCell<SpareLists> = RefCell::default();
}

/// A list emptied and made a list of the same things borrowed from any script, in the same room: Rust collects an
/// emptied list's `filter_map` into the room it already has, and `filter` couldn't change its type.
#[allow(clippy::unnecessary_filter_map)]
fn emptied<T, U>(mut list: Vec<T>) -> Vec<U> {
    list.clear();
    list.into_iter().filter_map(|_| None).collect()
}

/// Keeps a list, once its things are dealt with, unless it has no room, too much, or enough of its kind are kept.
fn keep<T, U>(kept: &mut Vec<Vec<U>>, list: Vec<T>) {
    if list.capacity() == 0 || list.capacity() > MOST_ROOM || kept.len() >= MOST_KEPT {
        return;
    }
    kept.push(emptied(list));
}

/// Gives back the kept lists of one kind, adding to those already there.
fn give_back<T>(kept: &mut Vec<Vec<T>>, more: Vec<Vec<T>>) {
    if kept.is_empty() {
        *kept = more;
    } else {
        kept.extend(more.into_iter().take(MOST_KEPT.saturating_sub(kept.len())));
    }
}

impl SpareLists {
    /// The lists kept on this thread, taken for a script being read.
    pub(crate) fn take() -> Self {
        SPARE_LISTS
            .with(|spare| spare.try_borrow_mut().map(|mut spare| std::mem::take(&mut *spare)).unwrap_or_default())
    }

    /// Gives back the lists a script's reading didn't use.
    pub(crate) fn give_back(self) {
        SPARE_LISTS.with(|spare| {
            if let Ok(mut spare) = spare.try_borrow_mut() {
                give_back(&mut spare.items, self.items);
                give_back(&mut spare.statements, self.statements);
                give_back(&mut spare.entries, self.entries);
                give_back(&mut spare.words, self.words);
                give_back(&mut spare.parameters, self.parameters);
            }
        });
    }

    pub(crate) fn items<'a>(&mut self) -> Vec<Item<'a>> {
        self.items.pop().unwrap_or_default()
    }

    pub(crate) fn statements<'a>(&mut self) -> Vec<Statement<'a>> {
        self.statements.pop().unwrap_or_default()
    }

    /// A list for a call's parts or a list's entries, with room for a few from the start.
    pub(crate) fn entries<'a>(&mut self) -> Vec<Entry<'a>> {
        self.entries.pop().unwrap_or_else(|| Vec::with_capacity(4))
    }

    pub(crate) fn words<'a>(&mut self) -> Vec<Word<'a>> {
        self.words.pop().unwrap_or_default()
    }

    pub(crate) fn parameters<'a>(&mut self) -> Vec<Parameter<'a>> {
        self.parameters.pop().unwrap_or_default()
    }

    fn item(&mut self, item: Item<'_>) {
        match item {
            Item::Statement(statement) => self.statement(statement),
            Item::Procedure(procedure) => self.procedure(procedure),
            _ => {}
        }
    }

    fn procedure(&mut self, procedure: Procedure<'_>) {
        keep(&mut self.words, procedure.path);
        keep(&mut self.parameters, procedure.parameters);
        self.block(procedure.body);
    }

    fn block(&mut self, mut block: Vec<Statement<'_>>) {
        for statement in block.drain(..) {
            self.statement(statement);
        }
        keep(&mut self.statements, block);
    }

    fn statement(&mut self, statement: Statement<'_>) {
        match statement.kind {
            StatementKind::Name { value, .. }
            | StatementKind::Expression(value)
            | StatementKind::Instead(value)
            | StatementKind::Return(Some(value)) => self.expression(value),
            StatementKind::Change { place, value } => {
                self.expression(place);
                self.expression(value);
            }
            StatementKind::If { branches, otherwise } => {
                for (condition, body) in branches {
                    self.expression(condition);
                    self.block(body);
                }
                if let Some(body) = otherwise {
                    self.block(body);
                }
            }
            StatementKind::While { condition, body } => {
                self.expression(condition);
                self.block(body);
            }
            StatementKind::ForEach { source, body, .. } => {
                self.expression(source);
                self.block(body);
            }
            StatementKind::Match { subject, cases } => {
                self.expression(subject);
                for case in cases {
                    keep(&mut self.words, case.values);
                    self.block(case.body);
                }
            }
            StatementKind::Break | StatementKind::Return(None) | StatementKind::Fail(_) => {}
        }
    }

    fn expression(&mut self, expression: Expression<'_>) {
        match expression.kind {
            ExpressionKind::Call { parts } => self.entries_of(parts),
            ExpressionKind::Bracketed { receiver, entries } => {
                self.expression(*receiver);
                self.entries_of(entries);
            }
            ExpressionKind::Member { receiver: inner, .. }
            | ExpressionKind::Group(inner)
            | ExpressionKind::Try(inner)
            | ExpressionKind::Unary { operand: inner, .. } => self.expression(*inner),
            ExpressionKind::Binary { left, right, .. } => {
                self.expression(*left);
                self.expression(*right);
            }
            ExpressionKind::List(items) => {
                for item in items {
                    self.expression(item);
                }
            }
            _ => {}
        }
    }

    fn entries_of(&mut self, mut entries: Vec<Entry<'_>>) {
        for entry in entries.drain(..) {
            self.expression(entry.value);
        }
        keep(&mut self.entries, entries);
    }
}

/// Keeps the lists a script's syntax tree is made of, once it's done with, for the next script read on this thread.
pub fn recycle(script: Script<'_>) {
    SPARE_LISTS.with(|spare| {
        let Ok(mut spare) = spare.try_borrow_mut() else { return };
        let mut items = script.items;
        for item in items.drain(..) {
            spare.item(item);
        }
        keep(&mut spare.items, items);
    });
}
