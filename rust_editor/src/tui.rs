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
enum Mode { Normal, Insert, Command, SearchFwd, SearchBwd }

struct SearchState {
    regex: Option<Regex>,
    pattern: String,
    case_insensitive: bool,
    last_dir: i32, // 1: forward, -1: backward
}

impl SearchState {
    fn new() -> Self { Self { regex: None, pattern: String::new(), case_insensitive: false, last_dir: 1 } }
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
    let mut status: Option<String> = None;
    let mut mode = Mode::Normal;
    let mut cmdline: String = String::new(); // used for :cmd and /search
    let mut tabstop: usize = 4;
    let mut search = SearchState::new();

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

            // adjust scroll for cursor visibility
            let view_rows = content_rows as usize;
            if cy < scroll { scroll = cy; }
            if cy >= scroll + view_rows { scroll = cy + 1 - view_rows; }

            // compose visible text with optional search highlight
            let mut text = Text::default();
            let hl_style = Style::default().add_modifier(Modifier::REVERSED);
            for i in 0..view_rows {
                let li = scroll + i;
                if li < lines.len() {
                    if let Some(re) = &search.regex {
                        let line = &lines[li];
                        let mut spans: Vec<Span> = Vec::new();
                        let mut last = 0usize;
                        for m in re.find(line) {
                            let s = m.start();
                            let e = m.end();
                            if s > last { spans.push(Span::raw(line[last..s].to_string())); }
                            spans.push(Span::styled(line[s..e].to_string(), hl_style));
                            last = e;
                        }
                        if last < line.len() { spans.push(Span::raw(line[last..].to_string())); }
                        text.lines.push(Line::from(spans));
                    } else {
                        text.lines.push(Line::from(lines[li].clone()));
                    }
                } else {
                    text.lines.push(Line::from("~"));
                }
            }
            let content = Paragraph::new(text)
                .block(Block::default().borders(Borders::NONE));
            f.render_widget(content, chunks[0]);

