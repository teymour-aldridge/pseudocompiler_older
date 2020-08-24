//! Performs lexical analysis on a string.

#[cfg(test)]
mod integration_tests;
#[cfg(test)]
mod unit_tests;

use thiserror::Error as ThisError;

#[derive(Debug, Copy, Clone)]
pub enum Keyword {
    Function,
    EndFunction,
    If,
    Then,
    ElseIf,
    Else,
    EndIf,
    Switch,
    Case,
    Default,
    EndSwitch,
    While,
    EndWhile,
    Do,
    Until,
    For,
    To,
    Next,
}

#[derive(Debug, Clone)]
pub enum Punctuation {
    OpenRoundBracket,
    CloseRoundBracket,
    ByRef,
    ByVal,
    Colon,
    Comma,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Equals,
}

#[derive(Debug, Clone)]
/// A single token lexed from the input stream.
pub enum Token {
    Keyword(Keyword),
    Ident(String),
    Punctuation(Punctuation),
    Operator(Operator),
    Integer(i64),
    String(String),
    Comment(String),
    MultiLineComment(String),
    Float(f64),
}

pub fn lex(input: &mut str) -> Result<Vec<Token>, LexError> {
    let mut cursor  = Cursor::new(input.to_string());
    cursor.lex_statement()?;
    Ok(cursor.output)
}

#[derive(Copy, Clone, Debug)]
pub struct Loc {
    line: u32,
    col: u32,
}

