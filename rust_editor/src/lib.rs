use std::io::{self, Write, Read};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use regex::RegexBuilder;

enum BufferKind {
    Help,
    Buffers,
    Windows,
}

struct Buffer {
    lines: Vec<String>,
    filename: Option<PathBuf>,
    modified: bool,
    kind: Option<BufferKind>,
}

#[derive(Clone)]
struct View {
    buf: usize,
    cx: usize,
    cy: usize,
    scroll: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Layout { Horizontal, Vertical }

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { Normal, Insert, VisualChar, VisualLine }

#[derive(Clone)]
enum Register {
    Charwise(String),
    Linewise(Vec<String>),
}

#[derive(Clone)]
struct UndoSnap {
    lines: Vec<String>,
    cx: usize,
    cy: usize,
}

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
}

fn hide_cursor() {
    print!("\x1b[?25l");
}

fn show_cursor() {
    print!("\x1b[?25h");
}

fn draw_status_line(filename: &Option<PathBuf>, cx: usize, cy: usize, modified: bool, mode: Mode, status: &Option<String>, width: usize) {
    let name = filename.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "[No Name]".to_string());
    let mod_mark = if modified { " [+]" } else { "" };
    let right = status.as_ref().map(|s| s.as_str()).unwrap_or("");
    let mode_tag = match mode { Mode::Normal => "[N]", Mode::Insert => "[I]", Mode::VisualChar => "[V]", Mode::VisualLine => "[VL]" };
    let base = format!(" {} {} - {}:{}{} ", mode_tag, name, cy + 1, cx + 1, mod_mark);
    let mut line = base;
    if !right.is_empty() {
        let pad = if width > line.len() + right.len() + 3 { width - line.len() - right.len() - 3 } else { 1 };
        line.push_str(&" ".repeat(pad));
        line.push_str(right);
        line.push(' ');
    }
    // Inverse colors
    print!("\x1b[7m{:width$}\x1b[0m\r\n", line, width = width);
}

fn draw_rows(lines: &Vec<String>, scroll: usize, rows: usize, width: usize) {
    for i in 0..rows {
        let li = scroll + i;
        if li < lines.len() {
            let line = &lines[li];
            let mut s = line.clone();
            if s.len() > width { s.truncate(width); }
            print!("{s}\r\n");
        } else {
            print!("~\r\n");
        }
    }
}

fn flush() {
    let _ = io::stdout().flush();
}

fn set_raw_mode() -> Option<String> {
    // 保存: stty -g の出力を覚え、復帰時に適用
    let saved = Command::new("stty").arg("-g").output().ok()?.stdout;
    let saved = String::from_utf8_lossy(&saved).trim().to_string();
    let _ = Command::new("stty").args(["raw", "-echo"]).status();
    Some(saved)
}

fn restore_mode(saved: &Option<String>) {
    if let Some(s) = saved {
        let _ = Command::new("stty").arg(s).status();
    } else {
        let _ = Command::new("stty").arg("sane").status();
    }
}

fn open_file(path: &Path) -> Vec<String> {
    match fs::read_to_string(path) {
        Ok(content) => content.replace('\r', "").split('\n').map(|s| s.to_string()).collect(),
        Err(_) => Vec::new(),
    }
}

fn save_file(path: &Path, lines: &Vec<String>) -> io::Result<()> {
    let mut s = String::new();
    for (i, line) in lines.iter().enumerate() {
        s.push_str(line);
        if i + 1 < lines.len() { s.push('\n'); }
    }
    fs::write(path, s)
}

fn get_term_size() -> (usize, usize) {
    // stty size -> rows cols
    if let Ok(out) = Command::new("stty").arg("size").output() {
        if out.status.success() {
            if let Ok(text) = String::from_utf8(out.stdout) {
                let mut it = text.split_whitespace();
                if let (Some(rs), Some(cs)) = (it.next(), it.next()) {
                    if let (Ok(r), Ok(c)) = (rs.parse::<usize>(), cs.parse::<usize>()) {
                        if r >= 2 && c >= 2 { return (c, r); }
                    }
                }
            }
        }
    }
    (80, 24)
}

fn trim_trailing_whitespace(lines: &mut Vec<String>) -> usize {
    let mut count = 0;
    for line in lines.iter_mut() {
        let trimmed = line.trim_end_matches(|c: char| c == ' ' || c == '\t');
        if trimmed.len() != line.len() {
            *line = trimmed.to_string();
            count += 1;
        }
    }
    count
}

fn retab_lines(lines: &mut Vec<String>, tabstop: usize) -> usize {
    let mut changed = 0;
    for line in lines.iter_mut() {
        if line.contains('\t') {
            let mut col = 0usize;
            let mut out = String::with_capacity(line.len());
            for ch in line.chars() {
                if ch == '\t' {
                    let spaces = tabstop - (col % tabstop);
                    out.push_str(&" ".repeat(spaces));
                    col += spaces;
                } else {
                    out.push(ch);
                    col += 1;
                }
            }
            if *line != out { *line = out; changed += 1; }
        }
    }
    changed
}

