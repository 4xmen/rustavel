// ============================================================================
// GRAVEYARD – Attribute-based #[derive(Resource)]
//
// This design is intentionally abandoned.
//
// Why it was rejected
// -------------------
// The implementation itself was sound: compile-time safety, coherence-
// friendly container handling, pure infallible mapping, and clear separation
// from serialization/authorization all worked as intended. Rust was not the
// problem.
//
// The real issues were:
//
// 1. Developer Experience (DX)
//    Mapping logic was scattered across struct-level and field-level
//    attributes (`#[source]`, `#[transform]`, `#[compute]`). Readers had to
//    reconstruct the full conversion by hunting through attributes instead of
//    seeing a single, linear declaration. The overloaded meaning of `#[source]`
//    (type on the struct, field name on fields) added unnecessary cognitive
//    load.
//
// 2. Future bottlenecks (solvable, but real)
//    - Transforms were hard-wired to by-value, forcing clones for read-only
//      cases and making multi-field use of the same source field painful.
//    - No first-class support for `&Source` in the generated impls, which
//      collides with the common pattern of holding only a reference in
//      handlers and services.
//    These were not fundamental limitations of the philosophy; they were
//    consequences of the attribute-driven surface and could have been fixed.
//    Still, they would have become friction points as soon as the system left
//    the pure research examples.
//
// Decision
// --------
// Keep the underlying contract (`FromResource`, structural Vec/Option
// handling, purity rules). Replace the attribute DSL with a focused
// declarative mapping:
//
//     resource! {
//         TodoResource from Todo {
//             id,
//             name = title,
//             created_at = format_date(created_at),
//             is_overdue = is_overdue(&this),
//         }
//     }
//
// The Resource remains an ordinary Rust struct (so serde, docs, and other
// derives keep working). Mapping becomes a single readable block, transform
// and compute collapse into ordinary expressions, and ownership/borrow
// conflicts are left to the compiler instead of being papered over by the
// macro.
//
// This version is preserved only for historical and comparative study.
// ============================================================================





/// Derives a compile-time mapping from a source struct to this resource.
///
/// # Attributes
///
/// |      Attribute    | Position | Meaning |
/// |-------------------|----------|------------------------------------------------------------|
/// | `#[source(Path)]` |  struct  | The source type this resource is built from. **Required.** |
/// | `#[source(name)]` |  field   | Read the value from a differently named source field.        |
/// | `#[transform(f)]` |  field   | Convert one field's value with `fn f(SourceFieldTy) -> FieldTy`. |
/// | `#[compute(f)]`   |  field   | Derive the value from the whole source with `fn f(&Source) -> FieldTy`. |
///
/// # Generated items
///
/// * `impl macros_core::FromResource<Source> for TheResource` — the mapping.
/// * `impl From<Source> for TheResource` — so `TheResource::from(x)` and
///   `x.into()` work as shown in the design document.
/// * `TheResource::from_source(x)` and `TheResource::collection(iter)` —
///   inherent methods, which give better error messages than trait calls.
///
/// # Example
///
/// ```ignore
/// #[derive(Resource)]
/// #[source(Todo)]
/// struct TodoResource {
///     id: i64,
///     #[source(title)]
///     name: String,
///     #[transform(format_date)]
///     created_at: String,
///     #[compute(is_overdue)]
///     is_overdue: bool,
/// }
/// ```
#[proc_macro_derive(Resource, attributes(source, transform, compute))]
pub fn derive_resource(input: TokenStream) -> TokenStream {
    // `parse_macro_input!` already emits a `compile_error!` on malformed input,
    // so from here on we only deal with *semantic* errors (missing `#[source]`,
    // conflicting attributes, unsupported shapes, ...).
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    resource::derive(input)
        .unwrap_or_else(Error::into_compile_error)
        .into()
}
