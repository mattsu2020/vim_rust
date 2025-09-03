use crate::class::{parse_class, Class};
use std::collections::HashSet;

#[derive(Clone)]
enum StateKind {
    Match,
    Char(u8),
    Any,
    Class(Class),
    Split,
}

#[derive(Clone)]
struct State {
    kind: StateKind,
    out: usize,
    out1: usize,
}

pub struct Prog {
    start: usize,
    states: Vec<State>,
    anchor_start: bool,
    anchor_end: bool,
}

#[derive(Clone)]
struct Fragment {
    start: usize,
    outs: Vec<(usize, bool)>, // false -> out, true -> out1
}

fn patch(outs: Vec<(usize, bool)>, target: usize, states: &mut Vec<State>) {
    for (idx, second) in outs {
        if second {
            states[idx].out1 = target;
        } else {
            states[idx].out = target;
        }
    }
}

fn literal(states: &mut Vec<State>, c: u8) -> Fragment {
    let idx = states.len();
    states.push(State { kind: StateKind::Char(c), out: usize::MAX, out1: usize::MAX });
    Fragment { start: idx, outs: vec![(idx, false)] }
}

fn any_state(states: &mut Vec<State>) -> Fragment {
    let idx = states.len();
    states.push(State { kind: StateKind::Any, out: usize::MAX, out1: usize::MAX });
    Fragment { start: idx, outs: vec![(idx, false)] }
}

fn class_state(states: &mut Vec<State>, class: Class) -> Fragment {
    let idx = states.len();
    states.push(State { kind: StateKind::Class(class), out: usize::MAX, out1: usize::MAX });
    Fragment { start: idx, outs: vec![(idx, false)] }
}

fn concat(a: Fragment, b: Fragment, states: &mut Vec<State>) -> Fragment {
    patch(a.outs, b.start, states);
    Fragment { start: a.start, outs: b.outs }
}

fn star(f: Fragment, states: &mut Vec<State>) -> Fragment {
    let idx = states.len();
    states.push(State { kind: StateKind::Split, out: f.start, out1: usize::MAX });
    patch(f.outs, idx, states);
    Fragment { start: idx, outs: vec![(idx, true)] }
}

fn plus(f: Fragment, states: &mut Vec<State>) -> Fragment {
    let idx = states.len();
    states.push(State { kind: StateKind::Split, out: f.start, out1: usize::MAX });
    patch(f.outs, idx, states);
    Fragment { start: f.start, outs: vec![(idx, true)] }
}

fn question(f: Fragment, states: &mut Vec<State>) -> Fragment {
    let idx = states.len();
    states.push(State { kind: StateKind::Split, out: f.start, out1: usize::MAX });
    let mut outs = f.outs;
    outs.push((idx, true));
    Fragment { start: idx, outs }
}

fn parse_atom(chars: &mut &[u8], states: &mut Vec<State>) -> Option<Fragment> {
    if chars.is_empty() {
        return None;
    }
    let c = chars[0];
    *chars = &chars[1..];
    match c {
        b'.' => Some(any_state(states)),
        b'(' => {
            let f = parse_seq(chars, states)?;
            if chars.first() != Some(&b')') {
                return None;
            }
            *chars = &chars[1..];
            Some(f)
        }
        b'[' => {
            let mut buf = vec![b'['];
            buf.extend_from_slice(chars);
            let (class, len) = parse_class(&buf)?;
            *chars = &chars[len - 1..];
            Some(class_state(states, class))
        }
        _ => Some(literal(states, c)),
    }
}

fn parse_piece(chars: &mut &[u8], states: &mut Vec<State>) -> Option<Fragment> {
    let mut f = parse_atom(chars, states)?;
    loop {
        if let Some(&c) = chars.first() {
            match c {
                b'*' => {
                    *chars = &chars[1..];
                    f = star(f, states);
                }
                b'+' => {
                    *chars = &chars[1..];
                    f = plus(f, states);
                }
                b'?' => {
                    *chars = &chars[1..];
                    f = question(f, states);
                }
                _ => break,
            }
        } else {
            break;
        }
    }
    Some(f)
}

fn parse_seq(chars: &mut &[u8], states: &mut Vec<State>) -> Option<Fragment> {
    let mut f = parse_piece(chars, states)?;
    while let Some(next) = {
        let save = *chars;
        let r = parse_piece(chars, states);
        if r.is_none() {
            *chars = save;
        }
        r
    } {
        f = concat(f, next, states);
    }
    Some(f)
}

pub fn compile(pattern: &str) -> Option<Prog> {
    let mut bytes = pattern.as_bytes();
    let mut anchor_start = false;
    let mut anchor_end = false;
    if bytes.first() == Some(&b'^') {
        anchor_start = true;
        bytes = &bytes[1..];
    }
    if bytes.last() == Some(&b'$') {
        anchor_end = true;
        bytes = &bytes[..bytes.len() - 1];
    }
    let mut states = Vec::new();
    let f = parse_seq(&mut bytes, &mut states)?;
    if !bytes.is_empty() {
        return None;
    }
    let match_idx = states.len();
    states.push(State { kind: StateKind::Match, out: usize::MAX, out1: usize::MAX });
    patch(f.outs, match_idx, &mut states);
    Some(Prog { start: f.start, states, anchor_start, anchor_end })
}

fn add_state(idx: usize, states: &[State], list: &mut Vec<usize>, visited: &mut HashSet<usize>) {
    if !visited.insert(idx) {
        return;
    }
    match states[idx].kind {
        StateKind::Split => {
            add_state(states[idx].out, states, list, visited);
            add_state(states[idx].out1, states, list, visited);
        }
        _ => list.push(idx),
    }
}

fn step(clist: &[usize], c: u8, states: &[State], nlist: &mut Vec<usize>, ic: bool) {
    let mut visited = HashSet::new();
    for &s in clist {
        match &states[s].kind {
            StateKind::Char(ch) => {
                let ok = if ic {
                    ch.to_ascii_lowercase() == c.to_ascii_lowercase()
                } else {
                    *ch == c
                };
                if ok {
                    add_state(states[s].out, states, nlist, &mut visited);
                }
            }
            StateKind::Any => {
                add_state(states[s].out, states, nlist, &mut visited);
            }
            StateKind::Class(class) => {
                if class.matches(c, ic) {
                    add_state(states[s].out, states, nlist, &mut visited);
                }
            }
            _ => {}
        }
    }
}

pub fn search(prog: &Prog, text: &[u8], ic: bool) -> Option<(usize, usize)> {
    let starts: Vec<usize> = if prog.anchor_start { vec![0] } else { (0..=text.len()).collect() };
    for &i in &starts {
        let mut clist = Vec::new();
        let mut visited = HashSet::new();
        add_state(prog.start, &prog.states, &mut clist, &mut visited);
        let mut j = i;
        let mut match_pos = None;
        while j < text.len() {
            if clist.iter().any(|&s| matches!(prog.states[s].kind, StateKind::Match)) {
                match_pos = Some(j);
            }
            let mut nlist = Vec::new();
            step(&clist, text[j], &prog.states, &mut nlist, ic);
            clist = nlist;
            j += 1;
        }
        if clist.iter().any(|&s| matches!(prog.states[s].kind, StateKind::Match)) {
            match_pos = Some(j);
        }
        if let Some(end) = match_pos {
            if !prog.anchor_end || end == j {
                return Some((i, end));
            }
        }
    }
    None
}
