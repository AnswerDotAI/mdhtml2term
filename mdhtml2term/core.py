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

INLINE = dict(strong='bold', emph='italic', code='cyan', strike='strike', link='underline bright_blue', highlight='reverse')

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

def _inline(nodes, text=None, style=''):
    if text is None: text = Text()
    for node in nodes:
        kind = node['type']
        sty = f"{style} {INLINE.get(kind, '')}".strip()
        if kind in ('text', 'code'): text.append(node['text'], style=sty or None)
        elif kind in ('soft_break', 'hard_break'): text.append('\n')
        elif kind == 'image': text.append(f"[{node['alt'] or 'image'}]", style=f'{style} dim'.strip())
        elif kind == 'math_inline': text.append(node['tex'], style=sty or None)
        elif kind == 'footnote_ref': text.append(f"[{node['label']}]", style=sty or None)
        else:
            href = node['url'] if kind == 'link' else None
            if href: sty = f'{sty} link {href}'
            start = len(text)
            _inline(node.get('children', []), text, sty)
            if href and text.plain[start:] != href: text.append(f' ({href})', style='dim')
    return text

def _list(node, theme):
    items = []
    for i,item in enumerate(node['items'], node['start']):
        bullet = f'{i}. ' if node['ordered'] else '• '
        check = '' if item['checked'] is None else '[x] ' if item['checked'] else '[ ] '
        items.append(Prefixed(_blocks(item['blocks'], theme), bullet + check, ' ' * cell_len(bullet), style='bold'))
    return Group(*items)

def _caption(node):
    text = _inline(node['caption'])
    if number := node['number']: text = Text(number['display'] + (': ' if text.plain else '')) + text
    return text

def _table(node, theme):
    head = next((r for r in node['head'] if r['type'] == 'row'), None)
    rows = [r for r in [*node['rows'], *node['foot']] if r['type'] == 'row']
    first = head if head is not None else rows[0] if rows else None
    table = Table(box=box.SIMPLE_HEAD, show_header=head is not None, show_edge=False, pad_edge=False)
    if first is not None:
        for cell in first['cells']:
            table.add_column(Group(*_blocks(cell['content'], theme)) if head is not None else '', justify=cell['align'] or 'left')
        for row in rows: table.add_row(*(Group(*_blocks(c['content'], theme)) for c in row['cells']))
    return Group(_caption(node), table) if node['caption'] or node['number'] else table

def _block(node, theme):
    kind = node['type']
    if kind == 'heading':
        prefix = '#' * node['level'] + ' ' + (node['number']['display'] + ' ' if node['number'] else '')
        return _inline(node['children'], Text(prefix, style='bold magenta'), 'bold magenta')
    if kind == 'paragraph': return _inline(node['children'])
    if kind == 'code_block': return Syntax(node['text'].rstrip('\n'), node['lang'] or 'text', theme=theme)
    if kind == 'list': return _list(node, theme)
    if kind == 'block_quote': return Prefixed(_blocks(node['children'], theme), '│ ', style='dim')
    if kind == 'thematic_break': return Text('─' * 40, style='dim')
    if kind == 'table': return _table(node, theme)
    if kind == 'math_block':
        text = node['tex'] + (f" ({node['number']['display']})" if node['number'] else '')
        return Text(text)
    if kind == 'figure': return Group(*_blocks(node['body'], theme), _caption(node))
    if kind == 'panel':
        title = _inline(node['title']['children'], style='bold') if node['title'] else Text(node['label'], style='bold')
        return Group(title, *_blocks(node['body'], theme))
    if kind == 'definition_list':
        out = []
        for item in node['items']:
            out.extend(_inline(term['children'], style='bold') for term in item['terms'])
            for definition in item['definitions']: out.extend(_blocks(definition, theme))
        return Group(*out)
    if kind in ('raw_block', 'script_block', 'operation', 'html_block'): return None
    return Group(*_blocks(node['children'], theme))

def _blocks(nodes, theme): return [r for node in nodes if (r := _block(node, theme)) is not None]

@delegates(mdhtml.parse_md)
def md_blocks(
    md:str, # Markdown source
    theme:str='ansi_dark', # Pygments theme for code blocks
    **kwargs
):
    "One Rich renderable per top-level semantic block of `md`."
    doc = mdhtml.parse_md(md, **kwargs).bake(refs='lenient', links=False)
    notes = [Prefixed(_blocks(body, theme), f'[{label}] ', ' ' * (cell_len(label) + 3)) for label,body in doc.footnotes.items()]
    return _blocks(doc.blocks, theme) + notes
