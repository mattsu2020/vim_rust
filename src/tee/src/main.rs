use clap::Parser;
use std::fs::OpenOptions;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;

/// Read from standard input and write to standard output and files
#[derive(Parser, Debug)]
#[command(author, version, about = "Rust implementation of tee")]
struct Args {
    /// Append to the given files, do not overwrite
    #[arg(short = 'a', long = "append")]
    append: bool,

    /// Files to write to
    #[arg(value_name = "FILE", required = true)]
    files: Vec<PathBuf>,
}

fn run(args: Args) -> io::Result<()> {
    let stdin = io::stdin();
    let mut reader = BufReader::new(stdin.lock());

    let stdout = io::stdout();
    let mut out_writer = BufWriter::new(stdout.lock());

    let mut file_writers = Vec::new();
    for path in &args.files {
        let mut opts = OpenOptions::new();
        opts.write(true).create(true);
        if args.append {
            opts.append(true);
        } else {
            opts.truncate(true);
        }
        let file = opts.open(path)?;
        file_writers.push(BufWriter::new(file));
    }

    let mut buffer = [0u8; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        out_writer.write_all(&buffer[..n])?;
        out_writer.flush()?;
        for writer in &mut file_writers {
            writer.write_all(&buffer[..n])?;
            writer.flush()?;
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    run(args)
}
