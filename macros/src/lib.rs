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
use quote::quote;
use quote::spanned::Spanned;
use std::collections::HashSet;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Attribute, DeriveInput, Field, LitStr, parse_macro_input};
use syn::{Error, Result};
use syn::{GenericArgument, PathArguments, Type, TypePath};

// Define an enum for the validation rules
// This enum represents all supported rules, making it easy to extend later by adding new variants
// Each variant can hold parameters if needed (e.g., Min holds the min value as u32)
#[derive(Debug, Clone)]
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

                 println!("{:?}", self);

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
            println!("{}",seg.ident);
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

// Implement a way to display the rule for testing purposes
impl Rule {
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

            Rule::Email => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" email must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_email(&self.#field_ident) {
                        errors.add(#field_name, "email");
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_email(value) {
                                errors.add(#field_name, "email");
                            }
                        }
                    }
                }
            }
            Rule::Url => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" url must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_url(&self.#field_ident) {
                        errors.add(#field_name, "url");
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_url(value) {
                                errors.add(#field_name, "url");
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
                                    errors.add(#field_name, "min");
                                }
                            }
                        }
                    } else {
                        quote! {
                            if self.#field_ident.len() < #val as usize {
                                errors.add(#field_name, "min");
                            }
                        }
                    }
                } else if is_numeric_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = self.#field_ident {
                                if value < #val {
                                    errors.add(#field_name, "min");
                                }
                            }
                        }
                    } else {
                        quote! {
                            if self.#field_ident < #val {
                                errors.add(#field_name, "min");
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
                                    errors.add(#field_name, "max");
                                }
                            }
                        }
                    } else {
                        quote! {
                            if self.#field_ident.len() > #val as usize {
                                errors.add(#field_name, "max");
                            }
                        }
                    }
                } else if is_numeric_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = self.#field_ident {
                                if value < #val {
                                    errors.add(#field_name, "max");
                                }
                            }
                        }
                    } else {
                        quote! {
                            if self.#field_ident > #val {
                                errors.add(#field_name, "max");
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
                                    errors.add(#field_name, "size");
                                }
                            }
                        }
                    } else {
                        quote! {
                            if self.#field_ident.len() != #val as usize {
                                errors.add(#field_name, "size");
                            }
                        }
                    }
                } else if is_numeric_type(field_ty) {
                    if is_option {
                        quote! {
                            if let Some(value) = &self.#field_ident {
                                if *value != #val {
                                    errors.add(#field_name, "size");
                                }
                            }
                        }
                    } else {
                        quote! {
                            if self.#field_ident != #val {
                                errors.add(#field_name, "size");
                            }
                        }
                    }
                } else {
                    return Error::new_spanned(
                        field_name,
                        format!(" invalid data type for size  `{}`", field_name),
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
                                    errors.add(#field_name, "starts_with");
                                }
                            }
                        }
                    } else {
                        quote! {
                            if !self.#field_ident.starts_with(#prefix) {
                                errors.add(#field_name, "starts_with");
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
                                    errors.add(#field_name, "ends_with");
                                }
                            }
                        }
                    } else {
                        quote! {
                            if !self.#field_ident.ends_with(#suffix) {
                                errors.add(#field_name, "ends_with");
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
                        errors.add(#field_name, "ascii");
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_ascii(value) {
                                errors.add(#field_name, "ascii");
                            }
                        }
                    }
                }
            }
            Rule::Alphanumeric => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" alphanumeric must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_ascii_alphanumeric(&self.#field_ident) {
                        errors.add(#field_name, "alphanumeric");
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_ascii_alphanumeric(value) {
                                errors.add(#field_name, "alphanumeric");
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
                        errors.add(#field_name, "hex_color");
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_hex_color(value) {
                                errors.add(#field_name, "hex_color");
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
                            errors.add(#field_name, "lowercase");
                     }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if value != &value.to_lowercase() {
                                errors.add(#field_name, "lowercase");
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
                            errors.add(#field_name, "uppercase");
                     }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if value != &value.to_uppercase() {
                                errors.add(#field_name, "uppercase");
                            }
                        }
                    }
                }
            }
            // Rule::In(values) => {
            //     quote! {
            //         if let Some(value) = (&self.#field_ident).as_ref() {
            //             let allowed = #values.split(',').collect::<Vec<_>>();
            //             if !allowed.contains(&value.as_str()) {
            //                 errors.add(#field_name, "in");
            //             }
            //         }
            //     }
            // }
            // Rule::NotIn(values) => {
            //     quote! {
            //         if let Some(value) = (&self.#field_ident).as_ref() {
            //             let blocked = #values.split(',').collect::<Vec<_>>();
            //             if blocked.contains(&value.as_str()) {
            //                 errors.add(#field_name, "not_in");
            //             }
            //         }
            //     }
            // }
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
                        errors.add(#field_name, "ip");
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_ip(value) {
                                errors.add(#field_name, "ip");
                            }
                        }
                    }
                }
            }
            Rule::Date => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" date must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    if !macros_core::is_valid_date(&self.#field_ident) {
                        errors.add(#field_name, "date");
                    }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_date(value) {
                                errors.add(#field_name, "date");
                            }
                        }
                    }
                }
            }
            Rule::DateTime => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" datetime must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                        if !macros_core::is_valid_datetime(&self.#field_ident) {
                            errors.add(#field_name, "datetime");
                        }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_datetime(value) {
                                errors.add(#field_name, "datetime");
                            }
                        }
                    }
                }
            }
            Rule::Time => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" time must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                        if !macros_core::is_valid_time(&self.#field_ident) {
                            errors.add(#field_name, "time");
                        }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            if !macros_core::is_valid_time(value) {
                                errors.add(#field_name, "time");
                            }
                        }
                    }
                }
            }

            Rule::After(date) => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" date must be string  `{}`", field_name),
                    )
                    .to_compile_error()
                    .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    match macros_core::is_after_option(&self.#field_ident, #date) {
                                None => errors.add(#field_name, "date"),
                                Some(true) => (),
                                Some(false) => errors.add(#field_name, "after"),
                            }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            match macros_core::is_after_option(value, #date) {
                                None => errors.add(#field_name, "date"),
                                Some(true) => (),
                                Some(false) => errors.add(#field_name, "after"),
                         }
                        }
                    }
                }
            }

            Rule::Before(date) => {
                if !is_string_type(field_ty) {
                    return Error::new_spanned(
                        field_name,
                        format!(" date must be string  `{}`", field_name),
                    )
                        .to_compile_error()
                        .into();
                }
                let is_option = is_option_type(field_ty);
                if !is_option {
                    quote! {
                    match macros_core::is_before_option(&self.#field_ident, #date) {
                                None => errors.add(#field_name, "date"),
                                Some(true) => (),
                                Some(false) => errors.add(#field_name, "after"),
                            }
                    }
                } else {
                    quote! {
                        if let Some(value) = (&self.#field_ident).as_ref() {
                            match macros_core::is_before_option(value, #date) {
                                None => errors.add(#field_name, "date"),
                                Some(true) => (),
                                Some(false) => errors.add(#field_name, "before"),
                         }
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
            _ => quote! {},
        }
    }
}
