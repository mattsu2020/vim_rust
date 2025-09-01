use clap::{Arg, Command};
use libc::{c_char, c_int};
use std::ffi::CString;

extern "C" {
    fn xxd_main(argc: c_int, argv: *const *const c_char) -> c_int;
}

fn main() {
    // Define CLI with clap but allow any args to be forwarded to C implementation.
    let _ = Command::new("xxd")
        .arg(Arg::new("args").num_args(0..).allow_hyphen_values(true).trailing_var_arg(true))
        .disable_help_flag(true)
        .allow_external_subcommands(true)
        .get_matches_from(vec!["xxd"]);

    let args: Vec<String> = std::env::args().collect();
    let mut c_strings: Vec<CString> = args
        .iter()
        .map(|arg| CString::new(arg.as_str()).unwrap())
        .collect();
    let mut ptrs: Vec<*const c_char> = c_strings.iter().map(|s| s.as_ptr()).collect();
    ptrs.push(std::ptr::null());
    let ret = unsafe { xxd_main((c_strings.len()) as c_int, ptrs.as_ptr()) };
    std::process::exit(ret);
}
