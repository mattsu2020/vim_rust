use std::os::raw::c_int;

#[repr(C)]
struct AChar {
    c: u32,
    isolated: u32,
    initial: u32,
    medial: u32,
    final_: u32,
}

const fn achar(c: u32, isolated: u32, initial: u32, medial: u32, final_: u32) -> AChar {
    AChar { c, isolated, initial, medial, final_ }
}

// Unicode values for Arabic characters.
const A_HAMZA: u32 = 0x0621;
const A_ALEF_MADDA: u32 = 0x0622;
const A_ALEF_HAMZA_ABOVE: u32 = 0x0623;
const A_WAW_HAMZA: u32 = 0x0624;
const A_ALEF_HAMZA_BELOW: u32 = 0x0625;
const A_YEH_HAMZA: u32 = 0x0626;
const A_ALEF: u32 = 0x0627;
const A_BEH: u32 = 0x0628;
const A_TEH_MARBUTA: u32 = 0x0629;
const A_TEH: u32 = 0x062A;
const A_THEH: u32 = 0x062B;
const A_JEEM: u32 = 0x062C;
const A_HAH: u32 = 0x062D;
const A_KHAH: u32 = 0x062E;
const A_DAL: u32 = 0x062F;
const A_THAL: u32 = 0x0630;
const A_REH: u32 = 0x0631;
const A_ZAIN: u32 = 0x0632;
const A_SEEN: u32 = 0x0633;
const A_SHEEN: u32 = 0x0634;
const A_SAD: u32 = 0x0635;
const A_DAD: u32 = 0x0636;
const A_TAH: u32 = 0x0637;
const A_ZAH: u32 = 0x0638;
const A_AIN: u32 = 0x0639;
const A_GHAIN: u32 = 0x063A;
const A_TATWEEL: u32 = 0x0640;
const A_FEH: u32 = 0x0641;
const A_QAF: u32 = 0x0642;
const A_KAF: u32 = 0x0643;
const A_LAM: u32 = 0x0644;
const A_MEEM: u32 = 0x0645;
const A_NOON: u32 = 0x0646;
const A_HEH: u32 = 0x0647;
const A_WAW: u32 = 0x0648;
const A_ALEF_MAKSURA: u32 = 0x0649;
const A_YEH: u32 = 0x064A;
const A_FATHATAN: u32 = 0x064B;
const A_DAMMATAN: u32 = 0x064C;
const A_KASRATAN: u32 = 0x064D;
const A_FATHA: u32 = 0x064E;
const A_DAMMA: u32 = 0x064F;
const A_KASRA: u32 = 0x0650;
const A_SHADDA: u32 = 0x0651;
const A_SUKUN: u32 = 0x0652;
const A_MADDA_ABOVE: u32 = 0x0653;
const A_HAMZA_ABOVE: u32 = 0x0654;
const A_HAMZA_BELOW: u32 = 0x0655;
const A_PEH: u32 = 0x067E;
const A_TCHEH: u32 = 0x0686;
const A_JEH: u32 = 0x0698;
const A_FKAF: u32 = 0x06A9;
const A_GAF: u32 = 0x06AF;
const A_FYEH: u32 = 0x06CC;

const A_S_LAM_ALEF_MADDA_ABOVE: u32 = 0xFEF5;
const A_F_LAM_ALEF_MADDA_ABOVE: u32 = 0xFEF6;
const A_S_LAM_ALEF_HAMZA_ABOVE: u32 = 0xFEF7;
const A_F_LAM_ALEF_HAMZA_ABOVE: u32 = 0xFEF8;
const A_S_LAM_ALEF_HAMZA_BELOW: u32 = 0xFEF9;
const A_F_LAM_ALEF_HAMZA_BELOW: u32 = 0xFEFA;
const A_S_LAM_ALEF: u32 = 0xFEFB;
const A_F_LAM_ALEF: u32 = 0xFEFC;

const A_BYTE_ORDER_MARK: u32 = 0xFEFF;

