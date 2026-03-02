# Writing Cards

Card content is written in [Typst](https://typst.app) markup. Both the front and back of each card support full Typst syntax.

## Headings

Use `=` for headings, just like in Typst:

```typst
= What is ownership in Rust?

Every value has a single *owner*. When the owner goes out of scope, the value is dropped.
```

## Text formatting

```typst
This is *bold*, _italic_, and `monospace` text.

- Bullet list item
- Another item
  - Nested item

+ Numbered item
+ Another numbered item
```

## Math

Inline math with `$...$` and display math with `$ ... $` (with spaces):

```typst
Euler's identity: $e^(i pi) + 1 = 0$

The Gaussian integral:

$ integral_(-infinity)^(infinity) e^(-x^2) d x = sqrt(pi) $
```

## Code blocks

```typst
#raw(lang: "rust", "fn main() { println!(\"hello\"); }")
```

## Using the preamble

If you find yourself repeating the same `#set` or `#let` rules on every card, put them in the deck's **preamble** instead. The preamble is prepended to every card automatically.

```typst
// Example preamble
#set text(size: 14pt, font: "New Computer Modern")
#set page(margin: 1em)
#let highlight(body) = box(fill: yellow, inset: 4pt, body)
```

Then on any card you can use `#highlight[important text]` without re-defining the function.

## Syntax highlighting

The card editor provides Typst-aware syntax highlighting:

- **Purple:** keywords like `#set`, `#let`, `#show`, `#import`
- **Blue:** function calls like `#text`, `#image`, `#table`
- **Green:** string literals
- **Orange/gold:** math mode (`$...$`)
- **Gray:** comments (`// ...`)
- **Red:** headings (`= ...`)

## Tags

Each card can have tags (comma-separated) for organization. Tags are stored with the card but don't affect rendering.
