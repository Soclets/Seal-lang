//! `#[derive(Seal)]`: lets a Rust struct or enum cross a script's doors as a record or a `one_of` (D27, D35, D48).
//!
//! A struct with named fields stands for a record of the same name and fields. An enum stands for a `one_of`: each
//! of its cases, written in Rust's `CamelCase`, is the Seal case in `snake_case`, with named values when it carries
//! any. An enum whose cases carry nothing can also be a door's failures (D44).
//!
//! The macro reads the struct or enum with Rust's own `proc_macro` and nothing else, so it brings no dependencies.
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(clippy::pedantic)]

use std::fmt::Write;

use proc_macro::{Delimiter, TokenStream, TokenTree};

/// A field: its name, and its type as written.
struct Field {
    name: String,
    written_type: String,
}

/// An enum's case: its Rust name, and its fields when it carries values.
struct Case {
    name: String,
    fields: Vec<Field>,
}

enum Shape {
    Record(Vec<Field>),
    OneOf(Vec<Case>),
}

fn error(message: &str) -> TokenStream {
    format!("compile_error!({message:?});").parse().unwrap_or_default()
}

/// `HealedBy` to `healed_by`: a Rust case's name as Seal spells a case (D24).
fn snake_case(name: &str) -> String {
    let mut snake = String::new();
    for character in name.chars() {
        if character.is_ascii_uppercase() {
            if !snake.is_empty() {
                snake.push('_');
            }
            snake.push(character.to_ascii_lowercase());
        } else {
            snake.push(character);
        }
    }
    snake
}

/// Reads `name: Type, name: Type` from inside a struct's or a case's braces.
fn read_fields(stream: TokenStream) -> Result<Vec<Field>, &'static str> {
    let mut fields = Vec::new();
    let mut tokens = stream.into_iter().peekable();
    loop {
        // Skip attributes and `pub`, with whatever follows it in brackets.
        loop {
            match tokens.peek() {
                Some(TokenTree::Punct(punct)) if punct.as_char() == '#' => {
                    tokens.next();
                    tokens.next();
                }
                Some(TokenTree::Ident(ident)) if ident.to_string() == "pub" => {
                    tokens.next();
                    if matches!(tokens.peek(), Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Parenthesis)
                    {
                        tokens.next();
                    }
                }
                _ => break,
            }
        }
        let Some(first) = tokens.next() else { break };
        let TokenTree::Ident(name) = first else {
            return Err("`#[derive(Seal)]` needs fields with names, as `name: Type`");
        };
        match tokens.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == ':' => {}
            _ => return Err("`#[derive(Seal)]` needs fields with names, as `name: Type`"),
        }
        // The type runs to the next comma that isn't inside `< >`.
        let mut written_type = String::new();
        let mut depth = 0i32;
        for token in tokens.by_ref() {
            if let TokenTree::Punct(punct) = &token {
                match punct.as_char() {
                    '<' => depth += 1,
                    '>' => depth -= 1,
                    ',' if depth == 0 => break,
                    _ => {}
                }
            }
            written_type.push_str(&token.to_string());
            written_type.push(' ');
        }
        fields.push(Field { name: name.to_string(), written_type });
    }
    Ok(fields)
}

fn read_cases(stream: TokenStream) -> Result<Vec<Case>, &'static str> {
    let mut cases = Vec::new();
    let mut tokens = stream.into_iter().peekable();
    loop {
        while matches!(tokens.peek(), Some(TokenTree::Punct(punct)) if punct.as_char() == '#') {
            tokens.next();
            tokens.next();
        }
        let Some(first) = tokens.next() else { break };
        let TokenTree::Ident(name) = first else {
            return Err("`#[derive(Seal)]` couldn't read this enum's cases");
        };
        let mut fields = Vec::new();
        match tokens.peek() {
            Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
                fields = read_fields(group.stream())?;
                tokens.next();
            }
            Some(TokenTree::Group(_)) => {
                return Err(
                    "`#[derive(Seal)]` needs a case's values named, as `Hurt { amount: i64 }`, since a `one_of`'s values have names",
                );
            }
            _ => {}
        }
        match tokens.next() {
            None => {
                cases.push(Case { name: name.to_string(), fields });
                break;
            }
            Some(TokenTree::Punct(punct)) if punct.as_char() == ',' => {}
            Some(_) => return Err("`#[derive(Seal)]` can't read a case with `=` or anything else after it"),
        }
        cases.push(Case { name: name.to_string(), fields });
    }
    Ok(cases)
}

fn read_item(input: TokenStream) -> Result<(String, Shape), &'static str> {
    let mut tokens = input.into_iter();
    let mut kind = None;
    for token in tokens.by_ref() {
        if let TokenTree::Ident(ident) = &token {
            let word = ident.to_string();
            if word == "struct" || word == "enum" {
                kind = Some(word);
                break;
            }
        }
    }
    let Some(kind) = kind else {
        return Err("`#[derive(Seal)]` goes on a struct or an enum");
    };
    let Some(TokenTree::Ident(name)) = tokens.next() else {
        return Err("`#[derive(Seal)]` couldn't read the type's name");
    };
    match tokens.next() {
        Some(TokenTree::Group(group)) if group.delimiter() == Delimiter::Brace => {
            let shape = if kind == "struct" {
                Shape::Record(read_fields(group.stream())?)
            } else {
                Shape::OneOf(read_cases(group.stream())?)
            };
            Ok((name.to_string(), shape))
        }
        Some(TokenTree::Punct(punct)) if punct.as_char() == '<' => Err(
            "`#[derive(Seal)]` can't read a type with `< >`: Seal's records and `one_of`s have no blanks to fill in",
        ),
        _ => Err("`#[derive(Seal)]` needs a struct with named fields, as `struct Enemy { name: String, hp: i64 }`"),
    }
}

