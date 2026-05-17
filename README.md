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

By default `mdbook-svgdx` uses its built-in `svgdx` library support. When
`MDBOOK_SVGDX_BIN` is set in the environment, `mdbook-svgdx` invokes that
binary as a subprocess instead. When it is not set, the preprocessor falls back
to the built-in library if that feature is included in the build.

## Features

This crate provides a `builtin-svgdx` default feature that includes the `svgdx`
library internally and uses that for rendering. The alternative (with
`--no-default-features`) is to require `MDBOOK_SVGDX_BIN` and use a subprocess
in all cases. This provides a leaner `mdbook-svgdx` binary, but requires that
`svgdx` is installed locally and requires environment configuration, but
_potentially_ avoids needing to update the preprocessor itself with each
`svgdx` change.

## Developing

To test changes to `mdbook-svgdx`, update your `book.toml` with the following 'command' line under the `preprocessor.svgdx` block:

```toml
[preprocessor.svgdx]
command = "cargo run --manifest-path /path/to/mdbook-svgdx/Cargo.toml --quiet"
```

To test changes to the `svgdx` binary itself, point `mdbook-svgdx` at an external
`svgdx` executable via `MDBOOK_SVGDX_BIN` instead of modifying this repository's
dependency graph. For example:

```bash
export MDBOOK_SVGDX_BIN=/path/to/svgdx/target/debug/svgdx
mdbook build
```

## License

This repository is released under the MIT license; for more information see the [LICENSE](LICENSE) file.
