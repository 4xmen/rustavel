// DEPRECATED: Validation Macro Anti-Pattern
//
// This macro represents an anti-pattern in Rust due to several critical design flaws:
//
// 1. Excessive Runtime Type Checking: Relying on `Any` and `downcast_ref` breaks Rust's
//    compile-time type safety guarantees, introducing potential runtime overhead and
//    type-related vulnerabilities.
//
// 2. Reflection-like Behavior: The approach mimics runtime reflection, which is
//    fundamentally against Rust's zero-cost abstraction principles and type system
//    strengths.
//
// 3. Performance Overhead: Multiple `downcast_ref` calls for each field create
//    unnecessary runtime type checking and potential performance penalties.
//

use regex::Regex;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::marker::PhantomData;

//
// ──────────────────────────────────────────────────────────────
//                         Error Types
// ──────────────────────────────────────────────────────────────
//

/// Represents a single validation error for one field.
///
/// This enum is intentionally explicit so error reporting
/// can later be localized or serialized cleanly.
#[derive(Debug, Clone)]
pub enum FieldError {
    Required,
    Email,
    MinLength(usize),
    MaxLength(usize),
    MinValue(i64),
    MaxValue(i64),
    Confirmed { other: String },
}

/// A collection of validation errors indexed by field name.
///
/// This struct allows storing multiple validation errors for different fields.
/// Each field can have multiple validation errors associated with it.
///
/// # Example
/// ```
/// # use rustavel_core::http::request_validator::{FieldError,ValidationErrors};
/// let mut errors = ValidationErrors::new();
/// errors.add("email", FieldError::Required);
/// errors.add("email", FieldError::Email);
/// errors.add("password", FieldError::MinLength(8));
/// ```
#[derive(Debug, Default)]
pub struct ValidationErrors {
    /// A HashMap storing validation errors, where:
    /// - Key is the field name
    /// - Value is a vector of validation errors for that field
    pub errors: HashMap<String, Vec<FieldError>>,
}

impl ValidationErrors {
    /// Creates a new, empty ValidationErrors collection.
    ///
    /// # Returns
    /// An empty ValidationErrors instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Checks if there are no validation errors.
    ///
    /// # Returns
    /// `true` if no errors exist, `false` otherwise
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Adds a validation error for a specific field.
    ///
    /// # Arguments
    /// * `field` - The name of the field with the validation error
    /// * `error` - The specific validation error to add
    ///
    /// # Example
    /// ```
    /// # use rustavel_core::http::request_validator::{FieldError,ValidationErrors};
    /// let mut errors = ValidationErrors::new();
    /// errors.add("email", FieldError::Required);
    /// ```
    pub fn add(&mut self, field: impl Into<String>, error: FieldError) {
        // Get or create the error list for the field, then add the new error
        self.errors.entry(field.into()).or_default().push(error);
    }
}

//
// ──────────────────────────────────────────────────────────────
//                     Rule Representation
// ──────────────────────────────────────────────────────────────
//

/// A parsed validation rule extracted from a rule string.
///
/// Examples:
/// - "required"
/// - "min:8"
/// - "confirmed:password_confirmation"
#[derive(Debug, Clone)]
pub struct Rule {
    pub name: String,
    pub param: Option<String>,
}

/// Parses a rule string like:
/// "required|min:8|max:128|confirmed:password_confirmation"
///
/// into a vector of `Rule` structs.
///
/// This function is intentionally simple and deterministic.
/// No allocations happen during validation itself.
pub fn parse_rules(input: &str) -> Vec<Rule> {
    input
        .split('|')
        .filter(|s| !s.trim().is_empty())
        .map(|raw| {
            let mut parts = raw.splitn(2, ':');
            Rule {
                name: parts.next().unwrap().to_string(),
                param: parts.next().map(str::to_string),
            }
        })
        .collect()
}

//
// ──────────────────────────────────────────────────────────────
//                   Rule Application Helpers
// ──────────────────────────────────────────────────────────────
//
// These helpers are deliberately type-specific.
// This avoids Any, avoids downcasting, and keeps the compiler happy.
//

pub fn validate_required_string(field: &str, value: &String, errors: &mut ValidationErrors) {
    if value.trim().is_empty() {
        errors.add(field, FieldError::Required);
    }
}

pub fn validate_required_option_string(
    field: &str,
    value: &Option<String>,
    errors: &mut ValidationErrors,
) {
    if value.as_ref().map_or(true, |s| s.trim().is_empty()) {
        errors.add(field, FieldError::Required);
    }
}

pub fn validate_email(field: &str, value: &String, errors: &mut ValidationErrors) {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

    if !email_regex.is_match(value) {
        errors.add(field, FieldError::Email);
    }
}

