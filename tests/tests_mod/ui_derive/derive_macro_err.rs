use multiconst::FieldType;


#[derive(FieldType)]
struct Foo(#[field_type(crate = hello)] u32);


#[derive(FieldType)]
struct Bar(#[field_type(world)] u32);


#[derive(FieldType)]
#[field_type(aaaaa)]
struct Baz( u32);

#[derive(FieldType)]
#[field_type(aaaaa)]
struct Qux( u32);


#[derive(FieldType)]
#[field_type(pub fooasdasd)]
struct Another(u32);

// ensuring that errors are emitted for all attributes
#[derive(FieldType)]
#[field_type(foooo)]
#[field_type(bar)]
#[field_type(pub fooasdasd)]
struct ManyAttrs(u32);


fn main(){}