            // status
            let name = filename.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|| "[No Name]".to_string());
            let m = if modified { " [+]" } else { "" };
            let mode_tag = match mode { Mode::Normal => "[N]", Mode::Insert => "[I]", Mode::Command => ":", Mode::SearchFwd => "/", Mode::SearchBwd => "?" };
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
                // place cursor in content
                let Rect { x, y, .. } = chunks[0];
                let cur_y = y + (cy - scroll) as u16;
                let cur_x = x + (cx as u16);
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
                                    let mut pat = raw.as_str();
                                    let mut case_insensitive = false;
                                    // support \c for ignore case
                                    if pat.contains("\\c") { case_insensitive = true; }
                                    let pat_clean = pat.replace("\\c", "");
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
                                    if parts.len() >= 2 { filename = Some(PathBuf::from(parts[1])); }
                                    if let Some(ref p) = filename { match save_file(p, &lines) { Ok(_) => { modified = false; status = Some("written".into()); }, Err(_) => status = Some("write error".into()) } } else { status = Some("No file name".into()); }
                                }
                                else if cmd.starts_with("e!") || cmd.starts_with(":e!") {
                                    let parts: Vec<&str> = cmd.trim_start_matches(':').split_whitespace().collect();
                                    if parts.len() >= 2 {
                                        let p = PathBuf::from(parts[1]);
                                        let new_lines = open_file(&p);
                                        lines = if new_lines.is_empty() { vec![String::new()] } else { new_lines };
                                        filename = Some(p); cx = 0; cy = 0; scroll = 0; modified = false; status = Some("reloaded".into());
                                    }
                                }
                                else if cmd.starts_with("e ") || cmd.starts_with(":e ") {
                                    if modified { status = Some("No write since last change (:e! to force)".into()); }
                                    else {
                                        let p = PathBuf::from(cmd.split_whitespace().nth(1).unwrap_or(""));
                                        if !p.as_os_str().is_empty() {
                                            let new_lines = open_file(&p);
                                            lines = if new_lines.is_empty() { vec![String::new()] } else { new_lines };
                                            filename = Some(p); cx = 0; cy = 0; scroll = 0; modified = false; status = Some("edited".into());
                                        }
                                    }
                                }
                                else if cmd.starts_with("set ") || cmd.starts_with(":set ") {
                                    if let Some(pos) = cmd.find("ts=") { if let Ok(n) = cmd[pos+3..].trim().parse::<usize>() { tabstop = n.max(1); status = Some(format!("tabstop={}", tabstop)); } }
                                }
                                else if cmd == "help" || cmd == ":help" {
                                    status = Some("Use h j k l, i/ESC, :w :q".into());
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

                    // Normal/Insert modes
                    match (code, modifiers, mode) {
                        (KeyCode::Char(':'), _, Mode::Normal) => { mode = Mode::Command; cmdline.clear(); }
                        ,(KeyCode::Char('/'), _, Mode::Normal) => { mode = Mode::SearchFwd; cmdline.clear(); }
                        ,(KeyCode::Char('?'), _, Mode::Normal) => { mode = Mode::SearchBwd; cmdline.clear(); }
                        ,(KeyCode::Char('i'), _, Mode::Normal) => { mode = Mode::Insert; }
                        ,(KeyCode::Esc, _, Mode::Insert) => { mode = Mode::Normal; }
                        ,(KeyCode::Char('h'), _, Mode::Normal) | (KeyCode::Left, _, _) => { if cx>0 { cx-=1; } else if cy>0 { cy-=1; cx = lines[cy].len(); } }
                        ,(KeyCode::Char('l'), _, Mode::Normal) | (KeyCode::Right, _, _) => { if cx < lines[cy].len() { cx+=1; } else if cy+1 < lines.len() { cy+=1; cx=0; } }
                        ,(KeyCode::Char('k'), _, Mode::Normal) | (KeyCode::Up, _, _) => { if cy>0 { cy-=1; cx = cx.min(lines[cy].len()); } }
                        ,(KeyCode::Char('j'), _, Mode::Normal) | (KeyCode::Down, _, _) => { if cy+1 < lines.len() { cy+=1; cx = cx.min(lines[cy].len()); } }
                        ,(KeyCode::Home, _, _) | (KeyCode::Char('0'), _, Mode::Normal) => { cx = 0; }
                        ,(KeyCode::End, _, _) | (KeyCode::Char('$'), _, Mode::Normal) => { cx = lines[cy].len(); }
                        ,(KeyCode::Char('G'), _, Mode::Normal) => { cy = lines.len().saturating_sub(1); cx = 0; }
                        ,(KeyCode::Char('g'), _, Mode::Normal) => { cy = 0; cx = 0; }
                        ,(KeyCode::Enter, _, Mode::Insert) => { let cur = lines[cy].clone(); let (l, r) = cur.split_at(cx); lines[cy] = l.to_string(); lines.insert(cy+1, r.to_string()); cy+=1; cx=0; modified = true; }
                        ,(KeyCode::Backspace, _, Mode::Insert) => { if cx>0 { lines[cy].remove(cx-1); cx-=1; modified = true; } else if cy>0 { let prev_len = lines[cy-1].len(); let line = lines.remove(cy); lines[cy-1].push_str(&line); cy-=1; cx=prev_len; modified=true; } }
                        ,(KeyCode::Char(c), KeyModifiers::NONE, Mode::Insert) => { if c == '\t' { let spaces = " ".repeat(tabstop); for ch in spaces.chars() { lines[cy].insert(cx, ch); cx+=1; } } else { lines[cy].insert(cx, c); cx+=1; } modified = true; }
                        ,(KeyCode::Char('s'), KeyModifiers::CONTROL, _) => { if let Some(ref p) = filename { if save_file(p, &lines).is_ok() { modified = false; status = Some("written".into()); } else { status = Some("write error".into()); } } else { status = Some("No file name".into()); } }
                        ,(KeyCode::Char('n'), _, Mode::Normal) => { if let Some(_) = &search.regex { if let Some((ny, nx)) = find_next(&lines, cy, cx, &search, search.last_dir) { cy = ny; cx = nx; } } }
                        ,(KeyCode::Char('N'), _, Mode::Normal) => { if let Some(_) = &search.regex { if let Some((ny, nx)) = find_next(&lines, cy, cx, &search, -search.last_dir) { cy = ny; cx = nx; } } }
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
