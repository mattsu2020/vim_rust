use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::sync::{Mutex, OnceLock};

static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

fn with_logfile<F>(f: F) -> io::Result<()>
where
    F: FnOnce(&mut File) -> io::Result<()>,
{
    if let Some(m) = LOG_FILE.get() {
        if let Some(ref mut file) = *m.lock().unwrap() {
            return f(file);
        }
    }
    Ok(())
}

pub fn ch_logfile(path: &str, opt: &str) -> io::Result<()> {
    let mut file = if opt.contains('w') && !opt.contains('a') {
        File::create(path)?
    } else {
        OpenOptions::new().create(true).append(true).open(path)?
    };
    let ts = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(dur) => dur.as_secs().to_string(),
        Err(_) => String::from("time_error"),
    };
    writeln!(file, "==== start log session {} ====", ts)?;
    file.flush()?;
    let mutex = LOG_FILE.get_or_init(|| Mutex::new(None));
    *mutex.lock().unwrap() = Some(file);
    Ok(())
}

pub fn ch_log_active() -> bool {
    LOG_FILE
        .get()
        .and_then(|m| m.lock().ok())
        .map(|guard| guard.is_some())
        .unwrap_or(false)
}

pub fn ch_log(message: &str) -> io::Result<()> {
    with_logfile(|f| {
        writeln!(f, "{}", message)?;
        f.flush()
    })
}

pub fn ch_error(message: &str) -> io::Result<()> {
    with_logfile(|f| {
        writeln!(f, "ERR {}", message)?;
        f.flush()
    })
}

pub fn ch_log_literal(lead: &str, buf: &[u8]) -> io::Result<()> {
    with_logfile(|f| {
        write!(f, "{}'", lead)?;
        f.write_all(buf)?;
        writeln!(f, "'")?;
        f.flush()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn logging_works() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path().to_path_buf();
        ch_logfile(path.to_str().unwrap(), "").unwrap();
        assert!(ch_log_active());

        ch_log("hello").unwrap();
        ch_error("oops").unwrap();
        ch_log_literal("LIT ", b"text").unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(content.contains("hello"));
        assert!(content.contains("ERR oops"));
        assert!(content.contains("LIT 'text'"));
    }
}
