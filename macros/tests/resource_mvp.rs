// ============================================================================
// File: macros/tests/resource_mvp.rs
//
// Responsibility
//   End-to-end tests for `#[derive(Resource)]`, exercising all six MVP
//   features against hand-built in-memory data.
//
//   Deliberate constraints, mirroring the design document:
//     * No sqlx, no database, no async runtime. Sources are plain structs
//       built inline, which is the whole point: a Resource must not be able to
//       tell where its data came from.
//     * Tests live in `macros/tests/` (an integration test target) rather than
//       inside `src/`, because a proc macro cannot be invoked from within the
//       crate that defines it. This target consumes `macros` exactly the way
//       `app` will.
//     * `serde` appears only in the last test, to prove the final JSON shape.
//       The Resource system itself never touches it.
//
//   Reading order: each section maps 1:1 to a numbered feature in the design
//   document.
// ============================================================================

use macros::Resource;
use macros_core::{collection, FromResource};

// ---------------------------------------------------------------------------
// Fake "database rows". In the real app these carry `#[derive(sqlx::FromRow)]`;
// here they are ordinary structs, and nothing in the Resource layer notices.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Todo {
    id: i64,
    title: String,
    done: bool,
    // Stand-in for `time::PrimitiveDateTime`: a Unix timestamp. Using a plain
    // integer keeps the test dependency-free while still forcing a real type
    // change (`i64 -> String`) through `#[transform]`.
    created_at: i64,
}

#[derive(Clone)]
struct User {
    id: i64,
    username: String,
    posts: Vec<Post>,
    avatar: Option<Image>,
}

#[derive(Clone)]
struct Post {
    id: i64,
    title: String,
    comments: Vec<Comment>,
}

#[derive(Clone)]
struct Comment {
    id: i64,
    body: String,
}

#[derive(Clone)]
struct Image {
    url: String,
}

#[derive(Clone)]
struct Product {
    price: i64,
    discount: u8,
}

// ---------------------------------------------------------------------------
// 1. Direct field mapping
// ---------------------------------------------------------------------------

/// Fields with identical names and types need no attribute at all; the
/// generated code falls back to the identity `FromResource` impl.
#[derive(Resource)]
#[source(Todo)]
struct TodoResource {
    id: i64,
    title: String,
    done: bool,
}

#[test]
fn direct_mapping_copies_fields_with_matching_names_and_types() {
    let todo = Todo {
        id: 1,
        title: "buy milk".into(),
        done: false,
        created_at: 0,
    };

    let resource = TodoResource::from_source(todo);

    assert_eq!(resource.id, 1);
    assert_eq!(resource.title, "buy milk");
    assert!(!resource.done);
}

#[test]
fn direct_mapping_ignores_source_fields_the_resource_does_not_declare() {
    // `Todo::created_at` is simply not read. A resource is a *projection*, so
    // omitting a field is the normal case, not an error.
    let todo = Todo {
        id: 2,
        title: "ignored field".into(),
        done: true,
        created_at: 1_700_000_000,
    };

    let resource = TodoResource::from_source(todo);
    assert!(resource.done);
}

#[test]
fn from_and_into_are_both_available() {
    // The design document's API: `TodoResource::from(todo)`.
    let a = TodoResource::from(Todo {
        id: 3,
        title: "via From".into(),
        done: false,
        created_at: 0,
    });
    assert_eq!(a.id, 3);

    // ...and the `Into` side of it, which comes for free.
    let b: TodoResource = Todo {
        id: 4,
        title: "via Into".into(),
        done: false,
        created_at: 0,
    }
    .into();
    assert_eq!(b.id, 4);

    // ...and the underlying trait, for generic code.
    let c = <TodoResource as FromResource<Todo>>::from_resource(Todo {
        id: 5,
        title: "via trait".into(),
        done: false,
        created_at: 0,
    });
    assert_eq!(c.id, 5);
}

// ---------------------------------------------------------------------------
// 2. Rename field
// ---------------------------------------------------------------------------

/// `#[source(username)]` reads a differently named source field. The API key
/// stays `user_name`; only the *origin* changes.
#[derive(Resource)]
#[source(User)]
struct RenamedUserResource {
    id: i64,
    #[source(username)]
    user_name: String,
}

#[test]
fn source_attribute_reads_from_a_differently_named_field() {
    let user = User {
        id: 10,
        username: "ferris".into(),
        posts: Vec::new(),
        avatar: None,
    };

    let resource = RenamedUserResource::from_source(user);

    assert_eq!(resource.id, 10);
    assert_eq!(resource.user_name, "ferris");
}

// ---------------------------------------------------------------------------
// 3. Transform
// ---------------------------------------------------------------------------

/// Formats a timestamp for the API. Takes exactly one field, by value, and
/// owns the whole `i64 -> String` conversion.
fn format_timestamp(value: i64) -> String {
    format!("ts:{value}")
}