fn parse_line_ref(tok: &str, max: usize, current: usize) -> Option<usize> {
    let t = tok.trim();
    if t.is_empty() { return None; }
    let (mut base, mut rest) = if t.starts_with('.') { (current + 1, &t[1..]) }
        else if t.starts_with('$') { (max.max(1), &t[1..]) }
        else if let Some(d) = t.chars().next().filter(|c| c.is_ascii_digit()) { let _ = d; (t.parse::<usize>().ok()?, "") }
        else { return None };
    // offsets like +N or -N possibly chained
    let mut i = 0usize;
    while i < rest.len() {
        let bytes = rest.as_bytes();
        let sign = bytes[i] as char;
        if sign != '+' && sign != '-' { break; }
        i += 1;
        let mut j = i;
        while j < rest.len() && rest.as_bytes()[j].is_ascii_digit() { j += 1; }
        if i == j { break; }
        let n: usize = rest[i..j].parse().ok()?;
        if sign == '+' { base = base.saturating_add(n); } else { base = base.saturating_sub(n); }
        i = j;
    }
    Some(base.clamp(1, max.max(1)))
}

fn parse_range(arg: &str, max: usize, current: usize) -> Option<(usize, usize)> {
    // 支持: "%" 全体, "." 現在行, "$" 末行, 数値, 相対 ".+N", "$-N"
    let s = arg.trim();
    if s.is_empty() { return None; }
    if s == "%" { return Some((1, max.max(1))); }
    if let Some((a, b)) = s.split_once(',') {
        let start = parse_line_ref(a, max, current)?;
        let end = parse_line_ref(b, max, current)?;
        Some((start.min(end), start.max(end)))
    } else {
        let n = parse_line_ref(s, max, current)?;
        Some((n, n))
    }
}

fn delete_range(lines: &mut Vec<String>, range: (usize, usize)) -> usize {
    if lines.is_empty() { return 0; }
    let max = lines.len();
    let start = range.0.saturating_sub(1).min(max - 1);
    let end = range.1.saturating_sub(1).min(max - 1);
    let cnt = end + 1 - start;
    for _ in 0..cnt { lines.remove(start); }
    if lines.is_empty() { lines.push(String::new()); }
    cnt
}

fn join_current_with_next(lines: &mut Vec<String>, cy: usize) -> bool {
    if cy + 1 >= lines.len() { return false; }
    let next = lines.remove(cy + 1);
    lines[cy].push_str(&next);
    true
}

fn sort_lines_range(lines: &mut Vec<String>, range: (usize, usize)) {
    let max = lines.len();
    if max == 0 { return; }
    let start = range.0.saturating_sub(1).min(max - 1);
    let end = range.1.saturating_sub(1).min(max - 1);
    if start >= end { return; }
    let mut slice: Vec<String> = lines[start..=end].to_vec();
    slice.sort();
    for (i, s) in slice.into_iter().enumerate() {
        lines[start + i] = s;
    }
}

fn fmt_brace_style(lines: &mut Vec<String>, indent_width: usize) -> usize {
    // 非常に単純な { } ベースの整形
    let mut level = 0usize;
    let mut changed = 0;
    for line in lines.iter_mut() {
        let mut t = line.trim().to_string();
        // 行頭 '}' の場合は先にデントを下げる
        let mut pre_decrement = false;
        if t.starts_with('}') { pre_decrement = true; }

        let current_level = if pre_decrement { level.saturating_sub(1) } else { level };
        let new_line = format!("{}{}", " ".repeat(current_level * indent_width), t);
        if *line != new_line { *line = new_line; changed += 1; }

        // 次のレベル計算
        // '{' の個数分インクリメント、 '}' の個数分デクリメント（ただし負にしない）
        let open = t.chars().filter(|&c| c == '{').count();
        let close = t.chars().filter(|&c| c == '}').count();
        if pre_decrement { level = level.saturating_sub(1); }
        level = level.saturating_add(open);
        level = level.saturating_sub(close);
    }
    changed
}

fn convert_repl_to_rust(repl: &str, last_repl: &str) -> String {
    // Convert Vim-style: \1 -> $1, & -> $0, ~ -> last replacement.
    let mut out = String::with_capacity(repl.len());
    let mut chars = repl.chars().peekable();
    let mut escape = false;
    while let Some(ch) = chars.next() {
        if escape {
            out.push(ch);
            escape = false;
            continue;
        }
        if ch == '\\' {
            if let Some(nc) = chars.peek().copied() {
                if nc.is_ascii_digit() {
                    out.push('$');
                    out.push(nc);
                    let _ = chars.next();
                    continue;
                } else {
                    // escape literal next
                    escape = true;
                    continue;
                }
            } else {
                // trailing backslash, keep as is
                out.push(ch);
                continue;
            }
        } else if ch == '&' {
            out.push('$'); out.push('0');
            continue;
        } else if ch == '~' {
            out.push_str(last_repl);
            continue;
        }
        out.push(ch);
    }
    out
}

fn substitute_lines(lines: &mut Vec<String>, range: (usize, usize), pat: &str, repl: &str, flags: &str) -> Result<usize, String> {
    let mut builder = RegexBuilder::new(pat);
    if flags.contains('i') { builder.case_insensitive(true); }
    let re = builder.build().map_err(|e| format!("regex error: {}", e))?;
    let repl_conv = convert_repl_to_rust(repl, "");
    let start = range.0.saturating_sub(1);
    let end = range.1.saturating_sub(1).min(lines.len().saturating_sub(1));
    let mut total = 0usize;
    let global = flags.contains('g');
    for i in start..=end {
        let count = if global { re.find_iter(&lines[i]).count() } else { if re.is_match(&lines[i]) { 1 } else { 0 } };
        if count > 0 {
            let new_line = if global { re.replace_all(&lines[i], repl_conv.as_str()).to_string() } else { re.replace(&lines[i], repl_conv.as_str()).to_string() };
            lines[i] = new_line;
            total += count;
        }
    }
    Ok(total)
}

