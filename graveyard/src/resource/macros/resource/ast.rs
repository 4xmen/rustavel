// ============================================================================
// File: macros/src/resource/ast.rs
//
// Responsibility
//   Phase 1 of the derive: turn the raw `syn::DeriveInput` into a small,
//   fully-validated intermediate representation (IR) that the code generator
//   can consume without ever having to ask "is this valid?" again.
//
//   Two ideas drive the design of this file:
//
//   * Parse, don't validate later. Once `ResourceInput::parse` returns `Ok`,
//     every field carries exactly one resolved strategy (map / transform /
//     compute). `expand.rs` therefore contains no error handling at all — it is
//     a pure function from IR to tokens.
//
//   * Fail loudly and early, with good spans. A proc macro is a compiler
//     extension; its diagnostics are the only thing standing between the user
//     and a wall of confusing generated-code errors.
// ============================================================================

use super::Errors;

/// A fully parsed and validated `#[derive(Resource)]` input.
pub(crate) struct ResourceInput {
    /// Name of the resource struct, e.g. `TodoResource`.
    pub(crate) ident: syn::Ident,
    /// Generics of the resource struct, forwarded verbatim to every generated
    /// `impl` block so generic resources keep working.
    pub(crate) generics: syn::Generics,
    /// Path to the source type, taken from the struct-level `#[source(...)]`.
    pub(crate) source: syn::Path,
    /// One entry per named field, in declaration order.
    pub(crate) fields: Vec<ResourceField>,
}

/// One field of the resource together with how its value is produced.
pub(crate) struct ResourceField {
    /// Field name on the *resource* (this is also the JSON key by default).
    pub(crate) ident: syn::Ident,
    /// Field type as written by the user. `expand` inspects the written form to
    /// recognise `Vec<_>` / `Option<_>`; see `ty.rs` for why that is sound.
    pub(crate) ty: syn::Type,
    /// How this field's value is obtained from the source.
    pub(crate) strategy: FieldStrategy,
}

/// The three mutually exclusive ways a resource field can be filled.
///
/// Modelling this as an enum rather than three `Option`s is what makes the
/// invalid combinations (e.g. `#[compute]` together with `#[source]`)
/// unrepresentable once parsing has succeeded.
pub(crate) enum FieldStrategy {
    /// Direct mapping, possibly under a different source name.
    ///
    /// Covers MVP features 1 (direct mapping), 2 (rename), 5 (nested
    /// resources) and 6 (composition): nesting needs no attribute because the
    /// written type already carries the information.
    Map { source_field: syn::Ident },
    /// `#[transform(f)]`: one source field in, one resource field out.
    Transform {
        source_field: syn::Ident,
        function: syn::Path,
    },
    /// `#[compute(f)]`: the whole source in, one resource field out.
    Compute { function: syn::Path },
}

impl ResourceInput {
    /// Parses and validates the annotated item.
    ///
    /// Returns a combined `syn::Error` describing *every* problem found, so a
    /// struct with several bad attributes only needs one compile round-trip.
    pub(crate) fn parse(input: syn::DeriveInput) -> syn::Result<Self> {
        // Destructure up front: `data` must be consumed by value to move the
        // field types out, while `attrs`/`ident` are needed before that.
        let syn::DeriveInput {
            attrs,
            ident,
            generics,
            data,
            ..
        } = input;

        let mut errors = Errors::default();

        let source = parse_source_type(&attrs, &ident, &mut errors);
        let named_fields = extract_named_fields(data, &ident, &mut errors);

        let mut fields = Vec::new();
        for field in named_fields {
            match ResourceField::parse(field) {
                Ok(parsed) => fields.push(parsed),
                Err(error) => errors.push(error),
            }
        }

        // Surface everything collected so far before touching the `Option`s.
        errors.finish()?;

        Ok(Self {
            ident,
            generics,
            // Unreachable unless `parse_source_type` returned `Some` — if it
            // had not, it would have pushed an error and `finish()?` above
            // would already have returned.
            source: source.expect("source type is present when no error was recorded"),
            fields,
        })
    }
}

