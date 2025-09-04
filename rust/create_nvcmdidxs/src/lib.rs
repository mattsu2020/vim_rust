use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

fn nv_cmds() -> io::Result<Vec<i32>> {
    let src_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../src");
    let output = Command::new("cc")
        .current_dir(&src_dir)
        .args([
            "-I", ".",
            "-include", "ascii.h",
            "-include", "keymap.h",
            "-E", "-P", "nv_cmds.h",
        ])
        .output()?;
    let text = String::from_utf8(output.stdout).unwrap();
    let array_pos = text.find("nv_cmds[]").ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no array"))?;
    let mut chars = text[array_pos..].chars().enumerate();
    let mut braces = 0; let mut start_idx = None; let mut end_idx = None; let mut in_char = false;
    while let Some((i, c)) = chars.next() {
        if in_char {
            if c == '\\' { chars.next(); }
            else if c == '\'' { in_char = false; }
            continue;
        }
        match c {
            '\'' => in_char = true,
            '{' => {
                braces += 1;
                if braces == 1 { start_idx = Some(array_pos + i + 1); }
            },
            '}' => {
                if braces == 1 { end_idx = Some(array_pos + i); break; }
                braces -= 1;
            },
            _ => {}
        }
    }
    let start = start_idx.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no start"))?;
    let end = end_idx.ok_or_else(|| io::Error::new(io::ErrorKind::Other, "no end"))?;
    let body = &text[start..end];
    let mut nums = Vec::new();
    let re = regex::Regex::new(r"'(?:\\.|[^'])+'|0x[0-9A-Fa-f]+|-?\d+").unwrap();
    for m in re.find_iter(body) {
        let t = m.as_str();
        let val = if t.starts_with('\'') {
            let ch = &t[1..t.len() - 1];
            if let Some(rest) = ch.strip_prefix('\\') {
                if rest.chars().next().unwrap().is_ascii_digit() {
                    i32::from_str_radix(rest, 8).unwrap()
                } else {
                    rest.chars().next().unwrap() as i32
                }
            } else {
                ch.chars().next().unwrap() as i32
            }
        } else if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
            i32::from_str_radix(hex, 16).unwrap()
        } else {
            t.parse::<i32>().unwrap()
        };
        nums.push(val);
    }
    Ok(nums)
}

pub fn generate<W: Write>(mut out: W) -> io::Result<()> {
    let mut table: Vec<(i32, usize)> = nv_cmds()?
        .into_iter()
        .enumerate()
        .map(|(i, mut ch)| {
            if ch < 0 {
                ch = -ch;
            }
            (ch, i)
        })
        .collect();
    table.sort_by_key(|&(ch, _)| ch);

    let mut nv_max_linear = table.len().saturating_sub(1);
    for (i, &(ch, _)) in table.iter().enumerate() {
        if i != ch as usize {
            nv_max_linear = i.saturating_sub(1);
            break;
        }
    }

    writeln!(out, "/*")?;
    writeln!(out, " * Automatically generated code by the create_nvcmdidxs tool.")?;
    writeln!(out, " *")?;
    writeln!(out, " * Table giving the index in nv_cmds[] to lookup based on")?;
    writeln!(out, " * the command character.")?;
    writeln!(out, " */")?;
    writeln!(out)?;
    writeln!(out, "// nv_cmd_idx[<normal mode command character>] => nv_cmds[] index")?;
    writeln!(out, "static const unsigned short nv_cmd_idx[] =")?;
    writeln!(out, "{{")?;
    for (ch, idx) in &table {
        writeln!(out, "  /* {:>5} */ {:>3},", ch, idx)?;
    }
    writeln!(out, "}};")?;
    writeln!(out)?;
    writeln!(out, "// The highest index for which")?;
    writeln!(out, "// nv_cmds[idx].cmd_char == nv_cmd_idx[nv_cmds[idx].cmd_char]")?;
    writeln!(out, "static const int nv_max_linear = {};", nv_max_linear)?;
    Ok(())
}

pub fn generate_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let mut buf = Vec::new();
    generate(&mut buf)?;
    if let Ok(existing) = fs::read(path.as_ref()) {
        if existing == buf {
            return Ok(());
        }
    }
    fs::write(path, buf)
}
