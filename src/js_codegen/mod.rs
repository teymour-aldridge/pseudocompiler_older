//! Outputs Javascript code.
//!
//! This option exists because it's easier to just output Javascript directly, instead of having to
//! compile the entire codebase to WebAssembly (including compiling LLVM to Wasm) to run it
//! interactively in the browser.

/// A trait for outputting Javascript code from AST nodes.
trait JSCodegen {
    /// Outputs Javascript code for the AST node.
    fn output(&self) -> String;
}