pub fn validate_string_length(
    field: &str,
    value: &String,
    rule: &Rule,
    errors: &mut ValidationErrors,
) {
    let limit: usize = match rule.param.as_deref().and_then(|p| p.parse().ok()) {
        Some(v) => v,
        None => return,
    };

    let len = value.len();

    match rule.name.as_str() {
        "min" if len < limit => errors.add(field, FieldError::MinLength(limit)),
        "max" if len > limit => errors.add(field, FieldError::MaxLength(limit)),
        _ => {}
    }
}

pub fn validate_numeric_range<T>(field: &str, value: T, rule: &Rule, errors: &mut ValidationErrors)
where
    T: Into<i64> + Copy,
{
    let limit: i64 = match rule.param.as_deref().and_then(|p| p.parse().ok()) {
        Some(v) => v,
        None => return,
    };

    let value = value.into();

    match rule.name.as_str() {
        "min" if value < limit => errors.add(field, FieldError::MinValue(limit)),
        "max" if value > limit => errors.add(field, FieldError::MaxValue(limit)),
        _ => {}
    }
}

//
// ──────────────────────────────────────────────────────────────
//                         Macro
// ──────────────────────────────────────────────────────────────
//
// This macro generates a `validate()` method for the target struct.
//
// Key design goals:
//
// 1. Some runtime type checking using Any (can be replaced by Flexible wrappers)
// 2. All field access is compile-time checked
// 3. Works in async contexts (Axum, Tokio, etc.)
// 4. Clear, explicit, debuggable expansion

#[macro_export]
macro_rules! derive_validatable {
    (
        $struct_name:ident {
            $(
                $field_name:ident : $field_type:ty => $rules:literal
            ),* $(,)?
        }
    ) => {
        impl $struct_name {
            /// Validate the struct according to the declared rules.
            ///
            /// This method is synchronous, allocation-light,
            /// and safe to call inside async handlers.
            pub fn validate(&self) -> Result<(), ValidationErrors> {
                let mut errors = ValidationErrors::new();

                $(
                    // Parse rules once per field.
                    let rules = parse_rules($rules);

                    for rule in rules {
                        match rule.name.as_str() {
                            // ─────────────────────────────────────
                            // required
                            // ─────────────────────────────────────
                            "required" => {
                                // String
                                if let Some(v) = (&self.$field_name as &dyn std::any::Any)
                                    .downcast_ref::<String>()
                                {
                                    validate_required_string(
                                        stringify!($field_name),
                                        v,
                                        &mut errors,
                                    );
                                }

                                // Option<String>
                                if let Some(v) = (&self.$field_name as &dyn std::any::Any)
                                    .downcast_ref::<Option<String>>()
                                {
                                    validate_required_option_string(
                                        stringify!($field_name),
                                        v,
                                        &mut errors,
                                    );
                                }
                            }

                            // ─────────────────────────────────────
                            // email
                            // ─────────────────────────────────────
                            "email" => {
                                if let Some(v) = (&self.$field_name as &dyn std::any::Any)
                                    .downcast_ref::<String>()
                                {
                                    validate_email(
                                        stringify!($field_name),
                                        v,
                                        &mut errors,
                                    );
                                }
                            }

                            // ─────────────────────────────────────
                            // min / max
                            // ─────────────────────────────────────
                            "min" | "max" => {
                                if let Some(v) = (&self.$field_name as &dyn std::any::Any)
                                    .downcast_ref::<String>()
                                {
                                    validate_string_length(
                                        stringify!($field_name),
                                        v,
                                        &rule,
                                        &mut errors,
                                    );
                                }

                                if let Some(v) = (&self.$field_name as &dyn std::any::Any)
                                    .downcast_ref::<u16>()
                                {
                                    validate_numeric_range(
                                        stringify!($field_name),
                                        *v,
                                        &rule,
                                        &mut errors,
                                    );
                                }
                            }

                            // ─────────────────────────────────────
                            // confirmed (cross-field validation)
                            // ─────────────────────────────────────
                            "confirmed" => {
                                if let Some(other) = &rule.param {
                                    // confirmed only makes sense for String fields
                                    let current = match (&self.$field_name as &dyn std::any::Any)
                                        .downcast_ref::<String>()
                                    {
                                        Some(v) => v,
                                        None => continue, // Skip non-string fields safely
                                    };

                                    let confirmed_value: &String = match other.as_str() {
                                        "password_confirmation" => &self.password_confirmation,
                                        _ => continue,
                                    };

                                    if current != confirmed_value {
                                        errors.add(
                                            stringify!($field_name),
                                            FieldError::Confirmed {
                                                other: other.clone(),
                                            },
                                        );
                                    }
                                }
                            }


                            _ => {}
                        }
                    }
                )*

                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(errors)
                }
            }
        }
    };
}

//
// ──────────────────────────────────────────────────────────────
// 1. Core Traits
// ──────────────────────────────────────────────────────────────
//

/// A trait for numeric types that can be safely constructed
/// from either numbers or strings during deserialization.
///
/// This keeps the struct type-safe while allowing tolerant input.
pub trait FlexibleNumber: Sized {
    fn from_i128(n: i128) -> Option<Self>;
    fn from_f64(n: f64) -> Option<Self>;
    fn from_str(s: &str) -> Option<Self>;
}

