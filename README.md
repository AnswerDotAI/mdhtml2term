# mdhtml2term

Render [MDHTML](https://github.com/AnswerDotAI/mdhtml) Markdown in the terminal with [Rich](https://github.com/Textualize/rich).

`md_blocks` parses Markdown with `mdhtml` and returns one Rich renderable per top-level block. A terminal app can then print, fold or replace each block on its own. ipyai uses it to render AI replies.

## Install

```bash
pip install mdhtml2term
```

## Usage

```python
from rich.console import Console
from mdhtml2term import md_blocks

console = Console()
for block in md_blocks(markdown_text): console.print(block)
```

`md_blocks` takes a Markdown string and returns a list of Rich renderables. `theme` names the Pygments theme for code blocks and defaults to `ansi_dark`. Other keyword arguments go to `mdhtml.parse_md`. The shared Rust document engine resolves references and assigns numbers. Unresolved references show their target ids.

## Supported content

- Headings, paragraphs, block quotes, lists, code blocks, tables and horizontal rules. Block quotes and list items can hold any other block, including code and further quotes.
- Bold, italic, inline code, strikethrough, highlight and line breaks.
- Links, as OSC 8 hyperlinks followed by their address in brackets.
- Images, as their alt text in square brackets.
- Task list checkboxes, as `[ ]` and `[x]`.
- A numbered list that resumes after other content keeps its numbering.

Other inline elements show their text. Other block containers render their children. Raw payloads and unresolved template instructions are omitted.

Bibliographies use mdhtml's `::: bibliography` syntax with explicit citation-author terms, ids, optional `year` attributes and formatted entry definitions. Citations and bibliography layout are prepared in Rust. `md_blocks(..., cite_style='numeric')` selects numbers in bibliography order; `'author-year'` selects author-year text. None inherits frontmatter. Unresolved citation keys remain visible in drafts.
