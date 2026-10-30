

# Architecture Manifesto: Rust Explicitness with Laravel-Level DX

> **Status:** Accepted · **Version:** 1.0 · **Date:** 2026-10-30 **Scope:** All system infrastructure modules (auth, mail, cache, queue, session, etc.) built on `axum` \+ `sqlx`\
> **Audience:** All team members. This document is the default architectural contract. Any deviation must be justified through an ADR.


---

## Note on the Accuracy of This Document

- **Decisions and principles** in this document are the result of design reasoning and are binding on the team.
- **Code snippets are sketches** (design representations), not necessarily compilable code. Before copying them, verify compatibility with the current versions of `axum`, `sqlx`, and other crates.
- Appendix C lists which technical claims have been verified and which have not.

---

## 1\. One-Page Summary (TL;DR)

We want to redesign the important parts of Laravel in Rust, with **good DX** but **without magic**. These ten rules summarize the entire document:

1. **Rust code is the single source of truth.** Wiring, implementation selection, and contracts live in code and types—not in strings, configuration files, or runtime registries. Configuration only provides **values**.
2. **We do not build Containers, Service Providers, Facades, or Service Locators.** (See Section 4 for the reasoning.)
3. We have one **Composition Root**: the `main` function. It is the only place where all modules know about one another and are connected.
4. **Dependencies are injected through constructors.** Each module explicitly declares what it needs through `new(...)` or a `builder`.
5. **Dependency direction is unidirectional and acyclic.** A cycle means the boundaries have been drawn incorrectly.
6. **Queries use traits or direct calls; side effects use events.**
7. **Generics stay inside modules; non-generic types are used across module seams** and in `AppState`.
8. **Syntactic sugar (macros, derives, builders) is allowed only when it expands into code that could have been written manually and can be inspected** (`cargo expand`).
9. **Security is locked into the defaults.** The secure path must be the easy path, and the insecure path must be difficult or impossible.
10. **Every new module must have a DX design** (consumer-facing code) **and a boundary design** before implementation, and must pass the review gate (Section 12).

---

## 2\. Context: Why Laravel, and What We Do Not Want from It

### What We Want from Laravel (DX)

- Simple tasks should require **little code**.
- **Contracts and separation of responsibilities** should be clear (e.g., Guard / Provider / Session Driver).
- **Everything should be included out of the box** (batteries included), while every component remains replaceable.
- It should provide **clear extension points** (easy customization for 80% of use cases and full flexibility for the rest).
- Testing experience and error messages should be understandable.

### What We Do Not Want (Magic)

- **Hidden global state** (Facades such as `Auth::user()`).
- **Strings instead of types** (`'guard' => 'api'`, `config('x.y')`).
- **Runtime resolution** (autowiring through reflection, `bind`, and `make`).
- **Invisible bootstrapping order** (`register`/`boot`, package ordering).
- **Late-discovered errors** (a typo in a guard name is discovered in production).
- **Configuration-driven wiring** (changing one string in `.env` switches the entire implementation).

### Goals

- Wiring errors must be detected at **compile time**.
- Reading the code should be enough to understand where a dependency comes from (**traceability**).
- Testing must be possible by replacing dependencies **without special tooling**.
- Boilerplate must be kept to the **minimum defensible amount** (Section 10).

### Non-Goals (Things We Intentionally Do Not Do)

- A 1:1 copy of Laravel's API. We take the **ideas**, not the surface form.
- An Eloquent-style ORM. We use `sqlx` (Section 8).
- Package auto-discovery and automatic registration.
- A runtime plugin system (enabling/disabling modules without recompilation). If this becomes necessary, it requires a separate ADR.
- Managing Authorization inside the Auth module. (It remains separate, following the same boundary that exists in Laravel.)

---

## 3\. Key Terms

| Term | Meaning in This Document |
| --- | --- |
| **Module** | An infrastructure capability with a clear boundary (auth, mail, cache, etc.). Usually consists of multiple crates. |
| **Composition Root** | `main`; the place where modules are constructed and connected. |
| **Seam** | A boundary between two modules or between a module and infrastructure, where they communicate through a trait or public type. |
| **Deps** | A struct that explicitly declares a module's dependencies. |
| **AppState** | The shared application struct that holds constructed services and is split into smaller parts through `FromRef`. |
| **Port / Adapter** | Port: a trait that a module requires from the outside. Adapter: its implementation (sqlx, redis, smtp). |

---

## 4\. Core Decision: ADR-0001 "Explicit Composition, No Container"

### 4.1. Decision

> **We will not build or use any Container, Provider, Facade, or Service Locator.**Instead, we use a **Composition Convention**: constructor injection, one Composition Root, `AppState` with substates, and modules following a standard structure (Section 7).

### 4.2. Why It Was Confusing: A "Container" Performs Five Separate Jobs

Treating a "container" as one indivisible thing makes decision-making difficult. In Laravel, it performs five jobs together. Once separated, it becomes clear which capabilities we actually need and which are magic:

| # | Responsibility | In Laravel | In Our Architecture | Do We Need It? |
| --- | --- | --- | --- | --- |
| 1 | **Storage and access** (registering services and making them available) | `app()->make()` | `AppState` / struct fields | **Yes, we have it** |
| 2 | **Lifecycle management** (singleton, per-request) | `singleton`, `scoped` | `Arc<T>` for shared instances; extractors for per-request values | **Yes, we have it** |
| 3 | **Automatic wiring** (building a dependency graph through reflection on types) | autowiring | **Compiler +****`main`** | **No** (the primary magic) |
| 4 | **Late binding and string-based replacement** | `bind('x', ...)`, `config('cache.default')` | `match` / generics / builders in code | **No** |
| 5 | **Bootstrapping order** | Service Provider (`register` / `boot`) | Order of statements in `main`; enforced by the compiler | **No** |

Conclusion: Only responsibilities 1 and 2 are genuinely necessary. Rust provides both without any library.

### 4.3. Why Laravel Needs a Container and We Do Not

