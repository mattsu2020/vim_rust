use std::thread;
use std::time::{Duration, Instant};

#[test]
fn short_sleep_last_at_least_100ms() {
    let start = Instant::now();
    thread::sleep(Duration::from_millis(100));
    assert!(start.elapsed() >= Duration::from_millis(100));
}
