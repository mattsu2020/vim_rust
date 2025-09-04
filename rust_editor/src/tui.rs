use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossterm::{event, execute, terminal};
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{backend::CrosstermBackend, Terminal};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Style, Modifier};
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Borders, Paragraph};
use regex::{Regex, RegexBuilder};

#[derive(Clone)]
struct View { kind: ViewKind, cx: usize, cy: usize, scroll: usize, buf: Option<usize> }

fn open_file(path: &Path) -> Vec<String> {
    match fs::read_to_string(path) {
        Ok(content) => content.replace('\r', "").split('\n').map(|s| s.to_string()).collect(),
        Err(_) => Vec::new(),
    }
}

fn save_file(path: &Path, lines: &Vec<String>) -> std::io::Result<()> {
    let mut s = String::new();
    for (i, line) in lines.iter().enumerate() {
        s.push_str(line);
        if i + 1 < lines.len() { s.push('\n'); }
    }
    fs::write(path, s)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode { Normal, Insert, Command, SearchFwd, SearchBwd, VisualChar, VisualLine }

#[derive(Clone, Copy, PartialEq, Eq)]
enum SplitLayout { Horizontal, Vertical }

#[derive(Clone, Copy, PartialEq, Eq)]
enum ViewKind { Normal, BuffersList, Help }

#[derive(Clone)]
struct UndoSnap { buf: Option<usize>, lines: Vec<String>, cx: usize, cy: usize }

struct SearchState {
    regex: Option<Regex>,
    pattern: String,
    case_insensitive: bool,
    last_dir: i32, // 1: forward, -1: backward
}

impl SearchState {
    fn new() -> Self { Self { regex: None, pattern: String::new(), case_insensitive: false, last_dir: 1 } }
}

#[derive(Clone)]
enum Register { Charwise(String), Linewise(Vec<String>) }

#[derive(Clone)]
struct Buffer { lines: Vec<String>, filename: Option<PathBuf>, modified: bool }

// 単語モーション用の簡易ヘルパ
fn is_word_char(c: char) -> bool { c.is_alphanumeric() || c == '_' }

fn motion_w(lines: &Vec<String>, mut cx: usize, mut cy: usize, mut n: usize) -> (usize, usize) {
    while n > 0 {
        let line = &lines[cy];
        let mut x = cx;
        // 1) 現在の単語末尾まで進む
        if x < line.len() {
            let mut in_word = is_word_char(line.chars().nth(x).unwrap_or(' '));
            while x < line.len() {
                let ch = line.chars().nth(x).unwrap_or(' ');
                if in_word { if !is_word_char(ch) { break; } }
                else { if is_word_char(ch) { break; } }
                x += 1;
            }
        }
        // 2) 次の単語開始へ（改行越え含む）
        loop {
            if x < lines[cy].len() {
                let ch = lines[cy].chars().nth(x).unwrap_or(' ');
                if is_word_char(ch) { break; }
                x += 1;
            } else {
                if cy + 1 >= lines.len() { cx = x.min(lines[cy].len()); cy = cy; n = 1; break; }
                cy += 1; x = 0;
            }
        }
        cx = x; n -= 1;
    }
    (cx, cy)
}

fn motion_e(lines: &Vec<String>, mut cx: usize, mut cy: usize, mut n: usize) -> (usize, usize) {
    while n > 0 {
        // 先に次の単語に入る
        let (mut x, mut y) = (cx, cy);
        loop {
            if x >= lines[y].len() {
                if y + 1 >= lines.len() { cx = lines[y].len().saturating_sub(1); cy = y; n = 1; break; }
                y += 1; x = 0; continue;
            }
            let ch = lines[y].chars().nth(x).unwrap_or(' ');
            if is_word_char(ch) { break; }
            x += 1;
        }
        // 単語の末尾へ
        while x + 1 <= lines[y].len() {
            let ch = lines[y].chars().nth(x).unwrap_or(' ');
            if !is_word_char(ch) { if x > 0 { x -= 1; } break; }
            if x + 1 == lines[y].len() { break; }
            let nch = lines[y].chars().nth(x + 1).unwrap_or(' ');
            if !is_word_char(nch) { break; }
            x += 1;
        }
        cx = x; cy = y; n -= 1;
    }
    (cx, cy)
}

fn motion_b(lines: &Vec<String>, mut cx: usize, mut cy: usize, mut n: usize) -> (usize, usize) {
    while n > 0 {
        let mut x = cx; let mut y = cy;
        if x == 0 { if y == 0 { return (0,0); } y -= 1; x = lines[y].len(); }
        // 直前の単語境界へ
        let mut seen_word = false;
        while x > 0 {
            x -= 1;
            let ch = lines[y].chars().nth(x).unwrap_or(' ');
            if is_word_char(ch) { seen_word = true; }
            else if seen_word { x += 1; break; }
            if x == 0 { break; }
        }
        cx = x; cy = y; n -= 1;
    }
    (cx, cy)
}

fn prev_char_pos(lines: &Vec<String>, x: usize, y: usize) -> Option<(usize, usize)> {
    if x > 0 { Some((x - 1, y)) } else if y > 0 { let py = y - 1; let plen = lines[py].len(); if plen == 0 { None } else { Some((plen - 1, py)) } } else { None }
}

// 汎用ヘルパ: 分割レイアウトの領域計算
fn split_rect(layout: SplitLayout, rect: Rect, n: usize) -> Vec<Rect> {
    if n <= 1 { return vec![rect]; }
    let mut constraints: Vec<Constraint> = Vec::with_capacity(n);
    for i in 0..n {
        let mut p = (100 / n) as u16;
        if i == n - 1 { p = 100 - (p * (n as u16 - 1)); }
        constraints.push(Constraint::Percentage(p));
    }
    match layout {
        SplitLayout::Horizontal => Layout::default().direction(Direction::Vertical).constraints(constraints).split(rect).to_vec(),
        SplitLayout::Vertical => Layout::default().direction(Direction::Horizontal).constraints(constraints).split(rect).to_vec(),
    }
}

// アクティブなドキュメント（現在ビューが参照するバッファ or グローバル）へ読み取りアクセス
fn with_active_ro<T>(buffers: &Vec<Buffer>, view_buf: Option<usize>, lines: &Vec<String>, f: impl FnOnce(&Vec<String>) -> T) -> T {
    if let Some(bi) = view_buf { if let Some(b) = buffers.get(bi) { return f(&b.lines); } }
    f(lines)
}

// アクティブなドキュメントへ書き込みアクセス（lines/filename/modified を一括で扱う）
fn with_active_mut<T>(
    buffers: &mut Vec<Buffer>,
    view_buf: Option<usize>,
    lines: &mut Vec<String>,
    filename: &mut Option<PathBuf>,
    modified: &mut bool,
    f: impl FnOnce(&mut Vec<String>, &mut Option<PathBuf>, &mut bool) -> T,
) -> T {
    if let Some(bi) = view_buf {
        if let Some(b) = buffers.get_mut(bi) {
            return f(&mut b.lines, &mut b.filename, &mut b.modified);
        }
    }
    f(lines, filename, modified)
}

// 保存共通化: アクティブドキュメントを保存
fn save_active(
    buffers: &mut Vec<Buffer>,
    view_buf: Option<usize>,
    lines: &mut Vec<String>,
    filename: &mut Option<PathBuf>,
    modified: &mut bool,
) -> Result<(), String> {
    with_active_mut(buffers, view_buf, lines, filename, modified, |ls, fname, m| {
        if let Some(ref p) = fname {
            save_file(p, ls).map_err(|_| "write error".to_string())?;
            *m = false;
            Ok(())
        } else {
            Err("No file name".to_string())
        }
    })
}

pub fn run(args: &[String]) -> std::io::Result<()> {
    // initial state
    let mut filename: Option<PathBuf> = None;
    for a in args.iter().skip(1) { if !a.starts_with('-') { filename = Some(PathBuf::from(a)); break; } }
    let mut lines: Vec<String> = match filename.as_ref() { Some(p) => open_file(p.as_path()), None => Vec::new() };
    if lines.is_empty() { lines.push(String::new()); }
    let mut cx: usize = 0;
    let mut cy: usize = 0;
    let mut scroll: usize = 0;
    let mut modified = false;
    // 保持用バッファリスト（アクティブは直下の lines/filename/modified）
    let mut buffers: Vec<Buffer> = Vec::new();
    let mut status: Option<String> = None;
    let mut mode = Mode::Normal;
    let mut cmdline: String = String::new(); // used for :cmd and /search
    let mut tabstop: usize = 4;
    let mut search = SearchState::new();
    // visual/select + clipboard for basic y/d/p
    let mut visual_anchor: Option<(usize, usize)> = None; // (cx, cy)
    let mut clipboard: Option<Register> = None;
    // 2キーオペレーター待ち（例: dd）
    let mut pending_op: Option<char> = None;
    // オペレーター起点（cx, cy）
    let mut op_anchor: Option<(usize, usize)> = None;
    // 数値カウント (Normal のプレフィックス)
    let mut count: Option<usize> = None;
    // 直前の挿入を記録（'.' の繰り返し用）
    let mut last_insert: String = String::new();
    let mut insert_record: String = String::new();
    // last substitute
    let mut last_pat: String = String::new();
    let mut last_repl: String = String::new();
    let mut last_flags: String = String::new();
    // 単純な1段階のUndo
    let mut undo_snap: Option<UndoSnap> = None;

    // window splits (logical only for now; rendering is single view)
    let mut views: Vec<View> = vec![View { kind: ViewKind::Normal, cx, cy, scroll, buf: None }];
    let mut cur_view: usize = 0;
    let mut layout = SplitLayout::Horizontal;
    let mut last_normal_view: usize = 0;

    // setup terminal
    terminal::enable_raw_mode().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let mut stdout = std::io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen, crossterm::cursor::Hide).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let show_cmd = matches!(mode, Mode::Command | Mode::SearchFwd | Mode::SearchBwd);
            let content_rows = if show_cmd { size.height.saturating_sub(2) } else { size.height.saturating_sub(1) };
            let chunks = if show_cmd {
                Layout::default().direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(content_rows),
                        Constraint::Length(1),
                        Constraint::Length(1),
                    ]).split(size)
            } else {
                Layout::default().direction(Direction::Vertical)
                    .constraints([
                        Constraint::Length(content_rows),
                        Constraint::Length(1),
                    ]).split(size)
            };

            // sync current view state
            if !views.is_empty() {
                views[cur_view].cx = cx; views[cur_view].cy = cy; views[cur_view].scroll = scroll;
            }

            let hl_style = Style::default().add_modifier(Modifier::REVERSED);
            let sel_style = Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD);

            // areas to render
            let areas: Vec<Rect> = split_rect(layout, chunks[0], views.len());

            let n = views.len();
            for i in 0..n {
                let area = areas[i];
                let v = &mut views[i];
                // adjust scroll for visibility
                let view_rows = area.height as usize;
                if v.cy < v.scroll { v.scroll = v.cy; }
                if v.cy >= v.scroll + view_rows { v.scroll = v.cy + 1 - view_rows; }

                let mut text = Text::default();
                match v.kind {
                    ViewKind::Normal => {
                        for r in 0..view_rows {
                            let li = v.scroll + r;
                            if li < lines.len() {
                                let line = &lines[li];
                                let mut spans: Vec<Span> = Vec::new();
                                let mut last = 0usize;
                                let mut matches: Vec<(usize, usize, Style)> = Vec::new();
                                if let Some(re) = &search.regex {
                                    let mut start_at = 0usize;
                                    while let Some(m) = re.find_at(line, start_at) {
                                        let s = m.start(); let e = m.end(); matches.push((s, e, hl_style)); if e == start_at { break; } start_at = e;
                                    }
                                }
                                if i == cur_view && matches!(mode, Mode::VisualChar | Mode::VisualLine) {
                                    if let Some((ax, ay)) = visual_anchor { let (bx, by) = (cx, cy); if (li >= ay && li <= by) || (li >= by && li <= ay) { let (sx, sy, ex, ey) = ordered_region(ax, ay, bx, by); let (s, e) = if sy == ey { if li == sy { (sx.min(line.len()), (ex + 1).min(line.len())) } else { (0, 0) } } else if li == sy { (sx.min(line.len()), line.len()) } else if li == ey { (0, (ex + 1).min(line.len())) } else { (0, line.len()) }; if e > s { matches.push((s, e, sel_style)); } } }
                                }
                                matches.sort_by_key(|m| m.0);
                                for (s, e, st) in matches { if s > last { spans.push(Span::raw(line[last..s].to_string())); } spans.push(Span::styled(line[s..e].to_string(), st)); last = e; }
                                if last < line.len() { spans.push(Span::raw(line[last..].to_string())); }
                                text.lines.push(Line::from(spans));
                            } else { text.lines.push(Line::from("~")); }
                        }
                    }
                    ViewKind::BuffersList => {
                        let header = Line::from("Buffers:");
                        text.lines.push(header);
                        let mut start = v.scroll;
                        for (idx, b) in buffers.iter().enumerate().skip(start) {
                            if text.lines.len() >= view_rows { break; }
                            let name = b.filename.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "[No Name]".into());
                            let mark = if b.modified { '+' } else { ' ' };
                            let line = format!(" {:>3} {} {}", idx, mark, name);
                            if v.cy == (text.lines.len()) { text.lines.push(Line::from(Span::styled(line, sel_style))); }
                            else { text.lines.push(Line::from(line)); }
                        }
                        while text.lines.len() < view_rows { text.lines.push(Line::from("~")); }
                    }
                    ViewKind::Help => {
                        let help = [
                            "Rust TUI Vim (mini) Help",
                            "",
                            ":e {file} / :e! {file} / :w / :wq / :q / :q!",
                            ":badd {file} / :bn / :bp / :buffer {n} / :buffers",
                            ":split / :vsplit / :only / :close / :wincmd w (Ctrl-W w)",
                            ":read {file} / :write [range] {file}",
                            ":%s/pat/repl/[g][i]  (:& / :&& で再実行)",
                            "検索: /pattern (?pattern) / n / N  (\\c:ignore, \\C:match)",
                            "モード: Normal / Insert / Visual(v/V) / Command(:)",
                            "操作: h j k l / 0 $ gg G / i a I A / o O / x J / dd yy cc / p P / D Y / u / .",
                            "q でこのウィンドウを閉じる",
                        ];
                        for s in help.iter().skip(v.scroll) {
                            if text.lines.len() >= view_rows { break; }
                            text.lines.push(Line::from((*s).to_string()));
                        }
                        while text.lines.len() < view_rows { text.lines.push(Line::from("~")); }
                    }
                }
                let content = Paragraph::new(text).block(Block::default().borders(Borders::NONE));
                f.render_widget(content, area);
            }
            // propagate scroll of active view back to top-level
            if !views.is_empty() { scroll = views[cur_view].scroll; }

            // status
            let name = filename.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "[No Name]".to_string());
            let m = if modified { " [+]" } else { "" };
            let mode_tag = match mode { Mode::Normal => "[N]", Mode::Insert => "[I]", Mode::Command => ":", Mode::SearchFwd => "/", Mode::SearchBwd => "?", Mode::VisualChar => "[V]", Mode::VisualLine => "[VL]" };
            let right = status.clone().unwrap_or_default();
            let status_line = Line::from(vec![
                Span::raw(format!(" {} {} - {}:{}{} ", mode_tag, name, cy + 1, cx + 1, m)),
                Span::raw(right),
            ]);
            let status_p = Paragraph::new(status_line).block(Block::default().borders(Borders::NONE));
            f.render_widget(status_p, chunks[chunks.len()-1]);

            // command/search line
            if show_cmd {
                let prompt = match mode { Mode::Command => ':', Mode::SearchFwd => '/', Mode::SearchBwd => '?', _ => ':' };
                let cmd_p = Paragraph::new(Line::from(format!("{}{}", prompt, cmdline))).style(Style::default());
                f.render_widget(cmd_p, chunks[1]);
                // place cursor at cmdline end
                let Rect { x, y, .. } = chunks[1];
                let pos = (x + 1 + cmdline.len() as u16, y);
                f.set_cursor(pos.0, pos.1);
            } else {
                // place cursor in content area of current view
                let area = if views.len() <= 1 { chunks[0] } else {
                    match layout {
                        SplitLayout::Horizontal => {
                            let n = views.len();
                            let mut constraints: Vec<Constraint> = Vec::with_capacity(n);
                            for i in 0..n { let mut p = (100 / n) as u16; if i == n - 1 { p = 100 - (p * (n as u16 - 1)); } constraints.push(Constraint::Percentage(p)); }
                            Layout::default().direction(Direction::Vertical).constraints(constraints).split(chunks[0])[cur_view]
                        }
                        SplitLayout::Vertical => {
                            let n = views.len();
                            let mut constraints: Vec<Constraint> = Vec::with_capacity(n);
                            for i in 0..n { let mut p = (100 / n) as u16; if i == n - 1 { p = 100 - (p * (n as u16 - 1)); } constraints.push(Constraint::Percentage(p)); }
                            Layout::default().direction(Direction::Horizontal).constraints(constraints).split(chunks[0])[cur_view]
                        }
                    }
                };
                let Rect { x, y, .. } = area;
                let v = &views[cur_view];
                let cur_y = y + (v.cy - v.scroll) as u16;
                let cur_x = x + (v.cx as u16);
                f.set_cursor(cur_x, cur_y);
            }
        }).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // input
        if event::poll(Duration::from_millis(250)).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))? {
            match event::read().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))? {
                Event::Key(KeyEvent { code, modifiers, .. }) => {
                    if matches!(mode, Mode::Command | Mode::SearchFwd | Mode::SearchBwd) {
                        match (code, modifiers) {
                            (KeyCode::Esc, _) => { mode = Mode::Normal; cmdline.clear(); }
                            ,(KeyCode::Enter, _) => {
                                if mode == Mode::SearchFwd || mode == Mode::SearchBwd {
                                    // build regex from cmdline and jump
                                    let raw = cmdline.clone();
                                    let pat = raw.as_str();
                                    let mut case_insensitive = false;
                                    // support \c (ignore case) and \C (match case)
                                    if pat.contains("\\c") { case_insensitive = true; }
                                    if pat.contains("\\C") { case_insensitive = false; }
                                    let mut pat_clean = pat.replace("\\c", "");
                                    pat_clean = pat_clean.replace("\\C", "");
                                    let mut builder = RegexBuilder::new(&pat_clean);
                                    if case_insensitive { builder.case_insensitive(true); }
                                    match builder.build() {
                                        Ok(re) => {
                                            search.regex = Some(re);
                                            search.pattern = pat_clean;
                                            search.case_insensitive = case_insensitive;
                                            search.last_dir = if mode == Mode::SearchBwd { -1 } else { 1 };
                                            // jump to first match
                                            if let Some((ny, nx)) = find_next(&lines, cy, cx, &search, search.last_dir) { cy = ny; cx = nx; }
                                        }
                                        Err(e) => { status = Some(format!("regex error: {}", e)); }
                                    }
                                    cmdline.clear(); mode = Mode::Normal;
                                    continue;
                                }
                                let cmd = cmdline.trim();
                                if cmd == "q" || cmd == ":q" { if modified { status = Some("No write since last change (:q! to quit)".into()); } else { break; } }
                                else if cmd == "q!" || cmd == ":q!" { break; }
                                else if cmd.starts_with("wq") || cmd.starts_with(":wq") {
                                    if let Some(ref p) = filename { let _ = save_file(p, &lines); break; } else { status = Some("No file name".into()); }
                                }
                                else if cmd.starts_with("w") || cmd.starts_with(":w") {
                                    let parts: Vec<&str> = cmd.trim_start_matches(':').split_whitespace().collect();
                                    let active_bi = views[cur_view].buf;
                                    if let Some(bi) = active_bi {
                                        if let Some(b) = buffers.get_mut(bi) {
                                            if parts.len() >= 2 { b.filename = Some(PathBuf::from(parts[1])); }
                                            if let Some(ref p) = b.filename { match save_file(p, &b.lines) { Ok(_) => { b.modified = false; status = Some("written".into()); }, Err(_) => status = Some("write error".into()) } } else { status = Some("No file name".into()); }
                                        }
                                    } else {
                                        if parts.len() >= 2 { filename = Some(PathBuf::from(parts[1])); }
                                        if let Some(ref p) = filename { match save_file(p, &lines) { Ok(_) => { modified = false; status = Some("written".into()); }, Err(_) => status = Some("write error".into()) } } else { status = Some("No file name".into()); }
                                    }
                                }
                                else if cmd.starts_with(":badd ") || cmd.starts_with("badd ") {
                                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                                    if parts.len() >= 2 {
                                        let p = PathBuf::from(parts[1]);
                                        let mut l = open_file(&p);
                                        if l.is_empty(){ l.push(String::new()); }
                                        buffers.push(Buffer{ lines: l, filename: Some(p), modified: false });
                                        status = Some(format!("badd: #{}", buffers.len()-1));
                                    } else { status = Some("badd: missing file".into()); }
                                }
                                else if cmd == ":bn" || cmd == "bn" {
                                    if buffers.is_empty() { status = Some("no buffers".into()); } else { let v=&mut views[cur_view]; let idx=v.buf.unwrap_or(usize::MAX); v.buf=Some(if idx==usize::MAX {0} else {(idx+1)%buffers.len()}); cx=0; cy=0; scroll=0; }
                                }
                                else if cmd == ":bp" || cmd == "bp" {
                                    if buffers.is_empty() { status = Some("no buffers".into()); } else { let v=&mut views[cur_view]; let idx=v.buf.unwrap_or(0); v.buf=Some((idx+buffers.len()-1)%buffers.len()); cx=0; cy=0; scroll=0; }
                                }
                                else if cmd.starts_with(":buffer ") || cmd.starts_with("buffer ") || cmd.starts_with(":b ") || cmd.starts_with("b ") {
                                    let parts: Vec<&str> = cmd.split_whitespace().collect(); if parts.len()>=2 { if let Ok(n)=parts[1].parse::<usize>() { if n<buffers.len() { views[cur_view].buf=Some(n); cx=0; cy=0; scroll=0; } else { status=Some("buffer: out of range".into()); } } }
                                }
                                else if cmd == ":buffers" || cmd == "buffers" || cmd == ":ls" || cmd == "ls" {
                                    // 記録: 呼び出し元のビュー
                                    last_normal_view = cur_view;
                                    views.push(View { kind: ViewKind::BuffersList, cx: 1, cy: 1, scroll: 0, buf: None });
                                    cur_view = views.len() - 1;
                                }
                                else if cmd == "&" || cmd == ":&" || cmd == "&&" || cmd == ":&&" {
                                    if last_pat.is_empty() {
                                        status = Some("no previous substitute".into());
                                    } else {
                                        if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ let len=b.lines.len(); let range= if cmd.ends_with("&&") {(1,len)} else {(cy+1,cy+1)}; match substitute_lines(&mut b.lines, range, &last_pat, &last_repl, &last_flags){ Ok(n)=>{ if n>0 { b.modified=true; } status=Some(format!("substitutions: {}", n)); }, Err(e)=> status=Some(e) } } else { let len=lines.len(); let range= if cmd.ends_with("&&") {(1,len)} else {(cy+1,cy+1)}; match substitute_lines(&mut lines, range, &last_pat, &last_repl, &last_flags){ Ok(n)=>{ if n>0 { modified=true; } status=Some(format!("substitutions: {}", n)); }, Err(e)=> status=Some(e) } } }
                                        else { let len=lines.len(); let range= if cmd.ends_with("&&") {(1,len)} else {(cy+1,cy+1)}; match substitute_lines(&mut lines, range, &last_pat, &last_repl, &last_flags){ Ok(n)=>{ if n>0 { modified=true; } status=Some(format!("substitutions: {}", n)); }, Err(e)=> status=Some(e) } }
                                    }
                                }
                                else if cmd.starts_with('s') || cmd.starts_with(":s") || cmd.contains('s') && cmd.find('s').unwrap_or(usize::MAX) < 6 {
                                    if let Some((range_str, pat, repl_raw, flags)) = parse_substitute(cmd) {
                                        let repl = convert_repl_to_rust(&repl_raw, &last_repl);
                                        if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ let (start,end)= if let Some(rs)=range_str { parse_range(&rs, b.lines.len(), cy).unwrap_or((cy+1,cy+1)) } else { (cy+1,cy+1) }; match substitute_lines(&mut b.lines, (start,end), &pat, &repl, &flags){ Ok(n)=>{ if n>0 { b.modified=true; } status=Some(format!("substitutions: {}", n)); last_pat=pat; last_repl=repl_raw; last_flags=flags; }, Err(e)=> status=Some(e) } } else { let (start,end)= if let Some(rs)=range_str { parse_range(&rs, lines.len(), cy).unwrap_or((cy+1,cy+1)) } else { (cy+1,cy+1) }; match substitute_lines(&mut lines, (start,end), &pat, &repl, &flags){ Ok(n)=>{ if n>0 { modified=true; } status=Some(format!("substitutions: {}", n)); last_pat=pat; last_repl=repl_raw; last_flags=flags; }, Err(e)=> status=Some(e) } } }
                                        else { let (start,end)= if let Some(rs)=range_str { parse_range(&rs, lines.len(), cy).unwrap_or((cy+1,cy+1)) } else { (cy+1,cy+1) }; match substitute_lines(&mut lines, (start,end), &pat, &repl, &flags){ Ok(n)=>{ if n>0 { modified=true; } status=Some(format!("substitutions: {}", n)); last_pat=pat; last_repl=repl_raw; last_flags=flags; }, Err(e)=> status=Some(e) } }
                                    }
                                }
                                else if cmd.starts_with("e!") || cmd.starts_with(":e!") {
                                    let parts: Vec<&str> = cmd.trim_start_matches(':').split_whitespace().collect();
                                    if parts.len() >= 2 {
                                        let p = PathBuf::from(parts[1]);
                                        let new_lines = open_file(&p);
                                        if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ b.lines = if new_lines.is_empty(){ vec![String::new()] } else { new_lines }; b.filename=Some(p); b.modified=false; } }
                                        else { lines = if new_lines.is_empty() { vec![String::new()] } else { new_lines }; filename=Some(p); modified=false; }
                                        cx = 0; cy = 0; scroll = 0; status = Some("reloaded".into());
                                    }
                                }
                                else if cmd.starts_with("e ") || cmd.starts_with(":e ") {
                                    if modified { status = Some("No write since last change (:e! to force)".into()); }
                                    else {
                                        let p = PathBuf::from(cmd.split_whitespace().nth(1).unwrap_or(""));
                                        if !p.as_os_str().is_empty() {
                                            let new_lines = open_file(&p);
                                            if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ b.lines = if new_lines.is_empty(){ vec![String::new()] } else { new_lines }; b.filename=Some(p); b.modified=false; } }
                                            else { lines = if new_lines.is_empty() { vec![String::new()] } else { new_lines }; filename=Some(p); modified=false; }
                                            cx = 0; cy = 0; scroll = 0; status = Some("edited".into());
                                        }
                                    }
                                }
                                else if cmd.starts_with("set ") || cmd.starts_with(":set ") {
                                    if let Some(pos) = cmd.find("ts=") { if let Ok(n) = cmd[pos+3..].trim().parse::<usize>() { tabstop = n.max(1); status = Some(format!("tabstop={}", tabstop)); } }
                                }
                                else if cmd == ":split" || cmd == "split" || cmd == ":sp" || cmd == "sp" {
                                    // add a new horizontal view (clone current)
                                    let k = views[cur_view].kind;
                                    views[cur_view] = View { kind: k, cx, cy, scroll, buf: views[cur_view].buf };
                                    views.push(View { kind: ViewKind::Normal, cx, cy, scroll, buf: views[cur_view].buf });
                                    cur_view = views.len() - 1;
                                    layout = SplitLayout::Horizontal;
                                    status = Some("split".into());
                                }
                                else if cmd == ":vsplit" || cmd == "vsplit" || cmd == ":vsp" || cmd == "vsp" {
                                    let k = views[cur_view].kind;
                                    views[cur_view] = View { kind: k, cx, cy, scroll, buf: views[cur_view].buf };
                                    views.push(View { kind: ViewKind::Normal, cx, cy, scroll, buf: views[cur_view].buf });
                                    cur_view = views.len() - 1;
                                    layout = SplitLayout::Vertical;
                                    status = Some("vsplit".into());
                                }
                                else if cmd == ":only" || cmd == "only" {
                                    if views.len() > 1 { let keep = views[cur_view].clone(); views.clear(); views.push(keep); cur_view = 0; status = Some("only".into()); }
                                }
                                else if cmd == ":close" || cmd == "close" {
                                    if views.len() > 1 { views.remove(cur_view); cur_view = 0; let v = views[cur_view].clone(); cx = v.cx; cy = v.cy; scroll = v.scroll; status = Some("closed".into()); } else { status = Some("cannot close last window".into()); }
                                }
                                else if cmd == ":wincmd w" || cmd == "wincmd w" {
                                    if !views.is_empty() { let k = views[cur_view].kind; let b = views[cur_view].buf; views[cur_view] = View { kind: k, cx, cy, scroll, buf: b }; cur_view = (cur_view + 1) % views.len(); let v = views[cur_view].clone(); cx = v.cx; cy = v.cy; scroll = v.scroll; }
                                }
                                else if cmd == "help" || cmd == ":help" {
                                    last_normal_view = cur_view;
                                    views.push(View { kind: ViewKind::Help, cx: 0, cy: 0, scroll: 0, buf: None });
                                    cur_view = views.len() - 1;
                                }
                                cmdline.clear(); mode = Mode::Normal;
                            }
                            ,(KeyCode::Backspace, _) => { cmdline.pop(); }
                            ,(KeyCode::Char(c), KeyModifiers::NONE) => { cmdline.push(c); }
                            ,(KeyCode::Char('c'), KeyModifiers::CONTROL) => { mode = Mode::Normal; cmdline.clear(); }
                            ,_ => {}
                        }
                        continue;
                    }

                    // Normalモードの数値カウントを先に処理
                    if matches!(mode, Mode::Normal) {
                        if let KeyCode::Char(ch) = code {
                            if ch.is_ascii_digit() {
                                if ch == '0' {
                                    if count.is_some() {
                                        let v = count.get_or_insert(0);
                                        *v = v.saturating_mul(10);
                                        continue;
                                    }
                                } else {
                                    let v = count.get_or_insert(0);
                                    *v = v.saturating_mul(10).saturating_add((ch as u8 - b'0') as usize);
                                    continue;
                                }
                            }
                        }
                    }

                    // まず 2キーオペレーター処理（dd/yy/cc）を先に判定
                    if matches!(mode, Mode::Normal) {
                        if let Some('d') = pending_op {
                            match code {
                                KeyCode::Char('d') => {
                                    // dd: 現在行を削除（ヤンクはLinewiseに入れる）
                                    let n = count.take().unwrap_or(1);
                                    if let Some(bi) = views[cur_view].buf {
                                        if let Some(b) = buffers.get_mut(bi) {
                                            undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy });
                                            let mut taken: Vec<String> = Vec::new();
                                            for _ in 0..n { if cy < b.lines.len() { taken.push(b.lines.remove(cy)); } }
                                            if taken.is_empty() { taken.push(String::new()); }
                                            clipboard = Some(Register::Linewise(taken));
                                            if cy >= b.lines.len() { if b.lines.is_empty() { b.lines.push(String::new()); } cy = b.lines.len().saturating_sub(1); }
                                            cx = cx.min(b.lines[cy].len());
                                            b.modified = true;
                                        }
                                    } else {
                                        undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy });
                                        let mut taken: Vec<String> = Vec::new();
                                        for _ in 0..n { if cy < lines.len() { taken.push(lines.remove(cy)); } }
                                        if taken.is_empty() { taken.push(String::new()); }
                                        clipboard = Some(Register::Linewise(taken));
                                        if cy >= lines.len() { if lines.is_empty() { lines.push(String::new()); } cy = lines.len().saturating_sub(1); }
                                        cx = cx.min(lines[cy].len());
                                        modified = true;
                                    }
                                    pending_op = None; op_anchor=None;
                                    continue;
                                }
                                KeyCode::Char('w') | KeyCode::Char('e') | KeyCode::Char('b') | KeyCode::Char('$') => {
                                    if let Some((ax, ay)) = op_anchor.take() {
                                        let n = count.take().unwrap_or(1);
                                        let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                                        let (tx, ty, include_target) = match code {
                                            KeyCode::Char('w') => { let (tx,ty)=motion_w(src, ax, ay, n); (tx,ty,false) }
                                            ,KeyCode::Char('e') => { let (tx,ty)=motion_e(src, ax, ay, n); (tx,ty,true) }
                                            ,KeyCode::Char('b') => { let (tx,ty)=motion_b(src, ax, ay, n); (tx,ty,true) }
                                            ,_ => { let (tx,ty) = (src[ay].len(), ay); (tx,ty,true) }
                                        };
                                        // 決定範囲（charwise）
                                        let (mut sx, mut sy, mut ex, mut ey) = if (ty < ay) || (ty == ay && tx <= ax) {
                                            // 後方方向
                                            let end = prev_char_pos(src, ax, ay).unwrap_or((ax, ay));
                                            (tx, ty, end.0, end.1)
                                        } else {
                                            // 前方方向
                                            let end = if include_target { (tx, ty) } else { prev_char_pos(src, tx, ty).unwrap_or((ax, ay)) };
                                            (ax, ay, end.0, end.1)
                                        };
                                        // 正規化: 範囲が無効なら何もしない
                                        if sy > ey || (sy == ey && sx > ex) { pending_op=None; continue; }
                                        // 実行
                                        if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){
                                            undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy });
                                            clipboard = Some(yank_selection(&b.lines, sx, sy, ex, ey, true));
                                            delete_selection(&mut b.lines, sx, sy, ex, ey, true);
                                            b.modified = true;
                                        } }
                                        else { undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy }); clipboard = Some(yank_selection(&lines, sx, sy, ex, ey, true)); delete_selection(&mut lines, sx, sy, ex, ey, true); modified = true; }
                                        cx = sx; cy = sy; pending_op=None; continue;
                                    }
                                }
                                _ => { pending_op = None; }
                            }
                        } else if let Some('y') = pending_op {
                            match code {
                                KeyCode::Char('y') => {
                                    // yy: 行ヤンク（変更なし）
                                    let n = count.take().unwrap_or(1);
                                    if let Some(bi) = views[cur_view].buf { if let Some(b) = buffers.get(bi) { let mut v=Vec::new(); for i in 0..n { let li=cy.saturating_add(i); if li < b.lines.len() { v.push(b.lines[li].clone()); } } if v.is_empty(){ v.push(String::new()); } clipboard = Some(Register::Linewise(v)); } }
                                    else { let mut v=Vec::new(); for i in 0..n { let li=cy.saturating_add(i); if li < lines.len() { v.push(lines[li].clone()); } } if v.is_empty(){ v.push(String::new()); } clipboard = Some(Register::Linewise(v)); }
                                    pending_op = None; op_anchor=None; continue;
                                }
                                KeyCode::Char('w') | KeyCode::Char('e') | KeyCode::Char('b') | KeyCode::Char('$') => {
                                    if let Some((ax, ay)) = op_anchor.take() {
                                        let n = count.take().unwrap_or(1);
                                        let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                                        let (tx, ty, include_target) = match code {
                                            KeyCode::Char('w') => { let (tx,ty)=motion_w(src, ax, ay, n); (tx,ty,false) }
                                            ,KeyCode::Char('e') => { let (tx,ty)=motion_e(src, ax, ay, n); (tx,ty,true) }
                                            ,KeyCode::Char('b') => { let (tx,ty)=motion_b(src, ax, ay, n); (tx,ty,true) }
                                            ,_ => { let (tx,ty) = (src[ay].len(), ay); (tx,ty,true) }
                                        };
                                        let (mut sx, mut sy, mut ex, mut ey) = if (ty < ay) || (ty == ay && tx <= ax) {
                                            let end = prev_char_pos(src, ax, ay).unwrap_or((ax, ay));
                                            (tx, ty, end.0, end.1)
                                        } else {
                                            let end = if include_target { (tx, ty) } else { prev_char_pos(src, tx, ty).unwrap_or((ax, ay)) };
                                            (ax, ay, end.0, end.1)
                                        };
                                        if sy > ey || (sy == ey && sx > ex) { pending_op=None; continue; }
                                        let reg = yank_selection(src, sx, sy, ex, ey, true);
                                        clipboard = Some(reg);
                                        pending_op=None; continue;
                                    }
                                }
                                _ => { pending_op = None; }
                            }
                        } else if let Some('c') = pending_op {
                            match code {
                                KeyCode::Char('c') => {
                                    // cc: 行変更（空行にしてInsert, 削除前をヤンク）
                                    let n = count.take().unwrap_or(1);
                                    if let Some(bi) = views[cur_view].buf { if let Some(b) = buffers.get_mut(bi) {
                                        undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy });
                                        let mut taken: Vec<String> = Vec::new();
                                        taken.push(std::mem::replace(&mut b.lines[cy], String::new()));
                                        for _ in 1..n { if cy + 1 < b.lines.len() { taken.push(b.lines.remove(cy + 1)); } }
                                        clipboard = Some(Register::Linewise(taken));
                                        b.modified = true;
                                    } }
                                    else {
                                        undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy });
                                        let mut taken: Vec<String> = Vec::new();
                                        taken.push(std::mem::replace(&mut lines[cy], String::new()));
                                        for _ in 1..n { if cy + 1 < lines.len() { taken.push(lines.remove(cy + 1)); } }
                                        clipboard = Some(Register::Linewise(taken));
                                        modified = true;
                                    }
                                    cx = 0; mode = Mode::Insert; pending_op = None; visual_anchor=None; op_anchor=None; continue;
                                }
                                KeyCode::Char('w') | KeyCode::Char('e') | KeyCode::Char('b') | KeyCode::Char('$') => {
                                    if let Some((ax, ay)) = op_anchor.take() {
                                        let n = count.take().unwrap_or(1);
                                        let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                                        // Vimのcwはce相当
                                        let (tx, ty, include_target) = match code {
                                            KeyCode::Char('w') => { let (tx,ty)=motion_e(src, ax, ay, n); (tx,ty,true) }
                                            ,KeyCode::Char('e') => { let (tx,ty)=motion_e(src, ax, ay, n); (tx,ty,true) }
                                            ,KeyCode::Char('b') => { let (tx,ty)=motion_b(src, ax, ay, n); (tx,ty,true) }
                                            ,_ => { let (tx,ty) = (src[ay].len(), ay); (tx,ty,true) }
                                        };
                                        let (mut sx, mut sy, mut ex, mut ey) = if (ty < ay) || (ty == ay && tx <= ax) {
                                            let end = prev_char_pos(src, ax, ay).unwrap_or((ax, ay));
                                            (tx, ty, end.0, end.1)
                                        } else {
                                            let end = if include_target { (tx, ty) } else { prev_char_pos(src, tx, ty).unwrap_or((ax, ay)) };
                                            (ax, ay, end.0, end.1)
                                        };
                                        if sy > ey || (sy == ey && sx > ex) { pending_op=None; continue; }
                                        if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){
                                            undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy });
                                            clipboard = Some(yank_selection(&b.lines, sx, sy, ex, ey, true));
                                            delete_selection(&mut b.lines, sx, sy, ex, ey, true);
                                            b.modified = true;
                                        } }
                                        else { undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy }); clipboard = Some(yank_selection(&lines, sx, sy, ex, ey, true)); delete_selection(&mut lines, sx, sy, ex, ey, true); modified = true; }
                                        cx = sx; cy = sy; mode = Mode::Insert; pending_op=None; continue;
                                    }
                                }
                                _ => { pending_op = None; }
                            }
                        }
                    }

                    // Normal/Insert modes
                    match (code, modifiers, mode) {
                        (KeyCode::Char(':'), _, Mode::Normal) => { mode = Mode::Command; cmdline.clear(); }
                        ,(KeyCode::Char('/'), _, Mode::Normal) => { mode = Mode::SearchFwd; cmdline.clear(); }
                        ,(KeyCode::Char('?'), _, Mode::Normal) => { mode = Mode::SearchBwd; cmdline.clear(); }
                        ,(KeyCode::Char('v'), _, Mode::Normal) => { count=None; if matches!(mode, Mode::VisualChar) { mode = Mode::Normal; visual_anchor=None; } else { mode = Mode::VisualChar; visual_anchor = Some((cx, cy)); } }
                        ,(KeyCode::Char('V'), _, Mode::Normal) => { count=None; if matches!(mode, Mode::VisualLine) { mode = Mode::Normal; visual_anchor=None; } else { mode = Mode::VisualLine; visual_anchor = Some((cx, cy)); } }
                        ,(KeyCode::Char('i'), _, Mode::Normal) => { count=None; insert_record.clear(); mode = Mode::Insert; }
                        ,(KeyCode::Char('a'), _, Mode::Normal) => { count=None; let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines }; if cx < src[cy].len() { cx += 1; } insert_record.clear(); mode = Mode::Insert; }
                        ,(KeyCode::Char('A'), _, Mode::Normal) => { count=None; let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines }; cx = src[cy].len(); insert_record.clear(); mode = Mode::Insert; }
                        ,(KeyCode::Char('I'), _, Mode::Normal) => { count=None; let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines }; let line = &src[cy]; let first_nb = line.chars().position(|ch| ch != ' ' && ch != '\t').unwrap_or(0); cx = first_nb; insert_record.clear(); mode = Mode::Insert; }
                        ,(KeyCode::Char('o'), _, Mode::Normal) => {
                            count=None; if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy }); b.lines.insert(cy+1, String::new()); cy += 1; cx = 0; b.modified = true; } }
                            else { undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy }); lines.insert(cy+1, String::new()); cy += 1; cx = 0; modified = true; }
                            insert_record.clear(); mode = Mode::Insert;
                        }
                        ,(KeyCode::Char('O'), _, Mode::Normal) => {
                            count=None; if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy }); b.lines.insert(cy, String::new()); cx = 0; b.modified = true; } }
                            else { undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy }); lines.insert(cy, String::new()); cx = 0; modified = true; }
                            insert_record.clear(); mode = Mode::Insert;
                        }
                        ,(KeyCode::Esc, _, Mode::Insert) => { mode = Mode::Normal; last_insert = insert_record.clone(); insert_record.clear(); }
                        ,(KeyCode::Char('h'), _, Mode::Normal) | (KeyCode::Left, _, _) => {
                            let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                            let n = count.take().unwrap_or(1);
                            for _ in 0..n { if cx>0 { cx-=1; } else if cy>0 { cy-=1; cx = src[cy].len(); } }
                        }
                        ,(KeyCode::Char('l'), _, Mode::Normal) | (KeyCode::Right, _, _) => {
                            let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                            let n = count.take().unwrap_or(1);
                            for _ in 0..n { if cx < src[cy].len() { cx+=1; } else if cy+1 < src.len() { cy+=1; cx=0; } }
                        }
                        ,(KeyCode::Char('k'), _, Mode::Normal) | (KeyCode::Up, _, _) => {
                            let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                            let n = count.take().unwrap_or(1);
                            for _ in 0..n { if cy>0 { cy-=1; cx = cx.min(src[cy].len()); } }
                        }
                        ,(KeyCode::Char('j'), _, Mode::Normal) | (KeyCode::Down, _, _) => {
                            let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                            let n = count.take().unwrap_or(1);
                            for _ in 0..n { if cy+1 < src.len() { cy+=1; cx = cx.min(src[cy].len()); } }
                        }
                        ,(KeyCode::Home, _, _) | (KeyCode::Char('0'), _, Mode::Normal) => { cx = 0; }
                        ,(KeyCode::End, _, _) | (KeyCode::Char('$'), _, Mode::Normal) => { let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines }; cx = src[cy].len(); }
                        ,(KeyCode::Char('G'), _, Mode::Normal) => {
                            let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                            if let Some(nv) = count.take() { cy = nv.saturating_sub(1).min(src.len().saturating_sub(1)); cx = 0; }
                            else { cy = src.len().saturating_sub(1); cx = 0; }
                        }
                        ,(KeyCode::Char('g'), _, Mode::Normal) => { cy = 0; cx = 0; }
                        ,(KeyCode::Char('x'), _, Mode::Normal) => {
                            // カーソル位置の1文字削除（カウント対応）
                            let n = count.take().unwrap_or(1);
                            if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ let len=b.lines[cy].len(); let to_del = n.min(len.saturating_sub(cx)); if to_del>0 { let snapshot=b.lines.clone(); undo_snap = Some(UndoSnap { buf: Some(bi), lines: snapshot, cx, cy }); for _ in 0..to_del { b.lines[cy].remove(cx); } b.modified=true; } } }
                            else { let len=lines[cy].len(); let to_del = n.min(len.saturating_sub(cx)); if to_del>0 { let snapshot=lines.clone(); undo_snap = Some(UndoSnap { buf: None, lines: snapshot, cx, cy }); for _ in 0..to_del { lines[cy].remove(cx); } modified=true; } }
                        }
                        ,(KeyCode::Char('J'), _, Mode::Normal) => {
                            // 次の行と結合（カウント対応）
                            let times = count.take().unwrap_or(1);
                            if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ if cy+1 < b.lines.len() { undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy }); for _ in 0..times { if cy+1 < b.lines.len() { let next=b.lines.remove(cy+1); b.lines[cy].push_str(&next); } } b.modified=true; } } }
                            else { if cy+1 < lines.len() { undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy }); for _ in 0..times { if cy+1 < lines.len() { let next=lines.remove(cy+1); lines[cy].push_str(&next); } } modified=true; } }
                        }
                        ,(KeyCode::Char('y'), _, Mode::Normal) => { pending_op = Some('y'); op_anchor = Some((cx, cy)); }
                        ,(KeyCode::Char('c'), _, Mode::Normal) => { pending_op = Some('c'); op_anchor = Some((cx, cy)); }
                        ,(KeyCode::Char('d'), _, Mode::Normal) => { pending_op = Some('d'); op_anchor = Some((cx, cy)); }
                        ,(KeyCode::Char('d'), _, m) if matches!(m, Mode::VisualChar | Mode::VisualLine) => {
                            if let Some((ax, ay)) = visual_anchor {
                                let (sx, sy, ex, ey) = ordered_region(ax, ay, cx, cy);
                                if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ delete_selection(&mut b.lines, sx, sy, ex, ey, matches!(m, Mode::VisualChar)); b.modified=true; } }
                                else { delete_selection(&mut lines, sx, sy, ex, ey, matches!(m, Mode::VisualChar)); modified = true; }
                                mode = Mode::Normal; visual_anchor=None; cx = sx; cy = sy;
                            }
                        }
                        ,(KeyCode::Char('y'), _, m) if matches!(m, Mode::VisualChar | Mode::VisualLine) => {
                            if let Some((ax, ay)) = visual_anchor {
                                let (sx, sy, ex, ey) = ordered_region(ax, ay, cx, cy);
                                let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                                clipboard = Some(yank_selection(src, sx, sy, ex, ey, matches!(m, Mode::VisualChar)));
                                mode = Mode::Normal; visual_anchor=None;
                            }
                        }
                        ,(KeyCode::Char('c'), _, m) if matches!(m, Mode::VisualChar | Mode::VisualLine) => {
                            if let Some((ax, ay)) = visual_anchor {
                                let (sx, sy, ex, ey) = ordered_region(ax, ay, cx, cy);
                                let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines };
                                clipboard = Some(yank_selection(src, sx, sy, ex, ey, matches!(m, Mode::VisualChar)));
                                if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ delete_selection(&mut b.lines, sx, sy, ex, ey, matches!(m, Mode::VisualChar)); b.modified=true; } }
                                else { delete_selection(&mut lines, sx, sy, ex, ey, matches!(m, Mode::VisualChar)); modified=true; }
                                mode = Mode::Insert; visual_anchor=None; cx = sx; cy = sy;
                            }
                        }
                        ,(KeyCode::Char('p'), _, Mode::Normal) => {
                            let times = count.take().unwrap_or(1);
                            if let Some(orig) = clipboard.clone() {
                                if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy }); for _ in 0..times { let reg = orig.clone(); match reg { Register::Charwise(s)=>{ for ch in s.chars(){ b.lines[cy].insert(cx, ch); cx+=1; } }, Register::Linewise(mut ls)=>{ let insert_at=cy+1; for (i,l) in ls.drain(..).enumerate(){ b.lines.insert(insert_at+i, l);} cy=insert_at; cx=0; } } } b.modified=true; } }
                                else { undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy }); for _ in 0..times { let reg = orig.clone(); match reg { Register::Charwise(s)=>{ for ch in s.chars(){ lines[cy].insert(cx, ch); cx+=1; } }, Register::Linewise(mut ls)=>{ let insert_at=cy+1; for (i,l) in ls.drain(..).enumerate(){ lines.insert(insert_at+i, l);} cy=insert_at; cx=0; } } } modified=true; }
                            }
                        }
                        ,(KeyCode::Char('P'), _, Mode::Normal) => {
                            let times = count.take().unwrap_or(1);
                            if let Some(orig) = clipboard.clone() {
                                if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy }); for _ in 0..times { let reg = orig.clone(); match reg { Register::Charwise(s)=>{ let mut off=0; for ch in s.chars(){ b.lines[cy].insert(cx.saturating_sub(off+1), ch); off+=1; } }, Register::Linewise(mut ls)=>{ let insert_at=cy; for (i,l) in ls.drain(..).enumerate(){ b.lines.insert(insert_at+i, l);} cy=insert_at; cx=0; } } } b.modified=true; } }
                                else { undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy }); for _ in 0..times { let reg = orig.clone(); match reg { Register::Charwise(s)=>{ let mut off=0; for ch in s.chars(){ lines[cy].insert(cx.saturating_sub(off+1), ch); off+=1; } }, Register::Linewise(mut ls)=>{ let insert_at=cy; for (i,l) in ls.drain(..).enumerate(){ lines.insert(insert_at+i, l);} cy=insert_at; cx=0; } } } modified=true; }
                            }
                        }
                        ,(KeyCode::Char('D'), _, Mode::Normal) => {
                            // 行末まで削除（d$ 相当）
                            if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ if cx < b.lines[cy].len() { undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy }); b.lines[cy].replace_range(cx.., ""); b.modified = true; } } }
                            else { if cx < lines[cy].len() { undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy }); lines[cy].replace_range(cx.., ""); modified = true; } }
                        }
                        ,(KeyCode::Char('Y'), _, Mode::Normal) => {
                            // 行ヤンク（yy 相当、カウント対応）
                            let n = count.take().unwrap_or(1);
                            if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get(bi){ let mut v=Vec::new(); for i in 0..n { let li = cy.saturating_add(i); if li < b.lines.len() { v.push(b.lines[li].clone()); } } if v.is_empty(){ v.push(String::new()); } clipboard = Some(Register::Linewise(v)); } }
                            else { let mut v=Vec::new(); for i in 0..n { let li = cy.saturating_add(i); if li < lines.len() { v.push(lines[li].clone()); } } if v.is_empty(){ v.push(String::new()); } clipboard = Some(Register::Linewise(v)); }
                        }
                        ,(KeyCode::Char('u'), _, Mode::Normal) => {
                            if let Some(snap) = undo_snap.clone() {
                                if let Some(bi) = snap.buf { if let Some(b) = buffers.get_mut(bi) { b.lines = snap.lines; cx = snap.cx; cy = snap.cy; b.modified = true; } }
                                else { lines = snap.lines; cx = snap.cx; cy = snap.cy; modified = true; }
                            }
                        }
                        ,(KeyCode::Enter, _, Mode::Insert) => {
                            if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ let cur = b.lines[cy].clone(); let (l,r)=cur.split_at(cx); b.lines[cy]=l.to_string(); b.lines.insert(cy+1, r.to_string()); cy+=1; cx=0; b.modified=true; } }
                            else { let cur = lines[cy].clone(); let (l,r)=cur.split_at(cx); lines[cy]=l.to_string(); lines.insert(cy+1, r.to_string()); cy+=1; cx=0; modified=true; }
                            insert_record.push('\n');
                        }
                        ,(KeyCode::Enter, _, Mode::Normal) => {
                            // If buffers list is active, select and switch
                            if !views.is_empty() && matches!(views[cur_view].kind, ViewKind::BuffersList) {
                                let sel_row = views[cur_view].cy.saturating_sub(1); // row 0 is header
                                if sel_row < buffers.len() {
                                    // 割当: 呼び出し元のビューに選択バッファを割当
                                    let caller = last_normal_view.min(views.len().saturating_sub(1));
                                    if caller < views.len() { views[caller].buf = Some(sel_row); }
                                    // 一覧ビューを閉じて呼出元へ戻る
                                    let list_idx = cur_view;
                                    views.remove(list_idx);
                                    cur_view = caller.min(views.len().saturating_sub(1));
                                    status = Some("buffer switched".into());
                                }
                            }
                        }
                        ,(KeyCode::Char('q'), _, Mode::Normal) => {
                            if !views.is_empty() && matches!(views[cur_view].kind, ViewKind::BuffersList) {
                                views.remove(cur_view); cur_view = 0;
                            }
                        }
                        ,(KeyCode::Backspace, _, Mode::Insert) => {
                            if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ if cx>0 { b.lines[cy].remove(cx-1); cx-=1; b.modified=true; if !insert_record.is_empty() { insert_record.pop(); } } else if cy>0 { let prev_len=b.lines[cy-1].len(); let line=b.lines.remove(cy); b.lines[cy-1].push_str(&line); cy-=1; cx=prev_len; b.modified=true; } } }
                            else { if cx>0 { lines[cy].remove(cx-1); cx-=1; modified=true; if !insert_record.is_empty() { insert_record.pop(); } } else if cy>0 { let prev_len=lines[cy-1].len(); let line=lines.remove(cy); lines[cy-1].push_str(&line); cy-=1; cx=prev_len; modified=true; } }
                        }
                        ,(KeyCode::Char(c), KeyModifiers::NONE, Mode::Insert) => {
                            if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ if c=='\t' { let spaces = " ".repeat(tabstop); for ch in spaces.chars(){ b.lines[cy].insert(cx, ch); cx+=1; insert_record.push(' ');} } else { b.lines[cy].insert(cx, c); cx+=1; insert_record.push(c); } b.modified=true; } }
                            else { if c=='\t' { let spaces = " ".repeat(tabstop); for ch in spaces.chars(){ lines[cy].insert(cx, ch); cx+=1; insert_record.push(' ');} } else { lines[cy].insert(cx, c); cx+=1; insert_record.push(c); } modified=true; }
                        }
                        ,(KeyCode::Char('.'), _, Mode::Normal) => {
                            if !last_insert.is_empty() {
                                if let Some(bi)=views[cur_view].buf { if let Some(b)=buffers.get_mut(bi){ undo_snap = Some(UndoSnap { buf: Some(bi), lines: b.lines.clone(), cx, cy }); for ch in last_insert.chars(){ if ch=='\n' { let cur=b.lines[cy].clone(); let (l,r)=cur.split_at(cx); b.lines[cy]=l.to_string(); b.lines.insert(cy+1, r.to_string()); cy+=1; cx=0; } else { b.lines[cy].insert(cx, ch); cx+=1; } } b.modified=true; } }
                                else { undo_snap = Some(UndoSnap { buf: None, lines: lines.clone(), cx, cy }); for ch in last_insert.chars(){ if ch=='\n' { let cur=lines[cy].clone(); let (l,r)=cur.split_at(cx); lines[cy]=l.to_string(); lines.insert(cy+1, r.to_string()); cy+=1; cx=0; } else { lines[cy].insert(cx, ch); cx+=1; } } modified=true; }
                            }
                        }
                        ,(KeyCode::Char('s'), KeyModifiers::CONTROL, _) => {
                            // save active buffer or global
                            let active_bi = views[cur_view].buf;
                            if let Some(bi) = active_bi {
                                if let Some(b) = buffers.get_mut(bi) {
                                    if let Some(ref p) = b.filename { if save_file(p, &b.lines).is_ok() { b.modified = false; status = Some("written".into()); } else { status = Some("write error".into()); } }
                                    else { status = Some("No file name".into()); }
                                }
                            } else {
                                if let Some(ref p) = filename { if save_file(p, &lines).is_ok() { modified = false; status = Some("written".into()); } else { status = Some("write error".into()); } } else { status = Some("No file name".into()); }
                            }
                        }
                        ,(KeyCode::Char('n'), _, Mode::Normal) => { if let Some(_) = &search.regex { let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines }; if let Some((ny,nx))=find_next(src, cy, cx, &search, search.last_dir){ cy=ny; cx=nx; } } }
                        ,(KeyCode::Char('N'), _, Mode::Normal) => { if let Some(_) = &search.regex { let src = if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines }; if let Some((ny,nx))=find_next(src, cy, cx, &search, -search.last_dir){ cy=ny; cx=nx; } } }
                        ,(KeyCode::Char('w'), KeyModifiers::CONTROL, Mode::Normal) => { if !views.is_empty() { let k = views[cur_view].kind; let b = views[cur_view].buf; views[cur_view] = View { kind: k, cx, cy, scroll, buf: b }; cur_view = (cur_view + 1) % views.len(); let v = views[cur_view].clone(); cx = v.cx; cy = v.cy; scroll = v.scroll; } }
                        ,(KeyCode::Char('w'), KeyModifiers::NONE, Mode::Normal) => { let n=count.take().unwrap_or(1); let src= if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines }; let (nx,ny)=motion_w(src,cx,cy,n); cx=nx; cy=ny; }
                        ,(KeyCode::Char('e'), KeyModifiers::NONE, Mode::Normal) => { let n=count.take().unwrap_or(1); let src= if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines }; let (nx,ny)=motion_e(src,cx,cy,n); cx=nx; cy=ny; }
                        ,(KeyCode::Char('b'), KeyModifiers::NONE, Mode::Normal) => { let n=count.take().unwrap_or(1); let src= if let Some(bi)=views[cur_view].buf { buffers.get(bi).map(|b| &b.lines).unwrap_or(&lines) } else { &lines }; let (nx,ny)=motion_b(src,cx,cy,n); cx=nx; cy=ny; }
                        ,(KeyCode::Char('q'), KeyModifiers::CONTROL, _) => { if modified { status = Some("No write since last change (:q! to quit)".into()); } else { break; } }
                        ,_ => {}
                    }
                }
                _ => {}
            }
        }
    }

    // restore terminal
    terminal::disable_raw_mode().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    let mut stdout = std::io::stdout();
    execute!(stdout, crossterm::cursor::Show, crossterm::terminal::LeaveAlternateScreen).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
    Ok(())
}

