// ============================================================================
// File: macros-core/src/resource.rs
//
// Responsibility
//   Defines the single conversion trait the entire Resource system rests on:
//   `FromResource<Source>`, plus the identity blanket implementation and a
//   small `collection` helper.
//
//   Why a dedicated trait instead of `std::convert::From`?
//     1. Strictness. `From` is a general-purpose conversion trait with many
//        std impls (`i32 -> i64`, `&str -> String`, ...). If we used it for
//        direct field mapping, a source field silently changing from `i32` to
//        `i64` would keep compiling. With our own trait, only *identity* and
//        *derived resource* conversions exist, so any real type change is a
//        compile error. That is exactly the guarantee the design asks for.
//     2. Orphan rules. `impl From<Vec<Post>> for Vec<PostResource>` is illegal
//        outside the crate that defines `Vec` (E0117). Owning the trait keeps
//        the door open for future container support without hacks.
//     3. Error messages. A missing `FromResource<Foo>` bound reads as "this
//        resource cannot be built from Foo", which is the domain-level truth.
//
//   Cost model: every impl here is generic and `#[inline]`. After
//   monomorphisation the identity conversion compiles down to a plain move.
//   There is no dynamic dispatch, no registry, no reflection, no allocation
//   other than the `Vec` the caller explicitly asked for.
// ============================================================================

//! The conversion contract shared by every generated resource.
//!
//! A Resource is a *representation builder*: it takes a value that somebody
//! else already loaded and produces the shape the API should return. It has no
//! knowledge of databases, queries, caches or loading strategies, and it cannot
//! acquire any, because nothing in this module can perform I/O.

/// Builds a representation (`Self`) out of an already-prepared `Source` value.
///
/// This trait is the compile-time contract of the Resource system. If
/// `TodoResource: FromResource<Todo>` holds, then — and only then — a `Todo`
/// can be turned into a `TodoResource`. Nothing is looked up by name at run
/// time; the compiler resolves the whole mapping.
///
/// # Ownership
///
/// `from_resource` takes the source **by value**. A Resource is the last stop
/// before serialization, so moving out of the source avoids cloning every
/// `String` and `Vec`. When you need to keep the source, implement the trait
/// for a reference type as well (`impl FromResource<&Todo> for TodoResource`)
/// or clone at the call site.
///
/// # Examples
///
/// Hand-written implementation (this is what `#[derive(Resource)]` generates):
///
/// ```
/// use macros_core::FromResource;
///
/// struct Todo { id: i64, title: String }
/// struct TodoResource { id: i64, title: String }
///
/// impl FromResource<Todo> for TodoResource {
///     fn from_resource(source: Todo) -> Self {
///         Self {
///             id: FromResource::from_resource(source.id),
///             title: FromResource::from_resource(source.title),
///         }
///     }
/// }
///
/// let resource = TodoResource::from_resource(Todo { id: 1, title: "buy milk".into() });
/// assert_eq!(resource.id, 1);
/// ```
pub trait FromResource<Source>: Sized {
    /// Consumes `source` and produces the representation.
    ///
    /// Implementations must be pure and infallible: they may reshape, rename,
    /// format and derive values, but they must not perform I/O, hit a
    /// database, or fail. Anything that can fail belongs *before* the Resource
    /// layer, in the code that prepared the source value.
    fn from_resource(source: Source) -> Self;
}

// ---------------------------------------------------------------------------
// Identity implementation
// ---------------------------------------------------------------------------
//
// This is what makes "the field types already match" the default, zero-cost
// path. The derive macro emits `FromResource::from_resource(source.id)` for
// *every* plainly mapped field; when the source and target types are equal,
// this impl is selected and the call is a move.
//
// Coherence note for reviewers: this blanket impl does **not** conflict with a
// generated `impl FromResource<Todo> for TodoResource`. Overlap would require
// unifying `Self` with `Source`, i.e. proving `Todo == TodoResource`. They are
// distinct nominal types, so the compiler accepts both impls.
//
// The same reasoning explains why there is *no*
// `impl<S, R: FromResource<S>> FromResource<Vec<S>> for Vec<R>` here: it would
// overlap with this identity impl for `T = Vec<X>` and the compiler cannot
// rule that out. Container handling (`Vec<T>`, `Option<T>`) is therefore done
// structurally by the derive macro, which can see the written field type.
impl<T> FromResource<T> for T {
    #[inline(always)]
    fn from_resource(source: T) -> Self {
        source
    }
}

/// Maps any iterator of source values into a `Vec` of resources.
///
/// The derive macro also generates a more convenient inherent
/// `YourResource::collection(items)` method; this free function exists for the
/// cases where the resource type is only known through a generic parameter.
///
/// # Examples
///
/// ```
/// use macros_core::{collection, FromResource};
///
/// struct Todo { id: i64 }
/// struct TodoResource { id: i64 }
///
/// impl FromResource<Todo> for TodoResource {
///     fn from_resource(source: Todo) -> Self { Self { id: source.id } }
/// }
///
/// let resources: Vec<TodoResource> = collection(vec![Todo { id: 1 }, Todo { id: 2 }]);
/// assert_eq!(resources.len(), 2);
/// ```
#[inline]
pub fn collection<Source, Res, I>(items: I) -> Vec<Res>
where
    I: IntoIterator<Item = Source>,
    Res: FromResource<Source>,
{
    // `size_hint` is honoured by `collect`, so a `Vec` source allocates once.
    items
        .into_iter()
        .map(<Res as FromResource<Source>>::from_resource)
        .collect()
}