fn parse_substitute(cmd: &str) -> Option<(Option<String>, String, String, String)> {
    // Returns (range_str, pat, repl, flags)
    let c = cmd.trim_start_matches(':').trim();
    // Extract optional range prefix before 's'
    let mut idx = 0usize;
    let bytes = c.as_bytes();
    while idx < bytes.len() {
        let ch = bytes[idx] as char;
        if ch == 's' { break; }
        if !(ch == '%' || ch == '.' || ch == '$' || ch == ',' || ch == '+' || ch == '-' || ch.is_ascii_digit() || ch.is_whitespace()) {
            return None;
        }
        idx += 1;
    }
    if idx >= bytes.len() || bytes[idx] as char != 's' { return None; }
    let range_str = if idx == 0 { None } else { Some(c[..idx].trim().to_string()) };
    let mut i = idx + 1; // position after 's'
    if i >= c.len() { return None; }
    let sep = c.as_bytes()[i] as char;
    if sep.is_ascii_whitespace() { return None; }
    i += 1;
    let mut collect = |i: &mut usize| {
        let mut out = String::new();
        let mut esc = false;
        while *i < c.len() {
            let ch = c.as_bytes()[*i] as char;
            *i += 1;
            if esc { out.push(ch); esc = false; continue; }
            if ch == '\\' { esc = true; continue; }
            if ch == sep { break; }
            out.push(ch);
        }
        out
    };
    let pat = collect(&mut i);
    if i >= c.len() { return None; }
    // i currently at position of sep consumed; already moved past in collect?
    // collect consumed until sep and left i at position after sep
    let repl = collect(&mut i);
    // flags are rest
    let flags = c[i..].trim().to_string();
    Some((range_str, pat, repl, flags))
}

