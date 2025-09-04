use std::env;
use std::fs;
use std::path::PathBuf;

use regex::Regex;

fn main() {
    println!("cargo:rerun-if-changed=../src/option_rs.h");
    println!("cargo:rerun-if-changed=../src/optiondefs.h");

    // Generate the bindings for the FFI structures.
    let bindings = bindgen::Builder::default()
        .header("../src/option_rs.h")
        .allowlist_type("rs_opt_t")
        .allowlist_type("rs_opt_type")
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");

    // Parse option names from optiondefs.h to build a Rust table.
    let content = fs::read_to_string("../src/optiondefs.h")
        .expect("failed to read optiondefs.h");

    let re = Regex::new(r#"^\{"([^"]+)",\s*"([^"]*)",\s*([^,]+),"#).unwrap();
    let term_re = Regex::new(r#"^p_term\("([^"]+)""#).unwrap();

    let mut entries = Vec::new();

    for line in content.lines() {
        let t = line.trim();
        if let Some(caps) = re.captures(t) {
            let name = caps.get(1).unwrap().as_str().to_string();
            let short = caps.get(2).unwrap().as_str().to_string();
            let flags = caps.get(3).unwrap().as_str();
            let (rs_kind, c_kind) = if flags.contains("P_BOOL") {
                ("OptType::Bool", "crate::bindings::rs_opt_type_RS_OPT_BOOL")
            } else if flags.contains("P_NUM") {
                ("OptType::Number", "crate::bindings::rs_opt_type_RS_OPT_NUMBER")
            } else {
                ("OptType::String", "crate::bindings::rs_opt_type_RS_OPT_STRING")
            };
            entries.push((name, short, rs_kind.to_string(), c_kind.to_string()));
        } else if let Some(caps) = term_re.captures(t) {
            let name = caps.get(1).unwrap().as_str().to_string();
            entries.push((name, String::new(), "OptType::String".to_string(), "crate::bindings::rs_opt_type_RS_OPT_STRING".to_string()));
        }
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut out = String::new();
    out.push_str("pub static OPTION_TABLE: &[OptionDef] = &[\n");
    for (name, short, rs_kind, _) in &entries {
        out.push_str(&format!(
            "    OptionDef {{ name: \"{}\", short: \"{}\", opt_type: {} }},\n",
            name, short, rs_kind
        ));
    }
    out.push_str("];\n\n");
    out.push_str("pub static OPTION_DEFS: &[rs_opt_t] = &[\n");
    for (name, _, _, c_kind) in &entries {
        out.push_str(&format!(
            "    rs_opt_t {{ name: b\"{}\\0\".as_ptr() as *const std::os::raw::c_char, typ: {}, default_value: b\"\\0\".as_ptr() as *const std::os::raw::c_char }},\n",
            name, c_kind
        ));
    }
    out.push_str("];");

    fs::write(out_dir.join("option_defs.rs"), out).expect("failed to write option_defs.rs");
}
