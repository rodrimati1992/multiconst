use multiconst::multiconst;

#[test]
fn test_length_inference() {
    {
        multiconst! {
            const [A, B, C]: [u8; _] = [3, 5, 8];
        }
        assert_eq!(A, 3);
        assert_eq!(B, 5);
        assert_eq!(C, 8);
    }
    {
        multiconst! {
            const [A, _, B, C @ ..]: [u8; 6] = [3, 5, 8, 13, 21, 34];
        }
        assert_eq!(A, 3);
        assert_eq!(B, 8);
        assert_eq!(C, [13, 21, 34]);
    }
    {
        multiconst! {
            const [_, A @ .., _]: [u8; 6] = [3, 5, 8, 13, 21, 34];
        }
        assert_eq!(A, [5, 8, 13, 21]);
    }
    {
        multiconst! {
            const [A, _ @ .., B]: [u8; 6] = [3, 5, 8, 13, 21, 34];
        }
        assert_eq!(A, 3);
        assert_eq!(B, 34);
    }
    {
        multiconst! {
            const [A, .., B]: [u8; 6] = [3, 5, 8, 13, 21, 34];
        }
        assert_eq!(A, 3);
        assert_eq!(B, 34);
    }
}

#[test]
fn test_opaque_types() {
    {
        type X = [u8; 6];
        multiconst! {
            const [A, _, B, C @ ..]: X = [3, 5, 8, 13, 21, 34];
        }
        assert_eq!(A, 3);
        assert_eq!(B, 8);
        assert_eq!(C, [13, 21, 34]);
    }
    {
        type Y = (u8, u16, u32);
        multiconst! {
            const [(A, B, C), ..]: [Y; 3] = {
                let empty = (0, 0, 0);
                [(3, 5, 8), empty, empty]
            };
        }
        assert_eq!(A, 3);
        assert_eq!(B, 5);
        assert_eq!(C, 8);
    }
}

#[test]
fn test_array_of_tuples_of_arrays_destructuring() {
    {
        multiconst! {
            const [
                ([A, B], [C]),
                ([D, E], [F]),
                REM @ ..,
            ]: [([u8; 2], [u16; 1]); 4] = {
                [
                    ([10, 11], [12]),
                    ([20, 21], [22]),
                    ([30, 31], [32]),
                    ([40, 41], [42]),
                ]
            };
        }

        assert_eq!(A, 10);
        assert_eq!(B, 11);
        assert_eq!(C, 12);
        assert_eq!(D, 20);
        assert_eq!(E, 21);
        assert_eq!(F, 22);
        assert_eq!(REM, [([30, 31], [32]), ([40, 41], [42])]);
    }
}

/// Ensures that tuple elements don't need a type when they're not destructured
#[test]
fn test_array_of_tuples_ignoring_fields() {
    multiconst! {
        const [(A, _, C)]: [(u8, _, u32); _] = [(10, 11, 12)];
    }

    assert_eq!(A, 10);
    assert_eq!(C, 12);
}

#[test]
fn test_multiple_levels_of_length_inference() {
    multiconst! {
        const [[A, B @ ..], [C, D, E], H]: [[u32; _]; _] =
            [
                [10, 11, 12],
                [20, 21, 22],
                [30, 31, 32],
            ];
    }

    assert_eq!(A, 10);
    assert_eq!(B, [11, 12]);
    assert_eq!(C, 20);
    assert_eq!(D, 21);
    assert_eq!(E, 22);
    assert_eq!(H, [30, 31, 32]);
}
