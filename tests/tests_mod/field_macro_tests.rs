use multiconst::{field_name, TChars, TIdent, Usize, __::AssertSameTypes};

#[test]
fn field_name_test() {
    let _: AssertSameTypes<field_name!(0), Usize<0>>;
    let _: AssertSameTypes<field_name!(10), Usize<10>>;
    let _: AssertSameTypes<
        field_name!(foo),
        TIdent<(TChars<'f', 'o', 'o', ' ', ' ', ' ', ' ', ' '>,)>,
    >;
    let _: AssertSameTypes<
        field_name!(foo_bar_),
        TIdent<(TChars<'f', 'o', 'o', '_', 'b', 'a', 'r', '_'>,)>,
    >;
    let _: AssertSameTypes<
        field_name!(foo_baar),
        TIdent<(TChars<'f', 'o', 'o', '_', 'b', 'a', 'a', 'r'>,)>,
    >;
    let _: AssertSameTypes<
        field_name!(foo_baarb),
        TIdent<(
            TChars<'f', 'o', 'o', '_', 'b', 'a', 'a', 'r'>,
            TChars<'b', ' ', ' ', ' ', ' ', ' ', ' ', ' '>,
        )>,
    >;
    let _: AssertSameTypes<
        field_name!(hello_world_fae),
        TIdent<(
            TChars<'h', 'e', 'l', 'l', 'o', '_', 'w', 'o'>,
            TChars<'r', 'l', 'd', '_', 'f', 'a', 'e', ' '>,
        )>,
    >;
    let _: AssertSameTypes<
        field_name!(hello_world_faei),
        TIdent<(
            TChars<'h', 'e', 'l', 'l', 'o', '_', 'w', 'o'>,
            TChars<'r', 'l', 'd', '_', 'f', 'a', 'e', 'i'>,
        )>,
    >;
    let _: AssertSameTypes<
        field_name!(hello_world_faeib),
        TIdent<(
            TChars<'h', 'e', 'l', 'l', 'o', '_', 'w', 'o'>,
            TChars<'r', 'l', 'd', '_', 'f', 'a', 'e', 'i'>,
            TChars<'b', ' ', ' ', ' ', ' ', ' ', ' ', ' '>,
        )>,
    >;
}

#[test]
fn field_name_aliases_test() {
    mod implicit {
        multiconst::field_name_aliases! {
            pub foo,
            pub foo_bar_,
            pub foo_baar,
            pub foo_baarb,
        }
        multiconst::field_name_aliases! {
            pub hello_world_fae,
            pub hello_world_faei
        }
        multiconst::field_name_aliases! {
            pub hello_world_faeib
        }
    }
    mod explicit {
        multiconst::field_name_aliases! {
            pub zero = 0,
            pub ten = 10,
            pub alias_foo = foo,
            pub alias_foo_bar_ = foo_bar_,
            pub alias_foo_baar = foo_baar,
            pub alias_foo_baarb = foo_baarb,
        }
        multiconst::field_name_aliases! {
            pub alias_hello_world_fae = hello_world_fae,
            pub alias_hello_world_faei = hello_world_faei
        }
        multiconst::field_name_aliases! {
            pub alias_hello_world_faeib = hello_world_faeib
        }
    }

    macro_rules! ass {
        ($A:tt, $B:tt) => {
            let _: AssertSameTypes<field_name!($A), implicit::$A>;
            let _: AssertSameTypes<implicit::$A, explicit::$B>;
        };
    }

    let _: AssertSameTypes<field_name!(0), explicit::zero>;
    let _: AssertSameTypes<field_name!(10), explicit::ten>;

    ass! {foo, alias_foo}
    ass! {foo_bar_, alias_foo_bar_}
    ass! {foo_baar, alias_foo_baar}
    ass! {foo_baarb, alias_foo_baarb}
    ass! {hello_world_fae, alias_hello_world_fae}
    ass! {hello_world_faei, alias_hello_world_faei}
    ass! {hello_world_faeib, alias_hello_world_faeib}
}
