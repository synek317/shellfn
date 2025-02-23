# Changelog

## version 0.2.0 - 2025-02-23

- refresh crate - update Rust version, dependencies, style thanks to [caspermeijn](https://github.com/caspermeijn)

## version 0.1.1 - 2019-02-16

- handle fn arguments in `cmd` attribute parameter
```rust
#[shell(cmd = "python -m $MODULE")]
fn run(module: &str) -> Result<String, Box<Error>> {
    ""
}
```
- handle new return types:
  * `()` (same as no return type)
  * `()` + no_panic (same as no return type)
  * `Result<(), E>`
  * `Vec<T>`
  * `Vec<T>` + no_panic
  * `Vec<Result<T, E>>`
  * `Vec<Result<T, E>>` + no_panic
  * `Result<Vec<T>, E>`
  * `Result<Vec<T>, E>` + no_panic
  * `Result<Vec<Result<T, E1>>, E1>`

## version 0.1.0 - 2019-02-15

Initial release:

- `#[shell]` attribute
- cmd, no_panic attribute parameters
- set env variables from function arguments
- handle return types:
  * `void`
  * `void` + no_panic
  * `T`
  * `Result<T, E>`
  * `impl Iterator<Item=T>`
  * `impl Iterator<Item=T>` + no_panic
  * `impl Iterator<Item=Result<T, E>>`
  * `impl Iterator<Item=Result<T, E>>` + no_panic
  * `Result<impl Iterator<Item=T>, E>`
  * `Result<impl Iterator<Item=T>, E>` + no_panic
  * `Result<impl Iterator<Item=Result<T, E1>>, E1>`