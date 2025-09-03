use std::env;
use std::fs::File;
use std::io::{self, Read, Write};

fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut input: Box<dyn Read> = if args.len() > 1 && args[1] != "-" {
        Box::new(File::open(&args[1])?)
    } else {
        Box::new(io::stdin())
    };
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;
    io::stdout().write_all(&buffer)?;
    Ok(())
}

fn main() {
    if let Err(err) = run() {
        eprintln!("error: {}", err);
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn copy_data() {
        let data = b"hello";
        let mut input = &data[..];
        let mut output = Vec::new();
        std::io::copy(&mut input, &mut output).unwrap();
        assert_eq!(output, data);
    }
}