/// Normalises a title for display. Shows that a transform may keep the same
/// type and still be useful.
fn shout(value: String) -> String {
    value.to_uppercase()
}

#[derive(Resource)]
#[source(Todo)]
struct TransformedTodoResource {
    id: i64,
    #[transform(shout)]
    title: String,
    #[transform(format_timestamp)]
    created_at: String,
}

#[test]
fn transform_converts_a_single_field_value() {
    let todo = Todo {
        id: 7,
        title: "write tests".into(),
        done: false,
        created_at: 1_700_000_000,
    };

    let resource = TransformedTodoResource::from_source(todo);

    assert_eq!(resource.id, 7);
    assert_eq!(resource.title, "WRITE TESTS");
    assert_eq!(resource.created_at, "ts:1700000000");
}

/// Renaming and transforming compose: read `username`, expose `handle`.
fn at_prefixed(value: String) -> String {
    format!("@{value}")
}

#[derive(Resource)]
#[source(User)]
struct HandleResource {
    #[source(username)]
    #[transform(at_prefixed)]
    handle: String,
}

#[test]
fn transform_and_rename_can_be_combined_on_one_field() {
    let user = User {
        id: 1,
        username: "ferris".into(),
        posts: Vec::new(),
        avatar: None,
    };

    assert_eq!(HandleResource::from_source(user).handle, "@ferris");
}

// ---------------------------------------------------------------------------
// 4. Compute
// ---------------------------------------------------------------------------

/// Derived flag: does this product have any discount at all?
fn has_discount(product: &Product) -> bool {
    product.discount > 0
}

/// Derived value: the price the customer actually pays.
///
/// This is exactly the case `#[transform]` cannot express, because it needs two
/// fields at once.
fn real_price(product: &Product) -> i64 {
    product.price - (product.price * i64::from(product.discount)) / 100
}

#[derive(Resource)]
#[source(Product)]
struct ProductResource {
    price: i64,
    discount: u8,
    #[compute(has_discount)]
    has_discount: bool,
    #[compute(real_price)]
    real_price: i64,
}

#[test]
fn compute_derives_values_from_the_whole_source() {
    // The worked example from the design document.
    let resource = ProductResource::from_source(Product {
        price: 1200,
        discount: 15,
    });

    assert_eq!(resource.price, 1200);
    assert_eq!(resource.discount, 15);
    assert!(resource.has_discount);
    assert_eq!(resource.real_price, 1020);
}

#[test]
fn compute_handles_the_zero_discount_edge_case() {
    let resource = ProductResource::from_source(Product {
        price: 500,
        discount: 0,
    });

    assert!(!resource.has_discount);
    assert_eq!(resource.real_price, 500);
}

/// Regression guard: a computed field declared *after* a moved-out field must
/// still compile and still see the whole source. This only works because the
/// generated code hoists compute calls above the struct literal.
fn describe_todo(todo: &Todo) -> String {
    format!("{} ({})", todo.title, if todo.done { "done" } else { "open" })
}

#[derive(Resource)]
#[source(Todo)]
struct ComputeAfterMoveResource {
    title: String,
    #[compute(describe_todo)]
    summary: String,
}

#[test]
fn compute_runs_before_any_field_is_moved_out_of_the_source() {
    let resource = ComputeAfterMoveResource::from_source(Todo {
        id: 1,
        title: "ship it".into(),
        done: true,
        created_at: 0,
    });

    assert_eq!(resource.title, "ship it");
    assert_eq!(resource.summary, "ship it (done)");
}

// ---------------------------------------------------------------------------
// 5. Nested resources + 6. Composition
// ---------------------------------------------------------------------------

#[derive(Resource)]
#[source(Comment)]
struct CommentResource {
    id: i64,
    body: String,
}

#[derive(Resource)]
#[source(Post)]
struct PostResource {
    id: i64,
    title: String,
    // Two levels of nesting: `Vec<Comment>` -> `Vec<CommentResource>`. No
    // attribute is needed; the written type carries all the information.
    comments: Vec<CommentResource>,
}

#[derive(Resource)]
#[source(Image)]
struct ImageResource {
    url: String,
}

#[derive(Resource)]
#[source(User)]
struct UserResource {
    id: i64,
    #[source(username)]
    user_name: String,
    posts: Vec<PostResource>,
    avatar: Option<ImageResource>,
}

fn sample_user() -> User {
    User {
        id: 1,
        username: "ferris".into(),
        posts: vec![
            Post {
                id: 11,
                title: "hello".into(),
                comments: vec![
                    Comment { id: 101, body: "first".into() },
                    Comment { id: 102, body: "second".into() },
                ],
            },
            Post {
                id: 12,
                title: "world".into(),
                comments: Vec::new(),
            },
        ],
        avatar: Some(Image {
            url: "https://example.test/a.png".into(),
        }),
    }
}

