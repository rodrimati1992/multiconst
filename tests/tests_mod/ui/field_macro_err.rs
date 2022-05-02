use multiconst::field_name;

fn main(){
    let _: field_name!("hello");
    let _: field_name!(0x100);
    let _: field_name!(-1);
    let _: field_name!(10usize);
}

mod aliases {
    multiconst::field_name_aliases! { foo = }
    multiconst::field_name_aliases! { foo = "bar" }
    multiconst::field_name_aliases! { "foo" }
    multiconst::field_name_aliases! { foo = bar baz }
}