#[macro_export] macro_rules! register_models {
    ($($name:ident),* $(,)?) => {
        $(pub mod $name;)*
    };
}