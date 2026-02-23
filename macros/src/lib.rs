//! # CheckMate: Laravel-inspired Validation DSL for Rust
//!
//! CheckMate is a compile-time, type-safe, macro-generated validation DSL
//! for Rust, inspired by Laravel's `FormRequest` concept while respecting
//! Rust's idioms and type system.
//!
//! Unlike Laravel's runtime-only rule arrays, CheckMate generates Rust code
//! to enforce validation rules at compile time wherever possible, reducing
//! runtime errors and boilerplate.
//!
//! Key features and motivations:
//! - Handle culturally specific inputs, such as Persian and Arabic numerals,
//!   without runtime overhead.
//! - Provide compile-time type safety for strings, numbers, and custom rules.
//! - Offer a familiar Laravel-like DSL for Rust developers, while remaining
//!   idiomatic to Rust.
//! - Generate ergonomic validation code via macros, minimizing boilerplate.
//! - Support future extensions with async/database/file rules in a modular way.
//! - WIP: Extractor integration planned for seamless web framework usage.
//!
//! CheckMate aims to provide a solid foundation for safe, expressive, and
//! culturally-aware input validation in Rust projects, filling gaps that
//! existing validators do not address.
//! CheckMate: check/validate، smart

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::format_ident;
use quote::quote;
use quote::spanned::Spanned;
use std::collections::HashSet;
// use std::process::id;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, DeriveInput, Field, LitStr, parse_macro_input};
use syn::{Error, Result};
use syn::{GenericArgument, PathArguments, Type, TypePath};

// Define an enum for the validation rules
// This enum represents all supported rules, making it easy to extend later by adding new variants
// Each variant can hold parameters if needed (e.g., Min holds the min value as u32)
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Rule {
    // general
    Required,
    Nullable,
    Min(i64),
    Max(i64),
    Size(i64),
    // string
    Email,
    Confirmed(String), // The other field name for confirmation (e.g., "password_confirmation")
    Url,
    EndsWith(String),
    StartsWith(String),
    Ip,
    Ascii,
    Alphanumeric,
    HexColor,
    LowerCase,
    UpperCase,
    In(String),
    NotIn(String),
    // db
    Unique(String),
    Exists(String),
    // files
    File,
    Image,
    MimeTypes(String),
    Extensions(String),
    // date & times
    Date,
    DateTime,
    Time,
    After(String),
    Before(String),
    // other
    Array,
    Json,
}

