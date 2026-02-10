use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use quote::spanned::Spanned;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, DeriveInput, Field, LitStr, parse_macro_input};
use syn::{Error, Result};

// Define an enum for the validation rules
// This enum represents all supported rules, making it easy to extend later by adding new variants
// Each variant can hold parameters if needed (e.g., Min holds the min value as u32)
#[derive(Debug, Clone)]
enum Rule {
    Required,
    Nullable,
    Min(u32),
    Max(u32),
    Email,
    Confirmed(String), // The other field name for confirmation (e.g., "password_confirmation")
                       // Add more rules here in the future, e.g., Regex(String), Unique(String), etc.
}

// Implement a way to display the rule for testing purposes
impl Rule {
    fn as_str(&self) -> String {
        match self {
            Rule::Required => "required".to_string(),
            Rule::Nullable => "nullable".to_string(),
            Rule::Min(val) => format!("min:{}", val),
            Rule::Max(val) => format!("max:{}", val),
            Rule::Email => "email".to_string(),
            Rule::Confirmed(field) => format!("confirmed:{}", field),
        }
    }
}

#[proc_macro_derive(LaravelValidate, attributes(validating))]
pub fn laravel_validate(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree (DeriveInput represents the struct)
    let mut ast = parse_macro_input!(input as DeriveInput);

    // We'll collect all the parsed rules for each field in a String for display in tests
    let mut rules_display = String::new();

    // Check if the derive is on a struct (we assume it is, but in full version, add error handling)

    if let syn::Data::Struct(data_struct) = &mut ast.data {
        // collect all fields name required in safe validation like confirm
        let field_names: Vec<String> = data_struct
            .fields
            .iter()
            .filter_map(|field| field.ident.as_ref().map(|ident| ident.to_string()))
            .collect();

        // Iterate over each field in the struct
        for field in data_struct.fields.iter_mut() {
            // Find the #[validating] attribute on this field
            let validating_attr = find_validating_attr(field);

            if let Some(attr) = validating_attr {
                // Parse the rules from the attribute (either single string or list) into Vec<Rule>
                match parse_rules(&attr, &field_names) {
                    Ok(rules) => {
                        // For display, format as "field: rule1|rule2|..."
                        let field_name = field.ident.as_ref().unwrap().to_string();
                        let rules_str: Vec<String> = rules.iter().map(|r| r.as_str()).collect();
                        rules_display.push_str(&format!(
                            "{}: {}\\n",
                            field_name,
                            rules_str.join("|")
                        ));
                    }
                    Err(err) => {
                        // If parsing fails, return the compile error
                        return err.to_compile_error().into();
                    }
                }
            }
        }
    }

    // Generate a simple impl for the struct with an associated function to display the parsed rules
    // This is static (no &self needed) since rules are compile-time known
    // In tests, we can call Struct::display_parsed_rules()
    let struct_name = &ast.ident;
    let r#gen = quote! {
        impl #struct_name {
            pub fn display_parsed_rules() -> &'static str {
                #rules_display
            }
        }
    };

    r#gen.into()
}

/// Helper function to find the #[validating] attribute on a field
/// Clones the attribute if found
fn find_validating_attr(field: &Field) -> Option<Attribute> {
    field
        .attrs
        .iter()
        .find(|attr| attr.path().is_ident("validating"))
        .cloned()
}

/// Helper function to parse the rules from the #[validating] attribute into Vec<Rule>
/// Supports two formats:
/// - Single string: #[validating("required|email|max:180")] -> split by '|' and parse each
/// - List of strings: #[validating("required", "email", "max:180")] -> parse each
/// Returns syn::Error if any rule is invalid or unsupported
fn parse_rules(attr: &Attribute, fields_name: &Vec<String>) -> Result<Vec<Rule>> {
    let mut rules = Vec::new();

    // Parse the attribute's arguments as a punctuated list of LitStr
    let args: Punctuated<LitStr, Comma> = attr.parse_args_with(Punctuated::parse_terminated)?;

    let mut raw_rules: Vec<String> = Vec::new();
    if args.len() == 1 {
        // Single item: likely the pipe-separated string
        if let Some(lit_str) = args.first() {
            // Split by '|' and trim each part
            let rule_str = lit_str.value();
            raw_rules = rule_str.split('|').map(|s| s.trim().to_string()).collect();
        }
    } else {
        // Multiple items: each is a separate rule (LitStr)
        for lit in args {
            raw_rules.push(lit.value());
        }
    }

    // Now, parse each raw rule string into a Rule enum variant
    for raw in raw_rules {
        let rule = parse_single_rule(&raw, attr.__span(), fields_name)?;
        rules.push(rule);
    }

    Ok(rules)
}

/// Parse a single rule string like "required" or "max:180" or "confirmed:other_field" into Rule
/// Returns syn::Error if invalid format or unsupported rule
fn parse_single_rule(raw: &str, span: Span, fields_name: &Vec<String>) -> Result<Rule> {
    if raw.contains(':') {
        // Rules with parameters, like "max:180" or "confirmed:password_confirmation"
        let parts: Vec<&str> = raw.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(Error::new(span, format!("Invalid rule format: '{}'", raw)));
        }
        let name = parts[0].trim();
        let param = parts[1].trim();

        match name {
            "min" => {
                let val: u32 = param
                    .parse()
                    .map_err(|_| Error::new(span, format!("Invalid min value: '{}'", param)))?;
                Ok(Rule::Min(val))
            }
            "max" => {
                let val: u32 = param
                    .parse()
                    .map_err(|_| Error::new(span, format!("Invalid max value: '{}'", param)))?;
                Ok(Rule::Max(val))
            }
            "confirmed" => {
                if param.is_empty() {
                    return Err(Error::new(span, "Confirmed rule requires a field name"));
                }
                if !fields_name.contains(&param.to_string()) {
                    return Err(Error::new(
                        span,
                        format!("Invalid confirm field name '{}'", param),
                    ));
                }
                Ok(Rule::Confirmed(param.to_string()))
            }
            _ => Err(Error::new(
                span,
                format!("Unsupported rule with param: '{}'", name),
            )),
        }
    } else {
        // Simple rules without parameters
        match raw.trim() {
            "required" => Ok(Rule::Required),
            "nullable" => Ok(Rule::Nullable),
            "email" => Ok(Rule::Email),
            _ => Err(Error::new(span, format!("Unsupported rule: '{}'", raw))),
        }
    }
}
