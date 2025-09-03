use std::fmt::Debug;

#[derive(Debug, PartialEq)]
pub struct TestError(pub String);

pub type TestResult = Result<(), TestError>;

fn make_err(msg: Option<&str>, default: &str) -> TestError {
    TestError(msg.unwrap_or(default).to_string())
}

pub fn assert_true(cond: bool, msg: Option<&str>) -> TestResult {
    if cond {
        Ok(())
    } else {
        Err(make_err(msg, "assert_true failed"))
    }
}

pub fn assert_false(cond: bool, msg: Option<&str>) -> TestResult {
    assert_true(!cond, msg)
}

pub fn assert_equal<T: PartialEq + Debug>(expected: T, actual: T, msg: Option<&str>) -> TestResult {
    if expected == actual {
        Ok(())
    } else {
        Err(make_err(msg, &format!("left: {:?} right: {:?}", expected, actual)))
    }
}

pub fn assert_notequal<T: PartialEq + Debug>(expected: T, actual: T, msg: Option<&str>) -> TestResult {
    if expected != actual {
        Ok(())
    } else {
        Err(make_err(msg, "values are equal"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asserts_work() {
        assert_true(true, None).unwrap();
        assert_false(false, None).unwrap();
        assert_equal(1, 1, None).unwrap();
        assert_notequal(1, 2, None).unwrap();

        assert!(assert_true(false, None).is_err());
        assert!(assert_false(true, None).is_err());
        assert!(assert_equal(1, 2, None).is_err());
        assert!(assert_notequal(1, 1, None).is_err());
    }
}
