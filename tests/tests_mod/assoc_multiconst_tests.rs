use multiconst::associated_multiconst;

const EMPTY_STR_ARR: [String; 0] = [];

#[test]
fn associated_multiconst_test() {
    struct Generic<T>(T);

    impl<T> Generic<T> {
        associated_multiconst! {
            const [A, B, C]: [usize; _] = {
                let x = std::mem::size_of::<T>();

                [x, x * 2, x * 3]
            };
        }
    }

    assert_eq!(Generic::<u16>::A, 2);
    assert_eq!(Generic::<u16>::B, 4);
    assert_eq!(Generic::<u16>::C, 6);

    assert_eq!(Generic::<[u8; 5]>::A, 5);
    assert_eq!(Generic::<[u8; 5]>::B, 10);
    assert_eq!(Generic::<[u8; 5]>::C, 15);
}

#[test]
fn generic_associated_tuple_test() {
    struct SyntTupleType<T>(T);

    impl<T> SyntTupleType<T> {
        associated_multiconst! {
            const (A, B): (Option<T>, [T; 0]) = (None, []);
        }
    }

    assert_eq!(SyntTupleType::<u16>::A, None);
    assert_eq!(SyntTupleType::<u16>::B, [0u16; 0]);

    assert_eq!(SyntTupleType::<String>::A, None);
    assert_eq!(SyntTupleType::<String>::B, EMPTY_STR_ARR);
}

#[test]
fn generic_associated_tuple_alias_test() {
    type AliasTup<T> = (Option<T>, [T; 0]);

    struct AliasTupleType<T>(T);

    impl<T> AliasTupleType<T> {
        associated_multiconst! {
            const (A, B): AliasTup<T> = (None, []);
        }
    }

    assert_eq!(AliasTupleType::<u16>::A, None);
    assert_eq!(AliasTupleType::<u16>::B, [0u16; 0]);

    assert_eq!(AliasTupleType::<String>::A, None);
    assert_eq!(AliasTupleType::<String>::B, EMPTY_STR_ARR);
}
