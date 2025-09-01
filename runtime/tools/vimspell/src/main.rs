use std::collections::{BTreeSet, HashSet};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;
use hunspell::Hunspell;
use regex::escape;
use tempfile::NamedTempFile;

/// Generate Vim syntax highlighting for misspelled words
#[derive(Parser)]
struct Args {
    /// File to check
    input: PathBuf,
    /// Additional dictionary file
    #[arg(short, long)]
    dict: Option<PathBuf>,
}

fn load_local_dict(path: &PathBuf) -> Result<HashSet<String>> {
    let content =
        fs::read_to_string(path).with_context(|| format!("reading local dictionary {path:?}"))?;
    let mut set = HashSet::new();
    for line in content.lines() {
        let word = line.trim();
        if !word.is_empty() {
            set.insert(word.to_string());
        }
    }
    Ok(set)
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Use the default US English dictionary
    let spell = Hunspell::new(
        "/usr/share/hunspell/en_US.aff",
        "/usr/share/hunspell/en_US.dic",
    );

    // Load optional local dictionary
    let local_words = if let Some(path) = args
        .dict
        .or_else(|| std::env::var("LOCAL_DICT").ok().map(PathBuf::from))
    {
        if path.exists() {
            load_local_dict(&path)?
        } else {
            HashSet::new()
        }
    } else {
        HashSet::new()
    };

    let content = fs::read_to_string(&args.input)
        .with_context(|| format!("reading input file {:?}", args.input))?;
    let mut bad = BTreeSet::new();
    for token in content.split(|c: char| !c.is_alphabetic() && c != '\'') {
        if token.is_empty() {
            continue;
        }
        if local_words.contains(token) {
            continue;
        }
        if !spell.check(token) {
            bad.insert(token.to_string());
        }
    }

    let mut tmp = NamedTempFile::new()?;
    let path = tmp.path().to_path_buf();
    for word in bad {
        writeln!(tmp, "syntax match SpellErrors \"\\<{}\\>\"", escape(&word))?;
    }
    writeln!(tmp, "highlight link SpellErrors ErrorMsg\n")?;
    writeln!(tmp, "!rm {}", path.display())?;
    tmp.flush()?;
    tmp.into_temp_path().keep()?; // keep the file after program exits
    println!("{}", path.display());
    Ok(())
}
