use crate::core::schema::Schema;


pub fn test(){
    let schema = Schema::new();
    schema.drop_if_exists("users");
}