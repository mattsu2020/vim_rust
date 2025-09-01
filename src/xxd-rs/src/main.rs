use clap::Parser;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;

/// Simple hexdump utility similar to Vim's xxd.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Reverse operation: convert hexdump into binary
    #[arg(short = 'r', long)]
    reverse: bool,

    /// Plain hexdump style (no offsets)
    #[arg(short = 'p', long)]
    plain: bool,

    /// Input file, read from STDIN if omitted
    input: Option<PathBuf>,

    /// Output file for reverse, write to STDOUT if omitted
    output: Option<PathBuf>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut input: Box<dyn Read> = match &args.input {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(io::stdin()),
    };

    let mut output: Box<dyn Write> = match &args.output {
        Some(path) => Box::new(File::create(path)?),
        None => Box::new(io::stdout()),
    };

    if args.reverse {
        reverse(&mut input, &mut output, args.plain)?;
    } else {
        hexdump(&mut input, &mut output, args.plain)?;
    }
    Ok(())
}

fn hexdump<R: Read, W: Write>(input: &mut R, output: &mut W, plain: bool) -> io::Result<()> {
    let mut offset = 0usize;
    let mut buf = [0u8; 16];
    loop {
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        if plain {
            for b in &buf[..n] {
                write!(output, "{:02x}", b)?;
            }
            writeln!(output)?;
        } else {
            write!(output, "{:08x}: ", offset)?;
            for i in 0..16 {
                if i < n {
                    write!(output, "{:02x}", buf[i])?;
                } else {
                    write!(output, "  ")?;
                }
                if i % 2 == 1 {
                    output.write_all(b" ")?;
                }
            }
            output.write_all(b" ")?;
            for &b in &buf[..n] {
                let ch = if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' };
                output.write_all(&[ch as u8])?;
            }
            writeln!(output)?;
        }
        offset += n;
    }
    Ok(())
}

fn reverse<R: Read, W: Write>(input: &mut R, output: &mut W, plain: bool) -> io::Result<()> {
    if plain {
        let mut data = Vec::new();
        input.read_to_end(&mut data)?;
        let mut buf = Vec::new();
        let mut hi = None;
        for b in data.into_iter() {
            let v = match b {
                b'0'..=b'9' => Some(b - b'0'),
                b'a'..=b'f' => Some(b - b'a' + 10),
                b'A'..=b'F' => Some(b - b'A' + 10),
                _ => None,
            };
            if let Some(v) = v {
                if let Some(h) = hi {
                    buf.push(h << 4 | v);
                    hi = None;
                } else {
                    hi = Some(v);
                }
            }
        }
        output.write_all(&buf)?;
    } else {
        let mut text = String::new();
        input.read_to_string(&mut text)?;
        let mut buf = Vec::new();
        for line in text.lines() {
            if let Some(colon) = line.find(':') {
                let mut hex_part = &line[colon + 1..];
                hex_part = hex_part.trim_start();
                if let Some(idx) = hex_part.find("  ") {
                    hex_part = &hex_part[..idx];
                }
                for chunk in hex_part.split_whitespace() {
                    let bytes = chunk.as_bytes();
                    let mut i = 0;
                    while i + 1 < bytes.len() {
                        let byte = u8::from_str_radix(&chunk[i..i + 2], 16).unwrap_or(0);
                        buf.push(byte);
                        i += 2;
                    }
                }
            }
        }
        output.write_all(&buf)?;
    }
    Ok(())
}
