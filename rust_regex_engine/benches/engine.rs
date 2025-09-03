use criterion::{criterion_group, criterion_main, Criterion};
use std::ffi::CString;
use rust_regex_engine::{vim_regcomp, vim_regexec, vim_regfree, RegMatch};

fn bench_regexec(c: &mut Criterion) {
    let pat = CString::new("foo.*bar").unwrap();
    let text = CString::new("foo something bar").unwrap();
    let prog = vim_regcomp(pat.as_ptr(), 0);
    assert!(!prog.is_null());
    c.bench_function("vim_regexec", |b| {
        b.iter(|| {
            let mut rm = RegMatch {
                regprog: prog,
                startp: [std::ptr::null(); 10],
                endp: [std::ptr::null(); 10],
                rm_matchcol: 0,
                rm_ic: 0,
            };
            assert_eq!(vim_regexec(&mut rm, text.as_ptr(), 0), 1);
        })
    });
    vim_regfree(prog);
}

criterion_group!(benches, bench_regexec);
criterion_main!(benches);