fn find_next(lines: &Vec<String>, cy: usize, cx: usize, search: &SearchState, dir: i32) -> Option<(usize, usize)> {
    let re = search.regex.as_ref()?;
    if dir >= 0 {
        // forward: start from current line at cx+ (skip current position)
        let mut y = cy;
        let mut x = cx + 1;
        loop {
            if y >= lines.len() { return None; }
            let line = &lines[y];
            if x <= line.len() {
                if let Some(m) = re.find_at(line, x) { return Some((y, m.start())); }
            }
            y += 1; x = 0;
        }
    } else {
        // backward: search up to cx-1 in current line, then previous lines
        let mut y = cy;
        let mut x_end = cx.saturating_sub(1);
        loop {
            let line = &lines[y];
            let mut last_pos: Option<usize> = None;
            for m in re.find_iter(line) {
                if m.start() <= x_end { last_pos = Some(m.start()); } else { break; }
            }
            if let Some(pos) = last_pos { return Some((y, pos)); }
            if y == 0 { return None; }
            y -= 1; x_end = lines[y].len();
        }
    }
}

// helpers ported from ANSI mode
fn ordered_region(ax: usize, ay: usize, bx: usize, by: usize) -> (usize, usize, usize, usize) {
    if (by < ay) || (by == ay && bx < ax) { (bx, by, ax, ay) } else { (ax, ay, bx, by) }
}