impl Loc {
    pub fn new(line: u32, col: u32) -> Self {
        Self { line, col }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Span {
    start: Loc,
    stop: Loc,
}

impl Span {
    pub fn new(start: Loc, stop: Loc) -> Self {
        Self { start, stop }
    }
}

#[derive(Debug, Clone)]
struct Cursor {
    input: String,
    location: Loc,
    output: Vec<Token>,
    current_indentation: u32,
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    span: Span,
    token: String,
}

impl SpannedToken {
    pub fn new(span: Span, token: String) -> Self {
        Self { span, token }
    }
}

#[derive(ThisError, Debug)]
/// An error encountered in the program while trying to conduct lexical analysis on the file.
pub enum LexError {
    #[error("didn't expect this token")]
    UnexpectedToken(SpannedToken),
    #[error("something's up with the indentation")]
    /// This will be made more intelligible soon.
    IndentationError,
    #[error("the input ended unexpectedly")]
    UnexpectedEndOfInput,
}

impl Cursor {
    /// Creates a new cursor from a string input.
    fn new(string: String) -> Self {
        Self {
            input: string,
            output: vec![],
            location: Loc::new(0, 0),
            current_indentation: 0,
        }
    }
    /// Lexes an application of a function.
    ///
    /// The term "application" originally comes from Alonzo Church's lambda calculus which is a way
    /// of "doing" mathematics using only functions.
    fn lex_application(&mut self) -> Result<(), LexError> {
        self.lex_identifier()?;
        self.lex_specific_punctuation(Punctuation::OpenRoundBracket)?;
        loop {
            let next = self.peek().unwrap();
            if !next.is_alphanumeric() {
                break;
            }
            self.lex_identifier()?;
            if self.peek().unwrap() == ',' {
                self.lex_specific_punctuation(Punctuation::Comma)?;
            }
        }
        self.lex_specific_punctuation(Punctuation::CloseRoundBracket)?;
        Ok(())
    }
    /// Retrieves the current location of the cursor.
    fn save_loc(&self) -> Loc {
        *&self.location
    }
    /// Lexes any assignment.
    /// This includes the use of the "syntactic sugar" `+=`, `*=`  and `-=`.
    fn lex_assignment_statement(&mut self) -> Result<(), LexError> {
        self.lex_identifier()?;
        self.lex_specific_operator(Operator::Equals)?;
        self.lex_expression()?;
        Ok(())
    }
    /// Lexes any valid statement.
    fn lex_statement(&mut self) -> Result<(), LexError> {
        if let Some(token) = self.peek_token() {
            if token.contains('(') {
                self.lex_application()?;
                return Ok(());
            }
            match token {
                "function" => self.lex_function()?,
                "if" => self.lex_if_statement()?,
                "switch" => self.lex_switch_statement()?,
                "while" => self.lex_while_statement()?,
                "for" => self.lex_for_statement()?,
                _ => self.lex_assignment_statement()?,
            };
            Ok(())
        } else {
            Err(LexError::UnexpectedEndOfInput)
        }
    }
    fn count_indents(&self) -> u32 {
        let mut count = 0;
        let mut iterator = self.input.chars();
        while let Some(next) = iterator.next() {
            if next == ' ' {
                count += 1;
            } else if next == '\t' {
                count += 4
            } else {
                break;
            }
        }
        count
    }

    /// Lexes code in an indented block.
    fn lex_block(&mut self) -> Result<(), LexError> {
        loop {
            let indents = self.count_indents();
            if indents == self.current_indentation {
                self.lex_statement()?;
                self.lex_newline()?;
            } else {
                return if indents == (self.current_indentation - 1) {
                    Ok(())
                } else {
                    Err(LexError::IndentationError)
                };
            }
        }
    }
    /// Eats any whitespace tokens between where the cursor presently is and the next non-whitespace
    /// token.
    fn consume_whitespace(&mut self) {
        while let Some(next) = self.peek() {
            if next.is_whitespace() {
                self.eat();
            } else {
                break;
            }
        }
    }
    /// Retrieves the next character without advancing the position of the cursor.
    ///
    /// Returns `None` if there are no more tokens in the stream.
    #[inline(always)]
    fn peek(&self) -> Option<char> {
        self.input.chars().next()
    }
    /// Retrieves the next "token" (anything up to the next space).
    #[inline(always)]
    fn peek_token(&self) -> Option<&str> {
        self.input.split(' ').next()
    }
    /// Removes the next character and advances the position of the cursor.
    ///
    /// Returns `None` if there are no more tokens in the stream.
    #[inline(always)]
    fn eat(&mut self) -> Option<char> {
        if self.input.is_empty() {
            return None;
        }
        let result = self.input.remove(0);
        // increment location pointer
        if result == '\n' {
            self.location.line += 1;
        } else if result == '\t' {
            self.location.col += 4;
        } else {
            self.location.col += 1;
        }
        Some(result)
    }
    fn lex_specific_keyword(&mut self, keyword: Keyword) -> Result<(), LexError> {
        macro_rules! keywords {
            ($self:ident, $( [ $string:expr => $keyword:ident ] ),+) => {
                match keyword {
                    $($crate::lexer::Keyword::$keyword => {
                        let start = $self.save_loc();
                        if self.input.starts_with($string) {
                            for _ in 1..$string.len() {
                                self.eat();
                            }
                            $self.output.push($crate::lexer::Token::Keyword(
                                $crate::lexer::Keyword::$keyword
                            ));
                            return Ok(())
                        } else {
                            return Err($crate::lexer::LexError::UnexpectedToken(
                                $crate::lexer::SpannedToken::new(
                                    $crate::lexer::Span::new(start, $self.save_loc()), "".to_string()
                                )
                            ))
                        }
                    })+
                }
            };
        }
        keywords!(self,
            ["function" => Function],
            ["endfunction" => EndFunction],
            ["if" => If],
            ["then" => Then],
            ["elseif" => ElseIf],
            ["else" => Else],
            ["endif" => EndIf],
            ["switch" => Switch],
            ["case" => Case],
            ["default" => Default],
            ["endswitch" => EndSwitch],
            ["while" => While],
            ["endwhile" => EndWhile],
            ["do" => Do],
            ["until" => Until],
            ["for" => For],
            ["to" => To],
            ["next" => Next]
        )
    }
    fn lex_identifier(&mut self) -> Result<(), LexError> {
        let mut output = String::new();
        while let Some(next) = self.peek() {
            if next.is_alphanumeric() {
                output.push(next);
                self.eat();
            } else {
                self.output.push(Token::Ident(output));
                return Ok(());
            }
        }
        self.output.push(Token::Ident(output));
        Ok(())
    }
    /// Lexes the specified item of punctuation.
    fn lex_specific_punctuation(&mut self, punctuation: Punctuation) -> Result<(), LexError> {
        macro_rules! punctuation {
            ($self:ident, $punctuation:ident, $(($string:expr => $punct:ident)),+) => {
                let start = $self.save_loc();
                match $punctuation {
                    $(
                        $crate::lexer::Punctuation::$punct => {
                            if $self.input.starts_with($string) {
                                for _ in 1..$string.len() {
                                    self.eat();
                                }
                                $self.output.push(
                                    $crate::lexer::Token::Punctuation(
                                        $crate::lexer::Punctuation::$punct
                                    )
                                );
                                return Ok(())
                            } else {
                                // todo fix this
                                panic!("expected token")
                            }
                        }
                    )*
                }
            }
        }
        let start = self.save_loc();
        punctuation!(self, punctuation,
            ("(" => OpenRoundBracket),
            (")" => CloseRoundBracket),
            (":byRef" => ByRef),
            (":byVal" => ByVal),
            (":" => Colon),
            ("," => Comma)
        );
        Ok(())
    }
    /// Lexes `argument:byRef` and `argument:byVal`
    fn lex_optional_argument_modifier(&mut self) -> Result<(), LexError> {
        if let Some(next) = self.peek() {
            if next == ':' {
                if self.lex_specific_punctuation(Punctuation::ByRef).is_err() {
                    self.lex_specific_punctuation(Punctuation::ByVal)?;
                }
            }
        }
        Ok(())
    }
    /// Lexes a functions arguments.
    fn lex_function_arguments(&mut self) -> Result<(), LexError> {
        self.lex_specific_punctuation(Punctuation::OpenRoundBracket)?;
        self.lex_identifier()?;
        self.lex_specific_punctuation(Punctuation::CloseRoundBracket)?;
        Ok(())
    }
    /// Lexes a function definition.
    fn lex_function(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::Function)?;
        self.consume_whitespace();
        self.lex_identifier()?;
        self.lex_function_arguments()?;
        self.lex_block()?;
        self.lex_specific_keyword(Keyword::EndFunction)?;
        Ok(())
    }
    fn lex_float(&mut self) -> Result<(), LexError> {
        self.output.push(Token::Float(
            self.peek_token()
                .expect("missing token")
                .parse::<f64>()
                .expect("error parsing float"),
        ));
        Ok(())
    }
    /// Lexes an expression
    fn lex_expression(&mut self) -> Result<(), LexError> {
        let start = self.save_loc();
        while let Some(item) = self.peek() {
            if item.is_alphabetic() {
                if self
                    .peek_token()
                    .expect("unexpected end of input")
                    .contains("(")
                {
                    self.lex_application()?;
                } else {
                    self.lex_identifier()?;
                }
            } else if item.is_numeric() {
                if self
                    .peek_token()
                    .expect("unexpected end of input")
                    .contains(".")
                {
                    self.lex_float()?;
                } else {
                    self.lex_integer()?;
                }
            } else {
                if self.lex_any_operator().is_err() {
                    return Err(LexError::UnexpectedToken(SpannedToken::new(
                        Span::new(start, {
                            let mut loc = self.save_loc();
                            loc.col += 1;
                            loc
                        }),
                        item.to_string(),
                    )));
                }
            }
        }
        Ok(())
    }
    /// Lexes an if statement.
    fn lex_if_statement(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::If)?;
        self.lex_expression()?;
        self.lex_specific_keyword(Keyword::Then)?;
        self.lex_block()?;
        if self.lex_specific_keyword(Keyword::ElseIf).is_ok() {
            self.lex_expression()?;
            self.lex_specific_keyword(Keyword::Then)?;
            self.lex_block()?;
        }
        if self.lex_specific_keyword(Keyword::Else).is_ok() {
            self.lex_block()?;
        }
        self.lex_specific_keyword(Keyword::EndIf)?;
        Ok(())
    }
    /// Consumes as many newlines as is possible.
    fn consume_newlines(&mut self) {
        while let Some(next) = self.peek() {
            if next == '\n' {
                self.eat();
            } else {
                break;
            }
        }
    }
    /// Lexes a switch statement.
    fn lex_switch_statement(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::Switch)?;
        // for the moment expressions cannot be switched on
        // this is only temporary
        self.lex_identifier()?;
        self.lex_specific_punctuation(Punctuation::Colon)?;

