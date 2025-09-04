use rust_evalbuffer::{buflist_find_by_name, buflist_get_line};
use std::env;
use std::fs::{self, File};
use std::io::Write;

#[test]
fn loads_and_caches_buffer_and_get_line() {
    let mut path = env::temp_dir();
    path.push("evalbuffer_test.txt");
    let mut file = File::create(&path).unwrap();
    write!(file, "foo\nbar\n").unwrap();

    let path_str = path.to_str().unwrap();
    let buf = buflist_find_by_name(path_str).expect("buffer");
    assert_eq!(buf.lines, vec!["foo".to_string(), "bar".to_string()]);

    // Second call should use cached buffer.
    let cached = buflist_find_by_name(path_str).unwrap();
    assert_eq!(cached.lines, buf.lines);

    // Retrieve line via helper
    let line2 = buflist_get_line(path_str, 2).unwrap();
    assert_eq!(line2, "bar");

    fs::remove_file(path).unwrap();
}