fn yank_selection(lines: &Vec<String>, sx: usize, sy: usize, ex: usize, ey: usize, charwise: bool) -> Register {
    if sy == ey {
        if charwise {
            let line = &lines[sy];
            let start = sx.min(line.len());
            let end = (ex + 1).min(line.len());
            let s = if start <= end { line[start..end].to_string() } else { String::new() };
            Register::Charwise(s)
        } else {
            Register::Linewise(vec![lines[sy].clone()])
        }
    } else {
        if charwise {
            let mut out = String::new();
            let first = &lines[sy];
            let start = sx.min(first.len());
            out.push_str(&first[start..]);
            out.push('\n');
            for i in (sy + 1)..ey { out.push_str(&lines[i]); out.push('\n'); }
            let last = &lines[ey];
            let end = (ex + 1).min(last.len());
            out.push_str(&last[..end]);
            Register::Charwise(out)
        } else {
            let mut out = Vec::new();
            for i in sy..=ey { out.push(lines[i].clone()); }
            Register::Linewise(out)
        }
    }
}

fn delete_selection(lines: &mut Vec<String>, sx: usize, sy: usize, ex: usize, ey: usize, charwise: bool) {
    if sy == ey {
        if charwise {
            let line = &mut lines[sy];
            let start = sx.min(line.len());
            let end = (ex + 1).min(line.len());
            if start < end { line.replace_range(start..end, ""); }
        } else {
            lines.remove(sy);
            if lines.is_empty() { lines.push(String::new()); }
        }
    } else {
        if charwise {
            let first_tail = {
                let first = &lines[sy];
                let start = sx.min(first.len());
                first[..start].to_string()
            };
            let last_head = {
                let last = &lines[ey];
                let end = (ex + 1).min(last.len());
                last[end..].to_string()
            };
            for _ in (sy + 1)..=ey { lines.remove(sy + 1); }
            lines[sy] = first_tail + &last_head;
        } else {
            for _ in sy..=ey { lines.remove(sy); }
            if lines.is_empty() { lines.push(String::new()); }
        }
    }
}

