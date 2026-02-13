use macros::CheckMate;
// use macros_core::LaravelValidator;
// use macros_core::ValidationErrors;

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

    #[validating("unique:users,email|exists:users,email,id")]
    user_ref: String,

    #[validating("file|image|mimetypes:image/png,image/jpeg|extensions:png,jpg")]
    avatar: String,

    #[validating("date|datetime|time")]
    published_at: String,

    #[validating("before:2026-01-01","after:2024-01-01")]
    date_range: String,

    #[validating("array|json")]
    metadata: String,
}

#[test]
fn test_all_rules_parsed() {
    let expected = concat!(
    "email: required|email|max:180|lowercase\n",
    "password: nullable|min:8|max:128|confirmed:password_confirmation|uppercase\n",
    "code: size:10|ascii|alphanumeric\n",
    "endpoint: url|ip\n",
    "color: hex_color|starts_with:#|ends_with:ff\n",
    "role: in:admin,user,guest|not_in:banned,suspended\n",
    "user_ref: unique:users,email|exists:users,email,id\n",
    "avatar: file|image|mimetypes:image/png,image/jpeg|extensions:png,jpg\n",
    "published_at: date|datetime|time\n",
    "date_range: before:2026-01-01|after:2024-01-01\n",
    "metadata: array|json\n",
    );



    // assert_eq!(FullRuleCoverage::display_parsed_rules(), expected);

}
