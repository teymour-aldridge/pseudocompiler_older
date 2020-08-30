//! Constructs an AST from a token stream.
//!
//! This AST can then be operated on to output LLVM IR or Javascript code.

use crate::lexer::{Operator, Token};
use thiserror::Error as ThisError;

/// A program consists of a series of statements.
/// This function constructs an abstract syntax tree from the token outputted
/// by the lexer.
pub fn parse(tokens: Vec<Token>) -> Vec<Statement> {
    todo!()
}

#[derive(ThisError, Debug)]
pub enum ParseError {
    #[error("unexpected end of input")]
    UnexpectedEndOfInput,
}

/// This trait is used to parse tokens from the lexer's output.
pub trait Parse<OUTPUT = Self> {
    fn parse(cursor: &mut Cursor) -> Result<OUTPUT, ParseError>;
}

/// A cursor for reading from a stream of tokens.
///
/// Unlike in the case of the lexer, this doesn't need to keep track of `Span`s
//// because these are already inside the tokens.
pub struct Cursor {
    tokens: Vec<Token>,
}

impl Cursor {
    /// Construct a new cursor from the token stream.
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens }
    }
    /// Retrieves the next token in the input stream, without advancing the position
    /// of the cursor. If the stream has been exhausted, it will return an error of
    /// instance of `ParseError`. Using the `?` operator provides an ergonomic way
    /// to propagate errors within implementations of `Parse`.
    pub fn peek(&self) -> Result<Token, ParseError> {
        todo!()
    }
    pub fn eat(&mut self) -> Result<Token, ParseError> {
        todo!()
    }
}

/// An AST of sort `Statement`
///
/// Don't worry, completely over the top mathematical formulation will be coming to this
/// langauge soon.
///
/// There are a lot of statements in this language :P
pub enum Statement {
    ForStatement,
    WhileStatement,
    IfStatement,
    AssignmentStatement,
    DoUntilStatement,
    SwitchStatement,
}

impl Parse for Statement {
    fn parse(cursor: &mut Cursor) -> Result<Self, ParseError> {
        todo!()
    }
}

/// A block consists of zero or more statements.
type Block = Vec<Statement>;

/// A "do ... until ..." statement.
pub struct DoUntilStatement {
    predicate: Expression,
    block: Block,
}

/// A switch statement.
pub struct SwitchStatement {
    cases: Vec<SwitchCase>,
    default: Vec<DefaultCase>,
}

pub struct SwitchCase {
    predicate: Expression,
    block: Block,
}

pub struct DefaultCase {
    block: Block,
}

pub struct IfStatement {
    case_if: If,
    cases_elif: Vec<If>,
    case_else: Else,
}

/// In this form, `If` also handles "elif"  
pub struct If {
    predicate: Expression,
    block: Block,
}

pub struct Else {
    block: Block,
}

/// A for statement.
pub struct ForStatement {
    ident: String,
    start: u32,
    stop: u32,
    block: Block,
}

pub struct WhileStatement {
    predicate: Expression,
    block: Block,
}

/// An AST of sort `Expression`
pub struct Expression {
    /// The operator in question.
    operator: Operator,
    /// The operands on which the operator acts.
    operands: Vec<Box<Expression>>,
}

impl Parse for Expression {
    fn parse(cursor: &mut Cursor) -> Result<Self, ParseError> {
        todo!()
    }
}