fn convert_repl_to_rust(repl: &str, last_repl: &str) -> String {
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
                if nc.is_ascii_digit() { out.push('$'); out.push(nc); let _ = chars.next(); continue; }
                else { escape = true; continue; }
            } else { out.push(ch); continue; }
        } else if ch == '&' { out.push('$'); out.push('0'); continue; }
        else if ch == '~' { out.push_str(last_repl); continue; }
        out.push(ch);
    }
    out
}

fn substitute_lines(lines: &mut Vec<String>, range: (usize, usize), pat: &str, repl: &str, flags: &str) -> Result<usize, String> {
    let mut builder = RegexBuilder::new(pat);
    if flags.contains('i') { builder.case_insensitive(true); }
    let re = builder.build().map_err(|e| format!("regex error: {}", e))?;
    let start = range.0.saturating_sub(1);
    let end = range.1.saturating_sub(1).min(lines.len().saturating_sub(1));
    let mut total = 0usize;
    let global = flags.contains('g');
    for i in start..=end {
        let count = if global { re.find_iter(&lines[i]).count() } else { if re.is_match(&lines[i]) { 1 } else { 0 } };
        if count > 0 {
            let new_line = if global { re.replace_all(&lines[i], repl).to_string() } else { re.replace(&lines[i], repl).to_string() };
            lines[i] = new_line; total += count;
        }
    }
    Ok(total)
}