/// Reads the struct-level `#[source(path::to::Type)]` attribute.
///
/// # Why a struct-level attribute is unavoidable
///
/// A derive macro only ever sees the tokens of the item it is attached to. It
/// has no type information and no way to look at other files, so it genuinely
/// cannot guess which struct `TodoResource` is built from. Rather than invent a
/// new keyword, we reuse `source`: on the struct it names the source *type*, on
/// a field it names the source *field*. One word, one meaning ("where does this
/// come from"), which keeps the DSL as small as the design document demands.
fn parse_source_type(
    attrs: &[syn::Attribute],
    ident: &syn::Ident,
    errors: &mut Errors,
) -> Option<syn::Path> {
    let mut source: Option<syn::Path> = None;

    for attr in attrs {
        if attr.path().is_ident("source") {
            if source.is_some() {
                errors.push_spanned(
                    attr,
                    "duplicate `#[source(...)]`: a resource is built from exactly one source type",
                );
                continue;
            }

            // `parse_args::<syn::Path>()` accepts `Todo`, `crate::models::Todo`
            // and `super::Todo`. It rejects generic arguments such as
            // `Page<Todo>`; supporting those is a deliberate non-goal for now
            // (see the placeholder at the bottom of this file).
            match attr.parse_args::<syn::Path>() {
                Ok(path) => source = Some(path),
                Err(_) => errors.push_spanned(
                    attr,
                    "expected a source type path, e.g. `#[source(Todo)]` or `#[source(crate::models::Todo)]`",
                ),
            }
        } else if attr.path().is_ident("transform") || attr.path().is_ident("compute") {
            errors.push_spanned(
                attr,
                "`#[transform(...)]` and `#[compute(...)]` apply to fields, not to the struct",
            );
        }
    }

    if source.is_none() {
        errors.push_spanned(
            ident,
            "missing `#[source(...)]`: add the source type, e.g. `#[source(Todo)]`, \
             so the macro knows what this resource is built from",
        );
    }

    source
}

/// Ensures the item is a struct with named fields and hands them back.
///
/// Tuple structs are rejected on purpose: `#[source(username)]` maps *names*,
/// and positional mapping would silently survive a field reorder — precisely
/// the refactor hazard this system exists to eliminate.
fn extract_named_fields(
    data: syn::Data,
    ident: &syn::Ident,
    errors: &mut Errors,
) -> Vec<syn::Field> {
    match data {
        syn::Data::Struct(data) => match data.fields {
            syn::Fields::Named(named) => named.named.into_iter().collect(),
            syn::Fields::Unnamed(unnamed) => {
                errors.push_spanned(
                    unnamed,
                    "`#[derive(Resource)]` needs named fields; tuple structs cannot be mapped by name",
                );
                Vec::new()
            }
            syn::Fields::Unit => {
                errors.push_spanned(
                    ident,
                    "`#[derive(Resource)]` needs at least one named field; a unit struct has nothing to map",
                );
                Vec::new()
            }
        },
        syn::Data::Enum(data) => {
            errors.push_spanned(
                data.enum_token,
                "`#[derive(Resource)]` supports structs only. Map an enum with `#[transform(...)]` \
                 on the field that holds it",
            );
            Vec::new()
        }
        syn::Data::Union(data) => {
            errors.push_spanned(
                data.union_token,
                "`#[derive(Resource)]` supports structs only; unions cannot be mapped safely",
            );
            Vec::new()
        }
    }
}

