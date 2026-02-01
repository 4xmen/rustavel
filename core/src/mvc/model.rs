

pub trait Model: Sized + Send + Sync + 'static {
    type PrimaryKey;

    fn table() -> &'static str;
    fn primary_key() -> &'static str;
    fn columns() -> &'static [&'static str];

}