#[proc_macro_derive(CheckMate, attributes(validating))]
pub fn mate_validate(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree (DeriveInput represents the struct)
    let mut ast = parse_macro_input!(input as DeriveInput);

    // We'll collect all the parsed rules for each field in a String for display in tests
    // let mut rules_display = String::new();

    // Check if the derive is on a struct (we assume it is, but in full version, add error handling)
    let mut validations = Vec::new();

    if let syn::Data::Struct(data_struct) = &mut ast.data {
        // collect all fields name required in safe validation like confirm
        let field_names: HashSet<String> = data_struct
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
                        // check nullable can't be a non Option<_> type
                        if rules.iter().any(|r| matches!(r, Rule::Nullable)) {
                            if !is_option_type(&field.ty) {
                                return Error::new(
                                    field.ty.__span(),
                                    format!(
                                        "Field '{}' is marked as nullable but is not Option<T>",
                                        field.ident.as_ref().unwrap()
                                    ),
                                )
                                .to_compile_error()
                                .into();
                            }
                        }

                        let field_ident = field.ident.as_ref().unwrap();
                        let field_name = field_ident.to_string();
                        let field_ty = &field.ty;

                        // runtime validation code generation
                        for rule in &rules {
                            validations.push(rule.expand(field_ident, field_ty, &field_name));
                        }

                        // let rules_str: Vec<String> = rules.iter().map(|r| r.as_str()).collect();
                        // rules_display.push_str(&format!(
                        //     "{}: {}\n",
                        //     field_name,
                        //     rules_str.join("|")
                        // ));
                    }
                    Err(err) => {
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
    // let lit = LitStr::new(&rules_display, Span::call_site());
    let r#gen = quote! {

    // impl #struct_name {
    //     pub fn display_parsed_rules() -> &'static str {
    //             #lit
    //         }
    //     }



        impl  #struct_name {
            async fn validate(&self) -> Result<(), macros_core::ValidationErrors> {
                let mut errors = macros_core::ValidationErrors::new();

                #(#validations)*

                 // println!("{:?}", self);

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
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
fn parse_rules(attr: &Attribute, fields_name: &HashSet<String>) -> Result<Vec<Rule>> {
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
fn parse_single_rule(raw: &str, span: Span, fields_name: &HashSet<String>) -> Result<Rule> {
    if raw.contains(':') {
        // Rules with parameters, like "max:180" or "confirmed:password_confirmation"
        let parts: Vec<&str> = raw.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(Error::new(span, format!("Invalid rule format: '{}'", raw)));
        }
        let name = parts[0].trim();
        let param = parts[1].trim();

        match name {
            "starts_with" => Ok(Rule::StartsWith(param.to_string())),
            "ends_with" => Ok(Rule::EndsWith(param.to_string())),
            "in" => Ok(Rule::In(param.to_string())),
            "not_in" => Ok(Rule::NotIn(param.to_string())),
            "mimetypes" => Ok(Rule::MimeTypes(param.to_string())),
            "extensions" => Ok(Rule::Extensions(param.to_string())),
            "after" => {
                if is_valid_datetime(param) {
                    Ok(Rule::After(param.to_string()))
                } else {
                    Err(Error::new(
                        span,
                        format!("Invalid rule format: '{}'", param),
                    ))
                }
            }
            "before" => {
                if is_valid_datetime(param) {
                    Ok(Rule::Before(param.to_string()))
                } else {
                    Err(Error::new(
                        span,
                        format!("Invalid rule format: '{}'", param),
                    ))
                }
            }
            "unique" => Ok(Rule::Unique(param.to_string())),
            "exists" => Ok(Rule::Exists(param.to_string())),
            "min" => {
                let val: i64 = param
                    .parse()
                    .map_err(|_| Error::new(span, format!("Invalid min value: '{}'", param)))?;
                Ok(Rule::Min(val))
            }
            "max" => {
                let val: i64 = param
                    .parse()
                    .map_err(|_| Error::new(span, format!("Invalid max value: '{}'", param)))?;
                Ok(Rule::Max(val))
            }
            "size" => {
                let val: i64 = param
                    .parse()
                    .map_err(|_| Error::new(span, format!("Invalid size value: '{}'", param)))?;
                Ok(Rule::Size(val))
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
            "json" => Ok(Rule::Json),
            "image" => Ok(Rule::Image),
            "file" => Ok(Rule::File),
            "ip" => Ok(Rule::Ip),
            "ascii" => Ok(Rule::Ascii),
            "alphanumeric" => Ok(Rule::Alphanumeric),
            "hex_color" => Ok(Rule::HexColor),
            "lowercase" => Ok(Rule::LowerCase),
            "uppercase" => Ok(Rule::UpperCase),
            "date" => Ok(Rule::Date),
            "datetime" => Ok(Rule::DateTime),
            "time" => Ok(Rule::Time),
            "url" => Ok(Rule::Url),
            "array" => Ok(Rule::Array),
            _ => Err(Error::new(span, format!("Unsupported rule: '{}'", raw))),
        }
    }
}

fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(TypePath { path, .. }) = ty {
        if let Some(segment) = path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Returns `true` if `s` matches one of the supported date or
/// date‑time formats, otherwise `false`.
///
/// Supported separators: `-` or `/` between year, month and day.
/// Month and day may be 1 or 2 digits (no leading zero required).
/// Time part (optional) is separated from the date by a single space
/// and consists of hour, minute and optional second, each 1‑2 digits.
///
/// Examples of valid strings:
/// * `2025-31-01`
/// * `2025/1/31`
/// * `2025-01-31 20:21:01`
/// * `2025/1/31 1:1:11`
fn is_valid_datetime(s: &str) -> bool {
    // Split date and optional time
    let mut parts = s.splitn(2, ' ');
    let date_part = parts.next().unwrap();
    let time_part = parts.next();

    // ---------- date validation ----------
    // Accept '-' or '/' as separator, but it must be the same for the whole date.
    let sep = if date_part.contains('-') {
        '-'
    } else if date_part.contains('/') {
        '/'
    } else {
        return false;
    };
    let date_fields: Vec<&str> = date_part.split(sep).collect();
    if date_fields.len() != 3 {
        return false;
    }

    // Year: 1‑4 digits (we only care about format, not range)
    if !date_fields[0].chars().all(|c| c.is_ascii_digit()) {
        return false;
    }

    // Month and day: 1‑2 digits, no leading zeros required
    for f in &date_fields[1..] {
        if f.is_empty() || f.len() > 2 || !f.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
    }

    // ---------- time validation (optional) ----------
    if let Some(t) = time_part {
        // Time must be separated from date by exactly one space
        // and contain 2 or 3 fields separated by ':'
        let time_fields: Vec<&str> = t.split(':').collect();
        if time_fields.len() < 2 || time_fields.len() > 3 {
            return false;
        }

        // Hour and minute: 1‑2 digits
        for f in &time_fields[0..2] {
            if f.is_empty() || f.len() > 2 || !f.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }

        // Optional seconds: same rules if present
        if time_fields.len() == 3 {
            let sec = time_fields[2];
            if sec.is_empty() || sec.len() > 2 || !sec.chars().all(|c| c.is_ascii_digit()) {
                return false;
            }
        }
    }

    true
}

/// If the given type is `Option<T>`, returns `Some(&T)`.
/// Otherwise returns `None`.
fn extract_option_inner_type(ty: &Type) -> Option<&Type> {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            if segment.ident == "Option" {
                if let PathArguments::AngleBracketed(ref args) = segment.arguments {
                    if let Some(GenericArgument::Type(inner_ty)) = args.args.first() {
                        return Some(inner_ty);
                    }
                }
            }
        }
    }
    None
}

/// Returns `true` if the given type is `String` or `Option<String>`.
fn is_string_type(ty: &Type) -> bool {
    let ty = extract_option_inner_type(ty).unwrap_or(ty);

    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            return seg.ident == "String";
        }
    }

    false
}

/// Returns `true` if the given type is a supported numeric primitive
/// (i8, i16, i32, i64, u8, u16, u32, u64, f32, f64)
/// or an `Option` wrapping one of those types.
fn is_numeric_type(ty: &Type) -> bool {
    let ty = extract_option_inner_type(ty).unwrap_or(ty);

    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            let ident = seg.ident.to_string();

            return matches!(
                ident.as_str(),
                "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64"
            );
        }
    }

    false
}

