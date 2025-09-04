use std::io::{self, Write, Read};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn clear_screen() {
    print!("\x1b[2J\x1b[H");
}

fn hide_cursor() {
    print!("\x1b[?25l");
}

fn show_cursor() {
    print!("\x1b[?25h");
}

fn draw_status_line(filename: &Option<PathBuf>, cx: usize, cy: usize, modified: bool, status: &Option<String>, width: usize) {
    let name = filename.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "[No Name]".to_string());
    let mod_mark = if modified { " [+]" } else { "" };
    let right = status.as_ref().map(|s| s.as_str()).unwrap_or("");
    let base = format!(" {} - {}:{}{} ", name, cy + 1, cx + 1, mod_mark);
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

fn draw_rows(lines: &Vec<String>, scroll: usize, height: usize, width: usize) {
    for i in 0..(height - 2) {
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

    let mut lines: Vec<String> = match filename.as_ref() {
        Some(p) => open_file(p.as_path()),
        None => Vec::new(),
    };
    if lines.is_empty() { lines.push(String::new()); }
    let mut cx: usize = 0;
    let mut cy: usize = 0;
    let mut scroll: usize = 0;
    let mut modified = false;
    let mut status_msg: Option<String> = None;

    let saved = set_raw_mode();
    hide_cursor();
    clear_screen();
    draw_rows(&lines, scroll, height, width);
    draw_status_line(&filename, cx, cy, modified, &status_msg, width);
    // 初期カーソル移動
    print!("\x1b[{};{}H", (cy - scroll) + 1, cx + 1);
    flush();

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
                    let current = lines[cy].clone();
                    let (left, right) = current.split_at(cx);
                    lines[cy] = left.to_string();
                    lines.insert(cy + 1, right.to_string());
                    cy += 1;
                    cx = 0;
                    modified = true;
                },
                0x7F => {
                    if cx > 0 {
                        lines[cy].remove(cx - 1);
                        cx -= 1;
                        modified = true;
                    } else if cy > 0 {
                        let prev_len = lines[cy - 1].len();
                        let line = lines.remove(cy);
                        lines[cy - 1].push_str(&line);
                        cy -= 1;
                        cx = prev_len;
                        modified = true;
                    }
                },
                0x13 => { // Ctrl-S save
                    if let Some(ref p) = filename { if save_file(p.as_path(), &lines).is_ok() { modified = false; status_msg = Some("written".to_string()); } else { status_msg = Some("write error".to_string()); } }
                    else { status_msg = Some("No file name".to_string()); }
                }
                0x11 => { // Ctrl-Q quit (with check)
                    if modified { status_msg = Some("No write since last change (:q! to quit)".to_string()); }
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
                            match second[0] {
                                b'A' => { // up
                                    if cy > 0 { cy -= 1; cx = cx.min(lines[cy].len()); }
                                }
                                b'B' => { // down
                                    if cy + 1 < lines.len() { cy += 1; cx = cx.min(lines[cy].len()); }
                                }
                                b'C' => { // right
                                    if cx < lines[cy].len() { cx += 1; }
                                    else if cy + 1 < lines.len() { cy += 1; cx = 0; }
                                }
                                b'D' => { // left
                                    if cx > 0 { cx -= 1; }
                                    else if cy > 0 { cy -= 1; cx = lines[cy].len(); }
                                }
                                b'H' => { // Home
                                    cx = 0;
                                }
                                b'F' => { // End
                                    cx = lines[cy].len();
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
                                        "1"|"7" => cx = 0,          // Home variants
                                        "4"|"8" => cx = lines[cy].len(), // End variants
                                        "5" => { // PageUp
                                            let view = height - 2;
                                            if cy >= view { cy -= view; } else { cy = 0; }
                                            cx = cx.min(lines[cy].len());
                                        }
                                        "6" => { // PageDown
                                            let view = height - 2;
                                            if cy + view < lines.len() { cy += view; } else { cy = lines.len().saturating_sub(1); }
                                            cx = cx.min(lines[cy].len());
                                        }
                                        _ => {}
                                    }
                                }
                                _ => {}
                            }
                        }
                    } else if b == b'h' { if cx > 0 { cx -= 1; } else if cy > 0 { cy -= 1; cx = lines[cy].len(); } }
                    else if b == b'l' { if cx < lines[cy].len() { cx += 1; } else if cy + 1 < lines.len() { cy += 1; cx = 0; } }
                    else if b == b'k' { if cy > 0 { cy -= 1; cx = cx.min(lines[cy].len()); } }
                    else if b == b'j' { if cy + 1 < lines.len() { cy += 1; cx = cx.min(lines[cy].len()); } }
                    else if (0x20..=0x7E).contains(&b) || b == b'\t' {
                        lines[cy].insert(cx, if b == b'\t' { ' ' } else { b as char });
                        cx += 1;
                        modified = true;
                    }
                }
            }
        } else {
            match b {
                b'\r' | b'\n' => {
                    // コマンド確定
                    let c = cmd.trim();
                    if c == ":q" || c == "q" {
                        if modified { status_msg = Some("No write since last change (:q! to quit)".to_string()); }
                        else { break 'outer; }
                    } else if c == ":q!" || c == "q!" { break 'outer; }
                    if c.starts_with(":w") || c == "w" {
                        let parts: Vec<&str> = c.split_whitespace().collect();
                        if parts.len() >= 2 {
                            filename = Some(PathBuf::from(parts[1]));
                        }
                        if let Some(ref p) = filename {
                            match save_file(p.as_path(), &lines) {
                                Ok(_) => { modified = false; status_msg = Some("written".into()); }
                                Err(_) => { status_msg = Some("write error".into()); }
                            }
                        }
                    } else if c == ":wq" || c == "wq" {
                        if let Some(ref p) = filename {
                            let _ = save_file(p.as_path(), &lines);
                            break 'outer;
                        } else { status_msg = Some("No file name".into()); }
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
        clear_screen();
        // スクロール調整
        if cy < scroll { scroll = cy; }
        let view_rows = height - 2;
        if cy >= scroll + view_rows { scroll = cy + 1 - view_rows; }
        draw_rows(&lines, scroll, height, width);
        draw_status_line(&filename, cx, cy, modified, &status_msg, width);
        if cmd_mode { print!("\r\n:{}", cmd); }
        else { print!("\r\n"); }
        // カーソル位置（1-based）。スクロール分を引く
        let screen_y = (cy - scroll) + 1;
        let screen_x = cx + 1;
        print!("\x1b[{};{}H", screen_y, screen_x);
        flush();
    }

    show_cursor();
    clear_screen();
    flush();
    restore_mode(&saved);
    0
}