#[test]
fn nested_resources_are_resolved_through_the_written_type() {
    let resource = UserResource::from_source(sample_user());

    assert_eq!(resource.id, 1);
    assert_eq!(resource.user_name, "ferris");

    // Vec<Post> became Vec<PostResource>.
    assert_eq!(resource.posts.len(), 2);
    assert_eq!(resource.posts[0].id, 11);
    assert_eq!(resource.posts[0].title, "hello");

    // ...and the nesting recursed one level deeper.
    assert_eq!(resource.posts[0].comments.len(), 2);
    assert_eq!(resource.posts[0].comments[0].id, 101);
    assert_eq!(resource.posts[0].comments[1].body, "second");
    assert!(resource.posts[1].comments.is_empty());
}

#[test]
fn optional_nested_resources_are_mapped_through_option() {
    let present = UserResource::from_source(sample_user());
    assert_eq!(
        present.avatar.map(|image| image.url),
        Some("https://example.test/a.png".to_owned())
    );

    let mut without_avatar = sample_user();
    without_avatar.avatar = None;
    let absent = UserResource::from_source(without_avatar);
    assert!(absent.avatar.is_none());
}

// ---------------------------------------------------------------------------
// Collections
// ---------------------------------------------------------------------------

#[test]
fn generated_collection_helper_maps_every_item() {
    let todos = vec![
        Todo { id: 1, title: "a".into(), done: false, created_at: 0 },
        Todo { id: 2, title: "b".into(), done: true, created_at: 0 },
    ];

    // The API from the design document: `TodoResource::collection(todos)`.
    let resources = TodoResource::collection(todos);

    assert_eq!(resources.len(), 2);
    assert_eq!(resources[0].title, "a");
    assert!(resources[1].done);
}

#[test]
fn collection_accepts_any_into_iterator_not_just_vec() {
    // Arrays and lazy iterators work because the bound is `IntoIterator`.
    let from_iterator = TodoResource::collection((1..=3).map(|id| Todo {
        id,
        title: id.to_string(),
        done: false,
        created_at: 0,
    }));
    assert_eq!(from_iterator.len(), 3);
    assert_eq!(from_iterator[2].title, "3");

    // The free helper in `macros-core` reaches the same result; it exists for
    // generic call sites where the resource type is a type parameter.
    let from_helper: Vec<TodoResource> = collection([Todo {
        id: 9,
        title: "single".into(),
        done: false,
        created_at: 0,
    }]);
    assert_eq!(from_helper[0].id, 9);
}

// ---------------------------------------------------------------------------
// Final representation
// ---------------------------------------------------------------------------

/// The Resource system is serialization-agnostic, but the point of the whole
/// pipeline is the JSON that comes out the other end, so one test pins it.
///
/// Note how `#[derive(Serialize)]` and `#[derive(Resource)]` coexist without
/// interfering: each ignores the other's attributes.
#[derive(Resource, serde::Serialize)]
#[source(Product)]
struct SerializableProductResource {
    price: i64,
    discount: u8,
    #[compute(has_discount)]
    has_discount: bool,
    #[compute(real_price)]
    real_price: i64,
}

#[test]
fn serialized_output_matches_the_documented_json_shape() {
    let resource = SerializableProductResource::from_source(Product {
        price: 1200,
        discount: 15,
    });

    let actual: serde_json::Value = serde_json::to_value(&resource).expect("serializable");
    let expected = serde_json::json!({
        "price": 1200,
        "discount": 15,
        "has_discount": true,
        "real_price": 1020
    });

    assert_eq!(actual, expected);
}

#[test]
fn serialized_collection_is_a_plain_json_array() {
    let resources = SerializableProductResource::collection(vec![
        Product { price: 100, discount: 0 },
        Product { price: 200, discount: 50 },
    ]);

    let actual: serde_json::Value = serde_json::to_value(&resources).expect("serializable");

    assert!(actual.is_array());
    assert_eq!(actual[1]["real_price"], 100);
}

// ---------------------------------------------------------------------------
// Placeholder: compile-fail tests.
//
// The most valuable tests for a system whose selling point is "invalid mappings
// do not compile" are the ones asserting that bad code is *rejected*. Those
// need the `trybuild` crate, which is left out of the MVP to keep the
// dependency set minimal:
//
//   // macros/Cargo.toml
//   // [dev-dependencies]
//   // trybuild = "1"
//
//   // macros/tests/compile_fail.rs
//   // #[test]
//   // fn invalid_resources_are_rejected() {
//   //     let t = trybuild::TestCases::new();
//   //     t.compile_fail("tests/compile_fail/*.rs");
//   // }
//
// The cases worth pinning, each of which fails today with a readable error:
//   * missing struct-level `#[source(...)]`
//   * `#[source(does_not_exist)]` -> "no field `does_not_exist` on type `Todo`"
//   * mapping `i64` onto `String` -> unsatisfied `FromResource` bound
//   * `#[compute]` combined with `#[source]` -> our own diagnostic
//   * a transform function with the wrong argument or return type
//   * a tuple struct or an enum
// ---------------------------------------------------------------------------
