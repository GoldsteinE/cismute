# cismute

![docs badge](https://img.shields.io/docsrs/cismute)
![license: BSD-2-Clause-Patent](https://img.shields.io/crates/l/cismute)

Provides safe trivial transmutes in generic context, emulating specialization on stable Rust. These functions are designed for being optimized out by the compiler, so are probably zero-cost in most cases.

```rust
fn specialized_function<T: 'static>(x: T) -> String {
    // We have an efficient algorithm for `i32` and worse algorithm for any other type.
    // With `cismute` we can do:
    match cismute::owned::<T, i32>(x) {
        Ok(x) => format!("got an i32: {x}"),
        Err(x) => format!("got something else"),
    }
}

assert_eq!(specialized_function(42_i32), "got an i32: 42");
assert_eq!(specialized_function(":)"), "got something else");
```

`cismute::owned` works only for `'static` types. If your type is a reference, you should use `cismute::reference` or `cismute::mutable`.

```rust
fn specialized_function<T: 'static>(x: &T) -> String {
    // We have an efficient algorithm for `i32` and worse algorithm for any other type.
    // With `cismute` we can do:
    match cismute::reference::<T, i32>(x) {
        Ok(x) => format!("got an i32: {x}"),
        Err(x) => format!("got something else"),
    }
}

assert_eq!(specialized_function(&42_i32), "got an i32: 42");
assert_eq!(specialized_function(&":)"), "got something else");
```

There's also a more generic function `cismute::value` which can do all three. Writing all type arguments can be cumbersome, so you can also pass the type pair as an argument via `cismute::value_with`:

```rust
use cismute::Pair;

fn specialized_function<T: 'static>(x: &T) -> String {
    match cismute::value_with(Pair::<(T, i32)>, x) {
        Ok(x) => format!("got an i32: {x}"),
        Err(x) => format!("got something else"),
    }
}

assert_eq!(specialized_function(&42_i32), "got an i32: 42");
assert_eq!(specialized_function(&":)"), "got something else");
```

There are also `switch!()` macro and `switch()` function to match one value with multiple types.
