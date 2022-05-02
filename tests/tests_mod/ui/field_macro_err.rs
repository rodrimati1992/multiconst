use multiconst::field;

fn main(){
    let _: field!("hello");
    let _: field!(0x100);
    let _: field!(-1);
    let _: field!(10usize);
}