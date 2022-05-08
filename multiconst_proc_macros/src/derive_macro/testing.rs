use used_proc_macro::TokenStream;

use alloc::{
    format,
    string::{String, ToString},
    vec,
};

use crate::test_utils::StrExt;

fn parse_derive(s: &str) -> Result<String, String> {
    std::println!("\ninput: \n{}\n", s);
    let input = s.parse::<TokenStream>().unwrap();

    crate::derive_macro::derive_macro_impl(input)
        .map(|x| x.to_string())
        .map_err(|e| e.to_compile_error().to_string())
}

fn braced_derive(
    container_annots: &str,
    field0_annots: &str,
    field1_annots: &str,
) -> Result<String, String> {
    parse_derive(&format!(
        "
            {}
            struct Foo {{
                {}
                x: u32,
                {}
                y: u64,
            }}
        ",
        container_annots, field0_annots, field1_annots,
    ))
}

#[test]
fn visibility_overrides() {
    for vis in vec!["", "pub", "pub(crate)"] {
        {
            let res = braced_derive(&format!("#[field_type(pub)] {}", vis), "", "").unwrap();
            assert!(res.consecutive_unspace(&["Type = u32"]), "{}", res);
            assert!(res.consecutive_unspace(&["Type = u64"]), "{}", res);
        }
        {
            let res = braced_derive(&format!("#[field_type(priv)] {}", vis), "", "").unwrap();
            assert!(!res.consecutive_unspace(&["Type = u32"]), "{}", res);
            assert!(!res.consecutive_unspace(&["Type = u64"]), "{}", res);
        }
    }
}

#[test]
fn visibility_dependence() {
    for (type_vis, field_vis) in vec![
        ("", ""),
        ("pub(crate)", ""),
        ("", "pub(crate)"),
        ("", "pub"),
        ("pub(crate)", "pub"),
        ("pub", "pub"),
    ] {
        let res = braced_derive(type_vis, field_vis, field_vis).unwrap();
        assert!(res.consecutive_unspace(&["Type = u32"]), "{}", res);
        assert!(res.consecutive_unspace(&["Type = u64"]), "{}", res);
    }

    for (type_vis, field_vis) in vec![("pub", ""), ("pub", "pub(crate)")] {
        let res = braced_derive(type_vis, field_vis, field_vis).unwrap();
        assert!(!res.consecutive_unspace(&["Type = u32"]), "{}", res);
        assert!(!res.consecutive_unspace(&["Type = u64"]), "{}", res);
    }
}