        loop {
            self.lex_newline()?;
            self.consume_newlines();
            self.lex_indentation()?;
            self.consume_whitespace();
            if self.lex_specific_keyword(Keyword::Case).is_err() {
                break;
            }
            self.lex_expression()?;
            self.set_indentation_level(2);
            self.lex_block()?;
        }

        self.lex_specific_keyword(Keyword::Default)?;
        self.consume_whitespace();
        self.lex_specific_punctuation(Punctuation::Colon)?;
        self.set_indentation_level(2);
        self.lex_block()?;

        self.lex_specific_keyword(Keyword::EndSwitch)?;

        Ok(())
    }
    /// Lexes a while statement
    fn lex_while_statement(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::While)?;
        self.lex_expression()?;
        self.lex_block()?;
        self.lex_specific_keyword(Keyword::EndWhile)?;
        Ok(())
    }
    fn lex_do_statement(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::Do)?;
        self.lex_newline()?;
        self.lex_block()?;
        self.lex_specific_keyword(Keyword::Until)?;
        self.lex_expression()?;
        Ok(())
    }
    /// Lexes an integer.
    fn lex_integer(&mut self) -> Result<(), LexError> {
        if let Some(next) = self.peek_token() {
            match next.parse::<i64>() {
                Ok(integer) => {
                    self.output.push(Token::Integer(integer));
                    return Ok(());
                }
                Err(_) => {
                    return Err(LexError::UnexpectedToken(SpannedToken::new(
                        Span::new(self.save_loc(), {
                            let mut loc = self.save_loc();
                            loc.col += next.len() as u32;
                            loc
                        }),
                        next.to_string(),
                    )))
                }
            }
        }
        Ok(())
    }
    fn lex_any_operator(&mut self) -> Result<(), LexError> {
        // todo: move this out and consolidate all of these definitions into one
        macro_rules! operators {
            ($self:ident, $(($string:expr => $op:ident)),+) => {
                $(
                    if self.input.starts_with($string) {
                        self.output.push(
                            $crate::lexer::Token::Operator($crate::lexer::Operator::$op)
                        );
                        return Ok(());
                    }
                )+
                else {
                    panic!("x")
                }
            }
        }
        operators!(self,
            ("=" => Equals)
        );
    }
    /// Lexes a specific operator.
    fn lex_specific_operator(&mut self, operator: Operator) -> Result<(), LexError> {
        macro_rules! operators {
            ($self:ident, $operator:ident, $(($string:expr => $op:ident)),+) => {
                let start = $self.save_loc();
                match $operator {
                    $(
                        $crate::lexer::Operator::$op => {
                            if self.input.starts_with($string) {
                                for _ in 1..$string.len() {
                                    self.eat();
                                }
                                self.output.push($crate::lexer::Token::Operator(
                                    $crate::lexer::Operator::$op
                                ));
                                return Ok(())
                            }
                            else {
                                return Err(
                                    $crate::lexer::LexError::UnexpectedToken(
                                        $crate::lexer::SpannedToken::new(
                                            Span::new(start, self.save_loc()),
                                            self.peek().unwrap().to_string()
                                        )
                                    )
                                )
                            }
                        }
                    )+
                }
            };
        }
        operators!(
            self, operator,
                ("=" => Equals)
        );
    }
    /// Lexes a for statement
    fn lex_for_statement(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::For)?;
        self.lex_identifier()?;
        self.lex_specific_operator(Operator::Equals)?;
        self.lex_integer()?;
        self.lex_specific_keyword(Keyword::To)?;
        self.lex_newline()?;
        self.lex_block()?;
        // why do they actually do this???
        self.lex_specific_keyword(Keyword::Next)?;
        self.lex_identifier()?;
        Ok(())
    }
    fn set_indentation_level(&mut self, level: u32) {
        self.current_indentation = level;
    }
    fn lex_newline(&mut self) -> Result<(), LexError> {
        self.consume_whitespace();
        if let Some(token) = self.eat() {
            if token == '\n' {
                return Ok(());
            } else {
                panic!("expected a newline")
            }
        } else {
            panic!("unexpected end of input")
        }
    }
    fn lex_two_spaces(&mut self) -> Result<(), LexError> {
        let mut spaces = 1;
        while spaces < 2 {
            let next = self.peek().expect("unexpected end of input");
            if next == ' ' {
                spaces += 1;
                self.eat();
            } else {
                panic!("expected a space. didn't get a space")
            }
        }
        Ok(())
    }
    /// Lexes a unit of indentation.
    fn lex_indentation(&mut self) -> Result<(), LexError> {
        let next = self.eat().expect("unexpected end of input");
        if next == '\t' {
            return Ok(());
        } else {
            if next == ' ' {
                self.lex_two_spaces()?;
                #[allow(unused)]
                {
                    self.lex_two_spaces();
                }

                Ok(())
            } else {
                panic!("expected some indentation")
            }
        }
    }
}
