# rust-patterns
Rust patterns

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