fn parse_substitute(cmd: &str) -> Option<(Option<String>, String, String, String)> {
    let c = cmd.trim_start_matches(':').trim();
    // collect possible range prefix until 's'
    let mut idx = 0usize; let bytes = c.as_bytes();
    while idx < bytes.len() {
        let ch = bytes[idx] as char; if ch == 's' { break; }
        if !(ch == '%' || ch == '.' || ch == '$' || ch == ',' || ch == '+' || ch == '-' || ch.is_ascii_digit() || ch.is_whitespace()) { return None; }
        idx += 1;
    }
    if idx >= bytes.len() || bytes[idx] as char != 's' { return None; }
    let range_str = if idx == 0 { None } else { Some(c[..idx].trim().to_string()) };
    let mut i = idx + 1; if i >= c.len() { return None; }
    let sep = c.as_bytes()[i] as char; if sep.is_ascii_whitespace() { return None; } i += 1;
    let mut collect = |i: &mut usize| {
        let mut out = String::new(); let mut esc = false; while *i < c.len() { let ch = c.as_bytes()[*i] as char; *i += 1; if esc { out.push(ch); esc = false; continue; } if ch == '\\' { esc = true; continue; } if ch == sep { break; } out.push(ch); } out
    };
    let pat = collect(&mut i); if i >= c.len() { return None; }
    let repl = collect(&mut i);
    let flags = c[i..].trim().to_string();
    Some((range_str, pat, repl, flags))
}

