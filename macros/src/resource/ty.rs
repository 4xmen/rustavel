// ============================================================================
// File: macros/src/resource/ty.rs
//
// Responsibility
//   Phase 2 of the derive: look at the *written* type of a resource field and
//   decide how to walk it — `Vec<T>`, `Option<T>`, or a plain leaf.
//
//   Why does the macro have to do this at all? Because of Rust's coherence
//   rules. The natural way to express "a Vec of sources becomes a Vec of
//   resources" would be a blanket impl in `macros-core`:
//
//       impl<S, R: FromResource<S>> FromResource<Vec<S>> for Vec<R> { ... }
//
//   but that overlaps with the identity impl `impl<T> FromResource<T> for T`
//   when `T = Vec<X>`, and the compiler rejects the pair. Writing the impl in
//   the *user's* crate is not possible either: `Vec` is foreign and not
//   `#[fundamental]`, so `impl FromResource<Vec<Post>> for Vec<PostResource>`
//   is an orphan-rule violation (E0117).
//
//   The way out is to resolve the container structurally, at macro expansion
//   time, and emit `.into_iter().map(...)` / `.map(...)` explicitly. That keeps
//   `macros-core` coherent and costs nothing at run time.
//
//   Known limitation (documented rather than hidden): the analysis is
//   *syntactic*. A field written as `type Posts = Vec<PostResource>;` reads as
//   a leaf, so an aliased collection of nested resources will not compile.
//   Write the container out, or use `#[transform(...)]`.
// ============================================================================

use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};

/// How a resource field's type must be traversed during conversion.
///
/// The recursion means nested containers such as `Option<Vec<PostResource>>`
/// work without any special casing.
pub(crate) enum TypeShape {
    /// A type with no recognised container wrapper. Converted in one step via
    /// `FromResource` — which resolves to the identity impl when the source and
    /// resource types are equal, and to a derived impl when the field holds a
    /// nested resource.
    Leaf,
    /// `Option<T>`; mapped with `Option::map`.
    Option(Box<TypeShape>),
    /// `Vec<T>`; mapped with `into_iter().map(...).collect()`.
    Vec(Box<TypeShape>),
}

/// Classifies a field type by its written shape.
///
/// Only the last path segment is inspected, so `Vec<T>`, `std::vec::Vec<T>` and
/// `alloc::vec::Vec<T>` are all recognised. Anything else — references, arrays,
/// `Box<T>`, `HashMap<K, V>`, aliases — is treated as a leaf.
pub(crate) fn analyze(ty: &syn::Type) -> TypeShape {
    // `qself` is the `<T as Trait>::Assoc` form; such a type carries no usable
    // container information for us, so it falls through to `Leaf`.
    let syn::Type::Path(type_path) = ty else {
        return TypeShape::Leaf;
    };
    if type_path.qself.is_some() {
        return TypeShape::Leaf;
    }

    let Some(segment) = type_path.path.segments.last() else {
        return TypeShape::Leaf;
    };

    let is_vec = segment.ident == "Vec";
    let is_option = segment.ident == "Option";
    if !is_vec && !is_option {
        return TypeShape::Leaf;
    }

    // Both containers take exactly one type argument. Requiring "exactly one"
    // guards against a user-defined `Vec<T, A>`-lookalike being misread.
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return TypeShape::Leaf;
    };
    let mut type_arguments = arguments.args.iter().filter_map(|argument| match argument {
        syn::GenericArgument::Type(inner) => Some(inner),
        _ => None,
    });
    let (Some(inner), None) = (type_arguments.next(), type_arguments.next()) else {
        return TypeShape::Leaf;
    };

    let inner_shape = Box::new(analyze(inner));
    if is_vec {
        TypeShape::Vec(inner_shape)
    } else {
        TypeShape::Option(inner_shape)
    }
}

