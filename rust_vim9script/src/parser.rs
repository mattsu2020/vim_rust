use crate::ast::Ast;

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

fn parse_comparison(expr: &str) -> Option<Ast> {
    if let Some((left, right)) = expr.split_once('<') {
        let left_ast = parse_addition(left.trim())?;
        let right_ast = parse_addition(right.trim())?;
        Some(Ast::LessThan(Box::new(left_ast), Box::new(right_ast)))
    } else {
        parse_addition(expr)
    }
}

pub fn parse_line(line: &str) -> Option<Ast> {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("echo ") {
        parse_comparison(rest).map(|ast| Ast::Echo(Box::new(ast)))
    } else {
        parse_comparison(line)
    }
}
