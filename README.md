# mdhtml2term

Render [MDHTML](https://github.com/AnswerDotAI/mdhtml) Markdown for the terminal. Rust apps get ANSI rows. Python apps get [Rich](https://github.com/Textualize/rich) renderables.

## Rust

```rust
for row in mdhtml2term::render(markdown, 72) { println!("{row}"); }
```

Pass the whole message so citations can refer to a bibliography below them. The width is in terminal columns, excluding any gutter. Render again after a resize. Each row contains text and SGR styles, with no trailing newline or cursor commands. Fenced code uses syntax highlighting when its language is recognized.

Use `sections(markdown, width)` to select a paragraph, heading, list or code fence without losing document-wide citations and footnote numbering. Each section includes its Markdown source and rendered rows. Collected endnotes form a final section. `highlight(text, language, width)` renders literal code. For an editable buffer, `highlight_spans(text, language)` returns the original text in token-sized slices with SGR prefixes. `render_html(html, width)` handles notebook HTML output.

## Python install

```bash
pip install mdhtml2term
```

## Python usage

```python
from rich.console import Console
from mdhtml2term import md_blocks

console = Console()
for block in md_blocks(markdown_text): console.print(block)
```

`md_blocks` takes a Markdown string and returns a list of Rich renderables. `theme` names the Pygments theme for code blocks and defaults to `ansi_dark`. Other keyword arguments go to `mdhtml.parse_md`. The shared Rust document engine resolves references and assigns numbers. Unresolved references show their target ids.

## Python rendering

- Headings, paragraphs, block quotes, lists, code blocks, tables and horizontal rules. Block quotes and list items can hold any other block, including code and further quotes.
- Bold, italic, inline code, strikethrough, highlight and line breaks.
- Links, as OSC 8 hyperlinks followed by their address in brackets.
- Images, as their alt text in square brackets.
- Task list checkboxes, as `[ ]` and `[x]`.
- A numbered list that resumes after other content keeps its numbering.

Other inline elements show their text. Other block containers render their children. Raw payloads and unresolved template instructions are omitted.

Bibliographies use mdhtml's `::: bibliography` syntax with explicit citation-author terms, ids, optional `year` attributes and formatted entry definitions. Citations and bibliography layout are prepared in Rust. `md_blocks(..., cite_style='numeric')` selects numbers in bibliography order; `'author-year'` selects author-year text. None inherits frontmatter. Unresolved citation keys remain visible in drafts.
