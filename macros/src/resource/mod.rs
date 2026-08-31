// ============================================================================
// File: macros/src/resource/mod.rs
//
// Responsibility
//   Wires the three phases of `#[derive(Resource)]` together and owns the
//   small error-accumulation helper they share.
//
//   The pipeline is deliberately split into three stages, in the order a
//   compiler front-end would use them:
//
//     1. `ast`    — parse the annotated struct into our own small IR. Nothing
//                   is generated here; the phase only validates and describes.
//     2. `ty`     — analyse the *written* type of each field (`Vec<T>`,
//                   `Option<T>`, plain) so containers can be mapped
//                   structurally.
//     3. `expand` — turn the IR into tokens.
//
//   Why an IR instead of generating tokens while parsing? Because it lets us
//   validate everything first and report *all* attribute mistakes in one
//   compile run, instead of stopping at the first one.
// ============================================================================

mod ast;
mod expand;
mod ty;

use proc_macro2::TokenStream;

/// Runs the full derive pipeline: parse → validate → expand.
///
/// Returns the generated tokens, or a `syn::Error` that the caller renders as
/// one or more `compile_error!` invocations.
pub(crate) fn derive(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let model = ast::ResourceInput::parse(input)?;
    Ok(expand::expand(&model))
}

/// Collects several `syn::Error`s so the user sees every problem at once.
///
/// `syn::Error::combine` chains errors into a single value that renders as
/// multiple `compile_error!` calls, each with its own span. Reporting only the
/// first mistake would force a slow fix-compile-fix loop on a struct with many
/// misannotated fields.
#[derive(Default)]
pub(crate) struct Errors {
    inner: Option<syn::Error>,
}

impl Errors {
    /// Records one more error.
    pub(crate) fn push(&mut self, error: syn::Error) {
        if let Some(existing) = &mut self.inner {
            existing.combine(error);
        } else {
            self.inner = Some(error);
        }
    }

    /// Records an error attached to the span of `tokens`.
    ///
    /// Span accuracy is the difference between "error in this file" and an
    /// underline directly beneath the offending attribute, so every diagnostic
    /// in this crate goes through a spanned constructor.
    pub(crate) fn push_spanned<T: quote::ToTokens>(&mut self, tokens: T, message: &str) {
        self.push(syn::Error::new_spanned(tokens, message));
    }

    /// `Ok(())` when nothing was recorded, otherwise the combined error.
    pub(crate) fn finish(self) -> syn::Result<()> {
        match self.inner {
            Some(error) => Err(error),
            None => Ok(()),
        }
    }
}
