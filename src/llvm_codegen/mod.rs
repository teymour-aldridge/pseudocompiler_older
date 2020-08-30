//! Generates LLVM IR from the AST. This can then be fed into LLVM to produce an executable binary.

/// Outputs LLVM IR from the AST.
trait LLVMCodegen {
    /// Output the LLVM IR for this AST node.
    fn output(&self) -> String;
}
