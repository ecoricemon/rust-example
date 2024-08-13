# rust-patterns
Rust patterns

## Trait impl detect

[Source code](impl-detect/src/main.rs)

Detects whether T implements some traits

When to use
- Want to know if a type implements some traits at run-time.

## Generic trait object

[Source code](generic-trait-object/src/main.rs)

Trait object from trait with generic methods

When to use
- Want to make a trait obejct from a trait having some generic methods.
- Generic methods require 'static lifetime such as `foo<T: 'static>()`.

## Heterogeneous functions in a list

[Source code](different-signature-fn-list/src/main.rs)

When to use
- When you want to manager functions that have different signatures from each other.
- You need to call them in your code.

## ECS: System and Query

[Source code](ecs-system-query/src/main.rs)

When to use
- When you want to see how to implement ECS in terms of system and query.


## WASM web worker with Webpack

[Source code](wasm-worker/src/lib.rs)

When to use
- When you want to use web worker and bundle your JS and wasm with Webpack.