static A_CHARS: &[AChar] = &[
    achar(A_HAMZA, 0xFE80, 0, 0, 0),
    achar(A_ALEF_MADDA, 0xFE81, 0, 0, 0xFE82),
    achar(A_ALEF_HAMZA_ABOVE, 0xFE83, 0, 0, 0xFE84),
    achar(A_WAW_HAMZA, 0xFE85, 0, 0, 0xFE86),
    achar(A_ALEF_HAMZA_BELOW, 0xFE87, 0, 0, 0xFE88),
    achar(A_YEH_HAMZA, 0xFE89, 0xFE8B, 0xFE8C, 0xFE8A),
    achar(A_ALEF, 0xFE8D, 0, 0, 0xFE8E),
    achar(A_BEH, 0xFE8F, 0xFE91, 0xFE92, 0xFE90),
    achar(A_TEH_MARBUTA, 0xFE93, 0, 0, 0xFE94),
    achar(A_TEH, 0xFE95, 0xFE97, 0xFE98, 0xFE96),
    achar(A_THEH, 0xFE99, 0xFE9B, 0xFE9C, 0xFE9A),
    achar(A_JEEM, 0xFE9D, 0xFE9F, 0xFEA0, 0xFE9E),
    achar(A_HAH, 0xFEA1, 0xFEA3, 0xFEA4, 0xFEA2),
    achar(A_KHAH, 0xFEA5, 0xFEA7, 0xFEA8, 0xFEA6),
    achar(A_DAL, 0xFEA9, 0, 0, 0xFEAA),
    achar(A_THAL, 0xFEAB, 0, 0, 0xFEAC),
    achar(A_REH, 0xFEAD, 0, 0, 0xFEAE),
    achar(A_ZAIN, 0xFEAF, 0, 0, 0xFEB0),
    achar(A_SEEN, 0xFEB1, 0xFEB3, 0xFEB4, 0xFEB2),
    achar(A_SHEEN, 0xFEB5, 0xFEB7, 0xFEB8, 0xFEB6),
    achar(A_SAD, 0xFEB9, 0xFEBB, 0xFEBC, 0xFEBA),
    achar(A_DAD, 0xFEBD, 0xFEBF, 0xFEC0, 0xFEBE),
    achar(A_TAH, 0xFEC1, 0xFEC3, 0xFEC4, 0xFEC2),
    achar(A_ZAH, 0xFEC5, 0xFEC7, 0xFEC8, 0xFEC6),
    achar(A_AIN, 0xFEC9, 0xFECB, 0xFECC, 0xFECA),
    achar(A_GHAIN, 0xFECD, 0xFECF, 0xFED0, 0xFECE),
    achar(A_TATWEEL, 0, 0x0640, 0x0640, 0x0640),
    achar(A_FEH, 0xFED1, 0xFED3, 0xFED4, 0xFED2),
    achar(A_QAF, 0xFED5, 0xFED7, 0xFED8, 0xFED6),
    achar(A_KAF, 0xFED9, 0xFEDB, 0xFEDC, 0xFEDA),
    achar(A_LAM, 0xFEDD, 0xFEDF, 0xFEE0, 0xFEDE),
    achar(A_MEEM, 0xFEE1, 0xFEE3, 0xFEE4, 0xFEE2),
    achar(A_NOON, 0xFEE5, 0xFEE7, 0xFEE8, 0xFEE6),
    achar(A_HEH, 0xFEE9, 0xFEEB, 0xFEEC, 0xFEEA),
    achar(A_WAW, 0xFEED, 0, 0, 0xFEEE),
    achar(A_ALEF_MAKSURA, 0xFEEF, 0, 0, 0xFEF0),
    achar(A_YEH, 0xFEF1, 0xFEF3, 0xFEF4, 0xFEF2),
    achar(A_FATHATAN, 0xFE70, 0, 0, 0),
    achar(A_DAMMATAN, 0xFE72, 0, 0, 0),
    achar(A_KASRATAN, 0xFE74, 0, 0, 0),
    achar(A_FATHA, 0xFE76, 0, 0xFE77, 0),
    achar(A_DAMMA, 0xFE78, 0, 0xFE79, 0),
    achar(A_KASRA, 0xFE7A, 0, 0xFE7B, 0),
    achar(A_SHADDA, 0xFE7C, 0, 0xFE7C, 0),
    achar(A_SUKUN, 0xFE7E, 0, 0xFE7F, 0),
    achar(A_MADDA_ABOVE, 0, 0, 0, 0),
    achar(A_HAMZA_ABOVE, 0, 0, 0, 0),
    achar(A_HAMZA_BELOW, 0, 0, 0, 0),
    achar(A_PEH, 0xFB56, 0xFB58, 0xFB59, 0xFB57),
    achar(A_TCHEH, 0xFB7A, 0xFB7C, 0xFB7D, 0xFB7B),
    achar(A_JEH, 0xFB8A, 0, 0, 0xFB8B),
    achar(A_FKAF, 0xFB8E, 0xFB90, 0xFB91, 0xFB8F),
    achar(A_GAF, 0xFB92, 0xFB94, 0xFB95, 0xFB93),
    achar(A_FYEH, 0xFBFC, 0xFBFE, 0xFBFF, 0xFBFD),
];