- PHP starts from scratch for **every request**. The service graph must be built each time (or lazily). Since the language is dynamic, reflection is the only way to discover "what this class needs." Providers also exist to separate registration from usage and solve package-ordering problems.
- Rust is a **long-running process**. We build the graph **once in****`main`**, and
  The compiler validates dependency types and construction requirements. Dependency cycles are prevented through explicit composition conventions and architectural review.

### 4.4. Magic → Explicit Replacement

| In Laravel | Problem | In Our Architecture |
| --- | --- | --- |
| `Auth::user()` / Facades (global state) | Hidden state, difficult testing | **Extractor** in the handler signature: `user: SessionAuth<User>` or an explicit handle (`State<Cache>`) |
| `'guard' => 'api'` (string name) | Runtime typos | **Types**: each guard/driver is a type |
| `config/*.php` selecting implementations through strings | Hidden wiring, late errors | Wiring in **code** through a builder; configuration only provides values (TTL, URL, etc.) and may select at most one **variant of a closed enum** in `main` |
| `middleware('auth:api')` (string alias) | Incorrect aliases, invisible behavior | **An extractor in the handler means the requirement is enforced** (missing it fails to compile), or use an explicit Tower `layer` |
| Implicit and forgettable `session->migrate()` | Forgetting it creates a vulnerability | The secure operation is **built into the API**; `session.login(&user)` rotates the ID itself |
| Multiple guards/providers for multiple models | Runtime mix-ups | `User` and `Customer` are **two distinct types**; mixing them up is a compile-time error |
| Exceptions | Invisible error paths | `Result<_, XError>` with a typed enum |
| `app()->make(Foo::class)` (autowiring) | Dependencies disappear from signatures | Dependencies are visible in `new(...)` / `Deps` |
| Service Provider (`register` / `boot`) | Hidden ordering | Statement order in `main` \+ compiler |
| Events registered by string/class with listeners defined elsewhere | Scattered control flow | Typed event enum + explicit channel (Section 9) |
| Eloquent (relations, scopes, casts, model events) | Too much magic | `sqlx` with explicit queries (Section 8) |

### 4.5. Consequences

**Positive**

- Wiring errors are found at **compile time**.
- Everything is **traceable** (`Go to Definition` is enough).
- Simple testing: pass a fake dependency to the constructor.
- **Circular dependencies are impossible**, which is itself a design signal.
- Zero tooling cost (no DI library is required).

**Negative (Accepted Trade-offs)**

- **No auto-discovery.** In Laravel, `composer require` registers a package automatically. Here, consumers must write a few lines of wiring. This is the cost of explicitness, and the design goal is to keep those lines to the bare minimum.
- `main` grows as the system grows. (Solutions: Sections 4.8 and 5.)
- Requires **discipline**: rules only work when followed. (Solutions: review gate and PR checklist.)

### 4.6. Considered and Rejected Alternatives

| Option | Why It Was Rejected |
| --- | --- |
| **Service Locator with a type map** (`HashMap<TypeId, Box<dyn Any>>` and `container.get::<T>()`) | Dependencies disappear from signatures; a missing service turns a compile-time error into a runtime panic/500. It destroys exactly the explicitness we are trying to achieve. |
| **`Extension<T>`****for services** | It is effectively a small Service Locator and has the same problem (a missing value at runtime). We use `State` for services; `Extension` is only for data derived from the request itself. |
| **DI libraries for Rust** | They add dependency and learning costs while repeating what the compiler and `main` provide for free. As far as we know, they are not sufficiently mature either. This can be reconsidered if the triggers in Section 13 occur. |
| **Global Singleton** (`static` / `OnceLock` for services) | It is a Facade disguised as Rust: tests are not isolated and dependencies are hidden from signatures. |
| **Convention-heavy framework** (such as Loco, following the Rails style in Rust) | A reasonable choice, but not aligned with our "explicitness first" priority. |

### 4.7. Reversibility Asymmetry (Why This Decision Is Not a "Fatal Mistake")

- **If we start explicitly and later need a container:** Every module already receives its dependencies through `new(deps)`. Adding a container only changes _how modules are constructed_ in `main`. Modules remain untouched.
- **If we start with a container and later want to become explicit:** Every module containing `container.get()` must be rewritten.

Therefore, explicitness is the cheapest option that does not close any future doors. Its worst-case cost is "a little extra boilerplate."

### 4.8. What Replaces the Container (Two Tools)

- **`AppState`****\+****`FromRef`****:** Handlers receive only the parts they need, not the entire state (Section 5).
- **`Module`****as a structural convention** (not a central trait): Every module has `Deps`, `new`/`builder`, and `routes` (Section 7).

---

## 5\. The Composition Ladder (From Simple to Complex)

Rule: **Move up only when you experience real pain.** "We might need it later" is not a reason.

### Level 0: Constructor Injection

Every service explicitly receives its dependencies. This is the **foundation of everything**; subsequent levels merely package it differently.

```rust
let hasher  = Argon2::default();
let users   = PgUsers::new(pool.clone());
let auth    = Auth::new(users, hasher, sessions);
let billing = Billing::new(pool.clone(), auth.clone());
```

### Level 1: `AppState` \+ Substate with `FromRef`

The risk with `AppState` is that it becomes a "god object" and every handler receives the entire thing. The solution: each handler extracts **only the part it needs**.

```rust
#[derive(Clone, FromRef)]          // Requires the `macros` feature in axum
struct AppState {
    pool:    MySqlPool,
    auth:    Auth,
    mail:    Mail,
    cache:   Cache,
}

async fn invoices(State(mail): State<Mail>, State(cache): State<Cache>) { /* ... */ }
```

Rules:

- Every `AppState` field must be **cheap to Clone** (internally using `Arc`). `MySqlPool` qualifies.
- Handlers and extractors must **never receive the complete****`State<AppState>`**, unless there is a specific documented reason.
- `AppState` contains no logic; it is only a **holder**.

### Level 2: Module = `Deps` \+ `new`/`builder` \+ `routes`

This replaces Service Providers. Every module declares "what I need" and "what I provide":

