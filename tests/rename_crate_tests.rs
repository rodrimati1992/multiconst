extern crate alloc as multiconst;
extern crate multiconst as mc;

use core::cmp::Ordering;

use mc::{FieldType, GetFieldType, Usize};

mod path {
    pub(crate) mod to {
        pub(crate) use mc;
    }
}

#[test]
fn single_token_crate_path() {
    #[derive(FieldType)]
    #[field_type(crate = mc)]
    struct Tupled(u8, Ordering, &'static str, bool);

    let _: GetFieldType<Tupled, Usize<0>> = 3u8;
    let _: GetFieldType<Tupled, Usize<1>> = Ordering::Less;
    let _: GetFieldType<Tupled, Usize<2>> = "helo";
    let _: GetFieldType<Tupled, Usize<3>> = true;
}

#[test]
fn leading_colon_crate_path() {
    #[derive(FieldType)]
    #[field_type(crate = ::mc)]
    struct Tupled(u8, Ordering, &'static str, bool);

    let _: GetFieldType<Tupled, Usize<0>> = 3u8;
    let _: GetFieldType<Tupled, Usize<1>> = Ordering::Less;
    let _: GetFieldType<Tupled, Usize<2>> = "helo";
    let _: GetFieldType<Tupled, Usize<3>> = true;
}

#[test]
fn colon_sep_crate_path() {
    #[derive(FieldType)]
    #[field_type(crate = crate::path::to::mc)]
    struct Tupled(u8, Ordering, &'static str, bool);

    let _: GetFieldType<Tupled, Usize<0>> = 3u8;
    let _: GetFieldType<Tupled, Usize<1>> = Ordering::Less;
    let _: GetFieldType<Tupled, Usize<2>> = "helo";
    let _: GetFieldType<Tupled, Usize<3>> = true;
}
