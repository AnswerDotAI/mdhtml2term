import pytest
from mdhtml2term import render, sections


def test_select_cited_prose_and_code():
    prose = 'Résumé: [@ruder]{ref=text}.'
    fence = '```python\nscore = 92\n```'
    md = prose + '\n\n' + fence + '\n\n::: bibliography\nHoward and Ruder {#ruder year=2018}\n: ULMFiT.\n:::'
    para, code, *_ = sections(md, 80, cite_style='numeric')
    assert para.source.strip() == prose and not para.code
    assert any('[1]' in row for row in para.rows)
    assert code.code and code.source.strip() == fence
    with pytest.raises(ValueError): render(md, 80, cite_style='unknown')