```rust
pub struct Deps {
    pub db:   MySqlPool,
    pub auth: Auth,        // Explicit dependency on another module
}

pub struct Billing { /* ... */ }

impl Billing {
    pub fn new(deps: Deps) -> Self { /* ... */ }

    // Library modules (auth, mail, cache): generic over state
    pub fn routes<S>() -> Router<S>
    where
        S: Clone + Send + Sync + 'static,
        Billing: FromRef<S>,
        Auth: FromRef<S>,
    { /* ... */ }
}
```

Two permitted approaches to state in `routes`:

| Style | Intended Module | How It Works |
| --- | --- | --- |
| **A) Generic over****`S`****with a****`FromRef`****bound** (default) | Reusable infrastructure modules (auth, mail, cache) | `Router<S>` with the bound `Billing: FromRef<S>`. The application's state structure does not leak into the module. |
| **B) Module-specific state** (closed with `with_state`) | Self-contained application modules (e.g., an admin panel) | The module returns a `Router` that owns its state and is connected in `main` (possibly using `nest_service`; verify before use, Appendix C). |

### Level 3: Trait Boundary (Port), When Modules Need to Know About Each Other

When Module A needs Module B but must not be coupled to B's implementation, **A defines the interface it requires itself** (not B). This is equivalent to Laravel Contracts, but establishes the correct dependency direction.

```rust
// Inside the Billing module (the consumer defines its requirement)
#[async_trait::async_trait]
pub trait CustomerLookup: Send + Sync {
    async fn by_id(&self, id: CustomerId) -> Result<Option<Customer>, LookupError>;
}

// Inside main (the only place that knows about both)
let billing = Billing::new(Deps {
    customers: Arc::new(AuthCustomerLookup::new(auth.clone())),  // adapter
    ..deps
});
```

### Level 4: What Is Forbidden

Service Locators, type maps, `static`/`OnceLock` for services, and DI crates (Sections 4.6 and 14).

### When Should You Move Up a Level?

Only when there are **measurable pain triggers** (Section 13).

---

## 6\. Design Principles (With Rationale and Good/Bad Examples)

### P0. Rust Code Is the Single Source of Truth

- Implementation selection happens in code.
- Configuration provides values (URL, TTL, cookie name, maximum connections), and may select at most one **variant of a closed enum** in `main`:

```rust
// Good: Selection in code from a closed, compile-time-known set
let cache = match cfg.cache.driver {
    CacheDriver::Memory => Cache::new(MemoryStore::default()),
    CacheDriver::Redis  => Cache::new(RedisStore::connect(&cfg.cache.url).await?),
    CacheDriver::File   => Cache::new(FileStore::new(&cfg.cache.path)),
};

// Bad: A configuration string resolved by a runtime registry
let cache = registry.make(&cfg.cache.driver_name)?;
```

### P1. Explicit over Implicit

Dependencies must be visible in the **signature** (constructor, `Deps`, or extractor parameter). If you need text search to understand "where did this come from?", we have violated this principle.

### P2. Compile Time over Runtime

Every error that can be made unrepresentable through types should be made unrepresentable through types. For example: `User` and `Customer` are two types; `Password` and `PasswordHash` are two types.

### P3. Syntactic Sugar Only If It Is "Writable by Hand and Inspectable"

- Allowed: builders, extractors, `derive` implementations that generate only a simple impl, and macros that are understandable and inspectable through `cargo expand`.
- Forbidden: macros that hide **control flow**, or anything that performs global registration (such as `inventory`-style auto-registration), unless approved through an ADR.
- Test: "If I remove this macro, how much code would I have to write manually, and would it mean exactly the same thing?"

### P4. One Composition Root

Only `main` (or a `build_app()` function called by both `main` and tests) is allowed to know about **all** modules. No module should know about the complete `AppState`.

### P5. Unidirectional Dependencies

```text
          adapters (sqlx, axum, redis, smtp)
                 │  depend on
                 ▼
          module (service + ports/traits)
                 │  depends on
                 ▼
          core (types and traits; no IO)
```

