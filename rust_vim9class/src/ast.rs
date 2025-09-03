pub struct Class {
    pub name: String,
}

/// Parse a very small subset of a Vim9 class declaration.
/// Currently this only extracts the class name from strings like
/// "class Name".  Unknown input results in a class named `Anon`.
pub fn parse(src: &str) -> Class {
    let name = src
        .split_whitespace()
        .nth(1)
        .unwrap_or("Anon")
        .to_string();
    Class { name }
}
