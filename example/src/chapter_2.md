The following shows an example Markdown document including an svgdx diagram.
The next page contains the same content, but at top-level rather than in an
extra fenced code-block, so is rendered including the svgdx content.

---

````markdown

## Moby Dick; or, The Whale.

### By Herman Melville.

#### Loomings.

Call me **Ishmael**. Some years ago—never mind how long precisely—having little or
no money in my purse, and nothing particular to interest me on shore, I thought I
would sail about a little and see the _watery_ part of the world.

```svgdx
<svg width="75%">
 <rect id="ishmael" wh="20 10" text="Ishmael"/>
 <circle id="world" xy="^:h 20" r="10" class="d-fill-blue" text="the world"/>
 <line start="#ishmael" end="#world" text="sails" text-loc="t" class="d-arrow"/>
</svg>
```

It is a way I have of driving off the spleen and regulating the circulation.
````
