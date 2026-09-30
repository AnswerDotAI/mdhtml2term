from rich.console import Console
from rich.syntax import Syntax
from mdhtml2term import md_blocks

def render(r, width=60, term=False):
    c = Console(width=width, force_terminal=term, color_system='truecolor' if term else None)
    with c.capture() as cap: c.print(r)
    return cap.get()

MD = """# Title

Some **bold**, `code`, and [a link](http://x.com).

```python
print(1)
```

- one
- two
  - nested

> quoted line

| a | bb |
|:--|--:|
| 1 | 2 |
"""

def test_md_blocks():
    bs = md_blocks(MD)
    assert render(bs[0]).strip() == '# Title'
    assert 'a link (http://x.com)' in render(bs[1])
    para = render(bs[1], term=True)
    assert '\x1b]8;' in para and 'http://x.com' in para  # OSC 8: links must be real, not costume
    assert '\x1b[1mbold' in para
    assert isinstance(bs[2], Syntax) and bs[2].code == 'print(1)'
    out = render(bs[3])
    assert '• one' in out and '  • nested' in out
    assert render(bs[4]).startswith('│ quoted line')
    head, rule, row = render(bs[5]).splitlines()
    assert head.split() == ['a', 'bb'] and '─' in rule
    assert row.rstrip().index('2') == head.rstrip().rindex('b')  # right-aligned column

def test_containers():
    out = render(md_blocks("> - quoted item\n>\n> ```py\n> x = 1\n> ```\n>\n> > nested quote\n")[0])
    assert '│ • quoted item' in out and '│ x = 1' in out and '│ │ nested quote' in out
    assert '[a chart]' in render(md_blocks('![a chart](x.png) and more')[0])
    assert render(md_blocks('1. a\n2. b\n\npara\n\n3. c\n')[2]).strip() == '3. c'
    out = render(md_blocks('- [ ] todo\n- [x] done\n')[0])
    assert '• [ ] todo' in out and '• [x] done' in out
    lines = render(md_blocks('- step\n\n  ```py\n  x = 1\n  ```\n')[0]).splitlines()
    assert lines[0].rstrip() == '• step' and lines[1].startswith('  x = 1')
    out = render(md_blocks('- [docxlite](https://pypi.org/p/docxlite) - read docx')[0])
    assert 'docxlite (https://pypi.org/p/docxlite)' in out
    out = render(md_blocks('<https://x.y> and https://a.b/c')[0])
    assert out.strip() == 'https://x.y and https://a.b/c'
