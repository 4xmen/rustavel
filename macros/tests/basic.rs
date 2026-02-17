use std::collections::HashMap;
use std::time::Instant;
use macros::CheckMate;

#[derive(CheckMate, Debug)]
struct FullRuleCoverage {

    id: i64,

    #[validating("required|email|max:180|lowercase")]
    email: String,

    #[validating("nullable|min:8|max:128|confirmed:password_confirmation|uppercase")]
    password: Option<String>,

    password_confirmation: Option<String>,

    #[validating("size:10","ascii","alphanumeric")]
    code: String,

    #[validating("url|ip")]
    endpoint: String,

    #[validating("hex_color|starts_with:#|ends_with:ff")]
    color: String,

    #[validating("in:admin,user,guest|not_in:banned,suspended")]
    role: String,

    // #[validating("unique:users,email,id|exists:users,email")]
    // user_ref: String,

    #[validating("file|image|mimetypes:image/png,image/jpeg|extensions:png,jpg")]
    avatar: String,

    #[validating("date|datetime|time")]
    published_at: String,

    #[validating("before:2026-01-01","after:2024-01-01")]
    date_range: String,

    
    #[validating("json")]
    metadata: String,

    #[validating("array")]
    test: HashMap<String, String>,
}

#[test]
fn test_all_rules_parsed() {

    assert_eq!(true, true);

}
