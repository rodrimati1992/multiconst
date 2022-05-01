use multiconst::multiconst as mc;

#[test]
fn basic_pattern() {
    mc! {
        const (A, B, C): (u8, &str, Option<()>) = (3, "5", Some(()));
    }

    assert_eq!(A, 3);
    assert_eq!(B, "5");
    assert_eq!(C, Some(()));
}

#[test]
fn ignore_pattern() {
    mc! {
        const (A, _, C): (u8, _, u32) = (3, 5, 8);
    }

    assert_eq!(A, 3);
    assert_eq!(C, 8);
}

#[test]
fn remainder_pattern() {
    {
        mc! {
            const (A, .., C): (u8, u32) = (3, 5);
        }

        assert_eq!(A, 3);
        assert_eq!(C, 5);
    }
    {
        mc! {
            const (A, .., C): (u8, _, u32) = (3, 5, 8);
        }

        assert_eq!(A, 3);
        assert_eq!(C, 8);
    }
    {
        mc! {
            const (A, .., C): (u8, _,  _, u32) = (3, 5, 8, 13);
        }

        assert_eq!(A, 3);
        assert_eq!(C, 13);
    }
    {
        mc! {
            const (A, .., C): (u8, _, _,  _, &str) = (3, 5, 8, 13, "21");
        }

        assert_eq!(A, 3);
        assert_eq!(C, "21");
    }

    mc! { const (..): (_, _, _, _, _) = (3, 5, 8, 13, 21); }

    mc! { const (..): (_, _, _, _, _) = (3, 5, 8, 13, 21); }
}

#[test]
fn type_alias_remainder_pattern() {
    {
        type X = (u8, &'static str);
        mc! {
            const (A, .., C): X = (3, "5");
        }

        assert_eq!(A, 3);
        assert_eq!(C, "5");
    }
    {
        type X = (u8, u8, &'static str);
        mc! {
            const (A, .., C): X = (3, 5, "8");
        }

        assert_eq!(A, 3);
        assert_eq!(C, "8");
    }
    {
        type X = (u8, u8, u8, &'static str);
        mc! {
            const (A, .., C): X = (3, 5, 8, "13");
        }

        assert_eq!(A, 3);
        assert_eq!(C, "13");
    }
    {
        type X = (u8, u8, u8, u8, &'static str);
        mc! {
            const (A, .., C): X = (3, 5, 8, 13, "21");
        }

        assert_eq!(A, 3);
        assert_eq!(C, "21");
    }
    {
        type X = (u8, u8, u8, u8, &'static str);
        mc! { const (..): X = (3, 5, 8, 13, "21"); }
    }
}

#[test]
fn skipping_one() {
    {
        mc! {
            const (A, B, C, D, ..): (&str, u16, char, Option<u32>, _) =
                ("3", 5, 'a', Some(8), [13]);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
        assert_eq!(C, 'a');
        assert_eq!(D, Some(8));
    }
    {
        mc! {
            const (A, B, C, .., D): (&str, u16, char, _, [u64; 1]) =
                ("3", 5, 'a', Some(8), [13]);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
        assert_eq!(C, 'a');
        assert_eq!(D, [13]);
    }
    {
        mc! {
            const (A, B, .., C, D): (&str, u16, _, Option<u32>, [u64; 1]) =
                ("3", 5, 'a', Some(8), [13]);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
        assert_eq!(C, Some(8));
        assert_eq!(D, [13]);
    }
    {
        mc! {
            const (A, .., B, C, D): (&str, _, u16, Option<u32>, [u64; 1]) =
                ("3", 'a', 5, Some(8), [13]);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
        assert_eq!(C, Some(8));
        assert_eq!(D, [13]);
    }
    {
        mc! {
            const (.., A, B, C, D): (_, char, u16, Option<u32>, [u64; 1]) =
                ("3", 'a', 5, Some(8), [13]);
        }

        assert_eq!(A, 'a');
        assert_eq!(B, 5);
        assert_eq!(C, Some(8));
        assert_eq!(D, [13]);
    }
}

#[test]
fn skipping_two() {
    {
        mc! {
            const (A, B, C, ..): (&str, u16, char, _, _) =
                ("3", 5, 'a', Some(8), [13]);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
        assert_eq!(C, 'a');
    }
    {
        mc! {
            const (A, B, .., C): (&str, u16, _, _, [u64; 1]) =
                ("3", 5, 'a', Some(8), [13]);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
        assert_eq!(C, [13]);
    }
    {
        mc! {
            const (A, .., B, C): (&str, _, _, Option<u32>, [u64; 1]) =
                ("3", 'a', 5, Some(8), [13]);
        }

        assert_eq!(A, "3");
        assert_eq!(B, Some(8));
        assert_eq!(C, [13]);
    }
    {
        mc! {
            const (.., A, B, C): (_, _, u16, Option<u32>, [u64; 1]) =
                ("3", 'a', 5, Some(8), [13]);
        }

        assert_eq!(A, 5);
        assert_eq!(B, Some(8));
        assert_eq!(C, [13]);
    }
}

#[test]
fn skipping_three() {
    {
        mc! {
            const (A, B, ..): (&str, u16, _, _, _) =
                ("3", 5, 'a', Some(8), [13]);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
    }
    {
        mc! {
            const (A, .., B): (&str, u16, _, _, [u64; 1]) =
                ("3", 5, 'a', Some(8), [13]);
        }

        assert_eq!(A, "3");
        assert_eq!(B, [13]);
    }
    {
        mc! {
            const (.., A, B): (&str, _, _, Option<u32>, [u64; 1]) =
                ("3", 'a', 5, Some(8), [13]);
        }

        assert_eq!(A, Some(8));
        assert_eq!(B, [13]);
    }
}

#[test]
fn skipping_four() {
    {
        mc! {
            const (A, ..): (&str, u16, _, _, _) =
                ("3", 5, 'a', Some(8), [13]);
        }

        assert_eq!(A, "3");
    }
    {
        mc! {
            const (.., A): (&str, _, _, Option<u32>, [u64; 1]) =
                ("3", 'a', 5, Some(8), [13]);
        }

        assert_eq!(A, [13]);
    }
}

#[test]
fn trailing_rem_pattern() {
    {
        mc! {
            const (A, B, C, D, ..): (&str, u16, Option<u32>, [u64; 1], _) =
                ("3", 5, Some(8), [13], 21);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
        assert_eq!(C, Some(8));
        assert_eq!(D, [13]);
    }
    {
        mc! {
            const (A, B, C, ..): (&str, u16, Option<u32>, _, _) =
                ("3", 5, Some(8), [13], 21);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
        assert_eq!(C, Some(8));
    }
    {
        mc! {
            const (A, B, ..): (&str, u16, _, _, _) = ("3", 5, Some(8), [13], 21);
        }

        assert_eq!(A, "3");
        assert_eq!(B, 5);
    }
    {
        mc! {
            const (A, ..): (&str, _, _, _, _) = ("3", 5, Some(8), [13], 21);
        }

        assert_eq!(A, "3");
    }
}

#[test]
fn nested_pattern() {}
