//! Checks that the types are correct

use std::collections::HashMap;

/// Maintains a "jar" containing all the bound variables and their types.
pub struct BindingJar {
    bindings: HashMap<String, Type>,
}

#[derive(PartialEq)]
pub struct Path {
    /// The parts of the path.
    parts: Vec<String>,
}

/// A type.
///
/// Types are inferred. Entire programs are statically typed.
pub struct Type {
    /// A unique identifier for each type.
    id: i32,
    /// The name of the type
    name: String,
    /// The location in which the type is located
    location: Path,
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.location == other.location
    }
}