//
// ──────────────────────────────────────────────────────────────
// 2. Macro: implement FlexibleNumber for primitives
// ──────────────────────────────────────────────────────────────
//
/// This macro is intentionally boring and readable.
/// No magic, no recursion, no macro hell.
///
/// It generates correct and explicit conversions.
macro_rules! impl_flexible_number {
    (
        ints: [$($int:ty),* $(,)?],
        floats: [$($float:ty),* $(,)?]
    ) => {
        $(
            impl FlexibleNumber for $int {
                fn from_i128(n: i128) -> Option<Self> {
                    <$int>::try_from(n).ok()
                }

                fn from_f64(n: f64) -> Option<Self> {
                    if n.fract() == 0.0 {
                        <$int>::try_from(n as i128).ok()
                    } else {
                        None
                    }
                }

                fn from_str(s: &str) -> Option<Self> {
                    s.parse::<$int>().ok()
                }
            }
        )*

        $(
            impl FlexibleNumber for $float {
                fn from_i128(n: i128) -> Option<Self> {
                    Some(n as $float)
                }

                fn from_f64(n: f64) -> Option<Self> {
                    Some(n as $float)
                }

                fn from_str(s: &str) -> Option<Self> {
                    s.parse::<$float>().ok()
                }
            }
        )*
    };
}

impl_flexible_number!(
    ints: [
        i8, i16, i32, i64, i128,
        u8, u16, u32, u64, u128
    ],
    floats: [
        f32, f64
    ]
);

//
// ──────────────────────────────────────────────────────────────
// 3. Generic Flexible<T> wrapper
// ──────────────────────────────────────────────────────────────
//

/// A wrapper that allows deserializing a value from:
/// - number
/// - numeric string
///
/// while keeping `T` fully type-safe after deserialization.
#[derive(Debug, Clone)]
pub struct Flexible<T>(pub T);

impl<'de, T> Deserialize<'de> for Flexible<T>
where
    T: FlexibleNumber,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor<T>(PhantomData<T>);

        impl<'de, T> serde::de::Visitor<'de> for Visitor<T>
        where
            T: FlexibleNumber,
        {
            type Value = Flexible<T>;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("a number or a string containing a number")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                T::from_i128(v as i128)
                    .map(Flexible)
                    .ok_or_else(|| E::custom("numeric overflow"))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                T::from_i128(v as i128)
                    .map(Flexible)
                    .ok_or_else(|| E::custom("numeric overflow"))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                T::from_f64(v)
                    .map(Flexible)
                    .ok_or_else(|| E::custom("invalid floating-point value"))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                T::from_str(v)
                    .map(Flexible)
                    .ok_or_else(|| E::custom("invalid numeric string"))
            }
        }

        deserializer.deserialize_any(Visitor(PhantomData))
    }
}

//
// ──────────────────────────────────────────────────────────────
// 4. Flexible Boolean
// ──────────────────────────────────────────────────────────────
//

/// Boolean deserializer tolerant to:
/// - true / false
/// - "true" / "false"
/// - "1" / "0"
/// - 1 / 0
#[derive(Debug, Clone)]
pub struct FlexibleBool(pub bool);

impl<'de> Deserialize<'de> for FlexibleBool {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = FlexibleBool;

            fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.write_str("bool, string, or numeric boolean")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> {
                Ok(FlexibleBool(v))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(FlexibleBool(v != 0))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                match v {
                    "true" | "1" => Ok(FlexibleBool(true)),
                    "false" | "0" => Ok(FlexibleBool(false)),
                    _ => Err(E::custom("invalid boolean value")),
                }
            }
        }

        deserializer.deserialize_any(Visitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct RegisterPayload {
        email: String,
        password: String,
        password_confirmation: String, // For 'confirmed' example
        age: u16,
        bio: Option<String>,
    }

    derive_validatable! {
        RegisterPayload {
            email:     String       => "required|email|max:180",
            password:  String       => "required|min:8|max:128",
            password:  String       => "required|min:8|max:128|confirmed:password_confirmation",
            age:       u16          => "required|min:13|max:120",
            bio:       Option<String> => "min:10|max:500",
        }
    }
    #[test]
    fn test_invalid_registration_payload() {
        let input = RegisterPayload {
            email: "not-email".to_string(),
            password: "123".to_string(),
            password_confirmation: "123".to_string(),
            age: 11,
            bio: Some("short".to_string()),
        };

        let validation_result = input.validate();
        assert!(validation_result.is_err());

        if let Err(errors) = validation_result {

            // Specific error checks
            assert!(errors.errors.contains_key("email"), "Should have email validation error");
            assert!(errors.errors.contains_key("password"), "Should have password validation error");
            assert!(errors.errors.contains_key("age"), "Should have age validation error");
        }
    }


}
