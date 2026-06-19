//! A Laravel-inspired model factory for Rust, driven by `#[derive(Factory)]`.
//!
//! Annotate a struct's fields, then build instances fluently. The single
//! `create` method returns one value or a `Vec` depending on whether `count`
//! was called — see the builder types below.
//!
//! ```ignore
//! use macros_core::factory::Pawn; // brings the trait (for `::factory()`) into scope
//!
//! #[derive(Pawn)]
//! struct User {
//!     #[fake(name)]
//!     name: String,
//!     #[value("2020-01-01")]
//!     created_at: String,
//! }
//!
//! let many = User::factory().count(10).create(); // Vec<User>
//! let one = User::factory().create();            // User
//! ```


/// A type that can produce populated instances of itself for tests and seeding.
///
/// Implemented automatically by `#[derive(Pawn)]`; you should not implement
/// it by hand.
pub trait Pawn: Sized {
    /// Builds a single instance, populating every field from its attribute
    /// (`#[fake]`, `#[generator]`, `#[value]`) or from `Default` when no
    /// attribute is present.
    fn definition() -> Self;

    /// Starts a fluent builder.
    ///
    /// The returned builder produces a single instance via
    /// [`FactoryBuilder::create`]; call [`FactoryBuilder::count`] to produce many.
    fn factory() -> FactoryBuilder<Self> {
        FactoryBuilder::new()
    }
}

/// A mutation applied to every produced instance after it is built.
type State<T> = Box<dyn Fn(&mut T)>;

/// Builds a single instance by default.
///
/// Calling [`FactoryBuilder::count`] transitions to a [`FactoryBatchBuilder`]
/// whose `create` returns a `Vec`. This type-state split is what lets one
/// `create` method return either a single value or many while staying fully
/// type-checked at compile time.
#[must_use = "a factory builder does nothing until you call `.create()`"]
pub struct FactoryBuilder<T: Pawn> {
    states: Vec<State<T>>,
}

impl<T: Pawn> FactoryBuilder<T> {
    /// Creates an empty builder. Prefer [`Factory::factory`].
    pub fn new() -> Self {
        Self { states: Vec::new() }
    }

    /// Registers a mutation run against every instance after it is built.
    ///
    /// States are applied in registration order.
    pub fn state<F>(mut self, mutate: F) -> Self
    where
        F: Fn(&mut T) + 'static,
    {
        self.states.push(Box::new(mutate));
        self
    }

    /// Switches to building `count` instances, returning a batch builder.
    pub fn count(self, count: usize) -> FactoryBatchBuilder<T> {
        FactoryBatchBuilder {
            count,
            states: self.states,
        }
    }

    /// Builds and returns a single instance.
    pub fn create(self) -> T {
        build_one(&self.states)
    }
}

impl<T: Pawn> Default for FactoryBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Builds a `Vec` of instances. Created by [`FactoryBuilder::count`].
#[must_use = "a factory builder does nothing until you call `.create()`"]
pub struct FactoryBatchBuilder<T: Pawn> {
    count: usize,
    states: Vec<State<T>>,
}

impl<T: Pawn> FactoryBatchBuilder<T> {
    /// Registers a mutation run against every instance after it is built.
    pub fn state<F>(mut self, mutate: F) -> Self
    where
        F: Fn(&mut T) + 'static,
    {
        self.states.push(Box::new(mutate));
        self
    }

    /// Updates how many instances will be built.
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }

    /// Builds and returns all instances.
    pub fn create(self) -> Vec<T> {
        (0..self.count).map(|_| build_one(&self.states)).collect()
    }
}

/// Builds one instance from its definition and applies every state mutation in order.
fn build_one<T: Pawn>(states: &[State<T>]) -> T {
    let mut instance = T::definition();
    for mutate in states {
        mutate(&mut instance);
    }
    instance
}