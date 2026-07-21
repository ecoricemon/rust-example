# Rust Patterns

Examples of useful Rust patterns.

## Detecting Trait Implementations

[Source code](impl-detect/src/main.rs)

Detects whether `T` implements certain traits.

When to use:

- You want to know at runtime whether a type implements certain traits.

## Self-Referential Types

[Source code](self-referential/README.md)

A workaround for self-referential types.

When to use:

- You need to work with a type such as `&'a Type<'a>`.

## Generic Trait Objects

[Source code](generic-trait-object/src/main.rs)

Creating a trait object from a trait with generic methods.

When to use:

- You want to create a trait object from a trait that has generic methods.
- The generic methods require a `'static` lifetime, as in `foo<T: 'static>()`.

## Heterogeneous functions in a list

[Source code](different-signature-fn-list/src/main.rs)

When to use:

- You want to manage functions with different signatures in a single collection.
- You need to call those functions from your code.

## ECS: System and Query

[Source code](ecs-system-query/src/main.rs)

When to use:

- You want to see how to implement an ECS using systems and queries.

## WASM Web Workers with Vite

[Source code](wasm-worker/src/lib.rs)

When to use:

- You want to use Web Workers and bundle your JavaScript and WASM with Vite.
