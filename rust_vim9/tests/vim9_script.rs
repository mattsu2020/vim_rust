use rust_vim9::execute_script;

#[test]
fn runs_simple_script() {
    let script = "1 + 2\n3 + 4";
    let result = execute_script(script);
    assert_eq!(result, vec![3, 7]);
}
