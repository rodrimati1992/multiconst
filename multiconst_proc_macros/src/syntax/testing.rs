use used_proc_macro::TokenStream;

use alloc::{
    string::{String, ToString},
    vec,
};

use crate::{
    parsing::ParseBuffer,
    syntax::{Crate, Path},
    test_utils::StrExt,
};

fn parse_path(s: &str) -> Result<(String, String), String> {
    let input = s.parse::<TokenStream>().unwrap();
    let input = &mut ParseBuffer::new(input);
    let crate_kw = Crate::new_dummy();

    Path::parse(input)
        .map(|x| {
            (
                x.tokens.to_string(),
                input.collect::<TokenStream>().to_string(),
            )
        })
        .map_err(|e| e.to_compile_error(&crate_kw).to_string())
}

#[test]
fn test_path() {
    for before in vec![
        "::Hello",
        "Hello",
        "<Foo>",
        "<Foo>::",
        "<Foo>::Bar",
        "<Foo>::Bar::<Baz>",
        "Foo::<Bar>",
    ] {
        for after in vec!["(world)", "()", "{}", "{world: A}", ""] {
            let concat = [before, after].concat();
            let (path, rem) = parse_path(&concat).unwrap();

            assert!(
                path.consecutive_unspace(&[before]),
                "{:?} {:?}",
                concat,
                before
            );
            assert!(
                rem.consecutive_unspace(&[after]),
                "{:?} {:?}",
                concat,
                after
            );
        }
    }

    {
        let err = parse_path("(world)").unwrap_err();

        assert!(err.consecutive_unspace(&["expected", "path"]), "{}", err);
    }
    {
        let err = parse_path("<(world)").unwrap_err();

        assert!(err.consecutive_unspace(&["incomplete", "type"]), "{}", err);
    }
}