/// Returns `true` if the given type is either a `Vec<String>` or `HashMap<String, String>`
/// or an `Option` wrapping one of those types.
fn is_string_collection_type(ty: &Type) -> bool {
    let ty = extract_option_inner_type(ty).unwrap_or(ty);

    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            let ident = seg.ident.to_string();

            // Check for Vec<String>
            if ident == "Vec" {
                if let syn::PathArguments::AngleBracketed(ref args) = seg.arguments {
                    if let Some(syn::GenericArgument::Type(syn::Type::Path(inner_path))) =
                        args.args.first()
                    {
                        if inner_path.path.is_ident("String") {
                            return true;
                        }
                    }
                }
            }

            // Check for HashMap<String, String>
            if ident == "HashMap" {
                if let syn::PathArguments::AngleBracketed(ref args) = seg.arguments {
                    if let (
                        Some(syn::GenericArgument::Type(syn::Type::Path(key_path))),
                        Some(syn::GenericArgument::Type(syn::Type::Path(value_path))),
                    ) = (args.args.get(0), args.args.get(1))
                    {
                        if key_path.path.is_ident("String") && value_path.path.is_ident("String") {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}

/// Returns `true` if the given type is a supported date|time primitive
/// (PrimitiveDateTime, Date)
/// or an `Option` wrapping one of those types.
fn is_datetime_types(ty: &Type) -> bool {
    let ty = extract_option_inner_type(ty).unwrap_or(ty);

    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            let ident = seg.ident.to_string();

            return matches!(ident.as_str(), "PrimitiveDateTime" | "Date");
        }
    }

    false
}

/// Returns `true` if the given type is `PrimitiveDateTime` or `Option<PrimitiveDateTime>`.
fn is_datetime_type(ty: &Type) -> bool {
    let ty = extract_option_inner_type(ty).unwrap_or(ty);

    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            return seg.ident == "PrimitiveDateTime";
        }
    }

    false
}
/// Returns `true` if the given type is `Date` or `Option<Date>`.
fn is_date_type(ty: &Type) -> bool {
    let ty = extract_option_inner_type(ty).unwrap_or(ty);

    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            return seg.ident == "Date";
        }
    }

    false
}

/// Returns `true` if the given type is `Time` or `Option<Dime>`.
fn is_time_type(ty: &Type) -> bool {
    let ty = extract_option_inner_type(ty).unwrap_or(ty);

    if let Type::Path(type_path) = ty {
        if let Some(seg) = type_path.path.segments.last() {
            return seg.ident == "Time";
        }
    }

    false
}

// Implement a way to display the rule for testing purposes
impl Rule {
    #[allow(dead_code)]
    fn as_str(&self) -> String {
        match self {
            Rule::Required => "required".to_string(),
            Rule::Nullable => "nullable".to_string(),
            Rule::Min(val) => format!("min:{}", val),
            Rule::Max(val) => format!("max:{}", val),
            Rule::Size(val) => format!("size:{}", val),
            Rule::Email => "email".to_string(),
            Rule::Confirmed(field) => format!("confirmed:{}", field),
            Rule::Url => "url".to_string(),
            Rule::Ip => "ip".to_string(),
            Rule::Ascii => "ascii".to_string(),
            Rule::Alphanumeric => "alphanumeric".to_string(),
            Rule::HexColor => "hex_color".to_string(),
            Rule::LowerCase => "lowercase".to_string(),
            Rule::UpperCase => "uppercase".to_string(),
            Rule::In(val) => format!("in:{}", val),
            Rule::NotIn(val) => format!("not_in:{}", val),
            Rule::Unique(map) => format!("unique:{}", map), // map: table_name,field or table_name,field,except_field
            Rule::Exists(map) => format!("exists:{}", map), // map: table,field
            Rule::File => "file".to_string(),
            Rule::Image => "image".to_string(),
            Rule::MimeTypes(types) => format!("mimetypes:{}", types),
            Rule::Extensions(types) => format!("extensions:{}", types),
            Rule::Date => "date".to_string(),
            Rule::DateTime => "datetime".to_string(),
            Rule::Time => "time".to_string(),
            Rule::After(date) => format!("after:{}", date),
            Rule::Before(date) => format!("before:{}", date),
            Rule::Array => "array".to_string(),
            Rule::Json => "json".to_string(),
            Rule::StartsWith(prefix) => format!("starts_with:{}", prefix),
            Rule::EndsWith(suffix) => format!("ends_with:{}", suffix),
        }
    }

