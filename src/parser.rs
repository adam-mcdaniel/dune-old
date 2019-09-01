extern crate honeycomb;
use honeycomb::{
    atoms::{any, eof, opt, rec, seq_no_ws, space, sym},
    language::{array, identifier, number, string},
    transform::to_number,
    Parser,
};

use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::tokens::{Expr, FnCall, Identifier, Literal, Name, Suite, Value, Builtin};

/// This parses a string literal
pub fn string_literal() -> Parser<Literal> {
    ((space() >> string() << space()) - Literal::String) % "a string literal"
}

/// This parses a number literal
pub fn number_literal() -> Parser<Literal> {
    ((space() >> (number() - to_number) << space()) - Literal::Number) % "a number literal"
}

/// This matches either a number or string literal
pub fn literal() -> Parser<Value> {
    (string_literal() | number_literal()) - Value::Literal
}

/// This matches a simple identifier
pub fn builtin() -> Parser<Value> {
    ident().is() >> (
        (seq_no_ws("ls") - |_| Builtin::List)
        | (seq_no_ws("mv") - |_| Builtin::Move)
        | (seq_no_ws("cd") - |_| Builtin::ChangeDir)
        | (seq_no_ws("rm") - |_| Builtin::Remove)
        | (seq_no_ws("pwd") - |_| Builtin::WorkingDir)
        | (seq_no_ws("exit") - |_| Builtin::Exit)
    ) - Value::Builtin
}

/// This matches a simple identifier
pub fn ident() -> Parser<Identifier> {
    ((space() >> identifier() << space()) - Identifier) % "an identifier"
}

/// This matches a value, succeeded by dot separated identifiers
pub fn dot_ident(values: Parser<Value>) -> Parser<(Box<Value>, Vec<Identifier>)> {
    ((values & ((sym('.') >> rec(ident)) * (1..)))
        - |v: (Value, Vec<Identifier>)| (Box::new(v.0), v.1))
        % "a dotted name"
}

/// This matches an identifier, a dotted name, or an indexed name
pub fn name() -> Parser<Name> {
    // Accept a dot name with the head value being one of
    // 1) group
    // 2) literal
    // 3) identifier
    ((dot_ident(group() | literal() | (ident() - Name::Name - Value::Name))
        - |d| Name::DotName(d.0, d.1))
        // Accept an indexed name with the head value being one of
        // 1) group
        // 2) literal
        // 3) identifier
        // Accept an identifier
        | (ident() - Name::Name))
        % "a dotted name, an indexed value, or an identifier"
}

/// This matches a function call, a value called with arguments
pub fn fncall() -> Parser<Value> {
    // The value being called can either be
    // 1) name (identifier, dotted name, indexed name)
    // 2) group
    // The arguments can be () enclosed and comma separated values
    // there can be 0 or more values.
    ((((builtin() | (name() - Value::Name) | rec(group)) & array("(", rec(value), ")"))
        - |call_data: (Value, Vec<Value>)| {
            Value::FnCall(FnCall(Box::new(call_data.0), call_data.1))
        })
    | ((((builtin() | (name() - Value::Name) | rec(group)) & (rec(value) * (1..)))
        - |call_data: (Value, Vec<Value>)| {
            Value::FnCall(FnCall(Box::new(call_data.0), call_data.1))
        }))
    )
        % "a value followed by comma arguments"
}

/// This matches a grouped value, any () enclosed value
pub fn group() -> Parser<Value> {
    seq_no_ws("(") >> rec(value) << seq_no_ws(")")
}

/// This matches values that do not have the possibility of
/// entering a recursive loop.
pub fn flat_value() -> Parser<Value> {
    // Literal is not recursive
    literal()
}

/// This matches values that DO have a possibility of
/// entering a recursive loop.
pub fn recursive_value() -> Parser<Value> {
    // These values are POTENTIALLY recursive
    // They require the use of the `value` parser
    rec(fncall) | builtin() | (name() - Value::Name) | rec(group)
}

/// This represents an atomic value
pub fn value() -> Parser<Value> {
    rec(recursive_value) | rec(flat_value)
}

/// This stores to an identifier,
/// or assigns to an indexed value
pub fn assignment() -> Parser<Expr> {
    ((name() & (seq_no_ws("=") >> value())) - |(n, v)| Expr::Assignment(n, v))
        % "a valid assignment"
}

/// While a condition is true, execute a suite
pub fn while_loop() -> Parser<Expr> {
    (((seq_no_ws("while") >> value()) & rec(suite)) - |(n, v)| Expr::WhileLoop(n, v))
        % "a valid while loop"
}

/// If a condition is true, execute a suite
/// else, execute a suite
pub fn if_then_else() -> Parser<Expr> {
    ((((seq_no_ws("if") >> value()) & rec(suite)) & opt(seq_no_ws("else") >> rec(suite)))
        - |((condition, then_body), else_body_opt)| {
            let else_body = match else_body_opt {
                Some(body) => body,
                None => Suite(Vec::new()),
            };

            Expr::IfThenElse(condition, then_body, else_body)
        })
        % "a valid if else statement"
}

/// A fundamental language expression
pub fn expr() -> Parser<Expr> {
    opt(comment() * (..))
        >> (((assignment() << opt(seq_no_ws(";"))) % "a valid assignment")
            | while_loop()
            | if_then_else()
            | (((value() - Expr::Value) << opt(seq_no_ws(";"))) % "a value"))
        << opt(comment() * (..))
}

/// A series of instructions enclosed with {}
pub fn suite() -> Parser<Suite> {
    ((seq_no_ws("{") >> (expr() * (..)) << seq_no_ws("}")) - Suite)
        % "a curly brace enclosed list of expressions"
}

/// Matches a comment in source code
pub fn comment() -> Parser<()> {
    (seq_no_ws("#") >> ((sym('\n').isnt() >> any()) * (..))) - |_| ()
}

/// A series of expressions
pub fn program() -> Parser<Suite> {
    ((expr() * (..)) - Suite) << eof()
}
