
# Rustavel

**Rustavel** is an experimental, Rust-first backend toolkit inspired by the *developer experience* of Laravel —  
not its runtime model, not its magic, and not its ORM assumptions.

This project explores a simple question:

> Can we offer a familiar, productive DX for backend developers  
> while staying idiomatic, explicit, and safe in Rust?

Rustavel is **not** a Laravel port.  
It is **not** an ORM.  
It is **not** a framework that hides SQL or system boundaries.

It is a growing collection of **opt-in tools** designed to feel comfortable for developers coming from Laravel / PHP,
while respecting the values of the Rust ecosystem: correctness, clarity, and performance.

---

## Philosophy

Rustavel follows a few strict principles:

- **Rust is the source of truth**  
  No runtime magic, no reflection, no hidden behavior.

- **DX matters, but never at the cost of safety**  
  If something cannot be expressed safely or clearly in Rust, we don’t force it.

- **Opt-in abstractions**  
  Nothing is mandatory. You can adopt individual parts without buying into the whole stack.

- **SQL-first, not ORM-first**  
  Data access is explicit. Rustavel does not impose an ORM model.

- **Familiar ideas, idiomatic Rust**  
  Laravel-inspired *concepts*, not Laravel-style implementations.

---

## Current Scope

Rustavel is under active development and currently focuses on:

- Application configuration (env-driven, explicit, and testable)
- Routing DSL built on top of `axum`
- Migration system with a Rust-based schema DSL
- CLI tooling inspired by `artisan`
- Template rendering via `minijinja`


The project intentionally avoids over-engineering and grows only when real usage justifies it.

---

## Data Access & ORM Stance

Rustavel **does not ship with an ORM**.

Instead:

- The default and recommended data access layer is **`sqlx`**
- SQL remains explicit and visible
- Models describe structure, not behavior
- Query execution is left to the user

A minimal, type-safe DSL exists only to describe **query shape and intent**, not execution.

> If you prefer another approach — raw SQL, `sqlx`, `sea-query`, or something else —  
> Rustavel does not stand in your way.

An ORM may exist **in the future**, but only if:
- it solves real problems,
- remains explicit,
- and earns its place through usage — not assumptions.

---

## Workspace Structure

Rustavel is organized as a Cargo workspace:

```

├── app        # Application layer
├── core       # Shared primitives and abstractions
├── artisan    # CLI tooling
├── database   # Migrations and schema-related code

```

Each crate has a clear responsibility and can evolve independently.

---

## Project Status

Rustavel is **early-stage** and **intentionally incomplete**.

This is not a finished framework —  
it is a foundation being shaped in the open.

APIs may evolve.
Names may change.
Boundaries may shift.

Stability will come *after* clarity.

---

## Contributing

Contributions are **highly welcome**.

Especially if you care about:
- clean DSL design
- safe abstractions
- developer experience without hidden costs
- bridging mental models between ecosystems

You do **not** need to agree with every design decision to contribute.
Discussion, alternatives, and critiques are encouraged.

> The goal is not to copy Laravel —  
> the goal is to build something *worthy of Rust*.

If you are unsure where to start:
- open an issue
- ask questions
- propose ideas
- or improve documentation

Every thoughtful PR matters.

---

## License

Rustavel is released under the **MIT License**.

Use it freely.
Fork it.
Experiment.
Build something better on top of it.

---


## A Note to the Laravel Community ❤️

Rustavel exists because Laravel exists.

For many developers — including the author of this project — Laravel was not just a framework,
but a way of learning how to think about backend systems, developer experience, and balance.

Routing clarity, migrations, expressive configuration, and a strong community culture
have shaped an entire generation of developers.

Rustavel does **not** aim to replace Laravel.
It exists for a different ecosystem, with different constraints, and different trade-offs.

If you come from Laravel and are curious about Rust:
- you are welcome here
- your feedback is valuable
- and your perspective matters

This project is built with deep respect for the ideas that Laravel popularized,
and with full awareness that many of them cannot — and should not — be copied directly into Rust.

If Rustavel feels familiar at times, that is intentional.
If it feels different, that is unavoidable — and often desirable.

Thank you, Laravel.

## For Newcomers

If you are new to Rust or backend development — you are welcome.

Rustavel is being built to be approachable, but it is still an **early-stage project**.
Some features you might expect from mature frameworks may not exist yet.

If you have requests such as:
- “Can it do X?”
- “Is there support for Y?”
- “Why doesn’t it work like Z?”

Feel free to ask.

We review all feature requests and ideas carefully.
Some may be accepted, some postponed, and some declined — always with an explanation.

Learning and exploration are part of the process.
Respectful questions are always encouraged.


## Graveyard

This directory contains ideas that were explored during development but ultimately abandoned.

For each idea, a short explanation (typically 5–10 lines) is provided **at the top of the source files as comments**, describing why the approach was discontinued and what issues were identified.
This is intentional: if the code is copied or reused elsewhere, its context and limitations should remain immediately visible.

The code is preserved largely in the state it was last touched. Some entries may still have passing tests, which reflects that an implementation can be functional while still being unsuitable or non-idiomatic.

**Why keep these ideas?**

* Some explorations may become useful again with new constraints, insights, or combinations
* They may serve as learning material or inspiration for others
* They help preserve the full design and decision-making history of the project

**Important note on responsibility (specific to this directory only):**
The following warning applies **only to the code contained in the Graveyard directory**, and not to the rest of the project.

If you choose to use any code from this directory, you do so at your own discretion and responsibility.
These implementations were intentionally left as-is after being deemed unsuitable for continuation, and should not be considered production-ready without careful review and revision.

**Workspace note:**
The Graveyard crates are not included in the workspace by default.
To build, test, or evaluate them, the corresponding workspace entries must be explicitly uncommented. This is done deliberately to avoid accidental usage.

If you manage to meaningfully improve or revive one of these ideas in an idiomatic and safe way, contributions or notes about your approach are always welcome.


## Final Note

Rustavel is an exploration.

If it turns out to be useful — great.  
If it inspires better tools elsewhere — even better.

Either way, the journey is the point.
