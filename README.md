# rust-patterns
Rust patterns

## Generic trait object

Trait object from trait with generic methods

When to use
- Want to make a trait obejct from a trait having some generic methods.
- Generic methods require 'static lifetime such as `foo<T: 'static>()`.