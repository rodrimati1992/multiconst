use crate::test_utils::StrExt;

use alloc::string::{String, ToString};

fn process_str(s: &str) -> Result<String, String> {
    let s = alloc::format!("crate {}", s);
    let ts = s.parse::<used_proc_macro::TokenStream>().unwrap();

    ::std::dbg!(&ts);

    crate::for_multiconst_macro::macro_impl(ts)
        .map(|x| x.to_string())
        .map_err(|e| e.to_string())
}

#[test]
fn no_inferred_length_error() {
    {
        let out = process_str("const [A, ..]: [u32; _] = expr;").unwrap_err();
        assert!(
            out.consecutive_in_self(&["infer", "length", "`..`"]),
            "{}",
            out
        );
    }
}

#[test]
fn mismatched_pattern_and_type_constructor() {
    {
        let out = process_str("const (A, B): [u32; 0] = expr;").unwrap_err();
        assert!(out.consecutive_in_self(&["mismatched", "type"]), "{}", out);
    }
    {
        let out = process_str("const [A, B]: (u32, u32) = expr;").unwrap_err();
        assert!(out.consecutive_in_self(&["mismatched", "type"]), "{}", out);
    }
}

#[test]
fn mismatched_tuple_lengths() {
    {
        let out = process_str("const (A, B): () = expr;").unwrap_err();
        assert!(
            out.consecutive_in_self(&["pattern has 2", "but type has 0"]),
            "{}",
            out
        );
    }
    {
        let out = process_str("const (A, B): (u32,) = expr;").unwrap_err();
        assert!(
            out.consecutive_in_self(&["pattern has 2", "but type has 1"]),
            "{}",
            out
        );
    }
    {
        let out = process_str("const (A, B): (u32, u32, u32) = expr;").unwrap_err();
        assert!(
            out.consecutive_in_self(&["pattern has 2", "but type has 3"]),
            "{}",
            out
        );
    }
}

#[test]
fn top_level_remainder_pattern() {
    {
        let out = process_str("const ..: u32 = 100;").unwrap_err();
        assert!(out.contains("`..` patterns"), "{}", out);
    }
}

#[test]
fn attributes_on_patterns() {
    {
        let out = process_str("#[foo] const #[bar] A: u32 = 100;").unwrap();
        assert!(
            out.consecutive_unspace(&["#[bar] #[foo] const A: u32"]),
            "{}",
            out
        );
    }
    {
        let out = process_str("#[foo] const (#[bar] A, #[baz] B): (u32, u32) = 100;").unwrap();
        assert!(
            out.consecutive_unspace(&["#[bar] #[foo] const A: u32", "#[baz] #[foo] const B: u32",]),
            "{}",
            out,
        );
    }
    {
        let out = process_str("#[foo] const [#[bar] A, #[baz] B @ ..]: [u32; 3] = 100;").unwrap();
        assert!(
            out.consecutive_unspace(&[
                "#[bar] #[foo] const A: u32",
                "#[baz] #[foo] const B: [u32;",
            ]),
            "{}",
            out,
        );
    }
    {
        let out = process_str("#[foo] const F{#[bar] a: A}: F = E;").unwrap();
        assert!(
            out.consecutive_unspace(&["#[bar] #[foo] const A:"]),
            "{}",
            out,
        );
    }
    {
        let out = process_str("#[foo] const F(#[bar] A): F = E;").unwrap();
        assert!(
            out.consecutive_unspace(&["#[bar] #[foo] const A:"]),
            "{}",
            out,
        );
    }
}

#[test]
fn visibility_qualifiers() {
    {
        let out = process_str("const A: u32 = 100;").unwrap();
        assert!(!out.contains("pub"), "{}", out);
    }
    {
        let out = process_str("pub const A: u32 = 100;").unwrap();
        assert_eq!(out.matches("pub").count(), 1, "{}", out);
        assert!(out.consecutive_unspace(&["pub const A"]), "{}", out);
    }
    {
        let out = process_str("pub(crate) const A: u32 = 100;").unwrap();
        assert_eq!(out.matches("pub").count(), 1, "{}", out);
        assert!(out.consecutive_unspace(&["pub(crate) const A"]), "{}", out);
    }
    {
        let out = process_str(
            "
            pub(crate) const (A, B): (u32, u64) = 100;
        ",
        )
        .unwrap();

        assert_eq!(out.matches("pub").count(), 2, "{}", out);

        assert!(
            out.consecutive_unspace(&["pub(crate) const A", "pub(crate) const B"]),
            "{}",
            out,
        );
    }
}
