use clap::{Arg, Command};
use std::fs::File;
use std::io::{self, BufRead, Read, Write};
use std::path::Path;

fn main() -> io::Result<()> {
    let matches = Command::new("xxd")
        .arg(Arg::new("reverse").short('r').long("reverse"))
        .arg(Arg::new("plain").short('p').long("plain"))
        .arg(Arg::new("uppercase").short('u').long("uppercase"))
        .arg(
            Arg::new("cols")
                .short('c')
                .value_name("cols")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("len")
                .short('l')
                .value_name("len")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("seek")
                .short('s')
                .value_name("seek")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("offset")
                .short('o')
                .value_name("offset")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(
            Arg::new("group")
                .short('g')
                .value_name("bytes")
                .value_parser(clap::value_parser!(usize)),
        )
        .arg(Arg::new("infile").index(1))
        .arg(Arg::new("outfile").index(2))
        .disable_help_flag(false)
        .get_matches();

    let reverse = matches.get_flag("reverse");
    let plain = matches.get_flag("plain");
    let uppercase = matches.get_flag("uppercase");
    let cols = *matches.get_one::<usize>("cols").unwrap_or(&16);
    let len_opt = matches.get_one::<usize>("len").copied();
    let seek = *matches.get_one::<usize>("seek").unwrap_or(&0);
    let base = *matches.get_one::<usize>("offset").unwrap_or(&0);
    let group = *matches.get_one::<usize>("group").unwrap_or(&2);
    let infile = matches
        .get_one::<String>("infile")
        .map(String::as_str)
        .unwrap_or("-");
    let outfile = matches
        .get_one::<String>("outfile")
        .map(String::as_str)
        .unwrap_or("-");

    if reverse {
        reverse_hex(infile, outfile, plain)
    } else {
        hexdump(
            infile, outfile, plain, cols, len_opt, seek, base, uppercase, group,
        )
    }
}

fn open_input(path: &str) -> io::Result<Box<dyn Read>> {
    if path == "-" {
        Ok(Box::new(io::stdin()))
    } else {
        Ok(Box::new(File::open(Path::new(path))?))
    }
}

fn open_output(path: &str) -> io::Result<Box<dyn Write>> {
    if path == "-" {
        Ok(Box::new(io::stdout()))
    } else {
        Ok(Box::new(File::create(Path::new(path))?))
    }
}

fn hexdump(
    infile: &str,
    outfile: &str,
    plain: bool,
    cols: usize,
    len_opt: Option<usize>,
    seek: usize,
    base: usize,
    uppercase: bool,
    group: usize,
) -> io::Result<()> {
    let mut reader = io::BufReader::new(open_input(infile)?);
    let mut writer = open_output(outfile)?;

    // Buffer reused for reading input. Its size matches the number of columns
    // displayed, ensuring one line of output per read.
    let mut buffer = vec![0u8; cols];

    let mut remaining_seek = seek;
    let mut total_read = 0usize;
    let mut offset = base;

    loop {
        let read_len = if remaining_seek > 0 {
            cols
        } else {
            match len_opt {
                Some(len) => {
                    if total_read >= len {
                        break;
                    }
                    (len - total_read).min(cols)
                }
                None => cols,
            }
        };

        let n = reader.read(&mut buffer[..read_len])?;
        if n == 0 {
            break;
        }
        let mut start = 0;
        if remaining_seek > 0 {
            if remaining_seek >= n {
                remaining_seek -= n;
                continue;
            } else {
                start = remaining_seek;
                remaining_seek = 0;
            }
        }

        let mut chunk = &buffer[start..n];
        if let Some(len) = len_opt {
            let remaining = len - total_read;
            if chunk.len() > remaining {
                chunk = &chunk[..remaining];
            }
        }
        total_read += chunk.len();

        if plain {
            for byte in chunk {
                if uppercase {
                    write!(writer, "{:02X}", byte)?;
                } else {
                    write!(writer, "{:02x}", byte)?;
                }
            }
            writeln!(writer)?;
            continue;
        }

        write!(writer, "{:08x}: ", offset)?;
        for j in 0..cols {
            if j < chunk.len() {
                if uppercase {
                    write!(writer, "{:02X}", chunk[j])?;
                } else {
                    write!(writer, "{:02x}", chunk[j])?;
                }
            } else {
                write!(writer, "  ")?;
            }
            if (j + 1) % group == 0 {
                write!(writer, " ")?;
            }
        }
        write!(writer, "|")?;
        for &b in chunk {
            let c = if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            };
            write!(writer, "{}", c)?;
        }
        writeln!(writer, "|")?;
        offset += cols;
    }
    Ok(())
}

fn reverse_hex(infile: &str, outfile: &str, plain: bool) -> io::Result<()> {
    let mut reader = open_input(infile)?;
    let mut writer = open_output(outfile)?;
    if plain {
        let mut contents = String::new();
        reader.read_to_string(&mut contents)?;
        let mut chars = contents.chars().filter(|c| !c.is_whitespace());
        while let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
            let byte = u8::from_str_radix(&format!("{}{}", h1, h2), 16).unwrap_or(0);
            writer.write_all(&[byte])?;
        }
    } else {
        let buf_reader = io::BufReader::new(reader);
        for line in buf_reader.lines() {
            let line = line?;
            let mut part = line.as_str();
            if let Some(pos) = part.find(':') {
                part = &part[pos + 1..];
            }
            if let Some(pos) = part.find('|') {
                part = &part[..pos];
            }
            let mut chars = part.chars().filter(|c| c.is_ascii_hexdigit());
            while let (Some(h1), Some(h2)) = (chars.next(), chars.next()) {
                let byte = u8::from_str_radix(&format!("{}{}", h1, h2), 16).unwrap_or(0);
                writer.write_all(&[byte])?;
            }
        }
    }
    Ok(())
}