    /// expand rule validator code generate
    /// depends on filed type
    fn expand(
        &self,
        field_ident: &syn::Ident,
        field_ty: &Type,
        field_name: &str,
    ) -> proc_macro2::TokenStream {
        match self {
            Rule::Required => {
                if is_option_type(field_ty) {
                    quote! {
                        if self.#field_ident.is_none() {
                            errors.add(#field_name, "required");
                        }
                    }
                } else if is_string_type(field_ty) {
                    quote! {
                        if self.#field_ident.is_empty() {
                            errors.add(#field_name, "required");
                        }
                    }
                } else {
                    quote! {}
                }
            }
            Rule::Confirmed(val) => {
                let confirm = format_ident!("{}", val);
                if is_option_type(field_ty) {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if let Some(value2) = (&self.#confirm).as_ref() {
                                if value != value2 {
                                    errors.add(#field_name, format!("{} not confirmed",&#field_name));
                                }
                            }
                        }
                    }
                } else {
                    quote! {
                        if &self.#field_ident != &self.#confirm{
                            errors.add(#field_name, format!("{} not confirmed",&#field_name));
                        }
                    }
                }
            }
            Rule::Email => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!("Email must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_email(&self.#field_ident) {
                        errors.add(#field_name, format!("Email is invalid `{}`",&self.#field_ident));
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_email(value) {
                               errors.add(#field_name, format!("Email is invalid `{}`",&value));
                            }
                        }
                    }
                }
            }
            Rule::Url => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!("Url must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_url(&self.#field_ident) {
                        errors.add(#field_name, format!("Url is invalid `{}`",&self.#field_ident));
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_url(value) {
                                errors.add(#field_name, format!("Email is invalid `{}`",&value));
                            }
                        }
                    }
                }
            }
            Rule::Min(val) => {
                let is_option = is_option_type(field_ty);

                if is_string_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = &self.#field_ident {
                                if value.len() < #val as usize {
                                    errors.add(#field_name, format!("String length must be at least `{}` characters",&#val));
                                }
                            }
                        }
                    } else {
                        quote! {
                            if self.#field_ident.len() < #val as usize {
                                 errors.add(#field_name, format!("String length must be at least `{}` characters",&#val));
                            }
                        }
                    }
                } else if is_numeric_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = self.#field_ident {
                                if macros_core::convert_to_i64(value) < #val  {
                                    errors.add(#field_name, format!("{} must be greater than or equal to {}",&#field_name,#val));
                                }
                            }
                        }
                    } else {
                        quote! {
                            if macros_core::convert_to_i64(self.#field_ident) < #val  {
                                errors.add(#field_name, format!("{} must be greater than or equal to {}",&#field_name,#val));
                            }
                        }
                    }
                } else {
                    return Error::new_spanned(
                        field_name,
                        format!("unsupported type for `min` rule:  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
            }
            Rule::Max(val) => {
                let is_option = is_option_type(field_ty);

                if is_string_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = &self.#field_ident {
                                if value.len() > #val as usize {
                                    errors.add(#field_name, format!("String length must not exceed {} characters.",&#val));
                                    errors.add(#field_name, "max");
                                }
                            }
                        }
                    } else {
                        quote! {
                            if self.#field_ident.len() > #val as usize {
                                errors.add(#field_name, format!("String length must not exceed {} characters.",&#val));
                            }
                        }
                    }
                } else if is_numeric_type(field_ty) {
                    if is_option {
                        quote! {
                            if let macros_core::convert_to_i64(Some(value)) = self.#field_ident {
                                if value < #val {
                                   errors.add(#field_name, format!("{} must be less than or equal to {}",&#field_name,#val));
                                }
                            }
                        }
                    } else {
                        quote! {
                            if macros_core::convert_to_i64(self.#field_ident) > #val {
                                errors.add(#field_name, format!("{} must be less than or equal to {}",&#field_name,#val));
                            }
                        }
                    }
                } else {
                    return Error::new_spanned(
                        field_name,
                        format!("unsupported type for `max` rule:  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
            }
            Rule::Size(val) => {
                let is_option = is_option_type(field_ty);

                if is_string_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = &self.#field_ident {
                                if value.len() != #val as usize {
                                    errors.add(#field_name, format!("String length must equal {} characters.",&#val));
                                }
                            }
                        }
                    } else {
                        quote! {
                            if self.#field_ident.len() != #val as usize {
                                errors.add(#field_name, format!("String length must equal {} characters.",&#val));
                            }
                        }
                    }
                } else if is_numeric_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = &self.#field_ident {

                                let num_digits = value.to_string().chars().count();
                                if num_digits != #val as usize {
                                    errors.add(#field_name, format!("The entered number must contain exactly {} digits.",#val));
                                }
                            }
                        }
                    } else {
                        quote! {
                            let num_digits = self.#field_ident.to_string().chars().count();
                            if num_digits != #val as usize {
                                errors.add(#field_name, format!("The entered number must contain exactly {} digits.",#val));
                            }
                        }
                    }
                } else {
                    return Error::new_spanned(
                        field_name,
                        format!("invalid data type for size  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
            }
            Rule::StartsWith(prefix) => {
                let is_option = is_option_type(field_ty);

                if is_string_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = &self.#field_ident {
                                if !value.starts_with(#prefix) {
                                    errors.add(#field_name, format!("String must start with :{}",&#prefix));
                                }
                            }
                        }
                    } else {
                        quote! {
                            if !self.#field_ident.starts_with(#prefix) {
                               errors.add(#field_name, format!("String must start with :{}",&#prefix));
                            }
                        }
                    }
                } else {
                    return Error::new_spanned(
                        field_name,
                        format!(" starts_with must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
            }
            Rule::EndsWith(suffix) => {
                let is_option = is_option_type(field_ty);

                if is_string_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = &self.#field_ident {
                                if !value.ends_with(#suffix) {
                                    errors.add(#field_name, format!("String must end with :{}",&#suffix));
                                }
                            }
                        }
                    } else {
                        quote! {
                            if !self.#field_ident.ends_with(#suffix) {
                                 errors.add(#field_name, format!("String must end with :{}",&#suffix));
                            }
                        }
                    }
                } else {
                    return Error::new_spanned(
                        field_name,
                        format!(" ends with must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
            }
            Rule::Ascii => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" ascii must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_ascii(&self.#field_ident) {
                        errors.add(#field_name, format!("Input must contain only ASCII characters. Invalid value: {}",&self.#field_ident));
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_ascii(value) {
                                errors.add(#field_name, format!("Input must contain only ASCII characters. Invalid value: {}",&value));
                            }
                        }
                    }
                }
            }
            Rule::Alphanumeric => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!("alphanumeric must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_ascii_alphanumeric(&self.#field_ident) {
                        errors.add(#field_name, format!("Input must contain only Alphanumeric characters. Invalid value: {}",&self.#field_ident));
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_ascii_alphanumeric(value) {
                                errors.add(#field_name, format!("Input must contain only Alphanumeric characters. Invalid value: {}",&value));
                            }
                        }
                    }
                }
            }
            Rule::HexColor => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" hex_color must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_hex_color(&self.#field_ident) {
                        errors.add(#field_name, format!("The provided value {} is not a valid hexadecimal color code",&self.#field_ident));
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_hex_color(value) {
                                errors.add(#field_name, format!("The provided value {} is not a valid hexadecimal color code",&value));
                            }
                        }
                    }
                }
            }
            Rule::LowerCase => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" lowercase must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                     if &self.#field_ident != &self.#field_ident.to_lowercase() {
                            errors.add(#field_name, format!("The input value {} must be in lowercase",&self.#field_ident));
                     }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if value != &value.to_lowercase() {
                                errors.add(#field_name, format!("The input value {} must be in lowercase",&value));
                            }
                        }
                    }
                }
            }
            Rule::UpperCase => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" uppercase must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                     if &self.#field_ident != &self.#field_ident.to_uppercase() {
                            errors.add(#field_name, format!("The input value {} must be in uppercase",&self.#field_ident));
                     }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if value != &value.to_uppercase() {
                               errors.add(#field_name, format!("The input value {} must be in uppercase",&value));
                            }
                        }
                    }
                }
            }
            Rule::In(values) => {
                if is_option_type(field_ty) {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            let allowed = #values.split(',').collect::<Vec<_>>();
                            if !allowed.contains(&value.as_str()) {
                                errors.add(#field_name, format!("The provided value `{}` must be one of the allowed options: {}",&value,&#values));
                            }
                        }
                    }
                } else {
                    quote! {
                        let allowed = #values.split(',').collect::<Vec<_>>();
                        if !allowed.contains(&self.#field_ident.as_str()) {
                           errors.add(#field_name, format!("The provided value `{}` must be one of the allowed options: {}",&self.#field_ident ,&#values));
                        }
                    }
                }
            }
            Rule::NotIn(values) => {
                if is_option_type(field_ty) {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            let allowed = #values.split(',').collect::<Vec<_>>();
                            if allowed.contains(&value.as_str()) {
                                errors.add(#field_name, format!("The provided value `{}` not must be one of the allowed options: {}",&value,&#values));
                            }
                        }
                    }
                } else {
                    quote! {
                        let allowed = #values.split(',').collect::<Vec<_>>();
                        if allowed.contains(&self.#field_ident.as_str()) {
                           errors.add(#field_name, format!("The provided value `{}` must not be one of the allowed options: {}",&self.#field_ident ,&#values));
                        }
                    }
                }
            }
            Rule::Ip => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" ip must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_ip(&self.#field_ident) {
                        errors.add(#field_name, format!("The provided value {} is not a valid IP address.",&self.#field_ident));
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_ip(value) {
                                errors.add(#field_name, format!("The provided value {} is not a valid IP address.",&value));
                            }
                        }
                    }
                }
            }
            Rule::Date => {
                let is_option = is_option_type(field_ty);
                if !is_string_type(field_ty) && !is_datetime_types(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" date must be string or date  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                if is_string_type(field_ty) {
                    if !is_option {
                        quote! {
                            if !macros_core::is_valid_date(&self.#field_ident) {
                                errors.add(#field_name, format!("The provided value {} is not a valid date. The required format is YYYY-MM-DD.",&self.#field_ident));
                            }
                        }
                    } else {
                        quote! {
                            if let Some(value) = (&self.#field_ident).as_ref() {
                                if !macros_core::is_valid_date(value) {
                                    errors.add(#field_name, format!("The provided value {} is not a valid date. The required format is YYYY-MM-DD.",&value));
                                }
                            }
                        }
                    }
                } else {
                    quote! {}
                }
            }
            Rule::DateTime => {
                let is_option = is_option_type(field_ty);
                if !is_string_type(field_ty) && !is_datetime_types(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" datetime must be string or datetime  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                if is_string_type(field_ty) {
                    if !is_option {
                        quote! {
                            if !macros_core::is_valid_datetime(&self.#field_ident) {
                                errors.add(#field_name, format!("The provided value {} is not a valid date‑time. The required format is YYYY‑MM‑DD HH:MM:SS",&self.#field_ident));
                            }
                        }
                    } else {
                        quote! {
                            if let Some(value) = (&self.#field_ident).as_ref() {
                                if !macros_core::is_valid_datetime(value) {
                                   errors.add(#field_name, format!("The provided value {} is not a valid date‑time. The required format is YYYY‑MM‑DD HH:MM:SS",&value));
                                }
                            }
                        }
                    }
                } else {
                    quote! {}
                }
            }
            Rule::Time => {
                if !is_string_type(field_ty) && !is_time_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" time must be string or time `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    if is_string_type(field_ty) {
                        quote! {
                            if !macros_core::is_valid_time(&self.#field_ident) {
                                errors.add(#field_name, "time");
                            }
                        }
                    } else {
                        quote! {}
                    }
                } else {
                    if is_string_type(field_ty) {
                        quote! {
                            if let Some(value) = (&self.#field_ident).as_ref() {
                                if !macros_core::is_valid_time(value) {
                                    errors.add(#field_name, "time");
                                }
                            }
                        }
                    } else {
                        quote! {}
                    }
                }
            }
            Rule::After(date) => {
                // check is date|time or string
                if !is_string_type(field_ty) && !is_datetime_types(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!("date must be string or date|time `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                // non option dates
                if !is_option {
                    // compare string date
                    if is_string_type(field_ty) {
                        quote! {
                        match macros_core::is_after_option(&self.#field_ident, #date) {
                                    None => errors.add(#field_name, format!("Date|time formats invalid: {},{}",&self.#field_ident,&#date)),
                                    Some(true) => (),
                                    Some(false) => errors.add(#field_name,format!("The provided value {} must be later than {}",
                                    &self.#field_ident, &#date)),
                                }
                        }
                    } else {
                        // compare datetime
                        if is_datetime_type(field_ty) {
                            quote! {
                                match macros_core::is_after_option_datetime_ex(self.#field_ident, #date) {
                                        None => errors.add(#field_name, format!("Datetime formats invalid: {},{}",&self.#field_ident,&#date.to_string())),
                                        Some(true) => (),
                                        Some(false) => errors.add(#field_name,
                                        format!("The provided value {} must be later than {}",
                                            &self.#field_ident.to_string(), &#date)),
                                }
                            }
                            // compare date
                        } else if is_date_type(field_ty) {
                            quote! {
                                match macros_core::is_after_option_date_ex(self.#field_ident, #date) {
                                        None => errors.add(#field_name, format!("Date formats invalid: {},{}",&self.#field_ident,&#date.to_string())),
                                        Some(true) => (),
                                        Some(false) => errors.add(#field_name,
                                        format!("The provided value {} must be later than {}",
                                            &self.#field_ident.to_string(),& #date)),
                               }
                            }
                        } else {
                            quote! {}
                        }
                    }
                } else {
                    if is_string_type(field_ty) {
                        // compare strings
                        quote! {
                            if let Some(value) = (&self.#field_ident).as_ref() {
                                match macros_core::is_after_option(value, #date) {
                                    None => errors.add(#field_name, format!("Date|time formats invalid: {},{}",&value,&#date)),
                                    Some(true) => (),
                                    Some(false) => errors.add(#field_name,
                                        format!("The provided value {} must be later than {}"
                                            ,&value.to_string(), &#date)),
                                }
                            }
                        }
                    } else {
                        // compare date time
                        if is_datetime_type(field_ty) {
                            quote! {
                            if let Some(value) = (self.#field_ident).as_ref() {
                                match macros_core::is_after_option_datetime_ex(value, #date) {
                                    None => errors.add(#field_name, format!("Datetime formats invalid: {},{}",&value,&#date)),
                                    Some(true) => (),
                                    Some(false)  => errors.add(#field_name,
                                        format!("The provided value {} must be later than {}",
                                            &value, &#date)),
                                }
                            }
                            }
                        } else if is_date_type(field_ty) {
                            quote! {
                            if let Some(value) = (self.#field_ident).as_ref() {
                                match macros_core::is_after_option_date_ex(value, #date) {
                                    None => errors.add(#field_name, format!("Date formats invalid: {},{}",&value,&#date)),
                                    Some(true) => (),
                                    Some(false)  => errors.add(#field_name,
                                        format!("The provided value {} must be later than {}",
                                            &value.to_string(), &#date)),
                                }
                            }
                            }
                        } else {
                            quote! {}
                        }
                    }
                }
            }
            Rule::Before(date) => {
                // check is date|time or string
                if !is_string_type(field_ty) && !is_datetime_types(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!("date must be string or date|time `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                // non option dates
                if !is_option {
                    // compare string date
                    if is_string_type(field_ty) {
                        quote! {
                        match macros_core::is_before_option(&self.#field_ident, #date) {
                                    None => errors.add(#field_name, format!("Date|time formats invalid: {},{}",&self.#field_ident,&#date)),
                                    Some(true) => (),
                                    Some(false) => errors.add(#field_name,format!("The provided value {} must be earlier than {}",
                                    &self.#field_ident, &#date)),
                                }
                        }
                    } else {
                        // compare datetime
                        if is_datetime_type(field_ty) {
                            quote! {
                                match macros_core::is_before_option_datetime_ex(self.#field_ident, #date) {
                                        None => errors.add(#field_name, format!("Datetime formats invalid: {},{}",&self.#field_ident,&#date.to_string())),
                                        Some(true) => (),
                                        Some(false) => errors.add(#field_name,
                                        format!("The provided value {} must be earlier than {}",
                                            &self.#field_ident.to_string(), &#date)),
                                }
                            }
                            // compare date
                        } else if is_date_type(field_ty) {
                            quote! {
                                match macros_core::is_before_option_date_ex(self.#field_ident, #date) {
                                        None => errors.add(#field_name, format!("Date formats invalid: {},{}",&self.#field_ident,&#date.to_string())),
                                        Some(true) => (),
                                        Some(false) => errors.add(#field_name,
                                        format!("The provided value {} must be earlier than {}",
                                            &self.#field_ident.to_string(),& #date)),
                               }
                            }
                        } else {
                            quote! {}
                        }
                    }
                } else {
                    if is_string_type(field_ty) {
                        // compare strings
                        quote! {
                            if let Some(value) = (&self.#field_ident).as_ref() {
                                match macros_core::is_before_option(value, #date) {
                                    None => errors.add(#field_name, format!("Date|time formats invalid: {},{}",&value,&#date)),
                                    Some(true) => (),
                                    Some(false) => errors.add(#field_name,
                                        format!("The provided value {} must be earlier than {}"
                                            ,&value.to_string(), &#date)),
                                }
                            }
                        }
                    } else {
                        // compare date time
                        if is_datetime_type(field_ty) {
                            quote! {
                            if let Some(value) = (self.#field_ident).as_ref() {
                                match macros_core::is_before_option_datetime_ex(value, #date) {
                                    None => errors.add(#field_name, format!("Datetime formats invalid: {},{}",&value,&#date)),
                                    Some(true) => (),
                                    Some(false)  => errors.add(#field_name,
                                        format!("The provided value {} must be earlier than {}",
                                            &value, &#date)),
                                }
                            }
                            }
                        } else if is_date_type(field_ty) {
                            quote! {
                            if let Some(value) = (self.#field_ident).as_ref() {
                                match macros_core::is_before    _option_date_ex(value, #date) {
                                    None => errors.add(#field_name, format!("Date formats invalid: {},{}",&value,&#date)),
                                    Some(true) => (),
                                    Some(false)  => errors.add(#field_name,
                                        format!("The provided value {} must be earlier than {}",
                                            &value.to_string(), &#date)),
                                }
                            }
                            }
                        } else {
                            quote! {}
                        }
                    }
                }
            }
            Rule::Json => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" json must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_json(&self.#field_ident) {
                        errors.add(#field_name, "json");
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_json(value) {
                                errors.add(#field_name, "json");
                            }
                        }
                    }
                }
            }
            Rule::Array => {
                if !is_string_collection_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(
                            " Array must be Vec<String> or HashMap<String, String>  `{}`",
                            field_name
                        ),
                    )
                    .to_compile_error()
                    .into();
                }
                quote! {}
            }
            Rule::Exists(meta) => {
                let mut tokens = TokenStream::new();
                let mut wanted_token = TokenStream::new();

                let db = meta.split(',').collect::<Vec<_>>();
                if db.len() != 2 {
                    return Error::new_spanned(
                        field_name,
                        format!("Exists must have to value like: table,column. Exm: users,email. Your meta: {} ", &meta),
                    )
                        .to_compile_error()
                        .into();
                }

                let table = db.get(0).unwrap().to_string();
                let column = db.get(1).unwrap().to_string();

                let is_option = is_option_type(field_ty);
                if is_string_type(field_ty) {
                    if is_option {
                        wanted_token = quote! {
                            let mut wanted = "";
                            if let Some(value) = self.#field_ident{
                                wanted = value;
                            }
                        }
                        .into();
                    } else {
                        wanted_token = quote! {
                            let wanted = &self.#field_ident.clone();
                        }
                        .into();
                    }
                } else if is_numeric_type(field_ty) {
                    if is_option {
                        wanted_token = quote! {
                            let mut wanted = "";
                            if let Some(value) = self.#field_ident{
                                wanted = value.to_string();
                            }
                        }
                        .into();
                    } else {
                        wanted_token = quote! {
                            let wanted = &self.#field_ident.to_string();
                        }
                        .into();
                    }
                } else {
                    return Error::new_spanned(
                        field_name,
                        format!(
                            "unsupported type requried numberic or string `{}`",
                            field_name
                        ),
                    )
                    .to_compile_error()
                    .into();
                }

                let db_token: TokenStream = quote! {
                    if !rustavel_core::db::get_static_schema()
                    .await.exists_record(#table, #column, wanted).await {
                        errors.add(#field_name, format!("The record not exists: {}", wanted));
                    }
                }
                .into();

                tokens.extend(wanted_token);
                tokens.extend(db_token);
                tokens.into()

            }
            Rule::Unique(meta) => {
                let mut tokens = TokenStream::new();
                let mut wanted_token2 = TokenStream::new();

                let db = meta.split(',').collect::<Vec<_>>();
                let db_len = db.len();
                if db_len < 2  || db_len > 3  {
                    return Error::new_spanned(
                        field_name,
                        format!("Unique must have to value like: table,column or table,column,except_field. Exm: users,email or users,email,id . Your meta: {} ", &meta),
                    ).to_compile_error()
                        .into();
                }
                let table = db.get(0).unwrap().to_string();
                let column = db.get(1).unwrap().to_string();


                let is_option = is_option_type(field_ty);
                if is_string_type(field_ty) {
                    if is_option {
                        wanted_token2 = quote! {
                            let mut wanted = "";
                            if let Some(value) = self.#field_ident{
                                wanted = value;
                            }
                        }
                            .into();
                    } else {
                        wanted_token2 = quote! {
                            let wanted = &self.#field_ident.clone();
                        }
                            .into();
                    }
                } else if is_numeric_type(field_ty) {
                    if is_option {
                        wanted_token2 = quote! {
                            let mut wanted = "";
                            if let Some(value) = self.#field_ident{
                                wanted = value.to_string();
                            }
                        }
                            .into();
                    } else {
                        wanted_token2 = quote! {
                            let wanted = &self.#field_ident.to_string();
                        }
                            .into();
                    }
                } else {
                    return Error::new_spanned(
                        field_name,
                        format!(
                            "unsupported type requried numberic or string `{}`",
                            field_name
                        ),
                    )
                        .to_compile_error()
                        .into();
                }

                let mut db_token: TokenStream = TokenStream::new();
                if db_len == 2  {
                    db_token = quote!{
                        if rustavel_core::db::get_static_schema()
                        .await.exists_record(#table, #column, wanted).await {
                            errors.add(#field_name, format!("The record exists: {}", wanted));
                        }
                    }.into();
                } else if db_len == 3 {

                    let except_val =  db.get(2).unwrap().to_string();
                    let except = format_ident!("{}",&except_val);
                    db_token = quote!{
                        if rustavel_core::db::get_static_schema()
                        .await.exists_record_except(#table, #column, wanted, #except_val,&macros_core::convert_to_string(&self.#except)).await {
                            errors.add(#field_name, format!("The record exists: {}", wanted));
                        }
                    }.into();


                }


                tokens.extend(wanted_token2);
                tokens.extend(db_token);
                tokens.into()
            }
            _ => quote! {},
        }
    }
}
