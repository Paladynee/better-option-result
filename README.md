# better_option_result 🦀

[![Crates.io](https://img.shields.io/crates/v/better_option_result.svg)](https://crates.io/crates/better_option_result)
[![Documentation](https://docs.rs/better_option_result/badge.svg)](https://docs.rs/better_option_result)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A library that provides enhanced versions of Rust's `Option` and `Result` types with consistent naming conventions.

## The Problem

Rust's standard library `Option` and `Result` types have fantastic methods, but their naming conventions are either inconsistent with the rest of the library, or too vague. Some examples include:

-   `Result::is_ok_and` and `Result::is_err_and` vs `Option::is_some_and` and `Option::is_none_or`.
-   What does `Option::or` do? Can you tell without looking at the signature, just by looking at the name?

These drawbacks make it harder to predict and remember method names, especially for newcomers.

For demonstration, try to guess the name of the `Result<T, E>` method that takes a `Result<U, E>` and returns another `Result,U, E>`. With this library, you just need to think what is changed, so in this case, `Result::into_ok_of_arg`.

In the standard library, the same method is called `and`, which is more concise, and in its specific context it makes sense, so we have stable aliases for them too.

## The Solution

`better_option_result` provides drop-in replacements for `Option` and `Result` with:

1. **Consistent naming conventions** following Rust's own guidelines:

    - `into_*` for methods that consume `self` and transform ownership
    - `as_*` for reference-based operations
    - `is_*` for boolean checks
    - `*_lazy` for lazily evaluated operations
    - `*_of_arg` for operations that return a new type based on the argument

2. **Full backward compatibility** with standard library method names:
    - Standard (core) library aliases available on demand with the `aliases` default feature flag.

## Features

-   🔄 **Comprehensive method aliases** that follow consistent naming patterns
-   🧠 **Intuitively predictable naming** - once you know the pattern, you can guess method names
-   🔗 **Complete compatibility** with the standard library
-   🔍 **Extended functionality** with additional Boolean logic operations
-   🛠️ **Zero dependencies**, we're even `#![no_std]`

## Installation

Add the library using `cargo`:
```bash
cargo add better_option_result
```

...or add it manually to your `Cargo.toml`:

```toml
[dependencies]
better_option_result = "*"
```

## Examples

todo

## Why Use BetterOkRes?

-   **Learning**: The consistent naming makes it easier to understand patterns
-   **Better code completion**: Logical grouping of related methods with prefix-based naming
-   **Greater expressivity**: The methods are aimed to be more descriptive
-   **Zero-cost abstraction**: Compiles down to the same machine code as the usual `Option` and `Result` types

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request or open an issue on [GitHub](https://github.com/Paladynee/better-option-result).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