impl ResourceField {
    /// Parses the attributes of a single field and resolves its strategy.
    fn parse(field: syn::Field) -> syn::Result<Self> {
        // Guaranteed by `extract_named_fields`, which rejects tuple structs.
        let ident = field
            .ident
            .clone()
            .expect("named fields are guaranteed by extract_named_fields");

        let mut errors = Errors::default();
        let mut source_field: Option<syn::Ident> = None;
        let mut transform: Option<syn::Path> = None;
        let mut compute: Option<syn::Path> = None;

        for attr in &field.attrs {
            if attr.path().is_ident("source") {
                if source_field.is_some() {
                    errors.push_spanned(attr, "duplicate `#[source(...)]` on this field");
                    continue;
                }
                match attr.parse_args::<syn::Ident>() {
                    Ok(name) => source_field = Some(name),
                    Err(_) => errors.push_spanned(
                        attr,
                        "expected a single source field name, e.g. `#[source(username)]`",
                    ),
                }
            } else if attr.path().is_ident("transform") {
                if transform.is_some() {
                    errors.push_spanned(attr, "duplicate `#[transform(...)]` on this field");
                    continue;
                }
                match attr.parse_args::<syn::Path>() {
                    Ok(path) => transform = Some(path),
                    Err(_) => errors.push_spanned(
                        attr,
                        "expected a function path, e.g. `#[transform(format_date)]`",
                    ),
                }
            } else if attr.path().is_ident("compute") {
                if compute.is_some() {
                    errors.push_spanned(attr, "duplicate `#[compute(...)]` on this field");
                    continue;
                }
                match attr.parse_args::<syn::Path>() {
                    Ok(path) => compute = Some(path),
                    Err(_) => errors.push_spanned(
                        attr,
                        "expected a function path, e.g. `#[compute(real_price)]`",
                    ),
                }
            }
            // Unknown attributes (`#[serde(...)]`, `#[doc]`, ...) are ignored
            // on purpose: a resource is normally also a serde struct, and a
            // derive macro must never claim attributes it did not declare.
        }

        // --- Cross-attribute validation -----------------------------------
        //
        // These checks encode the semantic split from the design document:
        // `transform` is a *field-level* conversion, `compute` is a
        // *whole-source* derivation. Mixing them is always a mistake, so it is
        // a compile error rather than a silent precedence rule.
        if compute.is_some() && transform.is_some() {
            errors.push_spanned(
                &ident,
                "`#[compute(...)]` and `#[transform(...)]` are mutually exclusive: `compute` \
                 already receives the whole source, so there is no single field left to transform",
            );
        }
        if compute.is_some() && source_field.is_some() {
            errors.push_spanned(
                &ident,
                "`#[compute(...)]` does not read a single field, so `#[source(...)]` has no \
                 meaning here. Remove one of them",
            );
        }

        errors.finish()?;

        // Resolve the strategy. Note the fallback: when no `#[source(...)]` is
        // given, the source field name equals the resource field name. That is
        // what makes case 1 (direct mapping) attribute-free.
        let strategy = match (compute, transform) {
            (Some(function), _) => FieldStrategy::Compute { function },
            (None, Some(function)) => FieldStrategy::Transform {
                source_field: source_field.unwrap_or_else(|| ident.clone()),
                function,
            },
            (None, None) => FieldStrategy::Map {
                source_field: source_field.unwrap_or_else(|| ident.clone()),
            },
        };

        Ok(Self {
            ident,
            ty: field.ty,
            strategy,
        })
    }
}

// ---------------------------------------------------------------------------
// Placeholders: attributes deliberately left out of the MVP.
//
// Each of these slots into the existing `FieldStrategy` enum without changing
// the generated trait impls, which is the main reason the IR is an enum.
// ---------------------------------------------------------------------------

// (1) Cloning instead of moving.
//     Two resource fields reading the same source field currently produce a
//     "use of moved value" error. A future opt-in flag would fix that:
//
//     #[source(title, clone)]
//
//     Parsing sketch: replace `attr.parse_args::<syn::Ident>()` with a small
//     `Punctuated<Ident, Token![,]>` parse and store `clone: bool` on
//     `FieldStrategy::Map`, then emit `source.title.clone()`.

// (2) Borrowing transforms.
//     `#[transform(f)]` passes the field by value today. Large values (a big
//     `String` that is only measured, say) would prefer:
//
//     #[transform(char_count, by_ref)]  ->  char_count(&source.title)

// (3) Renaming the *output* key.
//     Not needed: `#[serde(rename = "...")]` already owns that job and is
//     ignored by this macro. Keeping serialization concerns in serde is the
//     reason the Resource layer stays serialization-agnostic.

// (4) Generic source types, e.g. `#[source(Page<Todo>)]`.
//     `syn::Path` already accepts angle-bracketed arguments in many positions;
//     switching to `attr.parse_args::<syn::Type>()` and forwarding the type
//     would be the whole change. Left out because the MVP has no such source.

// (5) Runtime context, e.g. `#[context(RequestContext)]` on the struct.
//     Would flip the generated impl to `FromResourceWith` (sketched in
//     macros-core/src/resource.rs) while leaving field parsing untouched.