- `core` does not depend on `axum`, `sqlx`, or any IO.
- Modules do not depend on the **implementations** of other modules; they depend only on traits they define themselves (or on another module's public type when there is a direct, acyclic dependency).
- A cycle is a design error. Solutions: move the dependency behind a trait, turn one side into an event, or extract a third shared crate.

### P6. Queries Use Traits or Direct Calls; Side Effects Use Events

| Requirement | Mechanism |
| --- | --- |
| "Who is this user?", "What is the cache value?" (I need an answer) | Direct call or trait |
| "Do X after login" (side effect; no answer needed) | Event (Section 9) |

### P7. Generics Inside; Non-Generic Types Across Seams

- Inside a module/crate: generics and static dispatch are allowed.
- Across **seams**, in `AppState`, and in handler signatures: use **non-generic types**. For example, `Auth`, not `Auth<U, H, S, T>`.
- Reason: leaking generic parameters into `AppState` and every handler signature creates even more boilerplate than Laravel. The cost of dynamic dispatch is negligible compared to IO.

Recommended pattern: a public non-generic type that uses type erasure internally:

```rust
#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn send(&self, msg: Message) -> Result<(), TransportError>;
}

#[derive(Clone)]
pub struct Mail { transport: Arc<dyn Transport> }   // Non-generic public API

impl Mail {
    pub fn new(t: impl Transport + 'static) -> Self { Self { transport: Arc::new(t) } }
    pub async fn send(&self, msg: Message) -> Result<(), MailError> { /* ... */ }
}
```

**ADR-0002 (async in traits across seams):** `async fn` in traits is stable, but as far as we have verified, it is **not yet natively dyn-compatible** (its design is being tracked as part of Rust's 2026 project goals). Therefore, for traits intended to become `Arc<dyn ...>`, we use `async_trait` (or `dynosaur`). We consciously accept the cost of boxing each call. Once native support becomes stable, we will migrate in one pass. If a trait will never be used as `dyn` (only internal generics), use native `async fn`.

### P8. Secure by Default

- The secure path must be the **default** and the **easy** path; the insecure path must be explicit and difficult.
- Sensitive types (`Password`, `Secret<T>`, `Token`) must have **redacted****`Debug`****/****`Display`****implementations** so they never leak into logs.
- Constant-time secret comparison, secure cookie flags, expiration, rate-limit hooks, and non-disclosing error messages are part of the **reference implementation**, not merely "recommendations for consumers."

### P9. Typed Errors

- Every module has its own error enum (`thiserror`).
- `anyhow` is used only in `main` and application code, not in libraries or infrastructure modules.
- Converting errors into HTTP responses happens in the **Axum adapter layer** (`IntoResponse`), not in the core. Internal details (such as SQL errors) must not leak to users.

### P10. No Global State

No `static mut`, `lazy_static`/`OnceLock` for services, or `thread_local` for application state. (Exception: actual constants and stateless caches such as compiled regexes.)

### P11. DX Is a Design Outcome, Not an Add-on

Before implementation, **write the consumer-facing code first** (Section 12, Step 6). If the consumer-facing code is ugly or repetitive, change the design—not the documentation.

---

## 7\. Workspace Structure and Standard Module Pattern

### 7.1. Suggested Layout

```text
├── app                 # Application layer
├── core                # Shared primitives and abstractions
├── artisan             # CLI tooling
├── database            # Migrations and schema-related code
├── macros              # Project's macros
├── macros-corde        # Project's standalone macro libraries
├── graveyard           # DEPRECATED libraries
├── integration-tests   # Project integration tests
```

For small modules, multiple roles may be combined into a single crate using feature flags; however, the **logical boundaries** (folders/modules) must be preserved.

### 7.2. Module Anatomy (Template)

Every module has **all** of the following, even if it is simple:

| Component | Contents |
| --- | --- |
| **Domain Types** | IDs (newtypes), values, events |
| **Ports** | Minimal traits the module requires from the outside |
| **Service** | Non-generic public type, cheap to `Clone` |
| **Builder** | Service construction with secure defaults |
| **Config** | A `serde` struct for **values** (not implementation selection) |
| **Error** | Typed enum |
| **Events** | Enum of publishable events |
| **Adapters** | sqlx, axum, etc. in separate crates |
| **Testing** | In-memory implementation and helpers |
| **Docs** | One complete scenario (such as "web login" or "sending a welcome email") |

Skeleton:

```rust
// ---------- core ----------
pub struct CustomerId(/* ... */);                       // newtype
#[async_trait::async_trait]
pub trait CustomerStore: Send + Sync { /* minimal methods */ }
#[derive(Debug, thiserror::Error)]
pub enum BillingError { /* ... */ }

// ---------- <module> ----------
#[derive(Clone)]
pub struct Billing { inner: Arc<Inner> }                // cheap Clone
pub struct BillingBuilder { /* ... */ }
impl Billing { pub fn builder() -> BillingBuilder { /* ... */ } }
impl BillingBuilder {
    pub fn store(mut self, s: impl CustomerStore + 'static) -> Self { /* ... */ self }
    pub fn build(self) -> Result<Billing, BuildError> { /* ... */ }   // explicit construction error
}

// ---------- app ----------
// Extractors: generic S, with a FromRef bound
impl<S> FromRequestParts<S> for BillingCtx
where Billing: FromRef<S>, S: Send + Sync { /* ... */ }
pub fn routes<S>() -> Router<S> where S: Clone + Send + Sync + 'static, Billing: FromRef<S> { /* ... */ }
```

### 7.3. Composition Root (General Shape of `main`)

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1) Configuration: values only
    let cfg = Config::from_env()?;

    // 2) Infrastructure
    let pool = MySqlPoolOptions::new().max_connections(cfg.db.max).connect(&cfg.db.url).await?;
    let events = EventBus::new();

    // 3) Modules, in dependency order (the compiler enforces the order)
    let cache = Cache::builder().store(RedisStore::connect(&cfg.cache.url).await?).build()?;
    let mail  = Mail::builder().transport(SmtpTransport::from(&cfg.mail)).build()?;
    let auth  = Auth::builder()
        .identities(PgUsers::new(pool.clone()))
        .hasher(Argon2::default())
        .sessions(PgSessions::new(pool.clone()), cfg.session.clone())
        .events(events.sender())
        .build()?;

    // 4) State
    let state = AppState { pool, cache, mail, auth };

    // 5) Router
    let app = Router::new()
        .merge(auth_axum::routes())            // Library modules: generic over S
        .nest("/billing", billing::routes())
        .layer(auth_axum::session_layer())
        .with_state(state);

    axum::serve(tokio::net::TcpListener::bind(cfg.addr).await?, app).await?;
    Ok(())
}
```

Growth rule: When `main` becomes large, split it into **small functions without magic** (`build_infra`, `build_modules`, `build_router`), not into a runtime registry.

---

## 8\. Data Layer: `sqlx` Without an ORM

> Examples use PostgreSQL. Decision: **We type against one specific database** and do not invest in `sqlx::Any` or an abstraction for "all databases" unless a real need is documented.

### 8.1. Principles

1. **SQL is explicit.** There are no magical relations/scopes/casts/model events. Every query is visible in the code.
2. **`sqlx`****types do not cross seams.** Database rows (`Row`) stay in the adapter and are converted into domain types. Ports do not refer to `sqlx::Error`; errors are mapped to module errors.
3. **Table ownership:** Every table has **exactly one owning module**. Other modules read its data only through that module's service/Port. `JOIN` operations on another module's table are forbidden. (Exception: read/reporting models with an ADR.)
4. **Thin Repository:** Small functions/methods that execute and map one query. Business logic belongs in the service, not in an SQL wrapper.

### 8.2. Adapter Pattern

```rust
// core: error and Port without any reference to sqlx
#[async_trait::async_trait]
pub trait IdentityStore: Send + Sync {
    async fn by_login(&self, login: &str) -> Result<Option<User>, StoreError>;
}

// <module>-sqlx: adapter
#[derive(Clone)]
pub struct PgUsers { pool: MySqlPool }

#[derive(sqlx::FromRow)]
struct UserRow { id: i64, email: String, password_hash: String }

impl TryFrom<UserRow> for User { /* conversion and validation; error → StoreError */ }

#[async_trait::async_trait]
impl IdentityStore for PgUsers {
    async fn by_login(&self, login: &str) -> Result<Option<User>, StoreError> {
        let row = sqlx::query_as::<_, UserRow>(
            "SELECT id, email, password_hash FROM users WHERE email = $1",
        )
        .bind(login)
        .fetch_optional(&self.pool)
        .await
        .map_err(StoreError::from)?;

        row.map(User::try_from).transpose()
    }
}
```

### 8.3. Compile-Time Checked Queries

- Preferred: `query!`/`query_as!` macros that validate SQL at compile time.
- Requires `DATABASE_URL` at compile time **or** offline mode with stored metadata (`cargo sqlx prepare`, using the `.sqlx` directory committed to git).
- CI must verify that metadata is up to date (`cargo sqlx prepare --check`).
- Functions without macros (`query_as::<_, T>`) are allowed but must have adapter tests.

### 8.4. Transactions and Migrations

**Transactions**

- **The service (use case) defines the transaction boundary**, not individual repositories.
- Low-level adapter functions/methods receive a **connection/executor** instead of `&MySqlPool`, so they can be composed within a transaction:

```rust
pub async fn insert_user(conn: &mut PgConnection, u: &NewUser) -> Result<UserId, StoreError> { /* ... */ }
pub async fn insert_profile(conn: &mut PgConnection, p: &NewProfile) -> Result<(), StoreError> { /* ... */ }

// In the service/use case
let mut tx = pool.begin().await?;
let id = insert_user(&mut *tx, &new_user).await?;
insert_profile(&mut *tx, &new_profile.with(id)).await?;
tx.commit().await?;
```

- **Cross-module transactions are especially discouraged.** If two modules must change "at the same time," either the boundary is wrong or the **outbox pattern** should be used (writing the event to a table in the same transaction and processing it separately).

**Migrations**

- A central `database/src/migrations/` directory owned by the **application**.
- File naming: `<timestamp>_<module>_<description>.rs`; for example, `20261030120000_auth_create_sessions.rs`.
- Each module lists the **tables it owns** in its documentation, and its PR includes its migrations.
- **Open Question (OQ-2):** We have not yet investigated whether each module can carry its own migrations independently without conflicting with the `sqlx` migration history table.

### 8.5. Pool

- **One** `MySqlPool` or `SqlitePool` is created in `main` and cloned (cheaply; internally it uses `Arc`).
- Pool settings (maximum connections, timeout) come from `Config` and are visible in `main`, not hidden inside a module.

---

## 9\. Events, Configuration, and Errors (Shared Contracts)

### 9.1. Events

**Rules**

- Every module has an event `enum` describing **past facts** (`LoggedIn`, `PasswordChanged`), not commands (`SendWelcomeEmail`).
- An event carries an **identifier**, not a heavy object.
- The publisher does not wait for subscriber results, and subscriber errors do not affect it.
- Subscribers must be **idempotent**.
- Subscription happens explicitly in **`main`**, not through automatic registration.

```rust
// Inside the auth module
#[derive(Clone, Debug)]
pub enum AuthEvent {
    LoggedIn  { user_id: UserId },
    LoginFailed { login_hint: String },
    LoggedOut { user_id: UserId },
}

// Builder: the module receives only a Sender
Auth::builder().events(tx.clone());

// main: explicit subscriber
let mut rx = tx.subscribe();            // tokio::sync::broadcast
let mail = mail.clone();
tokio::spawn(async move {
    while let Ok(ev) = rx.recv().await {
        if let AuthEvent::LoggedIn { user_id } = ev { /* ... */ }
    }
});
```

**Honest limitation:** `broadcast` is in-memory. If the process dies, unsent events are lost, and a slow subscriber may miss messages. For **critical** side effects (such as issuing an invoice), use the outbox pattern (Section 8.4).

### 9.2. Configuration

- Every module has a `serde` struct for **values**, with secure `Default` values.
- Loading happens in `main` (from environment variables/files), and configuration reaches the module as **typed data**. There is no string-based `config("a.b.c")`.
- Validation happens in `build()` and returns a clear error (e.g., "`session.ttl` cannot be zero").
- Secrets are stored using `Secret<T>` (without `Debug`).
- Configuration **does not select implementations**; at most, it selects one variant of a closed enum (P0).

### 9.3. Errors

- One error enum per module using `thiserror`, with a separate **construction error** (`BuildError`) from runtime errors.
- Mapping to HTTP happens only in the Axum adapter; security-related messages must remain consistent (e.g., "Invalid email or password" without revealing whether the user exists).

---

## 10\. DX Budget: Where Should We Spend Boilerplate?

### 10.1. Numerical Targets (A Budget, Not a Sacred Rule)

| Scenario | Target |
| --- | --- |
| Connecting a module with default settings in `main` | Approximately 3–5 lines |
| Requiring authentication in a handler | 0 extra lines; only an extractor in the signature |
| Adding a new implementation for a Port (e.g., a new store) | Only one `impl` \+ one line in `main` |
| Writing a test with an authenticated user | One line (`acting_as`) |
| Builder construction error message | The exact name of what is missing |

If a design consistently exceeds this budget, treat it as a **sign of weak design**.

### 10.2. Permitted DX Tools (All Explicit and Visible)

- **Builder with secure defaults**, and when needed, a **typestate builder** so required fields are checked at compile time.
- **Extractors** instead of Facades: `SessionAuth<User>`, `BearerAuth<Customer>`.
- **`FromRef`****\+****`#[derive(FromRef)]`** for substates.
- A small, explicit **`prelude`** for each module.
- **`#[diagnostic::on_unimplemented]`** on key traits so compile errors explain what needs to be done.
- **`build_app()`** shared between `main` and tests.
- **Scenario-based documentation** (not just API lists): every module has one "complete scenario."

### 10.3. Boilerplate We Accept and Boilerplate We Reject

| We Accept (Valuable Explicitness) | We Reject (Noise Without Value) |
| --- | --- |
| Wiring statements in `main` | Repeating the same dependency in four modules (→ shared `Deps`) |
| Manual `impl FromRef` when no derive is available | Unnecessary scattered `.clone()` calls on types that are not cheap to clone |
| Defining a trait for a Port | Writing the same trait in multiple crates (→ shared crate) |
| Writing a `match` on a configuration variant | Driver name string + registry |
| Converting errors with `From`/`map_err` | `unwrap`/`expect` in runtime paths just to avoid dealing with errors |

### 10.4. Measurement

During every module review, perform a **consumer exercise**: someone who did not author the module connects it to a sample application using only the README. Record the number of wiring lines and where they encountered difficulties.

---

## 11\. Testing

| Layer | Approach |
| --- | --- |
| **Unit** | Service tested using an in-memory implementation from `<module>-testing`; no IO |
| **Adapter (database)** | `sqlx` tests against a real, isolated database (`#[sqlx::test]`, which provides a clean database for each test) |
| **HTTP** | `build_app()` \+ calling the Router without a network (e.g., using `tower::ServiceExt::oneshot`) |
| **Security** | Attack-oriented scenarios (cookie forgery, expired tokens, replay, injection) as permanent tests for every module |

Rules:

- Every Port has at least **two** implementations (one real, one in-memory). If building a second implementation is difficult, the Port is poorly designed.
- Tests use the **same****`build_app()`** as `main`, ensuring that tested wiring is the same as production wiring.
- Helpers such as `acting_as(user)` belong in `<module>-testing`, not production code.

---

## 12\. Process for Adding a New Module (auth, mail, cache, queue, ...)

This is the **most important operational section of this document**. Every new module must go through these steps in order. Each step has a **specific deliverable** recorded in the PR or the module design document (Appendix B template). **Implementation starts after Step 6, not before.**

```text
0 Scope ─▶ 1 Laravel Analysis ─▶ 2 Independent Axes ─▶ 3 Dependencies ─▶ 4 Contracts
                                                                            │
9 ADR & Gate ◀─ 8 Tests & Scenario ◀─ 7 Reference Implementation ◀─ 6 DX Design ◀─ 5 Config/Events/Security
```

### Step 0: Scope and Justification

**Questions:** What problem does this module solve? What is **outside** its scope? Which Laravel capability is our reference? Who are its consumers (handlers, other modules, CLI tools)?

**Deliverable:** A one-page scope document with a "Non-Goals" section.

**Exit criterion:** You can explain in two sentences what the module is and what it is **not**.

### Step 1: Laravel Analysis (List the Magic)

For the corresponding Laravel capability, classify each component in the following table:

| Laravel Component | Magic or Explicit? | Decision (Keep / Replace / Remove) | Reason |
| --- | --- | --- | --- |
| Example: Selecting a cache driver through `CACHE_DRIVER` | Magic (string + registry) | **Replace** with a closed enum + `match` in `main` | P0 |
| Example: `Cache::remember(...)` | Good explicitness (clear API) | **Keep** (as a method on `Cache`) | DX |
| Example: `Cache::` Facade | Magic (global state) | **Remove**; use `State<Cache>` | P10 |

**Deliverable:** Completed table.

**Exit criterion:** Every instance of "magic" has an explicit replacement or a reason for removal.

### Step 2: Independent Axes of Change

Ask: _What changes independently of what else?_ Each axis becomes either a **Port (trait)** or a **separate type**. Example questions:

- What defines **how** something is done (mechanism)? What defines **where** it is stored/sent (adapter)? What defines **who** someone is (identity)?
- If I change X, must Y also change? If yes, they are not independent axes.
- Which axes are **stateful** and which are **stateless**? (Keep them separate.)

**Deliverable:** A list of axes and a mapping of each axis to a Port/type.

**Exit criterion:** Every axis can be changed **without touching the others**.

### Step 3: Dependencies and Directions

- What does the module **require** (`Deps`), and what does it **provide**?
- Draw the dependency graph and check for **cycles**.
- Classify every interaction with another module as either a **query** (trait/direct call) or a **side effect** (event).
- Data ownership: which tables **belong to this module**? (Section 8.1, Rule 3)

**Deliverable:** Dependency graph + `Deps` table + list of owned tables.

**Exit criterion:** No cycles; no `JOIN` against another module's tables; every dependency has a reason.

### Step 4: Contracts (Ports)

- Keep them **minimal**: write only the methods consumers actually need.
- Separate stateful from stateless (e.g., `Guard` and `StatefulGuard`).
- Types: use newtypes for identifiers and secrets.
- Decide between **generics and****`dyn`** (P7), and consider **ADR-0002** for async in traits.
- Errors: typed enum (P9), with a separate `BuildError`.

**Deliverable:** Trait and type signatures (without implementations).

**Exit criterion:** No `sqlx`/`axum`/IO types appear in `/core`; no generic parameters leak into `AppState`.

### Step 5: Configuration, Events, and Security

- **Configuration:** Values, not implementation selection. Secure defaults. Validation in `build()`.
- **Events:** Event enum describing past facts. Which side effects are critical (→ outbox)?
- **Security:** Module threat checklist (e.g., information disclosure through errors, constant-time comparison, expiration, rate limiting, secrets not being logged, cookie flags, replay). Every outcome must be **enforced in the API**, not merely documented.

**Deliverable:** Configuration struct, event enum, threat-and-control table.

**Exit criterion:** The "insecure path" either does not exist or must be explicitly written.

### Step 6: DX Design (Write Consumer Code First)

**Before implementation**, write these three code snippets (they may be pseudocode):

1. **Wiring in****`main`** (how is the module connected?)
2. **One consumer handler** (how is the module used?)
3. **One consumer test** (how is the module replaced?)

Then compare them with the DX budget (Section 10.1). If the code is ugly, repetitive, or full of generics, **change the design** (return to Steps 2–4).

**Deliverable:** Three code snippets in the design document.

**Exit criterion:** The DX budget is met, and another team member can understand the consumer code without explanation.

### Step 7: Reference Implementation and the "Three Implementations Test"

- Implement `core` first, then the service, then adapters, and finally `app`.
- **Before finalizing Ports, write at least three implementations for them:** one in-memory, one real, and one "unusual" implementation that challenges the assumptions. If any implementation requires a hack, **the Port is wrong**.
    - Cache example: memory, Redis, file.
    - Mail example: SMTP, log/in-memory, cloud API.
    - Session example: memory, sqlx, signed cookie.

**Deliverable:** Three implementations using the same Port.

**Exit criterion:** The third implementation required no changes to `core`.

### Step 8: Testing and Documented Scenario

- Tests for the layers in Section 11 + security tests from Step 5.
- One **complete scenario** in the README (such as "web login with sessions" or "sending a welcome email"), including a responsibility map: what the module does and what the developer does (the same `[Module]/[Developer]` table).

**Exit criterion:** The consumer exercise (Section 10.4) succeeds.

### Step 9: ADR and Review Gate

Write a short ADR for every non-default decision (Appendix A template). Then pass this **gate**:

**Review Gate (Every Answer Must Be "Yes")**

1. Is every dependency visible in the signature/`Deps`?
2. Are there no circular dependencies?
3. Is `core` free from IO and framework dependencies?
4. Do no generics leak across seams or into `AppState`?
5. Are there no global state, string-based registries, or Service Locators?
6. Is every macro/derive inspectable and equivalent to handwritten code?
7. Are defaults secure and is the insecure path difficult?
8. Does configuration provide values only?
9. Does every Port have three implementations (or at least two) and tests?
10. Is the DX budget met, and did the consumer exercise succeed?
11. Is table ownership clear, with no cross-module `JOIN` operations?
12. Have all required ADRs been written?

---

## 13\. Review Triggers (When to Move Up a Level or Make a New Decision)

"More machinery" is justified only by **measurable pain**. Indicators:

| Indicator | Try This First | Only If That Is Insufficient |
| --- | --- | --- |
| `main` exceeds approximately 100 lines **and** much of it is mechanical/repetitive | Split into `build_infra` / `build_modules` / `build_router` functions | An ADR for a wiring tool (not a container) |
| A dependency is passed through four modules | Shared `Deps` | Reconsider module boundaries |
| Building the graph in tests is expensive | `build_app()` / `test_app()` function | More helpers in `*-testing` |
| Need for **runtime-pluggable modules** (enable/disable without recompilation) | — | **Only here** can a registry/plugin system be justified through a separate ADR |
| Generics leak into `AppState` and handlers | Type erasure across seams (P7) | Typestate builder or module redesign |

**ADR-0001 review process:** Use **data** (e.g., "in the last three modules, wiring lines increased from X to Y and caused these problems") to write a new ADR that declares the previous ADR **superseded**. Reviews without data are forbidden.

---

## 14\. Anti-Patterns (What We Reject in PRs)

| Anti-Pattern | Why | Use Instead |
| --- | --- | --- |
| `HashMap<TypeId, Box<dyn Any>>` as a container | Service Locator; runtime errors | `AppState` \+ `FromRef` |
| `Extension<T>` for services | Missing value = runtime error | `State<T>` |
| `static`/`OnceLock`/`lazy_static` for services | Facade disguised as Rust; tests are not isolated | Constructor injection |
| Complete `State<AppState>` in a handler | Unlimited dependencies | `State<TheOnlyNeeded>` |
| Driver/guard name as a string | Runtime typos | Type or closed enum |
| Macro that performs global registration or hides control flow | Magic | Builder/handwritten code |
| `Auth<U, H, S, T>` in `AppState`/handler | Generic explosion | Non-generic type + internal type erasure |
| Leaking `sqlx::Error`/`Row` across seams | Tool dependency | Module error + conversion in adapter |
| `JOIN` against another module's table | Breaks data ownership | That module's Port/service or a read model with an ADR |
| `unwrap()`/`expect()` in runtime paths | Panic instead of typed error | `Result` and module error |
| Logging secrets/tokens/passwords | Disclosure | `Secret<T>` with redacted Debug |
| Using events as "commands" or to obtain answers | Hidden dependency | Direct call/trait for queries |
| Trait with 15 methods | Large Port = large dependency | Several small Ports |
| `anyhow` in infrastructure modules | Untyped errors | `thiserror` |

---

## 15\. Open Questions and Risks

| ID | Topic | Status |
| --- | --- | --- |
| OQ-1 | **Generic explosion** in builders (e.g., `Auth::builder()` with several type parameters) | Default approach: type erasure across seams; validate early with a real module |
| OQ-2 | **Separate migrations for each module** without conflicts in the `sqlx` migration history table | Not investigated; central directory for now |
| OQ-3 | **Async in****`dyn`****:** migrating from `async_trait` when native support arrives | Requires official tracking |
| OQ-4 | **Transactions and consistency across modules** (outbox, saga) | Separate design when the first real need arises |
| OQ-5 | **Authorization** and its relationship with Auth | Outside current scope; preserve the boundary |
| OQ-6 | **Choosing between styles A and B** for module state | A is the default for now; B requires justification |

**Main Risks**

1. **Discipline erosion:** Rules remain effective only through the review gate and PR checklist.
2. **`AppState`****becoming a god object:** If handlers receive the complete `State<AppState>`, the benefits of substates disappear.
3. **`main`****growing too large:** Solve this by splitting it into functions, not by introducing a registry.
4. **The temptation of "convenience":** Whenever you are tempted to make something "automatic," apply the P3 test.

---

## Appendix A: ADR Template

```markdown
# ADR-XXXX: <Short Title>

- Status: Proposed | Accepted | Superseded by ADR-YYYY
- Date:
- Decision Makers:

## Context
What is the problem? What constraints do we have?

## Decision
One or two decisive sentences.

## Considered Options
| Option | Pros | Cons |

## Consequences
Positive / Negative (accepted) / What will this make harder later?

## Review Triggers
What evidence would challenge this decision?
```

## Appendix B: Module Design Document Template

```markdown
# Module Design: <Name>
## 0. Scope and Non-Goals
## 1. Laravel Analysis (Magic → Replacement Table)
## 2. Independent Axes of Change and Mapping to Ports/Types
## 3. Dependencies: Graph, Deps Table, Table Ownership
## 4. Contracts (Trait and Type Signatures), Generic/dyn Decision
## 5. Configuration, Events, Threats, and Controls
## 6. DX Design: Wiring in main, Consumer Handler, Consumer Test
## 7. Reference Implementations (At Least Three)
## 8. Complete Scenario + [Module]/[Developer] Table
## 9. ADRs and Review Gate Result
```

## Appendix C: PR Checklist for New Modules

```markdown
- [ ] Design document (Appendix B) is attached, and Step 6 (DX design) was written before implementation.
- [ ] `core` contains no IO/framework dependencies.
- [ ] Dependencies are injected only through `new`/`builder`/`Deps`.
- [ ] No static/OnceLock/Service Locator/Extension is used for services.
- [ ] No generics leak into `AppState` or handler signatures.
- [ ] Configuration provides values only; implementation selection uses code/closed enums.
- [ ] Errors are typed; secrets use `Secret<T>`.
- [ ] Every Port has at least two implementations and tests.
- [ ] Migrations are in `migrations/` with a module prefix; table ownership is clear.
- [ ] A complete scenario exists in the README; the consumer exercise was performed.
- [ ] Required ADRs have been written.
```

## Appendix D: Laravel-to-This-System Glossary

| Laravel | In This System |
| --- | --- |
| Service Container | Composition Root (`main`) + `AppState` \+ constructors |
| Service Provider | `Deps` \+ `new`/`builder` \+ `routes` (without register/boot) |
| Facade | Extractor or explicit handle (`State<Cache>`) |
| Contract (Interface) | Trait as a Port (defined by the consumer) |
| `config/*.php` | Typed `serde` struct for values |
| Driver selection through environment variables | `match` on a closed enum in `main` |
| Middleware + alias | Tower `layer` or extractor |
| Events / Listeners | Event enum + explicit channel + subscription in `main` |
| Eloquent | Not available; `sqlx` with explicit queries |
| Migrations | Central `migrations/` directory with module prefixes |
| `Auth::extend` / `Cache::extend` | One `impl Port for MyThing` \+ one line in `main` |
| Artisan | `cargo xtask` / CLI subcommands (outside this document's scope) |

## Appendix E: Examples Analyzed Using This Process (Summary)

### Auth (Previously Analyzed Together)

- **Axes:** Credential extraction, validation, identity storage (`IdentityStore`), state management (`SessionStore` / token), applying the result to the response.
- **Separation:** Stateful (session) from stateless (bearer).
- **In Rust:** Guard = **extractor type** (`SessionAuth<User>`, `BearerAuth<Customer>`); provider = `IdentityStore` with `type Identity` (so `User` and `Customer` cannot be confused).
- **Security in the API:** Session ID rotation inside `login`, constant-time comparison, `needs_rehash`.
- **Events:** `LoggedIn`, `LoginFailed`, `LoggedOut`.

### Cache

- **Axes:** Store (memory / redis / file / db), serialization, namespace/prefix, TTL, and clock.
- **Port:** A simple `CacheStore` (input/output bytes + TTL); a typed API built on top of it in the `Cache` service.
- **Typed keys** instead of plain strings:

```rust
const USER_COUNT: Key<u64> = Key::new("user_count");     // Value type is tied to the key
let n = cache.get(&USER_COUNT).await?;
let v = cache.remember(&USER_COUNT, ttl, || async { compute().await }).await?;
```

- **Store selection:** `match` on a closed enum in `main` (P0).
- **Events:** Usually unnecessary (no side effect).

### Mail

- **Axes:** Message composition (`Message`/templates), transport (SMTP / API / log / in-memory), template rendering, delivery strategy (synchronous / queue).
- **Port:** `Transport`. The `Mail` service is non-generic.
- **Target DX:** `mail.send(Welcome::new(&user)).await?`
- **Events:** `MailSent`, `MailFailed`; critical delivery uses the outbox.
- **Testing:** An in-memory transport that stores messages for assertions.

## Appendix F: Verification of Technical Claims (2026-10-30)

**Verified against official documentation/sources at the time of writing:**

- `FromRef` for substates exists in `axum`, and `#[derive(FromRef)]` requires the `macros` feature.
- For libraries, the recommended pattern is to keep `S` generic and add the bound `MyState: FromRef<S>` (the same approach assumed for module extractors).
- To compose nested Routers, the inner Router must specify its state type (e.g., `Router<AppState>`) so it can be combined with the outer Router. (Style A.)
- `async fn` in traits has been stable since Rust 1.75, but traits containing it are not natively dyn-compatible. Current solutions include `async_trait`/`dynosaur` or manual boxing, and native design work is being tracked as part of Rust's 2026 project goals.

**Not yet verified (must be checked before relying on them):**

- Connecting a Router that has been closed with `with_state` to an outer Router (Style B); probably through `nest_service`.
- `sqlx` details: `query!`/`query_as!` macros in offline mode and `cargo sqlx prepare --check`, `#[sqlx::test]` behavior, the `&mut *tx` pattern, and the exact `#[derive(sqlx::FromRow)]` name in the current version.
- Calling a Router without a network through `tower::ServiceExt::oneshot` and the exact `axum::serve` signature in the current version.
- `#[diagnostic::on_unimplemented]` (whether it is available and stable in the version used by the team).
- Separate migrations for each module in `sqlx` (OQ-2).
- The current status and maturity of `dynosaur`.

تمام شد. متن کامل در سند قابل ویرایش بالا قرار گرفت.

برای قرار دادن در ریپو، مسیر فایل:

```text
docs/architecture-manifesto.md
```

