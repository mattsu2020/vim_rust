use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

#[derive(Default)]
struct BuildNode {
    children: BTreeMap<u8, BuildNode>,
    is_word: bool,
}

#[derive(Default, Clone)]
pub struct SpellFile {
    pub byts: Vec<u8>,
    pub idxs: Vec<u32>,
}

fn insert(root: &mut BuildNode, word: &[u8]) {
    let mut node = root;
    for &b in word {
        node = node.children.entry(b).or_default();
    }
    node.is_word = true;
}

fn flatten(node: &BuildNode, byts: &mut Vec<u8>, idxs: &mut Vec<u32>) -> u32 {
    let node_idx = byts.len() as u32;
    byts.push(0); // placeholder for len
    idxs.push(0); // keep arrays aligned

    let mut entries: Vec<(u8, Option<&BuildNode>)> = Vec::new();
    if node.is_word {
        entries.push((0, None));
    }
    for (ch, child) in &node.children {
        entries.push((*ch, Some(child)));
    }

    byts[node_idx as usize] = entries.len() as u8;

    let mut positions: Vec<(Option<&BuildNode>, usize)> = Vec::new();
    for (ch, child_opt) in entries {
        byts.push(ch);
        idxs.push(0);
        let pos = byts.len() - 1;
        positions.push((child_opt, pos));
    }

    for (child_opt, pos) in positions {
        if let Some(child) = child_opt {
            let child_index = flatten(child, byts, idxs);
            idxs[pos] = child_index;
        } else {
            idxs[pos] = 1; // terminal flag
        }
    }

    node_idx
}

pub fn build_from_words(words: &[&str]) -> SpellFile {
    let mut root = BuildNode::default();
    for w in words {
        insert(&mut root, w.as_bytes());
    }
    let mut byts = Vec::new();
    let mut idxs = Vec::new();
    flatten(&root, &mut byts, &mut idxs);
    SpellFile { byts, idxs }
}

pub fn write_spellfile<P: AsRef<Path>>(path: P, dict: &SpellFile) -> io::Result<()> {
    let mut f = File::create(path)?;
    f.write_all(b"RSPF")?;
    let byts_len = dict.byts.len() as u32;
    let idxs_len = dict.idxs.len() as u32;
    f.write_all(&byts_len.to_le_bytes())?;
    f.write_all(&idxs_len.to_le_bytes())?;
    f.write_all(&dict.byts)?;
    for i in &dict.idxs {
        f.write_all(&i.to_le_bytes())?;
    }
    Ok(())
}

pub fn read_spellfile<P: AsRef<Path>>(path: P) -> io::Result<SpellFile> {
    let mut f = File::open(path)?;
    let mut magic = [0u8;4];
    f.read_exact(&mut magic)?;
    if &magic != b"RSPF" {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "bad magic"));
    }
    let mut buf4 = [0u8;4];
    f.read_exact(&mut buf4)?;
    let byts_len = u32::from_le_bytes(buf4) as usize;
    f.read_exact(&mut buf4)?;
    let idxs_len = u32::from_le_bytes(buf4) as usize;
    let mut byts = vec![0u8; byts_len];
    f.read_exact(&mut byts)?;
    let mut idxs = vec![0u32; idxs_len];
    for i in 0..idxs_len {
        f.read_exact(&mut buf4)?;
        idxs[i] = u32::from_le_bytes(buf4);
    }
    Ok(SpellFile { byts, idxs })
}
