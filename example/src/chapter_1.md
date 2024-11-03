# mdbook-svgdx

## Installation

**mdbook-svgdx** is a preprocessor which converts [svgdx](https://svgdx.net) format diagrams within your Markdown into inline SVG diagrams.

To use it, install mdbook-svgdx (e.g. with `cargo install mdbook-svgdx`), and add the following to your book.toml:

```toml
[preprocessor.svgdx]
```


## Usage

With the preprocessor installed and configured as part of your book, all that remains is to add diagrams in the Markdown source of your book.

This is done with the use of _fenced code blocks_ with the type (technically, 'info string') `svgdx`.

Let's start with a basic example - just a pair of connected rectangles with a rounded-rectangle outline.

```svgdx
<svg>
 <rect id="one" wh="20 10" text="one"/>
 <rect id="two" xy="^:h 10" wh="^" text="two"/>
 <line start="#one" end="#two"/>
 <rect surround="#one #two" margin="3" rx="2" class="d-dot"/>
</svg>
```

That wasn't so hard, was it? The XML for the diagram above is a fenced code block, with the type `svgdx` (as opposed to the below duplicate, which uses `xml`).

```xml
<svg>
 <rect id="one" wh="20 10" text="one"/>
 <rect id="two" xy="^:h 10" wh="^" text="two"/>
 <line start="#one" end="#two"/>
 <rect surround="#one #two" margin="3" rx="2" class="d-dot"/>
</svg>
```

## Themes:

### Light

```svgdx
<svg>
    <config theme="light"/>
    <rect wh="20 10" text="light"/>
</svg>
```

### Dark

```svgdx
<svg>
    <config theme="dark"/>
    <rect wh="20 10" text="dark"/>
</svg>
```

### Glass

```svgdx
<svg>
    <config theme="glass"/>
    <rect wh="20 10" text="glass"/>
</svg>
```