// ---------------------------------------------------------------------------
// Placeholders for features intentionally left out of the MVP.
//
// These are written as commented sketches on purpose: they document the
// intended extension points so that a future change does not have to redesign
// the trait, while keeping the compiled surface of the MVP minimal.
// ---------------------------------------------------------------------------

// (1) Envelope / pagination wrapper.
//     `collection()` returns a bare `Vec` today. Once pagination lands, the
//     HTTP layer (not the Resource) should wrap it:
//
// pub struct ResourceCollection<T> {
//     pub data: Vec<T>,
//     pub meta: CollectionMeta,
// }
//
// pub struct CollectionMeta {
//     pub total: u64,
//     pub per_page: u32,
//     pub current_page: u32,
// }
//
// impl<T> ResourceCollection<T> {
//     pub fn paginated<S, I>(items: I, meta: CollectionMeta) -> Self
//     where I: IntoIterator<Item = S>, T: FromResource<S> {
//         Self { data: collection(items), meta }
//     }
// }
//
//     Note that `meta` is runtime data, which is why it lives *outside* the
//     compile-time mapping and does not touch `FromResource`.

// (2) Request / authorization context.
//     Authorization is runtime by nature, so it must not leak into
//     `FromResource`. The clean extension is a second, separate trait:
//
// pub trait FromResourceWith<Source, Ctx>: Sized {
//     fn from_resource_with(source: Source, ctx: &Ctx) -> Self;
//     }
//
//     A future `#[derive(Resource)]` could emit this variant when the struct
//     carries `#[context(RequestContext)]`, and keep emitting the plain
//     `FromResource` impl when it does not. Existing code keeps compiling.

// (3) Conditional / "when loaded" fields.
//     Laravel's `whenLoaded()` is runtime state. The type-safe Rust analogue
//     is to make presence part of the *type*, not a string flag:
//
// pub enum Loaded<T> { Present(T), Missing }
//
// impl<S, T: FromResource<S>> FromResource<Loaded<S>> for Loaded<T> { ... }
//
//     Combined with `#[serde(skip_serializing_if = ...)]` on the resource
//     field this gives Laravel's behaviour with zero runtime lookups.

// (4) Fallible transforms.
//     Every conversion is infallible today, which is a deliberate constraint.
//     If a transform ever needs to fail, add a sibling trait rather than
//     making the happy path return `Result`:
//
// pub trait TryFromResource<Source>: Sized {
//     type Error;
//     fn try_from_resource(source: Source) -> Result<Self, Self::Error>;
// }

#[cfg(test)]
mod tests {
    use super::*;

    // A minimal source/resource pair defined by hand, so these tests exercise
    // the runtime crate on its own — no macro, no database, no serde.
    struct Todo {
        id: i64,
        title: String,
    }

    struct TodoResource {
        id: i64,
        title: String,
    }

    impl FromResource<Todo> for TodoResource {
        fn from_resource(source: Todo) -> Self {
            Self {
                id: FromResource::from_resource(source.id),
                title: FromResource::from_resource(source.title),
            }
        }
    }

    #[test]
    fn identity_conversion_returns_the_same_value() {
        // The blanket impl must behave as a plain move for equal types.
        let value: i64 = 42;
        let converted: i64 = FromResource::from_resource(value);
        assert_eq!(converted, 42);

        let text: String = "hello".to_owned();
        let converted: String = FromResource::from_resource(text);
        assert_eq!(converted, "hello");
    }

    #[test]
    fn hand_written_impl_builds_the_resource() {
        let resource = TodoResource::from_resource(Todo {
            id: 7,
            title: "write docs".to_owned(),
        });

        assert_eq!(resource.id, 7);
        assert_eq!(resource.title, "write docs");
    }

    #[test]
    fn collection_helper_maps_every_item_in_order() {
        let todos = vec![
            Todo { id: 1, title: "a".into() },
            Todo { id: 2, title: "b".into() },
        ];

        let resources: Vec<TodoResource> = collection(todos);

        assert_eq!(resources.len(), 2);
        assert_eq!(resources[0].id, 1);
        assert_eq!(resources[1].title, "b");
    }

    #[test]
    fn collection_helper_accepts_any_into_iterator() {
        // Not just `Vec`: arrays, iterators and adapters all work, because the
        // bound is `IntoIterator<Item = Source>` rather than a concrete type.
        let resources: Vec<TodoResource> = collection([Todo { id: 9, title: "x".into() }]);
        assert_eq!(resources[0].id, 9);

        let resources: Vec<TodoResource> =
            collection((1..=3).map(|id| Todo { id, title: id.to_string() }));
        assert_eq!(resources.len(), 3);
        assert_eq!(resources[2].title, "3");
    }
}
