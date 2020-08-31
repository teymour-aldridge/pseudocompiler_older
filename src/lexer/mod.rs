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
    Return,
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Punctuation {
    OpenRoundBracket,
    CloseRoundBracket,
    ByRef,
    ByVal,
    Colon,
    Comma,
    Quote,
}

#[derive(Debug, Clone)]
pub enum Operator {
    Equals,
    Times,
    Plus,
    Minus,
    Divide,
    Comparison,
    And,
    Or,
    Not,
    NotEquals,
    Increment
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
    let mut cursor = Cursor::new(input.to_string());
    while !cursor.input.is_empty() {
        cursor.lex_statement()?;
        cursor.consume_whitespace();
    }
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
    pub input: String,
    pub location: Loc,
    pub output: Vec<Token>,
    pub current_indentation: u32,
    /// The lexer maintains some internal state about how many opening brackets there are. This is
    /// useful for the parsing of expressions.
    pub current_parenthisis: u32,
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
            current_parenthisis: 0,
        }
    }
    /// Lexes an application of a function.
    ///
    /// The term "application" originally comes from Alonzo Church's lambda calculus which is a way
    /// of "doing" mathematics using only functions (to the best of my knowledge it's first appearance
    /// is in Church's work – it might have appeared earlier and I'm sure Church was inspired and
    /// helped by others).
    fn lex_application(&mut self) -> Result<(), LexError> {
        self.lex_identifier()?;
        self.lex_specific_punctuation(Punctuation::OpenRoundBracket)?;
        loop {
            if self.peek().unwrap() != ')' {
                self.lex_expression()?;

                self.consume_spaces();
                if self.peek().unwrap() != ')' {
                    self.lex_specific_punctuation(Punctuation::Comma)?;
                }
            } else {
                break;
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
        self.consume_spaces();
        if self.lex_specific_operator(Operator::Equals).is_err() {
            self.lex_specific_operator(Operator::Increment)?;
        };
        self.consume_spaces();
        self.lex_expression()?;
        Ok(())
    }
    /// Lexes any valid statement.
    fn lex_statement(&mut self) -> Result<(), LexError> {
        self.consume_newlines();
        self.consume_spaces();
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
                "return" => self.lex_return_statement()?,
                _ => self.lex_assignment_statement()?,
            };
            Ok(())
        } else {
            Err(LexError::UnexpectedEndOfInput)
        }
    }
    fn lex_return_statement(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::Return)?;
        self.consume_spaces();
        self.lex_expression()?;
        Ok(())
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
        self.current_indentation += self.count_indents();
        loop {
            let indents = self.count_indents();
            if indents == self.current_indentation {
                self.consume_spaces();
                self.lex_statement()?;
                self.lex_newline()?;
            } else {
                return if indents == (self.current_indentation - 2)
                    || indents == (self.current_indentation - 4)
                {
                    return Ok(());
                } else {
                    Err(LexError::IndentationError)
                };
            }
        }
    }
    /// Eats any spaces between where the cursor presently is and the next non-space
    fn consume_spaces(&mut self) {
        while let Some(next) = self.peek() {
            if next == '\n' {
                return;
            }
            if next.is_whitespace() {
                self.eat();
            } else {
                break;
            }
        }
    }
    /// Greedily gobbles up whitespace (including new lines) until it reaches the next non-whitespace
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
        self.input.split(|item| item == ' ' || item == '\n').next()
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
            self.location.col = 0;
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
                            for _ in 0..$string.len() {
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
        self.consume_spaces();
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
            ["next" => Next],
            ["return" => Return]
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
    /// Note that this function will panic if it is used to lex a closing bracket if there is no
    /// matching opening bracket (if `current_parenthisis` is none).
    fn lex_specific_punctuation(&mut self, punctuation: Punctuation) -> Result<(), LexError> {
        macro_rules! punctuation {
            ($self:ident, $punctuation:ident, $(($string:expr => $punct:ident)),+) => {
                match $punctuation {
                    $(
                        $crate::lexer::Punctuation::$punct => {
                            if $self.input.starts_with($string) {
                                for _ in 0..$string.len() {
                                    self.eat();
                                }
                                $self.output.push(
                                    $crate::lexer::Token::Punctuation(
                                        $crate::lexer::Punctuation::$punct
                                    )
                                );
                                if $crate::lexer::Punctuation::$punct == $crate::lexer::Punctuation::OpenRoundBracket {
                                    $self.current_parenthisis += 1;
                                } else if $crate::lexer::Punctuation::$punct == $crate::lexer::Punctuation::CloseRoundBracket {
                                    $self.current_parenthisis -= 1;
                                }
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
        punctuation!(self, punctuation,
            ("(" => OpenRoundBracket),
            (")" => CloseRoundBracket),
            (":byRef" => ByRef),
            (":byVal" => ByVal),
            (":" => Colon),
            ("," => Comma),
            ("\"" => Quote)
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
        loop {
            if self.peek().unwrap() == ')' {
                break;
            }
            self.lex_identifier()?;
            self.consume_spaces();
            if self.peek().unwrap() != ',' {
                break;
            }
            self.lex_specific_punctuation(Punctuation::Comma)?;
            self.consume_spaces();
        }
        self.lex_specific_punctuation(Punctuation::CloseRoundBracket)?;
        Ok(())
    }
    /// Lexes a function definition.
    fn lex_function(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::Function)?;
        self.consume_spaces();
        self.lex_identifier()?;
        self.consume_spaces();
        self.lex_function_arguments()?;
        self.consume_spaces();
        self.lex_newline()?;
        self.consume_newlines();
        self.lex_block()?;
        self.consume_spaces();
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
    fn lex_string(&mut self) -> Result<(), LexError> {
        self.lex_specific_punctuation(Punctuation::Quote)?;
        let mut output = String::new();
        while self.peek().unwrap() != '"' {
            output.push(self.peek().unwrap());
            self.eat();
        }
        self.output.push(Token::String(output));
        self.lex_specific_punctuation(Punctuation::Quote)?;
        Ok(())
    }
    /// Lexes an expression
    fn lex_expression(&mut self) -> Result<(), LexError> {
        let starting_brackets = *&self.current_parenthisis;
        self.consume_spaces();
        while let Some(item) = self.peek() {
            if item == '\n' {
                return Ok(());
            }
            self.consume_spaces();
            if item == '"' {
                self.lex_string()?;
            } else if item.is_alphabetic() || item == '(' || item == ')' {
                match item {
                    '(' => {
                        self.lex_specific_punctuation(Punctuation::OpenRoundBracket)?;
                    }
                    ')' => {
                        if self.current_parenthisis == starting_brackets {
                            return Ok(());
                        }
                        if self.current_parenthisis < 1 {
                            panic!(
                                "unmatched brackets; this is going to be made into a proper error"
                            )
                        }
                    }
                    _ => {}
                }
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
                self.consume_spaces();
                if self.lex_any_punctuation().is_err() {
                    if self.lex_any_operator().is_err() {
                        return Ok(());
                    };
                }
                self.consume_spaces();
            }
        }
        Ok(())
    }
    /// Lexes an if statement.
    fn lex_if_statement(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::If)?;
        self.consume_newlines();
        self.lex_expression()?;
        self.consume_spaces();
        self.lex_specific_keyword(Keyword::Then)?;
        self.lex_newline()?;
        self.lex_block()?;
        if self.lex_specific_keyword(Keyword::ElseIf).is_ok() {
            self.lex_expression()?;
            self.lex_specific_keyword(Keyword::Then)?;
            self.lex_block()?;
        }
        if self.lex_specific_keyword(Keyword::Else).is_ok() {
            self.lex_block()?;
        }
        self.consume_spaces();
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
            self.consume_spaces();
            if self.lex_specific_keyword(Keyword::Case).is_err() {
                break;
            }
            self.lex_expression()?;
            self.set_indentation_level(2);
            self.lex_block()?;
        }

        self.lex_specific_keyword(Keyword::Default)?;
        self.consume_spaces();
        self.lex_specific_punctuation(Punctuation::Colon)?;
        self.set_indentation_level(2);
        self.lex_block()?;

        self.lex_specific_keyword(Keyword::EndSwitch)?;

        Ok(())
    }
    /// Lexes a while statement
    fn lex_while_statement(&mut self) -> Result<(), LexError> {
        self.lex_specific_keyword(Keyword::While)?;
        self.consume_spaces();
        self.lex_expression()?;
        self.consume_spaces();
        self.lex_newline()?;
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
                    for _ in 0..next.len() {
                        self.eat();
                    }
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
    /// Lexes any item of punctuation.
    fn lex_any_punctuation(&mut self) -> Result<(), LexError> {
        macro_rules! punctuation {
            ($self:ident, $(($string:expr => $punct:ident)),+) => {
                $(
                     if self.input.starts_with($string) {
                        self.output.push(
                            $crate::lexer::Token::Punctuation($crate::lexer::Punctuation::$punct)
                        );
                        for _ in 0..$string.len() {
                           $self.eat();
                        }
                        return Ok(());
                     }
                )*
                else {
                    return Err($crate::lexer::LexError::UnexpectedEndOfInput)
                }
            }
        }
        punctuation!(self,
            ("(" => OpenRoundBracket),
            (")" => CloseRoundBracket),
            (":byRef" => ByRef),
            (":byVal" => ByVal),
            (":" => Colon),
            ("," => Comma),
            ("\"" => Quote)
        );
    }
    /// Lexes any operator
    fn lex_any_operator(&mut self) -> Result<(), LexError> {
        // todo: move this out and consolidate all of these definitions into one
        macro_rules! operators {
            ($self:ident, $(($string:expr => $op:ident)),+) => {
                $(
                    if self.input.starts_with($string) {
                        self.output.push(
                            $crate::lexer::Token::Operator($crate::lexer::Operator::$op)
                        );
                        for _ in 0..$string.len() {
                            $self.eat();
                        }
                        return Ok(());
                    }
                )+
                else {
                    return Err($crate::lexer::LexError::UnexpectedEndOfInput); 
                }
            }
        }
        operators!(self,
            ("==" => Comparison),
            ("!=" => NotEquals),
            ("=" => Equals),
            ("*" => Times),
            ("/" => Divide),
            ("+=" => Increment),
            ("+" => Plus),
            ("-" => Minus),
            ("AND" => And),
            ("OR" => Or),
            ("NOT" => Not)
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
                                for _ in 0..$string.len() {
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
                ("=" => Equals),
                ("==" => Comparison),
                ("!=" => NotEquals),
                ("*" => Times),
                ("+=" => Increment),
                ("+" => Plus),
                ("-" => Minus),
                ("/" => Divide),
                ("AND" => And),
                ("OR" => Or),
                ("NOT" => Not)
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
        self.consume_spaces();
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