fn field_types(fields: &[Field]) -> String {
    let mut out = String::from("vec![");
    for field in fields {
        let _ =
            write!(out, "({:?}.to_owned(), <{} as ::seal::SealType>::host_type()),", field.name, field.written_type);
    }
    out.push(']');
    out
}

fn fields_into(fields: &[Field], prefix: &str) -> String {
    let mut out = String::from("vec![");
    for field in fields {
        let _ = write!(out, "::seal::IntoSeal::into_seal({prefix}{}),", field.name);
    }
    out.push(']');
    out
}

fn fields_from(fields: &[Field]) -> String {
    let mut out = String::new();
    for field in fields {
        let _ = write!(out, "{}: ::seal::FromSeal::from_seal(fields.next()?)?,", field.name);
    }
    out
}

fn record(name: &str, fields: &[Field]) -> String {
    format!(
        "impl ::seal::SealType for {name} {{
            fn host_type() -> ::seal::HostType {{
                ::seal::describing({name:?}, || ::seal::HostType::Record {{ name: {name:?}.to_owned(), fields: {types} }})
            }}
        }}
        impl ::seal::IntoSeal for {name} {{
            fn into_seal(self) -> ::seal::Value {{
                ::seal::Value::Record({into})
            }}
        }}
        impl ::seal::FromSeal for {name} {{
            fn from_seal(value: ::seal::Value) -> ::core::option::Option<Self> {{
                let ::seal::Value::Record(fields) = value else {{ return ::core::option::Option::None }};
                let mut fields = fields.into_iter();
                ::core::option::Option::Some({name} {{ {from} }})
            }}
        }}",
        types = field_types(fields),
        into = fields_into(fields, "self."),
        from = fields_from(fields),
    )
}

fn one_of(name: &str, cases: &[Case]) -> String {
    let mut case_types = String::from("vec![");
    let mut into_arms = String::new();
    let mut from_arms = String::new();
    let mut failure_names = String::from("vec![");
    let mut failure_arms = String::new();
    for (number, case) in cases.iter().enumerate() {
        let seal_name = snake_case(&case.name);
        let rust_name = &case.name;
        let _ = write!(case_types, "({seal_name:?}.to_owned(), {}),", field_types(&case.fields));
        let mut bindings = String::new();
        for field in &case.fields {
            let _ = write!(bindings, "{},", field.name);
        }
        if case.fields.is_empty() {
            let _ = write!(
                into_arms,
                "{name}::{rust_name} => ::seal::Value::Case {{ case: {number}, fields: ::std::vec::Vec::new() }},"
            );
            let _ = write!(from_arms, "{number} => ::core::option::Option::Some({name}::{rust_name}),");
        } else {
            let _ = write!(
                into_arms,
                "{name}::{rust_name} {{ {bindings} }} => ::seal::Value::Case {{ case: {number}, fields: {} }},",
                fields_into(&case.fields, "")
            );
            let _ = write!(
                from_arms,
                "{number} => ::core::option::Option::Some({name}::{rust_name} {{ {} }}),",
                fields_from(&case.fields)
            );
        }
        let _ = write!(failure_names, "{seal_name:?}.to_owned(),");
        let _ = write!(failure_arms, "{name}::{rust_name} => {seal_name:?}.to_owned(),");
    }
    case_types.push(']');
    failure_names.push(']');
    let mut out = format!(
        "impl ::seal::SealType for {name} {{
            fn host_type() -> ::seal::HostType {{
                ::seal::describing({name:?}, || ::seal::HostType::OneOf {{ name: {name:?}.to_owned(), cases: {case_types} }})
            }}
        }}
        impl ::seal::IntoSeal for {name} {{
            fn into_seal(self) -> ::seal::Value {{
                match self {{ {into_arms} }}
            }}
        }}
        impl ::seal::FromSeal for {name} {{
            fn from_seal(value: ::seal::Value) -> ::core::option::Option<Self> {{
                let ::seal::Value::Case {{ case, fields }} = value else {{ return ::core::option::Option::None }};
                #[allow(unused_mut, unused_variables)]
                let mut fields = fields.into_iter();
                match case {{ {from_arms} _ => ::core::option::Option::None }}
            }}
        }}"
    );
    // An enum whose cases carry nothing can be a door's failures too (D44).
    if cases.iter().all(|case| case.fields.is_empty()) {
        let _ = write!(
            out,
            "impl ::seal::Failures for {name} {{
                fn names() -> ::std::vec::Vec<::std::string::String> {{ {failure_names} }}
                fn name(&self) -> ::std::string::String {{ match self {{ {failure_arms} }} }}
            }}"
        );
    }
    out
}

/// Makes a struct a Seal record, or an enum a Seal `one_of`, at the doors (D27, D35, D48).
#[proc_macro_derive(Seal)]
pub fn derive_seal(input: TokenStream) -> TokenStream {
    let generated = match read_item(input) {
        Ok((name, Shape::Record(fields))) => record(&name, &fields),
        Ok((name, Shape::OneOf(cases))) => one_of(&name, &cases),
        Err(message) => return error(message),
    };
    generated.parse().unwrap_or_else(|_| error("`#[derive(Seal)]` made code that doesn't read; please report it"))
}
