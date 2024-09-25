# mdbook-svgdx

A preprocessor for [mdBook][] to convert [svgdx][] fenced code blocks into inline SVG images.

[mdBook]: https://rust-lang.github.io/mdBook
[svgdx]: https://github.com/codedstructure/svgdx

## Installation

For now installation requires a working Rust toolchain, e.g. installed from [rustup.rs](https://rustup.rs).

Install `mdbook-svgdx` as follows:

```
cargo install mdbook-svgdx
```

## Usage

Your mdbook source must be configured to use the `mdbook-svgdx` preprocessor.
To do this, simply add the following line to your `book.toml` file.

```toml
[preprocessor.svgdx]
```

## Developing

To test changes to `mdbook-svgdx`, update your `book.toml` with the following 'command' line under the `preprocessor.svgdx` block:

```toml
[preprocessor.svgdx]
command = "cargo run --manifest-path /path/to/mdbook-svgdx/Cargo.toml --quiet"
```

In order to test changes to the `svgdx` library itself, update the appropriate `dependencies`
entry of [Cargo.toml](Cargo.toml) of this (mdbook-svgdx) repo to point to a local clone of `svgdx`,
rather than providing a version specifier:

```toml
svgdx = { path = "/path/to/svgdx", default-features = false }
```

## License

This repository is released under the MIT license; for more information see the [LICENSE](LICENSE) file.
