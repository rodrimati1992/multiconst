use multiconst::{field, TChars, TIdent, Usize, __::AssertSameTypes};

#[test]
fn field_value() {
    let _: AssertSameTypes<field!(0), Usize<0>>;
    let _: AssertSameTypes<field!(10), Usize<10>>;
    let _: AssertSameTypes<field!(foo), TIdent<(TChars<'f', 'o', 'o', ' ', ' ', ' ', ' ', ' '>,)>>;
    let _: AssertSameTypes<
        field!(foo_bar_),
        TIdent<(TChars<'f', 'o', 'o', '_', 'b', 'a', 'r', '_'>,)>,
    >;
    let _: AssertSameTypes<
        field!(foo_baar),
        TIdent<(TChars<'f', 'o', 'o', '_', 'b', 'a', 'a', 'r'>,)>,
    >;
    let _: AssertSameTypes<
        field!(foo_baarb),
        TIdent<(
            TChars<'f', 'o', 'o', '_', 'b', 'a', 'a', 'r'>,
            TChars<'b', ' ', ' ', ' ', ' ', ' ', ' ', ' '>,
        )>,
    >;
    let _: AssertSameTypes<
        field!(hello_world_fae),
        TIdent<(
            TChars<'h', 'e', 'l', 'l', 'o', '_', 'w', 'o'>,
            TChars<'r', 'l', 'd', '_', 'f', 'a', 'e', ' '>,
        )>,
    >;
    let _: AssertSameTypes<
        field!(hello_world_faei),
        TIdent<(
            TChars<'h', 'e', 'l', 'l', 'o', '_', 'w', 'o'>,
            TChars<'r', 'l', 'd', '_', 'f', 'a', 'e', 'i'>,
        )>,
    >;
    let _: AssertSameTypes<
        field!(hello_world_faeib),
        TIdent<(
            TChars<'h', 'e', 'l', 'l', 'o', '_', 'w', 'o'>,
            TChars<'r', 'l', 'd', '_', 'f', 'a', 'e', 'i'>,
            TChars<'b', ' ', ' ', ' ', ' ', ' ', ' ', ' '>,
        )>,
    >;
}