/// Builds the conversion expression for `input` according to `shape`.
///
/// `input` is the expression that yields the source value, typically
/// `source.title`. The result is an expression of the resource field's type.
///
/// The emitted code is intentionally boring:
///
/// | Shape | Generated |
/// |---|---|
/// | leaf | `FromResource::from_resource(source.f)` |
/// | `Option<T>` | `source.f.map(\|v\| ...)` |
/// | `Vec<T>` | `source.f.into_iter().map(\|v\| ...).collect::<Vec<_>>()` |
///
/// Performance note: for an all-identity `Vec<i64> -> Vec<i64>` the map closure
/// is `|v| v`, which LLVM removes entirely; the `collect` reuses the source
/// allocation because the iterator is `Vec`'s own `IntoIter`.
/// `span` should be the span of the resource field's declared type. Every
/// generated token is attributed to it via `quote_spanned!`, which is what
/// makes a failed `FromResource` bound underline the offending field instead of
/// the `#[derive(Resource)]` attribute. Diagnostics are a feature here, not a
/// detail: "clear compiler errors" is an explicit design priority.
pub(crate) fn conversion_expr(shape: &TypeShape, input: TokenStream, span: Span) -> TokenStream {
    match shape {
        TypeShape::Leaf => {
            // Both type parameters of `FromResource` are inferred here: `Self`
            // from the expected field type, `Source` from `input`. That is what
            // makes an incompatible mapping a clear "trait bound not satisfied"
            // error pointing at this field.
            quote_spanned! {span=>
                ::macros_core::resource::FromResource::from_resource(#input)
            }
        }
        TypeShape::Option(inner) => {
            // The binding name is deliberately unlikely to collide with user
            // identifiers; `quote!`-built idents are not hygienic, so the
            // prefix does the work instead.
            let inner_expr = conversion_expr(inner, quote!(__resource_item), span);
            quote_spanned! {span=>
                #input.map(|__resource_item| #inner_expr)
            }
        }
        TypeShape::Vec(inner) => {
            let inner_expr = conversion_expr(inner, quote!(__resource_item), span);
            quote_spanned! {span=>
                #input
                    .into_iter()
                    .map(|__resource_item| #inner_expr)
                    .collect::<::std::vec::Vec<_>>()
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Placeholders: container support intentionally left out of the MVP.
// ---------------------------------------------------------------------------

// (1) More containers. Each is one more `TypeShape` variant plus one arm in
//     `conversion_expr`; no change is needed anywhere else:
//
//     TypeShape::Box(inner)   -> ::std::boxed::Box::new(<inner on *#input>)
//     TypeShape::HashMap(v)   -> #input.into_iter().map(|(k, v)| (k, <inner>)).collect()
//     TypeShape::Array(inner) -> #input.map(|__resource_item| <inner>)   // [T; N]::map

// (2) A `#[resource(leaf)]` escape hatch, for the rare case where a field
//     genuinely *is* a `Vec` on both sides and must be moved wholesale rather
//     than walked. Today the identity impl handles that correctly anyway, so
//     the attribute would only be a compile-time optimisation.

// (3) Type-alias awareness. Fundamentally impossible in a derive macro (no type
//     information is available), so the honest fix is a good diagnostic rather
//     than a heuristic — e.g. detect a leaf whose name ends in `Resource` and
//     mention aliases in the error note.

#[cfg(test)]
mod tests {
    use super::*;

    /// Small helper so the tests read like the field types they describe.
    fn shape_of(source: &str) -> TypeShape {
        analyze(&syn::parse_str::<syn::Type>(source).expect("valid type"))
    }

    /// Renders a shape into a comparable string, e.g. "Vec(Option(Leaf))".
    fn describe(shape: &TypeShape) -> String {
        match shape {
            TypeShape::Leaf => "Leaf".to_owned(),
            TypeShape::Option(inner) => format!("Option({})", describe(inner)),
            TypeShape::Vec(inner) => format!("Vec({})", describe(inner)),
        }
    }

    #[test]
    fn plain_types_are_leaves() {
        assert_eq!(describe(&shape_of("i64")), "Leaf");
        assert_eq!(describe(&shape_of("String")), "Leaf");
        assert_eq!(describe(&shape_of("crate::api::PostResource")), "Leaf");
    }

    #[test]
    fn containers_are_recognised_including_qualified_paths() {
        assert_eq!(describe(&shape_of("Vec<PostResource>")), "Vec(Leaf)");
        assert_eq!(describe(&shape_of("std::vec::Vec<PostResource>")), "Vec(Leaf)");
        assert_eq!(describe(&shape_of("Option<PostResource>")), "Option(Leaf)");
    }

    #[test]
    fn nesting_recurses_in_the_written_order() {
        assert_eq!(
            describe(&shape_of("Option<Vec<PostResource>>")),
            "Option(Vec(Leaf))"
        );
        assert_eq!(describe(&shape_of("Vec<Vec<i64>>")), "Vec(Vec(Leaf))");
    }

    #[test]
    fn unsupported_shapes_fall_back_to_leaf() {
        // These must not be walked structurally; the identity impl (or a
        // `#[transform]`) has to handle them.
        assert_eq!(describe(&shape_of("[u8; 4]")), "Leaf");
        assert_eq!(describe(&shape_of("Box<PostResource>")), "Leaf");
        assert_eq!(describe(&shape_of("HashMap<String, i64>")), "Leaf");
    }
}