#[no_mangle]
pub extern "C" fn rust_editor_main(argc: i32, argv: *const *const i8) -> i32 {
    // 画面サイズ
    let (mut width, mut height) = get_term_size();
    if height < 3 { height = 3; }

    // 引数からファイル名（最初の非オプション）を拾う
    let mut filename: Option<PathBuf> = None;
    if argc > 1 && !argv.is_null() {
        for i in 1..(argc as isize) {
            let p = unsafe { *argv.offset(i) };
            if p.is_null() { continue; }
            let s = unsafe { std::ffi::CStr::from_ptr(p) };
            if let Ok(opt) = s.to_str() {
                if opt.starts_with('-') { continue; }
                filename = Some(PathBuf::from(opt));
                break;
            }
        }
    }

    let mut init_lines: Vec<String> = match filename.as_ref() {
        Some(p) => open_file(p.as_path()),
        None => Vec::new(),
    };
    if init_lines.is_empty() { init_lines.push(String::new()); }
    let mut buffers: Vec<Buffer> = vec![Buffer { lines: init_lines, filename: filename.clone(), modified: false, kind: None }];
    let mut views: Vec<View> = vec![View { buf: 0, cx: 0, cy: 0, scroll: 0 }];
    let mut cur_view: usize = 0;
    let mut status_msg: Option<String> = None;
    let mut tab_width: usize = 4; // Tab入力時の空白幅
    let mut layout = Layout::Horizontal;

    // substitute memory
    let mut last_sub_pat: String = String::new();
    let mut last_sub_repl: String = String::new();
    let mut last_sub_flags: String = String::new();

    let saved = set_raw_mode();
    hide_cursor();
    clear_screen();
    // 初期描画
    draw_all(&buffers, &mut views, cur_view, width, height, &status_msg, layout, Mode::Normal);

    // 簡易raw入力: 1バイトずつ読み、':' でコマンド、Enterで改行、Backspace(0x7F)で削除
    let mut cmd_mode = false;
    let mut cmd = String::new();
    let mut stdin = io::stdin();
    let mut buf = [0u8; 1];

    'outer: loop {
        // リサイズを毎ループで反映
        let (w, h) = get_term_size();
        if w >= 2 && h >= 3 { width = w; height = h; }
        if let Ok(n) = stdin.read(&mut buf) { if n == 0 { break; } } else { break; }
        let b = buf[0];
        if !cmd_mode {
            match b {
                b':' => { cmd_mode = true; cmd.clear(); },
                b'\r' | b'\n' => {
                    // split line at cursor
                    let v = &mut views[cur_view];
                    let bidx = v.buf;
                    let b = &mut buffers[bidx];
                    let current = b.lines[v.cy].clone();
                    let (left, right) = current.split_at(v.cx);
                    b.lines[v.cy] = left.to_string();
                    b.lines.insert(v.cy + 1, right.to_string());
                    v.cy += 1;
                    v.cx = 0;
                    b.modified = true;
                },
                0x7F => {
                    let v = &mut views[cur_view];
                    let b = &mut buffers[v.buf];
                    if v.cx > 0 {
                        b.lines[v.cy].remove(v.cx - 1);
                        v.cx -= 1;
                        b.modified = true;
                    } else if v.cy > 0 {
                        let prev_len = b.lines[v.cy - 1].len();
                        let line = b.lines.remove(v.cy);
                        b.lines[v.cy - 1].push_str(&line);
                        v.cy -= 1;
                        v.cx = prev_len;
                        b.modified = true;
                    }
                },
                0x13 => { // Ctrl-S save
                    let v = &views[cur_view];
                    let b = &mut buffers[v.buf];
                    if let Some(ref p) = b.filename { if save_file(p.as_path(), &b.lines).is_ok() { b.modified = false; status_msg = Some("written".to_string()); } else { status_msg = Some("write error".to_string()); } }
                    else { status_msg = Some("No file name".to_string()); }
                }
                0x11 => { // Ctrl-Q quit (with check)
                    let v = &views[cur_view];
                    if buffers[v.buf].modified { status_msg = Some("No write since last change (:q! to quit)".to_string()); }
                    else { break 'outer; }
                }
                _ => {
                    if b == 0x1B { // ESC sequence
                        // try to read two more bytes: [ A/B/C/D
                        let mut first = [0u8; 1];
                        let _ = stdin.read_exact(&mut first);
                        if first[0] == b'[' {
                            let mut second = [0u8; 1];
                            let _ = stdin.read_exact(&mut second);
                            let v = &mut views[cur_view];
                            let b = &mut buffers[v.buf];
                            match second[0] {
                                b'A' => { // up
                                    if v.cy > 0 { v.cy -= 1; v.cx = v.cx.min(b.lines[v.cy].len()); }
                                }
                                b'B' => { // down
                                    if v.cy + 1 < b.lines.len() { v.cy += 1; v.cx = v.cx.min(b.lines[v.cy].len()); }
                                }
                                b'C' => { // right
                                    if v.cx < b.lines[v.cy].len() { v.cx += 1; }
                                    else if v.cy + 1 < b.lines.len() { v.cy += 1; v.cx = 0; }
                                }
                                b'D' => { // left
                                    if v.cx > 0 { v.cx -= 1; }
                                    else if v.cy > 0 { v.cy -= 1; v.cx = b.lines[v.cy].len(); }
                                }
                                b'H' => { // Home
                                    v.cx = 0;
                                }
                                b'F' => { // End
                                    v.cx = b.lines[v.cy].len();
                                }
                                b'0'..=b'9' => {
                                    // Read until '~'
                                    let mut digits = vec![second[0]];
                                    let mut ch = [0u8;1];
                                    loop {
                                        if stdin.read(&mut ch).ok().unwrap_or(0) == 0 { break; }
                                        if ch[0] == b'~' { break; }
                                        digits.push(ch[0]);
                                        if digits.len() > 3 { break; }
                                    }
                                    let code = String::from_utf8_lossy(&digits);
                                    match code.as_ref() {
                                        "1"|"7" => v.cx = 0,          // Home variants
                                        "4"|"8" => v.cx = b.lines[v.cy].len(), // End variants
                                        "5" => { // PageUp
                                            let view_rows = height.saturating_sub(2);
                                            if v.cy >= view_rows { v.cy -= view_rows; } else { v.cy = 0; }
                                            v.cx = v.cx.min(b.lines[v.cy].len());
                                        }
                                        "6" => { // PageDown
                                            let view_rows = height.saturating_sub(2);
                                            if v.cy + view_rows < b.lines.len() { v.cy += view_rows; } else { v.cy = b.lines.len().saturating_sub(1); }
                                            v.cx = v.cx.min(b.lines[v.cy].len());
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else if b == 0x17 { // Ctrl-W
                        let mut next = [0u8;1];
                        let _ = stdin.read(&mut next);
                        match next[0] as char {
                            'w' | 'W' => { if views.len() > 1 { cur_view = (cur_view + 1) % views.len(); } },
                            _ => {}
                        }
                    } else if b == b'h' { let v=&mut views[cur_view]; let len = buffers[v.buf].lines[v.cy].len(); if v.cx > 0 { v.cx -= 1; } else if v.cy > 0 { v.cy -= 1; v.cx = len; } }
                    else if b == b'l' { let v=&mut views[cur_view]; let blen = buffers[v.buf].lines.len(); let llen = buffers[v.buf].lines[v.cy].len(); if v.cx < llen { v.cx += 1; } else if v.cy + 1 < blen { v.cy += 1; v.cx = 0; } }
                    else if b == b'k' { let v=&mut views[cur_view]; v.cy = v.cy.saturating_sub(1); let llen = buffers[v.buf].lines[v.cy].len(); v.cx = v.cx.min(llen); }
                    else if b == b'j' { let v=&mut views[cur_view]; let blen = buffers[v.buf].lines.len(); if v.cy + 1 < blen { v.cy += 1; let llen = buffers[v.buf].lines[v.cy].len(); v.cx = v.cx.min(llen); } }
                    else if b == b'J' { // Join
                        let v = &mut views[cur_view];
                        if join_current_with_next(&mut buffers[v.buf].lines, v.cy) { buffers[v.buf].modified = true; }
                    }
                    else if (0x20..=0x7E).contains(&b) || b == b'\t' {
                        let v = &mut views[cur_view];
                        let bbuf = &mut buffers[v.buf];
                        if b == b'\t' {
                            let spaces = " ".repeat(tab_width.max(1));
                            for ch in spaces.chars() { bbuf.lines[v.cy].insert(v.cx, ch); v.cx += 1; }
                        } else {
                            bbuf.lines[v.cy].insert(v.cx, b as char);
                            v.cx += 1;
                        }
                        bbuf.modified = true;
                    }
                }
            }
        } else {
            match b {
                b'\r' | b'\n' => {
                    // コマンド確定
                    let c = cmd.trim();
                    let vbuf = views[cur_view].buf;
                    if c == ":q" || c == "q" {
                        if buffers[vbuf].modified { status_msg = Some("No write since last change (:q! to quit)".to_string()); }
                        else { break 'outer; }
                    } else if c == ":q!" || c == "q!" { break 'outer; }

                    // substitution: [range]s/pat/repl/flags and repeat forms
                    if c == ":&" || c == "&" || c == ":&&" || c == "&&" {
                        if last_sub_pat.is_empty() {
                            status_msg = Some("no previous substitute".into());
                        } else {
                            let rng = if c.ends_with("&&") { (1, buffers[vbuf].lines.len()) } else { (views[cur_view].cy+1, views[cur_view].cy+1) };
                            // Use last_sub_repl and flags; allow ~ in repl to expand to last repl (which is itself)
                            match substitute_lines(&mut buffers[vbuf].lines, rng, &last_sub_pat, &last_sub_repl, &last_sub_flags) {
                                Ok(n) => { if n>0 { buffers[vbuf].modified = true; } status_msg = Some(format!("substitutions: {}", n)); },
                                Err(e) => { status_msg = Some(e); }
                            }
                        }
                    } else if let Some((rng, mut pat, mut repl, mut flags)) = parse_substitute(c) {
                        // Empty pattern -> last pattern
                        if pat.is_empty() { pat = last_sub_pat.clone(); }
                        // Expand replacement with last repl
                        let repl_conv = convert_repl_to_rust(&repl, &last_sub_repl);
                        // Save flags default if omitted
                        if flags.is_empty() { flags = last_sub_flags.clone(); }
                        let (start,end) = if let Some(rs) = rng { parse_range(&rs, buffers[vbuf].lines.len(), views[cur_view].cy).unwrap_or((views[cur_view].cy+1, views[cur_view].cy+1)) } else { (views[cur_view].cy+1, views[cur_view].cy+1) };
                        match substitute_lines(&mut buffers[vbuf].lines, (start,end), &pat, &repl_conv, &flags) {
                            Ok(n) => { 
                                // Update last substitute memory using raw (unconverted) repl string
                                last_sub_pat = pat.clone();
                                last_sub_repl = repl.clone();
                                last_sub_flags = flags.clone();
                                if n>0 { buffers[vbuf].modified = true; }
                                status_msg = Some(format!("substitutions: {}", n)); 
                            },
                            Err(e) => { status_msg = Some(e); }
                        }
                    } else if c.starts_with(":w") || c == "w" {
                        let parts: Vec<&str> = c.split_whitespace().collect();
                        // Support range write: :[range]w {file}
                        // Very simple parse: last token is filename, optional leading range handled earlier by parse_substitute branch not matching
                        if parts.len() >= 2 {
                            let fname = parts.last().unwrap().to_string();
                            // detect append mode via >>
                            let append = parts.iter().any(|&t| t == ">>" );
                            let (start,end) = if parts[0].starts_with(':') && parts[0].len() > 2 { // e.g., :1,10w file
                                if let Some((r, _)) = c.split_once('w') { parse_range(r.trim_start_matches(':'), buffers[vbuf].lines.len(), views[cur_view].cy).unwrap_or((1, buffers[vbuf].lines.len())) } else { (1, buffers[vbuf].lines.len()) }
                            } else { (1, buffers[vbuf].lines.len()) };
                            let path = PathBuf::from(fname);
                            let mut s = String::new();
                            for (i, line) in buffers[vbuf].lines[start-1..=end-1].iter().enumerate() {
                                if i>0 { s.push('\n'); }
                                s.push_str(line);
                            }
                            let res = if append { fs::OpenOptions::new().create(true).append(true).open(&path).and_then(|mut f| std::io::Write::write_all(&mut f, s.as_bytes())) } else { fs::write(&path, s) };
                            match res {
                                Ok(_) => { status_msg = Some("written".into()); },
                                Err(_) => { status_msg = Some("write error".into()); }
                            }
                        } else {
                            // fallback to current filename
                            if let Some(ref p) = buffers[vbuf].filename {
                                match save_file(p.as_path(), &buffers[vbuf].lines) {
                                    Ok(_) => { buffers[vbuf].modified = false; status_msg = Some("written".into()); }
                                    Err(_) => { status_msg = Some("write error".into()); }
                                }
                            } else { status_msg = Some("No file name".into()); }
                        }
                    } else if c == ":wq" || c == "wq" {
                        if let Some(ref p) = buffers[vbuf].filename {
                            let _ = save_file(p.as_path(), &buffers[vbuf].lines);
                            break 'outer;
                        } else { status_msg = Some("No file name".into()); }
                    } else if c.starts_with(":e!") || c.starts_with("e!") {
                        // 強制読み込み
                        let parts: Vec<&str> = c.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let p = PathBuf::from(parts[1]);
                            let new_lines = open_file(&p);
                            buffers[vbuf].lines = if new_lines.is_empty() { vec![String::new()] } else { new_lines };
                            buffers[vbuf].filename = Some(p);
                            views[cur_view].cx = 0; views[cur_view].cy = 0; views[cur_view].scroll = 0; buffers[vbuf].modified = false; status_msg = Some("reloaded".into());
                        } else { status_msg = Some("edit: missing file".into()); }
                    } else if c.starts_with(":e") || c.starts_with("e ") {
                        let parts: Vec<&str> = c.split_whitespace().collect();
                        if parts.len() >= 2 {
                            if buffers[vbuf].modified { status_msg = Some("No write since last change (:e! to force)".into()); }
                            else {
                                let p = PathBuf::from(parts[1]);
                                let new_lines = open_file(&p);
                                buffers[vbuf].lines = if new_lines.is_empty() { vec![String::new()] } else { new_lines };
                                buffers[vbuf].filename = Some(p);
                                views[cur_view].cx = 0; views[cur_view].cy = 0; views[cur_view].scroll = 0; buffers[vbuf].modified = false; status_msg = Some("edited".into());
                            }
                        } else { status_msg = Some("edit: missing file".into()); }
                    } else if c == ":trim" || c == "trim" {
                        let n = trim_trailing_whitespace(&mut buffers[vbuf].lines);
                        if n > 0 { buffers[vbuf].modified = true; }
                        status_msg = Some(format!("trim trailing ws: {} lines", n));
                    } else if c.starts_with(":retab") || c.starts_with("retab") {
                        let parts: Vec<&str> = c.split_whitespace().collect();
                        let ts = if parts.len() >= 2 { parts[1].parse::<usize>().unwrap_or(tab_width) } else { tab_width };
                        let n = retab_lines(&mut buffers[vbuf].lines, ts.max(1));
                        if n > 0 { buffers[vbuf].modified = true; }
                        status_msg = Some(format!("retab width={}, changed {} lines", ts, n));
                    } else if c.starts_with(":fmt") || c == "fmt" {
                        let parts: Vec<&str> = c.split_whitespace().collect();
                        let width = if parts.len() >= 2 { parts[1].parse::<usize>().unwrap_or(4) } else { 4 };
                        let n = fmt_brace_style(&mut buffers[vbuf].lines, width.max(1));
                        if n > 0 { buffers[vbuf].modified = true; }
                        status_msg = Some(format!("fmt indent={}, changed {} lines", width, n));
                    } else if c.starts_with(":sort") || c.starts_with("sort") {
                        let rest = c.strip_prefix(":sort").or_else(|| c.strip_prefix("sort")).unwrap_or("").trim();
                        let range = if rest.is_empty() { Some((1, buffers[vbuf].lines.len())) } else { parse_range(rest, buffers[vbuf].lines.len(), views[cur_view].cy) };
                        if let Some(r) = range { sort_lines_range(&mut buffers[vbuf].lines, r); buffers[vbuf].modified = true; status_msg = Some("sorted".into()); }
                        else { status_msg = Some("sort: bad range".into()); }
                    } else if c.starts_with(":delete") || c.starts_with("delete") || c == ":del" || c == "del" {
                        let rest = c.strip_prefix(":delete").or_else(|| c.strip_prefix("delete")).unwrap_or("");
                        let range = if rest.trim().is_empty() { Some((views[cur_view].cy + 1, views[cur_view].cy + 1)) } else { parse_range(rest, buffers[vbuf].lines.len(), views[cur_view].cy) };
                        if let Some(r) = range { let removed = delete_range(&mut buffers[vbuf].lines, r); buffers[vbuf].modified = true; status_msg = Some(format!("deleted {} line(s)", removed)); views[cur_view].cy = views[cur_view].cy.min(buffers[vbuf].lines.len() - 1); views[cur_view].cx = views[cur_view].cx.min(buffers[vbuf].lines[views[cur_view].cy].len()); }
                        else { status_msg = Some("delete: bad range".into()); }
                    } else if c == ":join" || c == "join" || c == ":j" || c == "j" {
                        if join_current_with_next(&mut buffers[vbuf].lines, views[cur_view].cy) { buffers[vbuf].modified = true; status_msg = Some("joined".into()); }
                        else { status_msg = Some("join: at last line".into()); }
                    } else if c.starts_with(":set") || c.starts_with("set") {
                        // set ts=4 など簡易実装
                        if let Some(eq) = c.find("ts=") {
                            let v = &c[eq+3..];
                            if let Ok(n) = v.trim().parse::<usize>() { tab_width = n.max(1); status_msg = Some(format!("tabstop={}", tab_width)); } else { status_msg = Some("set: bad ts value".into()); }
                        } else { status_msg = Some(format!("tabstop={}", tab_width)); }
                    } else if let Ok(n) = c.trim_start_matches(':').parse::<usize>() {
                        // :{number} でジャンプ
                        let tgt = n.saturating_sub(1).min(buffers[vbuf].lines.len().saturating_sub(1));
                        views[cur_view].cy = tgt; views[cur_view].cx = views[cur_view].cx.min(buffers[vbuf].lines[views[cur_view].cy].len());
                        status_msg = Some(format!("goto {}", n));
                    } else if c == ":ls" || c == "ls" {
                        let mut s = String::new();
                        for (i, b) in buffers.iter().enumerate() {
                            if i > 0 { s.push_str(" | "); }
                            let mark = if i == views[cur_view].buf { '%' } else { ' ' };
                            let modm = if b.modified { '+' } else { ' ' };
                            let name = b.filename.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "[No Name]".to_string());
                            s.push_str(&format!("{}{}{}:{}", mark, modm, i, name));
                            if s.len() > (width.saturating_sub(10)) { s.push_str(" ..."); break; }
                        }
                        status_msg = Some(s);
                    } else if c.starts_with(":badd ") || c.starts_with("badd ") {
                        let parts: Vec<&str> = c.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let p = PathBuf::from(parts[1]);
                            let mut lines = open_file(&p);
                            if lines.is_empty() { lines.push(String::new()); }
                            buffers.push(Buffer { lines, filename: Some(p.clone()), modified: false, kind: None });
                            status_msg = Some(format!("badd: {} (#{})", p.to_string_lossy(), buffers.len()-1));
                        } else { status_msg = Some("badd: missing file".into()); }
                    } else if c == ":bn" || c == "bn" {
                        if !buffers.is_empty() { views[cur_view].buf = (views[cur_view].buf + 1) % buffers.len(); }
                    } else if c == ":bp" || c == "bp" {
                        if !buffers.is_empty() { views[cur_view].buf = (views[cur_view].buf + buffers.len() - 1) % buffers.len(); }
                    } else if c.starts_with(":buffer ") || c.starts_with("buffer ") || c.starts_with(":b ") || c.starts_with("b ") {
                        let parts: Vec<&str> = c.split_whitespace().collect();
                        if parts.len() >= 2 { if let Ok(n) = parts[1].parse::<usize>() { if n < buffers.len() { views[cur_view].buf = n; } } }
                    } else if c == ":split" || c == "split" || c == ":sp" || c == "sp" {
                        // horizontal split
                        let v = views[cur_view].clone();
                        views.push(View { buf: v.buf, cx: v.cx, cy: v.cy, scroll: v.scroll });
                        cur_view = views.len() - 1;
                        layout = Layout::Horizontal;
                    } else if c == ":vsplit" || c == "vsplit" || c == ":vsp" || c == "vsp" {
                        // vertical split
                        let v = views[cur_view].clone();
                        views.push(View { buf: v.buf, cx: v.cx, cy: v.cy, scroll: v.scroll });
                        cur_view = views.len() - 1;
                        layout = Layout::Vertical;
                    } else if c == ":only" || c == "only" {
                        let keep = views[cur_view].clone();
                        views.clear();
                        views.push(keep);
                        cur_view = 0;
                    } else if c == ":close" || c == "close" {
                        if views.len() > 1 { views.remove(cur_view); cur_view = 0; } else { status_msg = Some("cannot close last window".into()); }
                    } else if c == ":wincmd w" || c == "wincmd w" {
                        if views.len() > 1 { cur_view = (cur_view + 1) % views.len(); }
                    } else if c.starts_with(":read ") || c.starts_with("read ") {
                        let parts: Vec<&str> = c.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let p = PathBuf::from(parts[1]);
                            let mut extra = open_file(&p);
                            let v = &mut views[cur_view];
                            let b = &mut buffers[v.buf];
                            if extra.is_empty() { extra.push(String::new()); }
                            let insert_at = v.cy + 1;
                            for (i, line) in extra.into_iter().enumerate() {
                                b.lines.insert(insert_at + i, line);
                            }
                            b.modified = true;
                            status_msg = Some("read ok".into());
                        } else { status_msg = Some("read: missing file".into()); }
                    } else if c == ":buffers" || c == "buffers" {
                        // Create a listing buffer and switch to it
                        let mut lines = Vec::new();
                        lines.push("Buffers:".to_string());
                        lines.push(" NR  Mod Current  Name".to_string());
                        for (i, b) in buffers.iter().enumerate() {
                            let cur = if i == views[cur_view].buf { '*' } else { ' ' };
                            let modm = if b.modified { '+' } else { ' ' };
                            let name = b.filename.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "[No Name]".to_string());
                            lines.push(format!("{:>3}   {}     {}     {}", i, modm, cur, name));
                        }
                        buffers.push(Buffer { lines, filename: None, modified: false, kind: Some(BufferKind::Buffers) });
                        views[cur_view].buf = buffers.len() - 1;
                    } else if c == ":windows" || c == "windows" {
                        let mut lines = Vec::new();
                        lines.push("Windows:".to_string());
                        lines.push(" NR  Buf  Cursor   Scroll".to_string());
                        for (i, v) in views.iter().enumerate() {
                            lines.push(format!("{:>3}  {:>3}  {:>4},{}   {}", i, v.buf, v.cy+1, v.cx+1, v.scroll));
                        }
                        buffers.push(Buffer { lines, filename: None, modified: false, kind: Some(BufferKind::Windows) });
                        views[cur_view].buf = buffers.len() - 1;
                    } else if c == ":help" || c.starts_with(":help ") || c == "help" || c.starts_with("help ") {
                        let mut lines = Vec::new();
                        lines.push("Rust Vim (mini) Help".into());
                        lines.push("Commands:".into());
                        lines.push(":e {file} | :e! {file} | :w | :w {file} | :wq | :q | :q!".into());
                        lines.push(":trim | :retab [n] | :fmt [indent] | :sort [range] | :delete [range] | :join".into());
                        lines.push(":set ts={n} | :{number}".into());
                        lines.push(":badd {file} | :bn | :bp | :buffer {n} | :ls | :buffers".into());
                        lines.push(":split | :vsplit | :only | :close | :wincmd w | Ctrl-W w".into());
                        lines.push(":read {file} | :write [range] {file} | :write >> {file}".into());
                        lines.push(":%s/pat/repl/[g][i] | :& (repeat on line) | :&& (repeat on buffer)".into());
                        buffers.push(Buffer { lines, filename: None, modified: false, kind: Some(BufferKind::Help) });
                        views[cur_view].buf = buffers.len() - 1;
                    }
                    cmd_mode = false; cmd.clear();
                }
                0x7F => { cmd.pop(); print!("\x08 \x08"); flush(); }
                _ => {
                    if (0x20..=0x7E).contains(&b) { cmd.push(b as char); print!("{}", b as char); flush(); }
                }
            }
        }

        // 再描画
        // mode will be updated later; placeholder gets replaced below
        // This call will be overwritten by the final call at loop bottom
        if cmd_mode { print!("\r\n:{}", cmd); } else { print!("\r\n"); }
        flush();
    }

    show_cursor();
    clear_screen();
    flush();
    restore_mode(&saved);
    0
}

fn draw_all(buffers: &Vec<Buffer>, views: &mut Vec<View>, cur_view: usize, width: usize, height: usize, status: &Option<String>, layout: Layout, mode: Mode) {
    clear_screen();
    let vcount = views.len().max(1);
    let content_rows_total = height.saturating_sub(2);
    match layout {
        Layout::Horizontal => {
            let seps = if vcount > 1 { vcount - 1 } else { 0 };
            let mut rows_remaining = content_rows_total.saturating_sub(seps);
            let mut offsets: Vec<(usize, usize)> = Vec::with_capacity(vcount); // (start_row, rows)
            for i in 0..vcount {
                let left = vcount - i;
                let rows = if left == 0 { 0 } else { rows_remaining / left + if rows_remaining % left != 0 { 1 } else { 0 } };
                offsets.push((offsets.last().map(|(s, r)| s + r + 1).unwrap_or(0), rows));
                rows_remaining = rows_remaining.saturating_sub(rows);
            }
            for (i, v) in views.iter_mut().enumerate() {
                let (start_row, rows) = offsets[i];
                print!("\x1b[{};{}H", start_row + 1, 1);
                let b = &buffers[v.buf];
                if v.cy < v.scroll { v.scroll = v.cy; }
                if rows > 0 && v.cy >= v.scroll + rows { v.scroll = v.cy + 1 - rows; }
                draw_rows(&b.lines, v.scroll, rows, width);
                if i + 1 < vcount { print!("{:->width$}\r\n", "-", width = width); }
            }
            let vb = &views[cur_view];
            let b = &buffers[vb.buf];
            draw_status_line(&b.filename, vb.cx, vb.cy, b.modified, mode, status, width);
            let mut cur_offset = 0usize;
            for i in 0..cur_view { cur_offset += offsets[i].1 + 1; }
            let screen_y = cur_offset + (vb.cy - vb.scroll) + 1;
            let screen_x = vb.cx + 1;
            print!("\x1b[{};{}H", screen_y, screen_x);
        }
        Layout::Vertical => {
            let seps = if vcount > 1 { vcount - 1 } else { 0 };
            let cols_total = width.saturating_sub(seps);
            let mut cols_remaining = cols_total;
            let mut col_widths: Vec<usize> = Vec::with_capacity(vcount);
            for i in 0..vcount {
                let left = vcount - i;
                let w = if left == 0 { 0 } else { cols_remaining / left + if cols_remaining % left != 0 { 1 } else { 0 } };
                col_widths.push(w);
                cols_remaining = cols_remaining.saturating_sub(w);
            }
            let mut col_offsets: Vec<usize> = Vec::with_capacity(vcount);
            let mut acc = 0usize;
            for (i, w) in col_widths.iter().enumerate() {
                col_offsets.push(acc);
                acc += *w + if i + 1 < vcount { 1 } else { 0 };
            }
            // Ensure visibility for each view
            for v in views.iter_mut() {
                if v.cy < v.scroll { v.scroll = v.cy; }
                if v.cy >= v.scroll + content_rows_total { v.scroll = v.cy + 1 - content_rows_total; }
            }
            // Draw rows combining columns
            for r in 0..content_rows_total {
                let mut line_out = String::with_capacity(width);
                for (i, v) in views.iter().enumerate() {
                    let b = &buffers[v.buf];
                    let li = v.scroll + r;
                    let mut s = if li < b.lines.len() { b.lines[li].clone() } else { "~".to_string() };
                    if s.len() > col_widths[i] { s.truncate(col_widths[i]); }
                    if s.len() < col_widths[i] { s.push_str(&" ".repeat(col_widths[i] - s.len())); }
                    line_out.push_str(&s);
                    if i + 1 < vcount { line_out.push('|'); }
                }
                println!("{}", line_out);
            }
            let vb = &views[cur_view];
            let b = &buffers[vb.buf];
            draw_status_line(&b.filename, vb.cx, vb.cy, b.modified, mode, status, width);
            let screen_y = (vb.cy - vb.scroll) + 1;
            let mut screen_x = vb.cx + 1;
            let vw = col_widths[cur_view];
            if screen_x > vw { screen_x = vw; }
            let x_off = col_offsets[cur_view];
            print!("\x1b[{};{}H", screen_y, x_off + screen_x);
        }
    }
}
