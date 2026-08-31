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