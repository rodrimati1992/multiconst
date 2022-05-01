#[macro_export]
macro_rules! multiconst {
    ($($args:tt)*) => {
        $crate::__::__priv_multiconst_proc_macro!{
            $crate

            $($args)*
        }
    };
}
