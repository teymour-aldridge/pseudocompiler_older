//! Various integration tests for the lexer.
//!
//! Tests are important! Please feel free to add more tests.
//!
//! If you submit a bug fix, please add a test which will fail if the bug is present.

use crate::lexer::lex;

fn assert_parses_ok(string: &str) {
    let mut string = string.to_string();
    match lex(string.as_mut_str()) {
        Ok(_) => {}
        Err(e) => {
            panic!("{:?}", e);
        }
    };
}

fn assert_parses_err(string: &str) {
    let mut string = string.to_string();
    assert!(lex(string.as_mut_str()).is_err());
}

#[test]
pub fn test_lexes_functions() {
    assert_parses_ok(
        &r#"
        function f(x, y, z)
            return x * y * z
        endfunction
    "#,
    );
}

// Procedures are not currently supported.
/*
#[test]
pub fn test_lexes_procedure_byref() {
    assert_parses_ok(
        &r#"
        procedure someFunction12(arg1:byVal, arg2:byRef)
            arg2 += 1
        endprocedure
    "#,
    );
}
*/

#[test]
pub fn test_lexes_while_statement() {
    assert_parses_ok(
        &r#"
        x = 12
        while x!=13
            x += 1
        endwhile
    "#,
    );
}

#[test]
pub fn test_lexes_assignment() {
    assert_parses_ok(
        &r#"
        fourtyTwo = 42
        fiftyFive = 12 + 8 * 3
        string = "string"
    "#,
    );
}

#[test]
pub fn test_lexes_assignment_inside_function() {
    assert_parses_ok(
        r#"
        function f(x, y, z, r3)
            a = "string" + "string2"
        endfunction
    "#,
    );
}

#[test]
pub fn test_lexes_simple_if_statement() {
    assert_parses_ok(
        r#"
        if x=="a" then
            print("hello world!")
        endif
    "#,
    );
}

#[test]
pub fn test_lexes_if_statement_with_complex_expression() {
    assert_parses_ok(
        r#"
        if x=="a" AND y=="b" AND z == 12 then
            print("hello world!")
        endif
    "#,
    );
}

#[test]
pub fn test_lexes_if_else() {}

#[test]
pub fn test_lexes_if_elif_else() {}

#[test]
pub fn test_lexes_superfluous_spaces() {}

#[test]
pub fn test_rejects_invalid_if() {
    assert_parses_err(
        &r#"
        if then
            print("hello")
        endif
    "#,
    );
}

#[test]
pub fn test_rejects_invalid_for() {}

#[test]
pub fn test_lexes_indentation() {}
