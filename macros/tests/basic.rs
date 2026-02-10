use macros::LaravelValidate;
#[derive(LaravelValidate)]
struct TestStruct {
    #[validating("required|email|max:180")]
    email: String,

    #[validating("required", "min:8", "max:128", "confirmed:password_confirmation")]
    password: String,

    password_confirmation: String,
}

#[test]
fn test_parsed_rules() {
    let expected = "email: required|email|max:180\\npassword: required|min:8|max:128|confirmed:password_confirmation\\n";
    assert_eq!(TestStruct::display_parsed_rules(), expected);
    println!("{:?}",expected)
}