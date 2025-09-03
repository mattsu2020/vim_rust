use crate::ast::Class;

/// Execute the class and return a simple result.  For demonstration purposes
/// the result is the length of the class name.
pub fn execute(class: &Class) -> i32 {
    class.name.len() as i32
}
