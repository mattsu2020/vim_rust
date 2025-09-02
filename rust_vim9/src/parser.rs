#[derive(Debug, Clone, PartialEq)]
pub enum Ast {
    Number(i64),
    Add(Box<Ast>, Box<Ast>),
    Echo(Box<Ast>),
}

pub fn parse_addition(expr: &str) -> Option<Ast> {
    let mut parts = expr.split('+');
    let first = parts.next()?.trim().parse().ok()?;
    let mut ast = Ast::Number(first);
    for p in parts {
        let n = p.trim().parse().ok()?;
        let right = Ast::Number(n);
        ast = Ast::Add(Box::new(ast), Box::new(right));
    }
    Some(ast)
}

pub fn parse_line(line: &str) -> Option<Ast> {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("echo ") {
        parse_addition(rest).map(|ast| Ast::Echo(Box::new(ast)))
    } else {
        parse_addition(line)
    }
}
