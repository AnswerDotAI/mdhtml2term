# mdhtml2term

Render [MDHTML](https://github.com/AnswerDotAI/mdhtml) Markdown as ANSI-styled terminal rows. Python and Rust use the same renderer.

## Python

Install with `pip install mdhtml2term`.

```python
from mdhtml2term import render

markdown = "# Results\n\nThe **baseline** scored 92%."
print(*render(markdown, width=72), sep="\n")
```

## Rust

Add the crate with `cargo add mdhtml2term`.

```rust
let markdown = "# Results\n\nThe **baseline** scored 92%.";
for row in mdhtml2term::render(markdown, 72) { println!("{row}"); }
```

Pass the available width in terminal columns, excluding any gutter. Render again after a resize. Each row contains text and SGR styles, with no trailing newline or cursor commands.

## Sections and citations

Use `render(markdown, width)` to display a document, or `sections(markdown, width)` to select individual blocks. A section has the original Markdown in `source`, its display lines in `rows`, and a `code` flag for code blocks. Collected footnote definitions form a final section. Pass the whole document to either function so citations can refer to a later `::: bibliography` and footnote numbering works across blocks.

Both functions inherit the document's citation style. In Python, pass `cite_style="numeric"` or `cite_style="author-year"` to override it. In Rust, use `render_with_style` or `sections_with_style` with a third argument such as `Some(mdhtml2term::CiteStyle::Numeric)`. Unresolved citation keys remain visible.

## HTML and code

Use `render_html(html, width)` for an HTML fragment such as notebook cell output. `highlight(text, language, width)` renders literal code. For an editor, `highlight_spans(text, language)` returns unwrapped `(SGR prefix, original text)` pairs that preserve tabs and newlines.
