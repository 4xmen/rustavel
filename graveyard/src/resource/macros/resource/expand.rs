// ============================================================================
// File: macros/src/resource/expand.rs
//
// Responsibility
//   Phase 3 of the derive: turn the validated IR into tokens. This file is a
//   pure function — it never returns an error, because `ast.rs` has already
//   rejected everything that is structurally invalid.
//
//   Guiding rule: generate the code a careful human would have written by
//   hand. No reflection, no string lookups, no trait objects, no allocation
//   beyond what the user asked for. If the expansion is unreadable, the error
//   messages the user sees will be unreadable too.
//
//   Three items are emitted per resource:
//     1. `impl FromResource<Source>` — the actual mapping.
//     2. `impl From<Source>`        — so `Resource::from(x)` / `x.into()` work.
//     3. inherent `from_source` + `collection` — the ergonomic surface.
// ============================================================================

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::spanned::Spanned;

use super::ast::{FieldStrategy, ResourceInput};
use super::ty;

/// Generates every impl block for one `#[derive(Resource)]` struct.
pub(crate) fn expand(input: &ResourceInput) -> TokenStream {
    let resource_ident = &input.ident;
    let source_ty = &input.source;

    // Forward the resource's own generics to every generated impl. For a
    // non-generic struct these three fragments are empty, so the common case
    // is unaffected.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let (compute_bindings, field_initializers) = build_field_code(input);

    quote! {
        // The real mapping. Everything else delegates to this one function, so
        // there is exactly one place where the conversion logic lives.
        #[automatically_derived]
        impl #impl_generics ::macros_core::resource::FromResource<#source_ty>
            for #resource_ident #ty_generics #where_clause
        {
            // A resource whose fields are *all* computed from constants would
            // not read `source`; silence that instead of forcing the user to
            // write `_source`.
            #[allow(unused_variables)]
            fn from_resource(source: #source_ty) -> Self {
                // Computed fields run first, while `source` is still whole.
                // A struct literal evaluates its fields in written order, so if
                // a `#[compute]` field appeared after a moved-out field it
                // would fail to borrow. Hoisting removes that ordering trap
                // entirely and makes field order in the struct irrelevant.
                #(#compute_bindings)*

                Self {
                    #(#field_initializers),*
                }
            }
        }

        // `From` is generated as a thin wrapper so the ergonomic API from the
        // design document works: `TodoResource::from(todo)` and `todo.into()`.
        // It is intentionally *not* the primary trait — `From` has too many
        // std impls to give the strict field-level checking we want.
        #[automatically_derived]
        impl #impl_generics ::core::convert::From<#source_ty>
            for #resource_ident #ty_generics #where_clause
        {
            #[inline]
            fn from(source: #source_ty) -> Self {
                <Self as ::macros_core::resource::FromResource<#source_ty>>::from_resource(source)
            }
        }

        // Inherent methods. These exist for diagnostics as much as ergonomics:
        // an inherent method has a single, concrete signature, so a wrong
        // argument type produces "expected `Todo`, found `X`" instead of a
        // trait-resolution error mentioning every `FromResource` impl in scope.
        #[automatically_derived]
        impl #impl_generics #resource_ident #ty_generics #where_clause {
            /// Builds this resource from one already-loaded source value.
            ///
            /// Equivalent to `Self::from(source)`; provided because it reads
            /// better at call sites where the target type is not obvious.
            #[inline]
            pub fn from_source(source: #source_ty) -> Self {
                <Self as ::macros_core::resource::FromResource<#source_ty>>::from_resource(source)
            }

            /// Builds a `Vec` of resources from any iterable of source values.
            ///
            /// Accepts `Vec`, arrays, iterators and adapters alike, because the
            /// bound is `IntoIterator` rather than a concrete container.
            ///
            /// Note that the return type is a plain `Vec`. Pagination metadata
            /// is runtime data and belongs in the HTTP layer, not here.
            #[inline]
            pub fn collection<__ResourceItems>(items: __ResourceItems) -> ::std::vec::Vec<Self>
            where
                __ResourceItems: ::core::iter::IntoIterator<Item = #source_ty>,
            {
                items
                    .into_iter()
                    .map(<Self as ::macros_core::resource::FromResource<#source_ty>>::from_resource)
                    .collect()
            }

            // Placeholder: a context-aware sibling, once authorization lands.
            //
            // pub fn from_source_with(source: #source_ty, ctx: &RequestContext) -> Self { ... }
            //
            // Placeholder: an envelope constructor, once pagination lands.
            //
            // pub fn paginated<I>(items: I, meta: CollectionMeta)
            //     -> ::macros_core::resource::ResourceCollection<Self> { ... }
        }
    }
}

/// Produces the hoisted `let` bindings for computed fields and the struct
/// literal initializers for every field, in declaration order.
///
/// Returning the two vectors together keeps the single pass over the fields —
/// and, more importantly, keeps the pairing between a computed field and its
/// binding obvious to a reader.
fn build_field_code(input: &ResourceInput) -> (Vec<TokenStream>, Vec<TokenStream>) {
    let mut compute_bindings = Vec::new();
    let mut field_initializers = Vec::new();

    for field in &input.fields {
        let field_ident = &field.ident;
        let field_ty = &field.ty;

        match &field.strategy {
            // -------------------------------------------------------------
            // 4. Compute: the whole source in, one value out.
            // -------------------------------------------------------------
            FieldStrategy::Compute { function } => {
                let binding = format_ident!("__resource_computed_{}", field_ident);

                // The explicit type annotation is not redundant: it pins the
                // expected return type at the *call site*, so a compute
                // function with the wrong return type is reported here, next
                // to the field, rather than deep inside the struct literal.
                compute_bindings.push(quote! {
                    let #binding: #field_ty = #function(&source);
                });

                field_initializers.push(quote! {
                    #field_ident: #binding
                });
            }

            // -------------------------------------------------------------
            // 3. Transform: one source field in, one value out.
            //
            // No structural walking happens here. Transform means "this
            // function owns the conversion completely", which is what allows
            // `PrimitiveDateTime -> String`. Its signature is checked by the
            // compiler as an ordinary function call.
            // -------------------------------------------------------------
            FieldStrategy::Transform {
                source_field,
                function,
            } => {
                field_initializers.push(quote! {
                    #field_ident: #function(source.#source_field)
                });
            }

            // -------------------------------------------------------------
            // 1, 2, 5, 6. Direct mapping, renaming, nested resources and
            // composition — all the same generated shape.
            //
            // The written field type decides how deep to walk; `FromResource`
            // then resolves to either the identity impl (types already match)
            // or a derived impl (the field holds another resource). Nesting
            // needs no attribute precisely because the type already says it.
            // -------------------------------------------------------------
            FieldStrategy::Map { source_field } => {
                let shape = ty::analyze(field_ty);
                let value =
                    ty::conversion_expr(&shape, quote!(source.#source_field), field_ty.span());

                field_initializers.push(quote! {
                    #field_ident: #value
                });
            }
        }
    }

    (compute_bindings, field_initializers)
}