extern "C" {
    static p_arshape: c_int;
    static p_tbidi: c_int;
}

fn find_achar(c: u32) -> Option<&'static AChar> {
    A_CHARS.binary_search_by_key(&c, |a| a.c).ok().map(|i| &A_CHARS[i])
}

fn chg_c_laa2i(hid_c: u32) -> u32 {
    match hid_c {
        A_ALEF_MADDA => A_S_LAM_ALEF_MADDA_ABOVE,
        A_ALEF_HAMZA_ABOVE => A_S_LAM_ALEF_HAMZA_ABOVE,
        A_ALEF_HAMZA_BELOW => A_S_LAM_ALEF_HAMZA_BELOW,
        A_ALEF => A_S_LAM_ALEF,
        _ => 0,
    }
}

fn chg_c_laa2f(hid_c: u32) -> u32 {
    match hid_c {
        A_ALEF_MADDA => A_F_LAM_ALEF_MADDA_ABOVE,
        A_ALEF_HAMZA_ABOVE => A_F_LAM_ALEF_HAMZA_ABOVE,
        A_ALEF_HAMZA_BELOW => A_F_LAM_ALEF_HAMZA_BELOW,
        A_ALEF => A_F_LAM_ALEF,
        _ => 0,
    }
}

fn can_join(c1: u32, c2: u32) -> bool {
    if let (Some(a1), Some(a2)) = (find_achar(c1), find_achar(c2)) {
        (a1.initial != 0 || a1.medial != 0) && (a2.final_ != 0 || a2.medial != 0)
    } else {
        false
    }
}

fn a_is_iso(c: u32) -> bool {
    find_achar(c).is_some()
}

fn a_is_ok(c: u32) -> bool {
    a_is_iso(c) || c == A_BYTE_ORDER_MARK
}

fn a_is_valid(c: u32) -> bool {
    a_is_ok(c) && c != A_HAMZA
}

fn arabic_maycombine_impl(two: u32) -> bool {
    unsafe {
        p_arshape != 0 && p_tbidi == 0 && matches!(two, A_ALEF_MADDA | A_ALEF_HAMZA_ABOVE | A_ALEF_HAMZA_BELOW | A_ALEF)
    }
}

fn arabic_combine_impl(one: u32, two: u32) -> bool {
    one == A_LAM && arabic_maycombine_impl(two)
}

#[no_mangle]
pub extern "C" fn arabic_maycombine(two: c_int) -> c_int {
    arabic_maycombine_impl(two as u32) as c_int
}

#[no_mangle]
pub extern "C" fn arabic_combine(one: c_int, two: c_int) -> c_int {
    arabic_combine_impl(one as u32, two as u32) as c_int
}

#[no_mangle]
pub extern "C" fn arabic_shape(
    c: c_int,
    ccp: *mut c_int,
    c1p: *mut c_int,
    prev_c: c_int,
    prev_c1: c_int,
    next_c: c_int,
) -> c_int {
    let c_u = c as u32;
    if !a_is_ok(c_u) {
        return c;
    }
    let c1 = unsafe { if c1p.is_null() { 0 } else { *c1p as u32 } };
    let curr_laa = arabic_combine_impl(c_u, c1);
    let prev_laa = arabic_combine_impl(prev_c as u32, prev_c1 as u32);
    let mut curr_c: u32;

    if curr_laa {
        curr_c = if a_is_valid(prev_c as u32) && can_join(prev_c as u32, A_LAM) && !prev_laa {
            chg_c_laa2f(c1)
        } else {
            chg_c_laa2i(c1)
        };
        if !c1p.is_null() {
            unsafe { *c1p = 0; }
        }
    } else {
        let curr_a = match find_achar(c_u) {
            Some(a) => a,
            None => return c,
        };
        let backward_combine = !prev_laa && can_join(prev_c as u32, c_u);
        let forward_combine = can_join(c_u, next_c as u32);
        curr_c = if backward_combine {
            if forward_combine {
                curr_a.medial
            } else {
                curr_a.final_
            }
        } else if forward_combine {
            curr_a.initial
        } else {
            curr_a.isolated
        };
    }

    if curr_c == 0 {
        curr_c = c_u;
    }

    if curr_c != c_u && !ccp.is_null() {
        if let Some(ch) = std::char::from_u32(curr_c) {
            let mut buf = [0u8; 4];
            let s = ch.encode_utf8(&mut buf);
            unsafe {
                *ccp = s.as_bytes()[0] as c_int;
            }
        }
    }

    curr_c as c_int
}
