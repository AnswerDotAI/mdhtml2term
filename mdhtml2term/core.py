"Render MDHTML Markdown as Rich renderables for the terminal, one per top-level block."
import mdhtml
from fastcore.meta import delegates
from rich import box
from rich.cells import cell_len
from rich.console import Group
from rich.segment import Segment
from rich.style import Style
from rich.syntax import Syntax
from rich.table import Table
from rich.text import Text

__all__ = ['md_blocks']

INLINE = {'strong': 'bold', 'em': 'italic', 'code': 'cyan', 'del': 'strike', 'a': 'underline bright_blue', 'mark': 'reverse'}
HEADINGS = {'h1', 'h2', 'h3', 'h4', 'h5', 'h6'}
BLOCKS = HEADINGS | {'p', 'pre', 'ul', 'ol', 'blockquote', 'hr', 'table', 'div', 'section', 'aside', 'figure', 'figcaption', 'details', 'summary', 'dl', 'dt', 'dd'}

class Prefixed:
    "Renders `rends` one after another, with `first` before the first line and `rest` before each later line"
    def __init__(self, rends, first, rest=None, style=''):
        self.rends,self.first,self.rest = rends,first,first if rest is None else rest
        self.style = Style.parse(style) if style else None

    def __rich_console__(self, console, options):
        opts = options.update_width(max(1, options.max_width - cell_len(self.first)))
        lines = [l for r in self.rends for l in console.render_lines(r, opts, pad=False)] or [[]]
        for i,line in enumerate(lines):
            yield Segment(self.first if i == 0 else self.rest, self.style)
            yield from line
            yield Segment.line()

def _elem(c, t, style=''):
    nm = getattr(c, 'name', None)
    if nm == '#text': return t.append(c.text, style=style or None)
    if nm == 'br': return t.append('\n')
    a = c.attrs or {}
    if nm == 'img': return t.append(f"[{a.get('alt') or 'image'}]", style=f'{style} dim'.strip())
    if nm == 'input' and a.get('type') == 'checkbox': return t.append('[x]' if 'checked' in a else '[ ]', style=style or None)
    sty = f"{style} {INLINE.get(nm, '')}".strip()
    href = nm == 'a' and a.get('href')
    if href: sty = f'{sty} link {href}'
    n = len(t)
    _inline(c, t, sty)
    if href and t.plain[n:] != href: t.append(f' ({href})', style='dim')

def _inline(node, t, style=''):
    for c in node.children: _elem(c, t, style)
    return t

def _find(el, name):
    for c in getattr(el, 'children', []):
        if getattr(c, 'name', None) == name: yield c
        else: yield from _find(c, name)

def _code_block(pre, theme):
    code = next(_find(pre, 'code'), None)
    if code is None: return _inline(pre, Text())
    src = ''.join(c.text for c in code.children if getattr(c, 'name', None) == '#text')
    cls = (code.attrs or {}).get('class') or ''
    lang = next((w.removeprefix('language-') for w in cls.split() if w.startswith('language-')), 'text')
    return Syntax(src.rstrip('\n'), lang, theme=theme)

def _list(el, theme):
    ordered = el.name == 'ol'
    n = int((el.attrs or {}).get('start', 1))
    items = []
    for li in el.children:
        if getattr(li, 'name', None) != 'li': continue
        bullet = f'{n}. ' if ordered else '• '
        items.append(Prefixed(_blocks(li, theme), bullet, ' ' * cell_len(bullet), style='bold'))
        n += 1
    return Group(*items)

def _cells(tr): return [c for c in tr.children if getattr(c, 'name', None) in ('td', 'th')]

def _table(el):
    head, rows = None, []
    for sec in el.children:
        nm = getattr(sec, 'name', None)
        if nm == 'thead': head = next(_find(sec, 'tr'), None)
        elif nm in ('tbody', 'tfoot'): rows += list(_find(sec, 'tr'))
        elif nm == 'tr': rows.append(sec)
    first = head if head is not None else rows[0] if rows else None
    t = Table(box=box.SIMPLE_HEAD, show_header=head is not None, show_edge=False, pad_edge=False)
    if first is None: return t
    for c in _cells(first): t.add_column(_inline(c, Text()) if head is not None else '', justify=(c.attrs or {}).get('align') or 'left')
    for tr in rows: t.add_row(*[_inline(c, Text()) for c in _cells(tr)])
    return t

def _block(el, theme):
    nm = el.name
    if nm in HEADINGS: return _inline(el, Text('#' * int(nm[1]) + ' ', style='bold magenta'), 'bold magenta')
    if nm == 'p': return _inline(el, Text())
    if nm == 'pre': return _code_block(el, theme)
    if nm in ('ul', 'ol'): return _list(el, theme)
    if nm == 'blockquote': return Prefixed(_blocks(el, theme), '│ ', style='dim')
    if nm == 'hr': return Text('─' * 40, style='dim')
    if nm == 'table': return _table(el)
    return Group(*_blocks(el, theme))

def _blocks(el, theme):
    "One renderable per block child of `el`, plus one `Text` per run of inline content between them"
    out, t = [], Text()
    def flush():
        nonlocal t
        t.rstrip()
        if t.plain: out.append(t)
        t = Text()
    for c in getattr(el, 'children', []):
        nm = getattr(c, 'name', None)
        if nm in BLOCKS:
            flush()
            out.append(_block(c, theme))
        elif nm == '#text' and not t.plain: t.append(c.text.lstrip())
        else: _elem(c, t)
    flush()
    return out

@delegates(mdhtml.md2dom)
def md_blocks(
    md:str, # Markdown source
    theme:str='ansi_dark', # Pygments theme for code blocks
    **kwargs
):
    "One Rich renderable per top-level block of `md`"
    return _blocks(mdhtml.md2dom(md, **kwargs), theme)