fn parse_line_ref(tok: &str, max: usize, current: usize) -> Option<usize> {
    let t = tok.trim(); if t.is_empty() { return None; }
    let (mut base, mut rest) = if t.starts_with('.') { (current + 1, &t[1..]) }
        else if t.starts_with('$') { (max.max(1), &t[1..]) }
        else if let Ok(n) = t.parse::<usize>() { (n, "") } else { return None };
    let mut i = 0usize; while i < rest.len() { let sign = rest.as_bytes()[i] as char; if sign != '+' && sign != '-' { break; } i+=1; let mut j = i; while j < rest.len() && rest.as_bytes()[j].is_ascii_digit() { j+=1; } if i==j { break; } let n: usize = rest[i..j].parse().ok()?; if sign=='+' { base = base.saturating_add(n); } else { base = base.saturating_sub(n); } i=j; }
    Some(base.clamp(1, max.max(1)))
}

fn parse_range(arg: &str, max: usize, current: usize) -> Option<(usize, usize)> {
    let s = arg.trim(); if s.is_empty() { return None; }
    if s == "%" { return Some((1, max.max(1))); }
    if let Some((a,b)) = s.split_once(',') { let start = parse_line_ref(a, max, current)?; let end = parse_line_ref(b, max, current)?; Some((start.min(end), start.max(end))) } else { let n = parse_line_ref(s, max, current)?; Some((n,n)) }
}
