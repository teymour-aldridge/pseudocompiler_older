//! Outputs Javascript code.
//!
//! This option exists because it's easier to just output Javascript directly, instead of having to
//! compile the entire codebase to WebAssembly (including compiling LLVM to Wasm) to run it
//! interactively in the browser.

/// Takes an AST and walks it, producing Javascript code as its output.
pub fn emit_js(input: ()) -> String {
    todo!()
}

/// Emits code for a for statement.
pub fn emit_for(input: ()) -> String {
    todo!()
}

/// Emits code for a while statement.
pub fn emit_while(input: ()) -> String {
    todo!()
}

/// Emits an if statement.
pub fn emit_if(input: ()) -> String {
    todo!()
}

/// Emits an assignment operation.
pub fn emit_assignment(input: ()) -> String {
    todo!()
}

/// Emits Javascript code for an expression.
pub fn emit_expression(input: ()) -> String {
    todo!()
}

/// Emit a Javascript function definition.
pub fn emit_function_def(input: ()) -> String {
    todo!()
}

/// Emits calls to outputted Javascript functions.
pub fn emit_function_call(input: ()) -> String {
    todo!()
}

/// Emits Javascript code from an operator.
pub fn emit_operator(input: ()) -> String {
    todo!()
}
